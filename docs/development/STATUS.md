# Implementation status

**Phase:** M1 — DEV-007 integrated; next GUI-dependent vertical slice awaits DEV-003 acceptance
**Active tasks:** DEV-003 native Slint docking validation remains open; DEV-008 remains Planned until DEV-003 acceptance
**Latest verified implementation milestone:** DEV-007 PR #18 merged as `3c53f1ee`; post-merge main CI run 42566143742 passed Windows/Linux
**Last updated:** 2026-11-09
**Branch policy:** Hybrid: PRs for substantial/risky/parallel tasks; direct main pushes allowed for small isolated changes

## Implemented

- Product requirements and architecture documentation.
- MIT license.
- Chat-driven implementation planning and status tracking.
- Minimal Rust workspace (`canexus-core`, `canexus-cli`), pinned toolchain and cross-platform Rust CI.
- DEV-004 typed CAN/CAN-FD frame and timestamp model (PR #11).
- DEV-005 deterministic virtual CAN fixture source with virtual clock (PR #15, merged).
- DEV-006 bounded, transport-neutral event ingestion with shared immutable batches, explicit overload and shutdown behavior (PR #17, merged).
- DEV-007 bounded, source-local RAM Trace history with versioned paginated queries, CAN filters and separate eviction/ingress-loss diagnostics (PR #18, merged).
- DEV-003 Slint/pbSlintDock feasibility spike (PR #12); production trace UI remains future work.

## Not implemented

No production Trace GUI, physical HAL, disk-backed history, real recorder, GUI trace viewer or protocol implementation yet. DEV-005 provides a headless virtual fixture source and DEV-006 provides bounded fan-out. The CLI is only a bootstrap entry point; DEV-003 is a feasibility spike.

## Verification evidence

DEV-002 PR #10 CI completed successfully on Windows and Linux (four jobs): https://github.com/JannisRln/CANexus-Studio/actions/runs/42499919323 . PR #10 was squash-merged to main at `d3ee9e87`. This verifies the PR head; post-merge main CI should be checked separately. No hardware tests were performed.

DEV-004 PR #11 CI succeeded at head `489f620b`, squash-merged as `e9090748`. DEV-003 PR #12 CI succeeded at head `8f42d11d`, squash-merged as `bef8513d`. Post-merge combined main CI has not yet been confirmed.

## DEV-005 integration and DEV-006 current work

- **DEV-005 Done:** PR [#15](https://github.com/JannisRln/CANexus-Studio/pull/15) was squash-merged on 2026-11-08 as `2ef21cbe87f3a03b5db57fc46efcc5cf2ba08a0f`. Final PR-head CI [run 42517487438](https://github.com/JannisRln/CANexus-Studio/actions/runs/42517487438) passed all Windows/Linux jobs at `4486f4bb`.
- **Post-merge main verification:** [run 42519479786](https://github.com/JannisRln/CANexus-Studio/actions/runs/42519479786) for `2ef21cbe` succeeded: Windows/Linux project-state, Rustfmt, Clippy `-D warnings`, workspace build and all-target tests; Linux Git hook tests passed. No local Cargo/rustfmt were executed. No native GUI/hardware tests performed.
- **DEV-006 In Progress:** branch `feat/DEV-006-bounded-batch-ingestion`, PR [#17](https://github.com/JannisRln/CANexus-Studio/pull/17). The initial head failed rustfmt in the Rust jobs; the formatting was corrected and a new CI run is required before any merge. The branch implements generic, nonblocking shared batches, bounded consumer queues, observable drops/failures and shutdown tests. It is **not yet merged** into main.
- **Parallel conflict:** PR [#14](https://github.com/JannisRln/CANexus-Studio/pull/14) is a duplicate DEV-005 implementation with overlapping files and an unmerged PR #13 dependency. Do not merge #14 unchanged now that #15 is integrated; review any distinct tests separately.

## DEV-006 verified integration and DEV-007 active work (2026-11-09)

- **DEV-006 Done:** [PR #17](https://github.com/JannisRln/CANexus-Studio/pull/17) squash-merged as `72dd5f2223b6071ec8c7b6eb570670c66c79660b` after final-head [CI run 42520106070](https://github.com/JannisRln/CANexus-Studio/actions/runs/42520106070) completed successfully on Windows/Linux: Rustfmt, Clippy `-D warnings`, workspace build, all-target tests and project-state validation (Linux Git-hook tests passed).
- **Main verification:** [run 42564374489](https://github.com/JannisRln/CANexus-Studio/actions/runs/42564374489) completed successfully at merge commit `72dd5f22`. Neither manual native GUI nor hardware tests were run; local Cargo and rustfmt are unavailable.
- **DEV-007 In Progress:** branch `feat/DEV-007-trace-history`, [PR #18](https://github.com/JannisRln/CANexus-Studio/pull/18). Implements source-local typed RAM retention, whole-batch FIFO eviction with independent ingress loss provenance, bounded pages and CAN filters with integration tests. Initial PR CI failed rustfmt; the formatter diagnostics were incorporated, and latest PR head must pass all required checks before merge.
- **Scope limit:** Global memory consumed by batches retained by other Arc owners is not bounded by history alone. Separate admission/credit accounting and durable loss notification are still open.
- **Coordination:** Older PR #14 duplicates DEV-005 and conflicts with already integrated work; PR #13 is stale against the integrated Clippy fix. Do not merge these old branches unchanged.

## DEV-007 verified integration (2026-11-09)

- **DEV-007 Done:** [PR #18](https://github.com/JannisRln/CANexus-Studio/pull/18) squash-merged as `3c53f1eec5956fbee79663e00bfc38622ccbd9fa`. Final PR-head [CI run 42565427749](https://github.com/JannisRln/CANexus-Studio/actions/runs/42565427749), head `28978ae881024684ce4cfb5ee9e4cea6763eaf89`, passed all four Windows/Linux jobs after Rustfmt and Clippy corrections.
- **Main verification:** [run 42566143742](https://github.com/JannisRln/CANexus-Studio/actions/runs/42566143742) completed **successfully** on the exact merge commit `3c53f1ee`: Rustfmt, Clippy `-D warnings`, workspace build, all-target tests and project-state checks on Windows and Linux; Linux pre-commit hook tests passed. Windows native interactive GUI and hardware checks were not run; no local Cargo/rustfmt were run.
- **Evidence:** 10 DEV-007 integration tests passed on Linux (PR job logs), alongside existing workspace tests; Windows all-target tests also passed. Includes bounded whole-batch eviction, source ordering guards, cursor invalidation, CAN filters, input-loss versus RAM-eviction accounting, shared-batch fan-out and virtual CAN replay.
- **Scope boundaries:** `max_event_bytes` counts inline retained observations; external `Arc` pinning, allocator overhead, asynchronous query cancellation, sort indexes, fixed-mode projection and disk history are not covered. `report_input_loss` requires discrete upstream loss reports; the current DEV-006 snapshot only exposes counters and the latest loss report.
- **Requirements remain partial:** DEV-007 task Done does **not** mean REQ-TRACE-045/047/050 or REQ-ARCH-007 fully verified at product level. GUI-dependent DEV-008 remains gated by DEV-003 native Slint dock/float behavior and manual Windows/Linux acceptance.

## Risks and blockers

- DEV-003 native floating-window lifecycle and manual Windows/Linux GUI interactions still need validation.
- Hardware support and recording format are later decisions, not blockers for virtual CAN.
- Confirm first CI run after workflow commit.

## Parallel work coordination

DEV-002, DEV-004, DEV-005, DEV-006 and DEV-007 are integrated. DEV-003 GUI acceptance is incomplete despite the merged feasibility spike. DEV-008 is still Planned and depends on DEV-003 acceptance; no GUI files were touched by DEV-007. PR #13 remains open but its original Clippy fix already exists on main; reconcile rather than blindly merge old document revisions.

## Next action

1. Validate native Slint/pbSlintDock floating-window and re-dock behavior on Windows and Linux for DEV-003; resolve any technical gaps before claiming full feasibility.
2. Once DEV-003 acceptance is established, mark DEV-008 Ready and connect DEV-005 virtual CAN → DEV-006 ingestion → DEV-007 history to a bounded virtualized Slint Trace model.
3. Schedule independently scoped follow-up for global pinned-batch memory accounting and a durable upstream loss-notification channel; neither is provided by the DEV-007 RAM budget alone.
4. Keep proposed product requirement statuses separate from DEV task completion, and do not merge stale conflicting PR #13/#14 unchanged.

Every future implementation chat must update this file with completed tasks, current commit, actual tests, failures and next action.

## DEV-003 historical branch verification (spike integrated, native validation open)

- Branch: `feat/DEV-003-slint-docking-spike`; owner: this implementation session (2026-11-08).
- Scope: sample Slint GUI using pbSlintDock Rust engine, workspace wiring, tests and cross-platform CI preparation.
- Local Rust builds/tests: not run (Cargo/rustc and external git clone are unavailable in the execution environment).
- CI: **passed** on Windows and Linux in PR #12 run [42501580558](https://github.com/JannisRln/CANexus-Studio/actions/runs/42501580558): project-state validation, `cargo fmt --all -- --check`, Clippy with `-D warnings`, workspace build and all-target tests. This is CI compilation/test evidence, not a native interactive GUI or floating-window test.
- Native floating-window lifecycle in CANexus: not implemented; engine-only float transition tests added.
- The DEV-003 spike was integrated; the task remains In Progress because native interaction and floating-window lifecycle acceptance remain unverified.

## DEV-016 integrated — rustfmt pre-commit hook

- Status: **Done**. PR [#16](https://github.com/JannisRln/CANexus-Studio/pull/16) squash-merged into main as `878dc7bff2e4c3c6091ad2f29351e23a94691758` on 2026-11-08.
- Scope: optional Git `.githooks/pre-commit` runs `cargo fmt --all` and checks formatting, refuses unstaged or partially staged Rust edits, does not silently stage; documented one-time hook activation and Linux integration tests.
- Verification: PR-head CI [run 42516618789](https://github.com/JannisRln/CANexus-Studio/actions/runs/42516618789) **passed** all four Windows/Linux jobs: project-state checks, formatting, Clippy, workspace build and tests; five hook integration tests passed on Linux. The initial PR run failed due to an existing Core test Clippy warning, fixed by a one-line assertion before merge.
- Post-merge main CI: not yet verified in this record. Manual Git-for-Windows hook execution not performed; the hook test is Linux-only. Rust binaries and automated tests were exercised in CI, not locally.
