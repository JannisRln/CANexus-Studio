# Implementation status

**Phase:** M1 — GUI migration architecture and ecosystem review prepared. DEV-024 is the next GUI task.
**Active tasks:** DEV-024 native and dependency proof is in progress on `spike/DEV-024-craft-desktop`, owned by this implementation session.
**Latest verified baseline:** DEV-028 [PR #28](https://github.com/JannisRln/CANexus-Studio/pull/28) merged as `2bf56dd`. Its [main CI](https://github.com/JannisRln/CANexus-Studio/actions/runs/45432818018) passed all Windows/Linux jobs.
**Last updated:** 2026-11-24
**Branch policy:** Hybrid: PRs for substantial/risky/parallel tasks; direct main pushes allowed for small isolated changes

The live baseline for the migration review is `f158f3e`. Its [exact-main CI](https://github.com/JannisRln/CANexus-Studio/actions/runs/45344406402) completed successfully on Windows and Linux. [ADR-012](../requirements/26-adr-012-egui-craft-ui.md) is the forward GUI direction. Older DEV-008/022 ownership and Slint host notes below are historical.

## Rust 1.100 and legacy PR reconciliation

- The workspace, local pin and CI use Rust 1.100.0. DEV-024 uses the same toolchain to evaluate the GUI dependency graph.
- The additional coverage from PRs #13 and #14 checks typed non-CAN identity, fractional timestamp conversion and ordered virtual replay of 32,768 frames at one tick. The current timed Virtual CAN adapter and the already integrated DEV-003/004/005 behavior remain the product baseline.
- Local Linux verification with Rust 1.100.0 passed Rustfmt, Core/CLI Clippy, Core/CLI build and 43 tests. Cross-platform workspace results are recorded by the integration PR and GitHub Actions. Native GUI, hardware and performance verification are separate.

## DEV-023 shared-batch budget

- Ownership: Core ingestion on `feat/DEV-023-batch-budget`. [PR #27](https://github.com/JannisRln/CANexus-Studio/pull/27) merged as `0b058dc`; its [main CI](https://github.com/JannisRln/CANexus-Studio/actions/runs/45429678132) passed on Windows and Linux.
- REQ-ARCH-006/007: `BoundedIngest::with_budget` reserves inline batch storage before staging, transfers it to shared immutable handles without a payload copy, and releases the charge on last-owner drop. Producer admission rejection counts and provenance are readable independently of subscriber queues.
- Fixtures: concurrent reservations, two producers with pinned handles, Trace eviction with an external owner, sparse flush, rejected admission and resumption, and storage release on producer drop and shutdown.
- Local verification on Linux with Rust 1.100.0: workspace Rustfmt, Core/CLI Clippy with warnings denied, Core/CLI all-target build, 47 Rust tests and project-state validation passed.
- Runtime handoff: DEV-028 passes one shared budget to its producers and exposes rejection counters. The budget covers inline batch storage and fixed metadata. Other pools, recorder reservations, live profile changes and process memory verification remain open.


## DEV-028 headless virtual capture sessions

- Ownership: Core on `feat/DEV-028-capture-session`, based on DEV-023 merge `0b058dc`.
- The host orders intake, flush deadlines, subscription changes and stop. Virtual adapters remain caller-driven in synthetic time. The service owns the live flush timer and uses bounded per-source receiver queues with a drop-newest policy. Stop tracks delivery until the batch is taken from the queue.
- Local Linux verification with Rust 1.100.0 passed workspace Rustfmt, Core/CLI Clippy with warnings denied, Core/CLI all-target build, 58 Core tests, project-state validation and five Git hook tests. Public session acceptance is covered in [capture_session.rs](../../crates/canexus-core/tests/capture_session.rs).

| Acceptance | Integration tests |
|---|---|
| RAM-only virtual CAN/CAN-FD, including idle real-time publication and first-frame deadlines | `running_service_publishes_one_fd_frame_without_follow_up_input`, `manual_clock_flushes_from_the_first_frame_and_not_before_its_deadline` |
| Concurrent timer and stop, receiver drain, idempotent completion and a pinned slow receiver | `timer_and_stop_share_one_publication_and_completion_at_the_deadline`, `stop_allows_a_waiting_receiver_to_drain_the_tail`, `stop_is_bounded_with_a_full_queue_and_a_pinned_external_batch` |
| Dynamic batch boundaries, independent loss counters and observed sequence bounds | `joining_and_leaving_close_the_partial_batch_at_the_documented_boundary`, `repeated_queue_overflow_is_local_and_reports_only_known_sequence_bounds` |
| Reconnect with reused source and sequence identity | `reconnect_rejects_old_results_and_keeps_old_deadlines_generation_local` |
| Shared batch memory, pin lifetime, recovery and 64-bit multi-GiB limits | `shared_budget_accounts_for_two_sources_and_recovers_after_the_last_reference`, `multi_gib_budget_is_representable_without_allocating_it` |

The Core regression test `receiver_winning_the_lock_after_expiry_cannot_take_the_tail` schedules a receiver ahead of the stop waiter at expiry and verifies the frozen loss result.

The follow-up `fix/DEV-028-retain-detached-loss` keeps the last receiver snapshot on each detached handle and maintains per-source detached loss totals in both live and stopped health. The integration test `detached_receiver_losses_survive_replacement_and_stop` verifies two overflows, a queued discard, replacement and final status. The local Linux workspace build, Clippy and all 61 Rust tests passed on Rust 1.100.0.

Latest loss details give the first and last observed sequence plus the actual count. Cumulative counts distinguish budget rejection, per-receiver overflow and queued deliveries discarded during detach or stop. Queue metadata and overall process memory have separate accounting. DEV-009 retains the latency and process-memory benchmark, DEV-010 owns physical adapters, and DEV-013/030 retain optional recording and disk Trace history. No capture latency or process-memory target is claimed here.

## Implemented

- Product requirements and architecture documentation.
- MIT license.
- Chat-driven implementation planning and status tracking.
- Minimal Rust workspace (`canexus-core`, `canexus-cli`), pinned toolchain and cross-platform Rust CI.
- DEV-004 typed CAN/CAN-FD frame and timestamp model (PR #11).
- DEV-005 deterministic virtual CAN fixture source with virtual clock (PR #15, merged).
- DEV-006 bounded, transport-neutral event ingestion with shared immutable batches, explicit overload and shutdown behavior (PR #17, merged).
- DEV-007 bounded, source-local RAM Trace history with versioned paginated queries, CAN filters and separate eviction/ingress-loss diagnostics (PR #18, merged).
- DEV-003 **Done within the documented minimal single-window feasibility scope** (PR #12): Slint/pbSlintDock workspace, dock callbacks, layout snapshot and engine-level float tests. DEV-022 owns the unimplemented native float lifecycle; DEV-008 owns production Trace UI.

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

## DEV-003 acceptance reconciliation and parallel work split (2026-11-09)

- **What is verified:** [PR #12](https://github.com/JannisRln/CANexus-Studio/pull/12) is merged into `main` as `bef8513d8aeb56d4bd1144be21fdfd4f9a73cb17`. Its [CI run 42501580558](https://github.com/JannisRln/CANexus-Studio/actions/runs/42501580558) passed project-state, formatting, Clippy `-D warnings`, build and workspace tests on both Windows and Linux. The current `main` also passed [CI run 42566700153](https://github.com/JannisRln/CANexus-Studio/actions/runs/42566700153), including GUI compilation/tests.
- **Exact DEV-003 scope:** pinned Rust 1.92/Slint 1.17.1 and pbSlintDock engine integration; one native `MainWindow` with three registered built-in panels, dock/split/tab/close/reopen callbacks, in-memory save/restore; engine tests for layout restore and float/redock state. This is a **feasibility spike** and does not claim native floating Slint windows, manual UI interaction or high-performance trace rendering.
- **Unfulfilled native criteria transferred, not waived:** DEV-022 owns creation/disposal and redocking of *actual* native Slint float windows, shared Rust-owned panel state, focus/DPI/multi-monitor behavior and Windows/Linux interaction evidence. REQ-UX-004/009 and REQ-PLG-012..014 remain at their separate product statuses; no third-party dynamic Slint plugin is implemented.
- **Scope decision:** DEV-DEC-009 explicitly closes minimal DEV-003 and makes DEV-008 Ready. DEV-008 can implement a performant single-window Trace against DEV-005/006/007 while DEV-022 explores native floating in isolated examples. DEV-023 separately researches globally pinned `Arc` batch budgets. [Parallel chat tasks](PARALLEL-CHATS.md) document disjoint file ownership and PR sequencing.
- **Evidence limit:** Existing Windows/Linux CI verifies compile, automated Rust tests and engine state transitions; **no native window interaction was executed here**. The task split must not be interpreted as product acceptance for floating/plugin UI.
- **Coordinated next deliverable:** This reconciliation is documentation only, with no GUI, Core, dependency or CI workflow changes.

## Risks and blockers

- DEV-022 native floating-window lifecycle and manual/display-backed Windows/Linux GUI interactions remain unimplemented/unverified.
- Hardware support and recording format are later decisions, not blockers for virtual CAN.
- Confirm first CI run after workflow commit.

## Parallel work coordination

DEV-002 through DEV-007 (except separately listed tasks) are integrated, including the DEV-003 minimal feasibility spike. DEV-008, DEV-022 and DEV-023 are Ready and have disjoint file ownership per PARALLEL-CHATS.md. Native float/interaction acceptance remains DEV-022; old PR #13 contains stale baseline changes and should not be merged unchanged.

## Next action

1. Claim DEV-024 on an isolated branch after verifying the current backlog and open PRs. Record exact Cargo sources and native Windows/Linux results using the [GUI migration plan](GUI-MIGRATION.md).
2. DEV-023 can proceed independently in Core ingestion with shared-batch budget evidence.
3. After DEV-024, stage DEV-025 desktop cutover and DEV-022 isolated native floating proof. DEV-008 integrates production Trace after the shell lands.
4. Reconcile shared status and backlog after each verified merge and record GUI, hardware and benchmark evidence separately from Rust CI.

## DEV-003 original PR evidence (minimal spike complete, native float split to DEV-022)

- Branch: `feat/DEV-003-slint-docking-spike`; owner: this implementation session (2026-11-08).
- Scope: sample Slint GUI using pbSlintDock Rust engine, workspace wiring, tests and cross-platform CI preparation.
- Local Rust builds/tests: not run (Cargo/rustc and external git clone are unavailable in the execution environment).
- CI: **passed** on Windows and Linux in PR #12 run [42501580558](https://github.com/JannisRln/CANexus-Studio/actions/runs/42501580558): project-state validation, `cargo fmt --all -- --check`, Clippy with `-D warnings`, workspace build and all-target tests. This is CI compilation/test evidence, not a native interactive GUI or floating-window test.
- Native floating-window lifecycle in CANexus: not implemented; engine-only float transition tests added.
- The DEV-003 minimal integration task is Done after explicit scope reconciliation (DEV-DEC-009). Native interaction and floating-window lifecycle remain **unverified** under DEV-022; engine-only tests never established native host correctness.

## DEV-016 integrated — rustfmt pre-commit hook

- Status: **Done**. PR [#16](https://github.com/JannisRln/CANexus-Studio/pull/16) squash-merged into main as `878dc7bff2e4c3c6091ad2f29351e23a94691758` on 2026-11-08.
- Scope: optional Git `.githooks/pre-commit` runs `cargo fmt --all` and checks formatting, refuses unstaged or partially staged Rust edits, does not silently stage; documented one-time hook activation and Linux integration tests.
- Verification: PR-head CI [run 42516618789](https://github.com/JannisRln/CANexus-Studio/actions/runs/42516618789) **passed** all four Windows/Linux jobs: project-state checks, formatting, Clippy, workspace build and tests; five hook integration tests passed on Linux. The initial PR run failed due to an existing Core test Clippy warning, fixed by a one-line assertion before merge.
- Post-merge main CI: not yet verified in this record. Manual Git-for-Windows hook execution not performed; the hook test is Linux-only. Rust binaries and automated tests were exercised in CI, not locally.
