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

| DEV-DEC-008 | 2026-11-09 | Accepted | DEV-007 retains shared immutable source-local batches with inline-event-byte and batch-slot limits, evicts complete oldest batches, and pages in capture sequence order rather than assuming correlated wall-clock time. | Preserves raw provenance and typed zero-copy fan-out, makes RAM eviction deterministic and keeps ingress loss distinct; global pinned-Arc accounting and multi-clock correlation remain separate follow-ups. |

| DEV-DEC-009 | 2026-11-09 | Accepted | Close DEV-003 as the **single-window Slint/pbSlintDock feasibility spike** when its pinned dependencies compile/test on both Windows/Linux and docking/layout/engine float tests pass; move unimplemented native floating-window lifecycle/OS interaction evidence to DEV-022. DEV-008 depends on the verified spike, not native floating. | This records actual PR #12 evidence without claiming native windows or hiding the gap; unblocks independently testable Trace GUI work. Native floating remains a separate deliverable and may be developed in parallel in an isolated example. |

## Pending

- Precise v1 release feature matrix.
- Rust crate topology and version pinning (DEV-002).
- Native Slint floating-window lifecycle and cross-platform interaction evidence (DEV-022); third-party UI panel/plugin ABI proof remains open.
- Hardware adapter and recording format for later milestones.
