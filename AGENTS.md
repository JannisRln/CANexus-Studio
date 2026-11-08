# AI contributor notes

This repository uses Markdown specifications under `docs/requirements/` as the sole source of truth for product requirements. Do not create GitHub Issues for requirements.

## Rules

- Read `docs/requirements/README.md` and relevant domain files before designing or implementing changes.
- Keep requirement IDs stable; document changed/superseded behavior.
- Separate **confirmed decisions** from **proposals**. Do not silently mark Proposed requirements as Accepted.
- All code-generated Doxygen comments must be **in English**.
- Application: Rust core; Slint UI with pbSlintDock; Windows and Linux targets.
- ECU projects must not be reorganized or populated with CANexus runtime files. They require only a single CANexus project configuration file, provisionally `canexus.toml`.
- Resolve project resources relative to that project's configuration file; do not require simultaneous worktree management.
- Keep UI responsive and data acquisition independent of display refresh.
- Do not implement a CAN-only generic event model: the architecture must allow future Ethernet and other transport plugins, while preserving efficient typed CAN hot paths.
- Trace must support chronological/fixed modes, column sorting/filtering, and independent per-view state.
- The unified Signal Explorer supports module-specific details; native plugins and scripts are distinct extension mechanisms.
- Prefer verifiable acceptance criteria and explicit failure handling.

- v0.7 Accepted decisions live in `docs/requirements/17-decisions-v0.7.md`; the canonical domain files are 12–16.
- Python tests live in user-referenced Python files, editable through external editors, and run through GUI/CLI using isolated per-script-session worker processes.
- Third-party Slint dockable panels and a fully configurable command dispatcher are product goals; prototype dynamic UI lifecycle/SDK stability before implementation.
- Do not claim generic flashing recovery, unrestricted vendor diagnostics support, or cross-device precise synchronization without evidence.

## Chat-only implementation workflow (mandatory)

- Before any implementation, read docs/development/README.md, STATUS.md, BACKLOG.md, DECISIONS.md and relevant requirements; inspect the current main commit and code.
- The repository is the only durable progress record across chats; never rely on previous chat memory as the implementation source of truth.
- Choose a Ready task with satisfied dependencies; independent tasks may run in parallel. Check active branches, open PRs and task ownership before starting to avoid duplicate work.
- Use task branches and pull requests for substantial, risky or parallel changes; direct pushes to main are allowed for small isolated changes. Do not use GitHub Issues for task tracking.
- Mark tasks Done only after integration into main and recorded verification. Coordinate STATUS.md and BACKLOG.md updates to avoid overwriting parallel work; never force-push main.
- Every delivery must report commit SHA, actual tests/CI results, failures, remaining tasks and next action. Do not claim CI passed unless a run confirms it.
- Add tests and maintain Windows/Linux CI; Rust checks become active after Cargo.toml exists.
- CAN Signal Generator and XCP-driven generation are not part of v1; retain extension points only.
