#!/usr/bin/env python3
"""Apply one deliberate source mutation for the gate self-test."""

from __future__ import annotations

import argparse
from pathlib import Path


SOURCE = Path(__file__).resolve().parents[1] / "crates" / "sim-core" / "src" / "lib.rs"
MUTATIONS = {
    "coefficient": (
        "fixed::split_output_with_rule(amount, &rule, 300_000)",
        "fixed::split_output_with_rule(amount, &rule, 310_000)",
    ),
    "remainder": (
        "let remainder = input - primary - secondary;",
        "let remainder = 0;",
    ),
    "phase-order": (
        "        self.intake()?;\n        self.maintenance()?;\n        self.starvation_and_death()?;\n        self.reproduction()?;",
        "        self.reproduction()?;\n        self.maintenance()?;\n        self.starvation_and_death()?;\n        self.intake()?;",
    ),
    "hash": (
        "        for stream in &self.rng {",
        "        for stream in std::iter::empty::<&Xoshiro256StarStar>() {",
    ),
}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("name", choices=sorted(MUTATIONS))
    parser.add_argument("action", choices=["apply", "restore"], nargs="?", default="apply")
    args = parser.parse_args()
    before, after = MUTATIONS[args.name]
    text = SOURCE.read_text(encoding="utf-8")
    if args.action == "restore":
        # Restore is intentionally idempotent: the caller keeps the pristine
        # source and invokes this after the mutation run.
        return 0
    if text.count(before) != 1:
        raise SystemExit(f"mutation target count is not one: {args.name}")
    SOURCE.write_text(text.replace(before, after), encoding="utf-8")
    return 0


if __name__ == "__main__":
    main()
