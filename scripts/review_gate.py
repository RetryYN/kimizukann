#!/usr/bin/env python3
"""Validate signed helix-review attestations for a pull request.

The checker is deliberately fail-closed.  It only treats a review as valid when
the ticket is for the current head, has a valid HMAC, and its evidence matches
the verification report (or is ``none`` for documentation-only changes).
"""

from __future__ import annotations

import argparse
import fnmatch
import hashlib
import hmac
import json
import os
import re
import subprocess
import sys
from pathlib import Path
from typing import Any, Iterable


HEX64 = re.compile(r"^[0-9a-fA-F]{64}$")
MARKER = re.compile(r"^\s*helix-review:\s*v1\s*$", re.I)
FIELD = re.compile(r"^\s*(reviewer|pr|sha|verdict|checklist|evidence|sig):\s*(.*?)\s*$", re.I)


class GateError(RuntimeError):
    pass


def _decode_many(raw: str) -> list[Any]:
    """Decode JSON arrays concatenated by ``gh api --paginate``."""
    raw = raw.strip()
    if not raw:
        return []
    decoder = json.JSONDecoder()
    out: list[Any] = []
    pos = 0
    while pos < len(raw):
        while pos < len(raw) and raw[pos].isspace():
            pos += 1
        if pos >= len(raw):
            break
        value, end = decoder.raw_decode(raw, pos)
        out.append(value)
        pos = end
    return out


def gh_api(endpoint: str, *, paginate: bool = False) -> Any:
    command = ["gh", "api"]
    if paginate:
        command.append("--paginate")
    command.append(endpoint)
    result = subprocess.run(command, text=True, capture_output=True, check=False)
    if result.returncode:
        detail = result.stderr.strip() or result.stdout.strip() or "gh api failed"
        raise GateError(detail)
    if not paginate:
        try:
            return json.loads(result.stdout)
        except json.JSONDecodeError as exc:
            raise GateError(f"invalid gh api JSON: {exc}") from exc
    values = _decode_many(result.stdout)
    if len(values) == 1:
        return values[0]
    flattened: list[Any] = []
    for value in values:
        flattened.extend(value if isinstance(value, list) else [value])
    return flattened


def read_json(path: str | None) -> Any | None:
    if not path:
        return None
    try:
        return json.loads(Path(path).read_text(encoding="utf-8"))
    except OSError as exc:
        raise GateError(f"cannot read {path}: {exc}") from exc
    except json.JSONDecodeError as exc:
        raise GateError(f"invalid JSON in {path}: {exc}") from exc


def canonical_identity(value: str) -> str:
    value = value.strip().strip("`").lstrip("@").lower()
    aliases = {
        "cursor-kimi": "kimi",
        "cursor-grok": "grok",
        "cursor-glm": "composer",
        "cursor-composer": "composer",
        "cursor-glm2": "glm2",
        "cursor-gemini": "gemini",
        "retryyn": "owner",
        "owner": "owner",
    }
    return aliases.get(value, value)


def parse_writer(body: str) -> str | None:
    match = re.search(r"(?im)^\s*(?:[-*]\s*)?writer\s*:\s*`?([A-Za-z0-9_-]+)`?\s*$", body)
    return canonical_identity(match.group(1)) if match else None


def parse_tickets(comments: Iterable[dict[str, Any]]) -> list[dict[str, str]]:
    tickets: list[dict[str, str]] = []
    for comment in comments:
        body = str(comment.get("body", ""))
        lines = body.splitlines()
        for index, line in enumerate(lines):
            if not MARKER.match(line):
                continue
            ticket: dict[str, str] = {"_created": str(comment.get("created_at", ""))}
            for candidate in lines[index + 1 : index + 14]:
                if MARKER.match(candidate):
                    break
                match = FIELD.match(candidate)
                if match:
                    ticket[match.group(1).lower()] = match.group(2).strip()
            tickets.append(ticket)
    return tickets


def checklist_hash(value: str) -> bool:
    return bool(HEX64.fullmatch(value.strip()))


def state_hash_values(value: Any) -> list[str]:
    found: list[str] = []
    if isinstance(value, dict):
        for key, child in value.items():
            if key in {"state_hash", "stateHash"}:
                if isinstance(child, str):
                    found.append(child)
                elif isinstance(child, list):
                    found.extend(str(item) for item in child if isinstance(item, (str, int)))
            found.extend(state_hash_values(child))
    elif isinstance(value, list):
        for child in value:
            found.extend(state_hash_values(child))
    return found


