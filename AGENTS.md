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
