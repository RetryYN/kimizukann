#!/usr/bin/env python3
"""Ensure the verification suite rejects four representative mutations."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

from mutate import MUTATIONS, SOURCE


ROOT = SOURCE.parents[3]
CLI = ["cargo", "run", "--manifest-path", "crates/sim-cli/Cargo.toml", "--", "verify", "--suite", "all"]


def run_verify() -> subprocess.CompletedProcess[str]:
    return subprocess.run(CLI, cwd=ROOT, text=True, capture_output=True, check=False)


def main() -> int:
    baseline = run_verify()
    if baseline.returncode != 0:
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
            if result.returncode == 0:
                failures.append(f"{name}: verify unexpectedly passed")
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