def report_digest(path: str | None) -> str | None:
    if not path:
        return None
    if not Path(path).is_file():
        return None
    report = read_json(path)
    if report is None:
        return None
    hashes = sorted(set(state_hash_values(report)))
    if not hashes:
        return None
    canonical = "\n".join(hashes).encode("utf-8")
    return hashlib.sha256(canonical).hexdigest()


def normalize_rule(raw: dict[str, Any]) -> dict[str, Any]:
    paths = raw.get("paths", raw.get("glob", []))
    if isinstance(paths, str):
        paths = [paths]
    writers = raw.get("writers", raw.get("writer", []))
    if isinstance(writers, str):
        writers = [writers]
    return {
        "paths": [str(path).replace("\\", "/") for path in paths],
        "all": [canonical_identity(str(v)) for v in raw.get("allOf", raw.get("all", []))],
        "any": [canonical_identity(str(v)) for v in raw.get("anyOf", raw.get("any", []))],
        "writers": [canonical_identity(str(v)) for v in writers],
        "one_non_writer": bool(raw.get("oneNonWriter", raw.get("one_non_writer", False))),
        "exclusive": bool(raw.get("exclusive", False)),
    }


def load_rules(path: str) -> list[dict[str, Any]]:
    data = read_json(path)
    if not isinstance(data, dict):
        raise GateError("review owners must be a JSON object")
    raw_rules = data.get("rules", [])
    if isinstance(raw_rules, dict):
        raw_rules = [dict(spec, paths=[glob]) for glob, spec in raw_rules.items()]
    if not isinstance(raw_rules, list):
        raise GateError("review owners rules must be an array")
    return [normalize_rule(rule) for rule in raw_rules if isinstance(rule, dict)]


def required_reviewers(files: Iterable[str], rules: list[dict[str, Any]]) -> tuple[set[str], list[set[str]], bool]:
    normalized_files = [str(path).replace("\\", "/") for path in files]
    all_required: set[str] = set()
    any_required: list[set[str]] = []
    one_non_writer = False
    for path in normalized_files:
        path_rules = [
            rule for rule in rules if any(fnmatch.fnmatchcase(path, pattern) for pattern in rule["paths"])
        ]
        # An exclusive rule applies only to this matching path.  Other changed
        # paths still contribute their own requirements (e.g. golden + crates).
        for rule in [rule for rule in path_rules if rule["exclusive"]] or path_rules:
            all_required.update(rule["all"])
            if rule["any"]:
                any_required.append(set(rule["any"]))
            one_non_writer = one_non_writer or rule["one_non_writer"]
    return all_required, any_required, one_non_writer


def verify_ticket(ticket: dict[str, str], pr: int, head_sha: str, secret: bytes) -> tuple[bool, str]:
    required = {"reviewer", "pr", "sha", "verdict", "checklist", "evidence", "sig"}
    missing = sorted(required - ticket.keys())
    if missing:
        return False, f"missing fields: {', '.join(missing)}"
    if ticket["pr"] != str(pr):
        return False, "PR number mismatch"
    if not re.fullmatch(r"[0-9a-fA-F]{40}", ticket["sha"]):
        return False, "ticket sha must be exactly 40 hexadecimal characters"
    if ticket["sha"].lower() != head_sha.lower():
        return False, "stale head sha"
    verdict = ticket["verdict"].lower()
    if verdict not in {"approve", "request-changes"}:
        return False, "invalid verdict"
    if not checklist_hash(ticket["checklist"]):
        return False, "checklist is not a sha256"
    if ticket["evidence"].lower() != "none" and not checklist_hash(ticket["evidence"]):
        return False, "evidence is not a sha256 or none"
    payload = "|".join(
        [ticket["reviewer"], ticket["pr"], ticket["sha"], verdict, ticket["checklist"], ticket["evidence"]]
    ).encode("utf-8")
    expected = hmac.new(secret, payload, hashlib.sha256).hexdigest()
    if not hmac.compare_digest(expected, ticket["sig"].strip().lower()):
        return False, "invalid signature"
    return True, "ok"


def docs_only(files: Iterable[str]) -> bool:
    paths = [str(path).replace("\\", "/") for path in files]
    return bool(paths) and all(path.startswith("docs/") or path.startswith("README") for path in paths)


