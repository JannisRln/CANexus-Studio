# Development backlog

Canonical implementation tasks. Statuses: Planned, Ready, In Progress, Blocked, Done. Every Done task needs committed work and verification evidence in STATUS.md. Product requirement statuses remain independent.

| ID | Milestone | Priority | Status | Dependencies | Acceptance criteria |
|---|---|---|---|---|---|
| DEV-001 | M0 | P0 | Done | None | Workflow docs and CI committed; first CI run verified |
| DEV-002 | M0 | P0 | Done | DEV-001 scaffolding | Rust workspace, pinned toolchain, formatting, Clippy, build and tests in CI |
| DEV-003 | M0 | P0 | Blocked | DEV-002 | Slint/pbSlintDock feasibility: native floating lifecycle, persisted float/re-dock, interactive Windows/Linux proof and documented decision; see DEV-016 |
| DEV-004 | M1 | P0 | Done | DEV-002 | Tested typed CAN/CAN FD frame and timestamp model |
| DEV-005 | M1 | P0 | In Progress | DEV-004 | Deterministic virtual CAN adapter and fixtures |
| DEV-006 | M1 | P0 | Planned | DEV-005 | Bounded event ingestion and shared batches, overload/shutdown tests |
| DEV-007 | M1 | P0 | Planned | DEV-006 | Bounded trace history, query API and loss provenance tests |
| DEV-008 | M1 | P0 | Planned | DEV-003, DEV-007 | Slint Trace panel showing virtual frames with virtualized rendering |
| DEV-009 | M1 | P0 | Planned | DEV-008 | End-to-end smoke test and repeatable benchmark harness |
| DEV-010 | M2 | P1 | Planned | DEV-009 | First hardware adapter behind HAL, documented platform support |
| DEV-011 | M2 | P1 | Planned | DEV-009 | DBC import and signal decoding with fixtures |
| DEV-012 | M2 | P1 | Planned | DEV-010 | Authorized manual transmission through shared scheduler |
| DEV-013 | M2 | P1 | Planned | DEV-009 | Recording format decision and failure-tested recorder prototype |
| DEV-014 | Later | P2 | Planned | DEV-011, DEV-012 | CAN Signal Generator, not v1 |
| DEV-015 | Later | P2 | Planned | XCP prerequisites | XCP-driven signal generation, not v1 |
| DEV-016 | M0 | P0 | Planned | DEV-003 spike | Implement native Slint floating windows and float/re-dock lifecycle; preserve panel state and layout across process restart; verify tabs, splits, close/reopen, float/re-dock and layout reload by interactive Windows/Linux smoke tests; record evidence and feasibility decision |

Start with the earliest Ready task whose dependencies are satisfied. Update this backlog and STATUS.md after every delivery. M0/M1 are an initial vertical slice, not the entire v1 feature matrix. Do not silently remove existing product requirements.
