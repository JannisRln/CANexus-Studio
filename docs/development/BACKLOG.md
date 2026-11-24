# Development backlog

Canonical implementation tasks. Statuses: Planned, Ready, In Progress, Blocked, Done. Every Done task needs committed work and verification evidence in STATUS.md. Product requirement statuses remain independent.

| ID | Milestone | Priority | Status | Dependencies | Acceptance criteria |
|---|---|---|---|---|---|
| DEV-001 | M0 | P0 | Done | None | Workflow docs and CI committed; first CI run verified |
| DEV-002 | M0 | P0 | Done | DEV-001 scaffolding | Rust workspace, pinned toolchain, formatting, Clippy, build and tests in CI |
| DEV-003 | M0 | P0 | Done | DEV-002 | Single-window Slint/pbSlintDock feasibility spike: three panels, docking/tab/splitter callbacks, layout snapshot and engine-level float tests; Windows/Linux Rust CI evidence. Native floating windows remain DEV-022. |
| DEV-004 | M1 | P0 | Done | DEV-002 | Tested typed CAN/CAN FD frame and timestamp model |
| DEV-005 | M1 | P0 | Done | DEV-004 | Deterministic virtual CAN adapter and fixtures |
| DEV-006 | M1 | P0 | Done | DEV-005 | Bounded event ingestion and shared batches, overload/shutdown tests |
| DEV-007 | M1 | P0 | Done | DEV-006 | Bounded trace history, query API and loss provenance tests |
| DEV-008 | M1 | P0 | Planned | DEV-007, DEV-023, DEV-025, DEV-028 | egui Trace from virtual CAN with bounded visible rows, independent chronological/Fixed filter and sort state, keyboard and loss evidence |
| DEV-009 | M1 | P0 | Planned | DEV-008 | End-to-end smoke test and repeatable benchmark harness |
| DEV-010 | M2 | P1 | Planned | DEV-009, DEV-029 | First hardware adapter behind HAL, documented platform support |
| DEV-011 | M2 | P1 | Planned | DEV-009 | DBC import and signal decoding with fixtures |
| DEV-012 | M2 | P1 | Planned | DEV-010, DEV-017 | Authorized manual transmission through shared scheduler |
| DEV-013 | M2 | P1 | Planned | DEV-009 | Recording format decision and failure-tested recorder prototype |
| DEV-014 | Later | P2 | Planned | DEV-011, DEV-012 | CAN Signal Generator, not v1 |
| DEV-015 | Later | P2 | Planned | XCP prerequisites | XCP-driven signal generation, not v1 |
| DEV-016 | M0 | P1 | Done | DEV-002 | Optional rustfmt pre-commit hook, safe handling of staged files, documented installation and hook tests in CI |
| DEV-017 | M2 | P1 | Planned | DEV-007, DEV-028, command permission decision | Versioned app-service/capability contracts and unified authorization; virtual fixture and stale-handle tests |
| DEV-018 | M2 | P1 | Planned | DEV-017 | Opt-in local stdio MCP read-only project/trace/health tools; bounded queries and independent-client tests on Windows/Linux |
| DEV-019 | Later | P1 | Planned | DEV-017; project schema decision | AI/MCP project create/validate/update with preview, atomic single-file persistence and separate-worktree tests |
| DEV-020 | Later | P1 | Planned | DEV-017, DEV-025; workspace/dashboard model | AI/MCP dashboard/widget/signal-binding CRUD, preview and local/shared persistence with toolkit-neutral panel lifecycle tests |
| DEV-021 | Later | P2 | Planned | DEV-018, DEV-019, DEV-020 | Optional in-app assistant sharing guarded service APIs; provider-choice, privacy, permissions and offline-mode tests |
| DEV-022 | M1 | P1 | Planned | DEV-024 | Isolated egui/Craft native float/redock proof, stable panel state, actual Windows/Linux interaction, scale/monitor recovery and explicit Wayland capability report. Maintain an independent example. |
| DEV-023 | M1 | P1 | Done | DEV-006, DEV-007 | Prototype bounded global pinned shared-batch byte accounting and deterministic admission/overload/shutdown tests; preserve typed no-copy delivery and nonblocking capture. |
| DEV-024 | M1 | P0 | Ready | DEV-003 baseline, ADR-012 | Isolated egui/Craft proof on the pinned toolchain with pinned single egui source, dependency/license audit, native launch and representative workspace/Trace/Plot sample on Windows and Linux |
| DEV-025 | M1 | P0 | Planned | DEV-024 | Native shell and component showcase, verified single-window parity, keyboard/accessibility evidence, locked builds and synchronized GUI/CI cutover |
| DEV-026 | Later | P1 | Planned | DEV-022, DEV-025 | Independently built dockable panel proof, versioned schema/manifest, state restore, missing/incompatible provider recovery and authorization |
| DEV-027 | Later | P2 | Planned | DEV-025 | Standalone contributor and build guide, license/asset inventory, Craft upstream candidates and documented affiliation decision path |
| DEV-028 | M1 | P0 | Done | DEV-006, DEV-007, DEV-023 | Runtime session host with scoped generations, timed sparse flush, ordered subscribe/detach, bounded stop/drain and out-of-band health. Reject late results and expose repeated overflow in deterministic tests. Coordinate byte admission with DEV-023. |
| DEV-029 | M2 | P1 | Planned | DEV-028, project schema decision | Portable loader, resource revision and local channel mappings. Verify relocation, missing resources and distinct worktrees without writing runtime state into ECU projects. |
| DEV-030 | M2 | P1 | Planned | DEV-013, DEV-023, DEV-028 | Verify all four REQ-LOG-006 combinations of optional durable recording and optional Trace spill, including RAM-only capture and filtered logging. Exercise two-session budgets, concurrent cache I/O, rollover, process interruption, full volume and independent reading of at least 8 GiB. |
| DEV-031 | M2 | P1 | Planned | DEV-030 | Indexed offline pages and virtual replay with seek, pause and speed. Verify stale-index rebuild, cancellation, bounded memory and stable position at segment boundaries. |
| DEV-032 | M2 | P1 | Planned | DEV-008, DEV-010, DEV-011, DEV-031 | Integrate Signal Explorer, linked Plot/Inspector, differences, channel statistics and persisted layout. Verify the complete live-to-offline workflow on Windows and Linux against the analysis baseline. |

Start with the earliest Ready task whose dependencies are satisfied. Update this backlog and STATUS.md after every delivery. M0/M1 are an initial vertical slice, not the entire v1 feature matrix. Optional AI/MCP backlog items DEV-018..021 are proposed planning placeholders; they are not Ready until the corresponding requirements, security and schema decisions are accepted. Do not silently remove existing product requirements.