def load_pr_data(pr: int, args: argparse.Namespace) -> tuple[dict[str, Any], list[dict[str, Any]], list[dict[str, Any]]]:
    metadata = read_json(args.pr_file)
    if metadata is None:
        metadata = gh_api(f"repos/{args.repo}/pulls/{pr}")
    files = read_json(args.files_file)
    if files is None:
        files = gh_api(f"repos/{args.repo}/pulls/{pr}/files?per_page=100", paginate=True)
    comments = read_json(args.comments_file)
    if comments is None:
        issue_comments = gh_api(f"repos/{args.repo}/issues/{pr}/comments?per_page=100", paginate=True)
        review_comments = gh_api(f"repos/{args.repo}/pulls/{pr}/comments?per_page=100", paginate=True)
        comments = list(issue_comments or []) + list(review_comments or [])
    if not isinstance(metadata, dict) or not isinstance(files, list) or not isinstance(comments, list):
        raise GateError("PR metadata, files, or comments has an unexpected shape")
    return metadata, files, comments


def run(args: argparse.Namespace) -> int:
    errors: list[str] = []
    try:
        metadata, file_entries, comments = load_pr_data(args.pr, args)
        body = str(metadata.get("body") or "")
        writer = parse_writer(body)
        if not writer:
            errors.append("PR body has no writer: field")
        head_sha = args.head_sha or str(metadata.get("head", {}).get("sha", ""))
        if not head_sha:
            errors.append("current head sha is unavailable")
        files = [str(entry.get("filename", "")) for entry in file_entries if isinstance(entry, dict)]
        if not files:
            errors.append("changed file list is empty")
        secret_text = os.environ.get("HELIX_ATTEST_SECRET", "").strip()
        if head_sha and not re.fullmatch(r"[0-9a-fA-F]{40}", head_sha):
            errors.append("head sha must be exactly 40 hexadecimal characters")
        if not secret_text:
            errors.append("HELIX_ATTEST_SECRET is not configured")
        owners = load_rules(args.owners)
        all_required, any_required, one_non_writer = required_reviewers(files, owners)
        digest = report_digest(args.report or os.environ.get("VERIFY_REPORT") or "report.json")
        needs_evidence = not docs_only(files)
        if needs_evidence and digest is None:
            errors.append("verify report has no state_hash digest")
        valid: list[dict[str, str]] = []
        if secret_text and head_sha:
            for ticket in parse_tickets(comments):
                ok, reason = verify_ticket(ticket, args.pr, head_sha, secret_text.encode("utf-8"))
                if not ok:
                    continue
                if needs_evidence and ticket.get("evidence", "").lower() != digest:
                    continue
                if not needs_evidence and ticket.get("evidence", "").lower() != "none":
                    continue
                ticket["_identity"] = canonical_identity(ticket["reviewer"])
                valid.append(ticket)
        if writer:
            for ticket in valid:
                if ticket["_identity"] == writer:
                    errors.append("writer's review ticket is invalid")
        latest: dict[str, dict[str, str]] = {}
        for ticket in valid:
            latest[ticket["_identity"]] = ticket
        approvals = {identity for identity, ticket in latest.items() if ticket["verdict"].lower() == "approve"}
        for reviewer in sorted(all_required):
            if reviewer not in approvals:
                errors.append(f"required reviewer missing: {reviewer}")
        for choices in any_required:
            if not (choices & approvals):
                errors.append("no approval satisfies one-of: " + ", ".join(sorted(choices)))
        if one_non_writer and not any(identity in approvals and identity != writer for identity in approvals):
            errors.append("a non-writer approval is required")
        requests = sum(1 for ticket in valid if ticket["verdict"].lower() == "request-changes")
        if requests >= 3:
            decision = re.search(r"(?im)^\s*helix-decision\s*:\s*(merge|close)\s*$", "\n".join(str(c.get("body", "")) for c in comments))
            if not decision:
                errors.append("three request-changes require helix-decision: merge|close")
            elif decision.group(1).lower() == "close":
                errors.append("PR is explicitly closed by helix-decision")
    except GateError as exc:
        errors.append(str(exc))
    if errors:
        print("review-gate: FAIL")
        for error in dict.fromkeys(errors):
            print(f"- {error}")
        return 1
    print("review-gate: PASS")
    return 0


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser(description=__doc__)
    result.add_argument("pr", type=int, help="pull request number")
    result.add_argument("--repo", default=os.environ.get("GITHUB_REPOSITORY", "RetryYN/kimizukann"))
    result.add_argument("--owners", default=".github/review-owners.json")
    result.add_argument("--report", help="verify report.json path")
    result.add_argument("--head-sha")
    result.add_argument("--pr-file")
    result.add_argument("--files-file")
    result.add_argument("--comments-file")
    return result


if __name__ == "__main__":
    sys.exit(run(parser().parse_args()))
