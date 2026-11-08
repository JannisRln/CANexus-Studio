# Chat-driven implementation

The repository, not chat history, is the durable source of truth. No GitHub Issues or pull requests for routine development. Commit directly to main.

## Every new chat

1. Read AGENTS.md, docs/development/STATUS.md, BACKLOG.md, DECISIONS.md, and relevant docs/requirements files.
2. Inspect current code and latest main commit; do not assume previous chat statements are current.
3. Pick the earliest Ready task with satisfied dependencies. Mark In Progress before work.
4. Implement a small verifiable increment, add tests, run available checks.
5. Commit changes and update STATUS.md and BACKLOG.md as part of the delivery.
6. Report commit SHA, changed files, checks actually run, known failures and next task.

## Task lifecycle

Planned -> Ready -> In Progress -> Done; Blocked when dependencies or decisions prevent progress. Done requires committed work and documented acceptance evidence. A failed or unrun test must never be reported as passed. Reopen regressed tasks.

## Source ownership

- STATUS.md: current reality, active task, latest verification, blockers and next action.
- BACKLOG.md: stable DEV task IDs, dependencies, priorities and acceptance criteria.
- DECISIONS.md: dated implementation decisions and supersessions.
- docs/requirements/: product scope and architecture decisions; task Done does not automatically mean requirement Verified.
- Git history and CI runs: implementation evidence.

## Definition of Done

Appropriate tests, documented errors/limitations, relevant platform builds where available, bounded resource behavior when applicable, committed code and updated project state. Never mark requirements Verified without evidence.

## CI

Run project-state validation on Windows and Linux now. Rust fmt, Clippy, build and test are conditional on Cargo.toml until the workspace is created. Native GUI and hardware verification need separate evidence.
