# Chat-driven implementation

The repository, not chat history, is the durable source of truth. No GitHub Issues for implementation tracking. Use short-lived branches and pull requests when parallelism, integration risk or review benefits justify them; direct pushes to main remain allowed for small independent changes.

## Every new chat

1. Read AGENTS.md, docs/development/STATUS.md, BACKLOG.md, DECISIONS.md, and relevant docs/requirements files.
2. Inspect current code and latest main commit; do not assume previous chat statements are current.
3. Pick a Ready task with satisfied dependencies, considering independent parallel tasks. Check active task ownership and branches in STATUS.md; mark In Progress before work.
4. Implement a small verifiable increment, add tests, run available checks.
5. Commit changes on a task branch or main, as appropriate. Update STATUS.md and BACKLOG.md when integrating into main; avoid concurrent conflicting edits to shared status files.
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

## Rust formatting pre-commit hook

Git does not activate version-controlled hooks automatically. After cloning, enable this repository's hook once per clone from the repository root:

```sh
git config --local core.hooksPath .githooks
```

This setting also applies to linked Git worktrees. The hook requires Git with POSIX shell support (provided by Git for Windows), Rust, and the pinned rustfmt component from `rust-toolchain.toml`.

Before each commit, the hook runs `cargo fmt --all` followed by the same formatting check as CI. If rustfmt modifies files or there are unstaged Rust edits (including partially staged changes), the hook stops the commit. Review and stage the intended files with `git add`, then commit again. The hook **never stages files automatically**. CI continues to enforce formatting even when hooks are disabled or bypassed.

To run the hook integration tests manually on Linux: `python -m unittest discover -s tests -p 'test_git_hooks.py' -v`.

## Branching, PRs and parallel work

- **Direct main:** documentation, CI fixes and small isolated changes with low merge risk. Verify applicable checks; do not bypass failing CI silently.
- **Task branch + PR:** larger features, risky refactors, shared interfaces, changes needing CI validation, or work parallel to another task. Branch naming: `feat/DEV-###-short-name`, `fix/DEV-###-short-name` or `chore/DEV-###-short-name`.
- Multiple independent tasks may be In Progress simultaneously when dependencies are satisfied and files/interfaces do not overlap. Record task ID, branch, owner/session, and integration dependencies in STATUS.md.
- Agree on interface contracts before parallel implementation; use small PRs and merge frequently. Rebase or update against main before merging; resolve conflicts and confirm CI.
- A PR task is not Done merely because a PR exists. Mark Done after merge to main and verification. Do not mark unmerged work as implemented on main.
- Shared BACKLOG.md/STATUS.md updates should be reconciled on main after merges, with no lost concurrent changes. Never force-push main.
- New chats must inspect open PRs, active branches and latest main before starting work; do not duplicate active tasks.
