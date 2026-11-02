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
- Prefer verifiable acceptance criteria and explicit failure handling.
