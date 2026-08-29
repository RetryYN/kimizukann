#!/usr/bin/env python3
"""Ensure the verification suite rejects four representative mutations."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

from mutate import MUTATIONS, SOURCE


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
    if failures:
        print("gate-selftest: FAIL")
        for failure in failures:
            print(f"- {failure}")
        return 1
    print("gate-selftest: PASS (4 mutations rejected)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
