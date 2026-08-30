#!/usr/bin/env python3
"""Lint the pull-request contract used by the kimizukann repository."""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from pathlib import Path
from typing import Any

from review_gate import canonical_identity, is_waiver_checklist, parse_tickets, verify_ticket


class LintError(RuntimeError):
    pass


LINE_LIMIT = 300
OWNER_LINE_LIMIT_MARKER = re.compile(r"(?im)^\s*helix-line-limit:\s*approve\s*$")
BENCHMARK_TARGET = re.compile(r"^crates/[^/]+/(?:examples|benches)(?:/|$)", re.I)
TRACE_PATH = "docs/20_design/trace.md"


def _decode_many(raw: str) -> list[Any]:
    raw = raw.strip()
    if not raw:
        return []
    decoder = json.JSONDecoder()
    values: list[Any] = []
    index = 0
    while index < len(raw):
        while index < len(raw) and raw[index].isspace():
            index += 1
        if index >= len(raw):
            break
        value, index = decoder.raw_decode(raw, index)
        values.append(value)
    if len(values) == 1:
        return values
    flattened: list[Any] = []
    for value in values:
        flattened.extend(value if isinstance(value, list) else [value])
    return flattened


def gh_api(endpoint: str, paginate: bool = False) -> Any:
    command = ["gh", "api"] + (["--paginate"] if paginate else []) + [endpoint]
    result = subprocess.run(command, text=True, capture_output=True, check=False)
    if result.returncode:
        raise LintError(result.stderr.strip() or result.stdout.strip() or "gh api failed")
    try:
        if paginate:
            values = _decode_many(result.stdout)
            return values[0] if len(values) == 1 else values
        return json.loads(result.stdout)
    except json.JSONDecodeError as exc:
        raise LintError(f"invalid gh api JSON: {exc}") from exc


