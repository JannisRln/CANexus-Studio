#!/usr/bin/env python3
"""Validate repository-backed chat implementation state."""
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[1]
REQUIRED = ["AGENTS.md", "LICENSE", "docs/requirements/README.md", "docs/development/README.md", "docs/development/STATUS.md", "docs/development/BACKLOG.md", "docs/development/DECISIONS.md"]
VALID = {"Planned", "Ready", "In Progress", "Blocked", "Done"}


def main():
    errors = [f"Missing {p}" for p in REQUIRED if not (ROOT / p).is_file()]
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    backlog = (ROOT / "docs/development/BACKLOG.md").read_text(encoding="utf-8")
    status = (ROOT / "docs/development/STATUS.md").read_text(encoding="utf-8")
    rows = [line for line in backlog.splitlines() if re.match(r"^\| DEV-\d{3} \|", line)]
    ids = []
    for row in rows:
        fields = [v.strip() for v in row.strip().strip("|").split("|")]
        if len(fields) != 6:
            errors.append(f"Expected 6 task columns: {row}")
            continue
        ids.append(fields[0])
        if fields[3] not in VALID:
            errors.append(f"Invalid status for {fields[0]}: {fields[3]}")
    if not ids or len(ids) != len(set(ids)):
        errors.append("Task IDs missing or duplicated")
    for task in re.findall(r"DEV-\d{3}", status):
        if task not in ids:
            errors.append(f"Unknown status task: {task}")
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    print(f"Project state valid: {len(ids)} tracked tasks")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
