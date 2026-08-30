#!/usr/bin/env python3
"""Ensure the verification suite rejects four representative mutations."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

from mutate import MUTATIONS, SOURCE
from review_gate import patch_id_equivalent


ROOT = SOURCE.parents[3]
CLI = ["cargo", "run", "--manifest-path", "crates/sim-cli/Cargo.toml", "--", "verify", "--suite", "all"]


def run_verify() -> subprocess.CompletedProcess[str]:
    return subprocess.run(CLI, cwd=ROOT, text=True, capture_output=True, check=False)


def report(result: subprocess.CompletedProcess[str]) -> dict[str, object] | None:
    """Extract the CLI JSON even when cargo writes build logs around it."""
    for line in reversed(result.stdout.splitlines()):
        try:
            value = json.loads(line)
        except json.JSONDecodeError:
            continue
        if isinstance(value, dict) and "status" in value:
            return value
    return None


def patch_id_cases() -> list[str]:
    """Check that a main-only merge preserves, but a code edit invalidates, a ticket."""
    failures: list[str] = []
    try:
        with tempfile.TemporaryDirectory(prefix="gate-patch-id-") as raw:
            repo = Path(raw)

            def git(*arguments: str) -> str:
                result = subprocess.run(
                    ["git", *arguments],
                    cwd=repo,
                    text=True,
                    capture_output=True,
                    check=False,
                )
                if result.returncode:
                    detail = result.stderr.strip() or result.stdout.strip() or "git failed"
                    raise RuntimeError(detail)
                return result.stdout.strip()

            git("init", "-b", "main")
            git("config", "user.name", "gate-selftest")
            git("config", "user.email", "gate-selftest@example.invalid")
            (repo / "base.txt").write_text("base\n", encoding="utf-8")
            git("add", ".")
            git("commit", "-m", "base")
            git("checkout", "-b", "feature")
            (repo / "feature.txt").write_text("feature\n", encoding="utf-8")
            git("add", ".")
            git("commit", "-m", "feature")
            ticket_sha = git("rev-parse", "HEAD")
            git("checkout", "main")
            (repo / "main.txt").write_text("main\n", encoding="utf-8")
            git("add", ".")
            git("commit", "-m", "unrelated main change")
            main_sha = git("rev-parse", "HEAD")
            git("update-ref", "refs/remotes/origin/main", main_sha)
            git("checkout", "feature")
            git("merge", "--no-ff", "main", "-m", "merge main")
            merged_sha = git("rev-parse", "HEAD")
            if not patch_id_equivalent(ticket_sha, merged_sha, cwd=repo):
                failures.append("patch-id merge case: ticket was not preserved")

            (repo / "feature.txt").write_text("feature changed\n", encoding="utf-8")
            git("add", ".")
            git("commit", "-m", "change PR content")
            changed_sha = git("rev-parse", "HEAD")
            if patch_id_equivalent(ticket_sha, changed_sha, cwd=repo):
                failures.append("patch-id edit case: changed ticket was accepted")
    except (OSError, RuntimeError) as exc:
        failures.append(f"patch-id cases: {exc}")
    return failures


def main() -> int:
    baseline = run_verify()
    baseline_report = report(baseline)
    if baseline.returncode != 0 or baseline_report is None or baseline_report.get("status") != "pass":
        print("gate-selftest: baseline verify failed", file=sys.stderr)
        print(baseline.stdout, baseline.stderr, file=sys.stderr)
        return 1
    failures: list[str] = []
    pristine = SOURCE.read_text(encoding="utf-8")
    for name, (before, after) in MUTATIONS.items():
        if pristine.count(before) != 1:
            failures.append(f"{name}: mutation target count is not one")
            continue
        try:
            SOURCE.write_text(pristine.replace(before, after), encoding="utf-8")
            result = run_verify()
            mutation_report = report(result)
            if result.returncode == 0:
                failures.append(f"{name}: verify unexpectedly passed")
            elif mutation_report is None:
                failures.append(f"{name}: verify emitted no failure JSON (likely compile failure)")
            elif mutation_report.get("status") != "fail":
                failures.append(f"{name}: verify JSON status is not fail")
        finally:
            SOURCE.write_text(pristine, encoding="utf-8")
    failures.extend(patch_id_cases())
    if failures:
        print("gate-selftest: FAIL")
        for failure in failures:
            print(f"- {failure}")
        return 1
    print("gate-selftest: PASS (4 mutations rejected; patch-id inheritance checked)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