def read_json(path: str | None) -> Any | None:
    if not path:
        return None
    try:
        return json.loads(Path(path).read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise LintError(f"cannot read JSON fixture {path}: {exc}") from exc


def section(body: str, title: str) -> str:
    pattern = re.compile(rf"(?ms)^##\s+{re.escape(title)}(?:\s*[（(][^\r\n]*)?\s*$\n?(.*?)(?=^##\s+|\Z)")
    match = pattern.search(body)
    return match.group(1) if match else ""


def meaningful(text: str) -> str:
    text = re.sub(r"<!--.*?-->", "", text, flags=re.S)
    text = re.sub(r"```", "", text)
    return "\n".join(line.strip() for line in text.splitlines() if line.strip() and line.strip() not in {"-", "`"})


def writer(body: str) -> str | None:
    match = re.search(r"(?im)^\s*(?:[-*]\s*)?writer\s*:\s*`?([A-Za-z0-9_-]+)`?\s*$", body)
    return match.group(1).lower() if match else None


def touched_stat(body: str) -> tuple[int, int, int, set[str]] | None:
    content = section(body, "触ったファイル")
    fences = re.findall(r"```(?:[^\n]*)\n(.*?)```", content, flags=re.S)
    text = "\n".join(fences) if fences else content
    files_match = re.search(r"(\d+)\s+files?\s+changed", text, re.I)
    ins_match = re.search(r"(\d+)\s+insertions?\(\+\)", text, re.I)
    del_match = re.search(r"(\d+)\s+deletions?\(-\)", text, re.I)
    if not (files_match and ins_match and del_match):
        return None
    listed: set[str] = set()
    for line in text.splitlines():
        match = re.match(r"\s*(?:`([^`]+)`|([^|\s]+))\s*\|", line)
        if match:
            listed.add((match.group(1) or match.group(2)).replace("\\", "/"))
    return int(files_match.group(1)), int(ins_match.group(1)), int(del_match.group(1)), listed


def is_test_path(path: str) -> bool:
    path = path.replace("\\", "/").lower()
    return bool(re.search(r"(^|/)(tests?|__tests__)(/|$)|(^|/)[^/]+_(test|spec)\.[^.]+$|\.spec\.[^.]+$", path))


def is_implementation(path: str) -> bool:
    path = path.replace("\\", "/").lower()
    return path.endswith((".rs", ".py", ".js", ".ts", ".tsx", ".dart")) and not is_test_path(path)


def added_forbidden(files: list[dict[str, Any]]) -> list[str]:
    forbidden = ("#" + "[ignore]", "allow(" + "clippy", "-" + "-no-verify")
    found: list[str] = []
    for entry in files:
        path = str(entry.get("filename", ""))
        normalized_path = path.replace("\\", "/")
        patch = str(entry.get("patch") or "")
        for line in patch.splitlines():
            if not line.startswith("+") or line.startswith("+++"):
                continue
            # Benchmark/example targets may use wall-clock APIs for measuring
            # performance.  Keep the exception path-scoped and limited to the
            # clippy allow directive; production and test code stays fail-closed.
            if forbidden[1] in line and BENCHMARK_TARGET.match(normalized_path):
                continue
            if any(word in line for word in forbidden):
                found.append(f"{path}: {line[1:].strip()}")
    return found


def fetch_data(
    args: argparse.Namespace,
) -> tuple[dict[str, Any], list[dict[str, Any]], list[dict[str, Any]], list[dict[str, Any]]]:
    metadata = read_json(args.pr_file) or gh_api(f"repos/{args.repo}/pulls/{args.pr}")
    # GitHub's pull-request files endpoint is the canonical merge-base
    # (three-dot) diff.  Comparing the body stat with this response keeps
    # synchronization merge commits (HEAD~1) from changing the expected PR
    # statistics.
    files = read_json(args.files_file)
    if files is None:
        files = gh_api(f"repos/{args.repo}/pulls/{args.pr}/files?per_page=100", paginate=True)
    commits = read_json(args.commits_file)
    if commits is None:
        commits = gh_api(f"repos/{args.repo}/pulls/{args.pr}/commits?per_page=100", paginate=True)
    comments = read_json(args.comments_file)
    if comments is None:
        issue_comments = gh_api(f"repos/{args.repo}/issues/{args.pr}/comments?per_page=100", paginate=True)
        review_comments = gh_api(f"repos/{args.repo}/pulls/{args.pr}/comments?per_page=100", paginate=True)
        comments = list(issue_comments or []) + list(review_comments or [])
    return metadata, files or [], commits or [], comments or []


def has_line_limit_exception(
    pr: int,
    metadata: dict[str, Any],
    comments: list[dict[str, Any]],
) -> bool:
    """Return whether an owner-approved line-limit waiver is present.

    The waiver requires both halves of the owner hand-off: an owner comment
    with the exact marker and a current-head HMAC-signed Claude ticket whose
    checklist records ``owner-instructed <timestamp>``.  A normal Claude
    approval, an owner ticket, or a body-only claim is not a waiver.
    """
    head_sha = str(metadata.get("head", {}).get("sha", ""))
    owner_marker = False
    for comment in comments:
        if not isinstance(comment, dict):
            continue
        user = comment.get("user")
        login = user.get("login", "") if isinstance(user, dict) else ""
        if canonical_identity(str(login)) == "owner" and OWNER_LINE_LIMIT_MARKER.search(str(comment.get("body", ""))):
            owner_marker = True
            break
    if not owner_marker:
        return False
    return any(
        is_waiver_checklist(ticket.get("checklist", ""))
        for ticket in valid_signed_tickets(
            pr, metadata, comments, reviewer="claude", verdict="waiver", exact_head=True
        )
    )


def valid_signed_tickets(
    pr: int,
    metadata: dict[str, Any],
    comments: list[dict[str, Any]],
    *,
    reviewer: str,
    verdict: str,
    exact_head: bool = False,
) -> list[dict[str, str]]:
    """Return structurally valid tickets for a named identity and verdict."""
    head_sha = str(metadata.get("head", {}).get("sha", ""))
    secret_text = os.environ.get("HELIX_ATTEST_SECRET", "").strip()
    if not secret_text or not re.fullmatch(r"[0-9a-fA-F]{40}", head_sha):
        return []
    expected_reviewer = canonical_identity(reviewer)
    expected_verdict = verdict.lower()
    valid: list[dict[str, str]] = []
    for ticket in parse_tickets(comments):
        if canonical_identity(ticket.get("reviewer", "")) != expected_reviewer:
            continue
        if ticket.get("verdict", "").lower() != expected_verdict:
            continue
        if exact_head and ticket.get("sha", "").lower() != head_sha.lower():
            continue
        accepted, _ = verify_ticket(ticket, pr, head_sha, secret_text.encode("utf-8"))
        if accepted:
            valid.append(ticket)
    return valid


def has_claude_trace_approval(
    pr: int,
    metadata: dict[str, Any],
    comments: list[dict[str, Any]],
) -> bool:
    """Require Claude's signed approval when generated trace is in the diff."""
    return bool(
        valid_signed_tickets(
            pr, metadata, comments, reviewer="claude", verdict="approve", exact_head=True
        )
    )


def run(args: argparse.Namespace) -> int:
    errors: list[str] = []
    try:
        metadata, files, commits, comments = fetch_data(args)
        body = str(metadata.get("body") or "")
        ref = str(metadata.get("head", {}).get("ref", ""))
        draft = bool(metadata.get("draft", False))
        suffix = ref.split("/", 1)[1] if "/" in ref else ""
        if not re.fullmatch(r"(?:task|rfc|hotfix)/[A-Za-z0-9][A-Za-z0-9._-]*", ref):
            errors.append("head branch must be task/<ID>, rfc/<ID>, or hotfix/<ID>")
        id_value = meaningful(section(body, "ID")).splitlines()[0] if meaningful(section(body, "ID")) else ""
        if not id_value:
            errors.append("ID section is empty")
        elif suffix and not (id_value == suffix or id_value.startswith(suffix + " ") or suffix.startswith(id_value + " ")):
            errors.append(f"ID ({id_value}) does not match branch suffix ({suffix})")
        kind = meaningful(section(body, "種別"))
        if kind not in {"feat", "fix", "design", "docs", "test", "ci", "rfc", "chore", "hotfix"}:
            errors.append("種別 must be one of feat/fix/design/docs/test/ci/rfc/chore/hotfix")
        if not re.search(r"(?im)^\s*Refs\s*:\s*\S+", section(body, "参照")):
            errors.append("参照 section must contain a Refs: line")
        for title in ("変更内容", "触ったファイル", "テスト", "検証手順", "参考", "チェックリスト"):
            if not meaningful(section(body, title)):
                errors.append(f"{title} section is empty")
        if not writer(body):
            errors.append("PR body must contain writer: <identity>")
        if not re.search(r"(?m)^\s*[-*]\s*追加/変更したテスト", section(body, "テスト")):
            errors.append("テスト section is missing the test-ID field")
        if not re.search(r"```[\s\S]*?\S[\s\S]*?```", section(body, "検証手順")):
            errors.append("検証手順 must contain a non-empty code block")
        checklist = section(body, "チェックリスト")
        if not re.search(r"\[[xX]\].*300 行以内", checklist):
            errors.append("300 行以内 checklist item is not checked")
        if not re.search(r"\[[xX]\].*(?:秘密情報|secret)", checklist, re.I):
            errors.append("secrets checklist item is not checked")
        if not draft and not meaningful(section(body, "CI")):
            errors.append("CI section is empty for a non-draft PR")

        stat = touched_stat(body)
        additions = sum(int(entry.get("additions", 0) or 0) for entry in files)
        deletions = sum(int(entry.get("deletions", 0) or 0) for entry in files)
        actual_names = {str(entry.get("filename", "")).replace("\\", "/") for entry in files}
        if stat is None:
            errors.append("触ったファイル must contain a complete git diff --stat")
        else:
            file_count, body_additions, body_deletions, listed = stat
            if (file_count, body_additions, body_deletions) != (len(actual_names), additions, deletions):
                errors.append("PR diff --stat does not match GitHub file statistics")
            if actual_names - listed:
                errors.append("触ったファイル stat omits: " + ", ".join(sorted(actual_names - listed)))
        if TRACE_PATH in {path.casefold() for path in actual_names}:
            if not has_claude_trace_approval(args.pr, metadata, comments):
                errors.append("trace.md changes require a current-head Claude signed approval")
        if additions + deletions > LINE_LIMIT:
            if "分割理由" not in body and "split reason" not in body.lower():
                errors.append("changed lines exceed 300 without a split reason")
            if not has_line_limit_exception(args.pr, metadata, comments):
                errors.append("changed lines exceed 300 without an owner-approved line-limit exception")
        errors.extend("forbidden addition: " + item for item in added_forbidden(files))

        implementation_index: int | None = None
        test_index: int | None = None
        for index, commit in enumerate(commits):
            if not isinstance(commit, dict):
                continue
            # Mainline merges are synchronization commits, not authored work;
            # do not make their generated subject fail the commit convention.
            if len(commit.get("parents", [])) > 1:
                continue
            commit_files = commit.get("files")
            if not commit_files and commit.get("sha"):
                try:
                    commit_files = gh_api(f"repos/{args.repo}/commits/{commit['sha']}").get("files", [])
                except LintError:
                    commit_files = []
            paths = [str(entry.get("filename", entry)) for entry in (commit_files or [])]
            if test_index is None and any(is_test_path(path) for path in paths):
                test_index = index
            if implementation_index is None and any(is_implementation(path) for path in paths):
                implementation_index = index
            message = str(commit.get("commit", {}).get("message", commit.get("message", "")))
            first_line = message.splitlines()[0] if message else ""
            if first_line and not re.fullmatch(r"(?:feat|fix|design|docs|test|ci|chore|rfc|hotfix)\([^)]+\):\s+.+", first_line):
                errors.append(f"invalid commit subject: {first_line}")
            if message and not re.search(r"(?im)^Refs:\s*\S+", message):
                errors.append(f"commit is missing Refs: {first_line}")
        if test_index is not None and implementation_index is not None and test_index > implementation_index:
            errors.append("test commit must precede implementation commit")
    except LintError as exc:
        errors.append(str(exc))
    if errors:
        print("pr-lint: FAIL")
        for error in dict.fromkeys(errors):
            print(f"- {error}")
        return 1
    print("pr-lint: PASS")
    return 0


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser(description=__doc__)
    result.add_argument("pr", type=int)
    result.add_argument("--repo", default=os.environ.get("GITHUB_REPOSITORY", "RetryYN/kimizukann"))
    result.add_argument("--pr-file")
    result.add_argument("--files-file")
    result.add_argument("--commits-file")
    result.add_argument("--comments-file")
    return result


if __name__ == "__main__":
    sys.exit(run(parser().parse_args()))
