# Implementation decision log

Product architecture ADRs remain in docs/requirements. Append superseding decisions rather than silently editing historical decisions.

| ID | Date | Status | Decision | Rationale |
|---|---|---|---|---|
| DEV-DEC-001 | 2026-11-08 | Accepted | ChatGPT is the implementation interface; repository files, commits and CI are persistent state. | Reliable handoff between chats. |
| DEV-DEC-002 | 2026-11-08 | Superseded by DEV-DEC-007 | Commit directly to main; do not create PRs unless requested. | Original workflow preference. |
| DEV-DEC-003 | 2026-11-08 | Accepted | Track implementation tasks in BACKLOG.md, not GitHub Issues. | Versioned planning next to code. |
| DEV-DEC-004 | 2026-11-08 | Accepted | Start with Rust, virtual CAN and a Slint Trace vertical slice. | Minimize integration risk. |
| DEV-DEC-005 | 2026-11-08 | Accepted | CAN Signal Generator and XCP-driven generation are post-v1. | Explicit release scope. |
| DEV-DEC-006 | 2026-11-08 | Accepted | CI on Windows and Linux validates docs now and Rust once a workspace exists. | Progressive verification. |

| DEV-DEC-007 | 2026-11-08 | Accepted | Use PRs and parallel branches when useful; allow direct pushes to main for small isolated changes. | Parallel development without unnecessary PR overhead; supersedes DEV-DEC-002. |

## Pending

- Precise v1 release feature matrix.
- Rust crate topology and version pinning (DEV-002).
- Slint/pbSlintDock feasibility (DEV-003).
- Hardware adapter and recording format for later milestones.
