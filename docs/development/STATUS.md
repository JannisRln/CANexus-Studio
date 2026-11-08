# Implementation status

**Phase:** M0 — Rust workspace completed; DEV-003 GUI feasibility in progress
**Active task:** DEV-003 In Progress on `feat/DEV-003-slint-docking-spike`; DEV-004 remains independently Ready
**Latest verified implementation milestone:** DEV-002 merged to main (`d3ee9e87`); DEV-003 PR #12 CI passed (not merged)
**Last updated:** 2026-11-08
**Branch policy:** Hybrid: PRs for substantial/risky/parallel tasks; direct main pushes allowed for small isolated changes

## Implemented

- Product requirements and architecture documentation.
- MIT license.
- Chat-driven implementation planning and status tracking.
- Minimal Rust workspace (`canexus-core`, `canexus-cli`), pinned toolchain and cross-platform Rust CI.

## Not implemented

No Slint GUI, virtual CAN, HAL, trace viewer or protocol implementation yet. The CLI is only a bootstrap entry point.

## Verification evidence

DEV-002 PR #10 CI completed successfully on Windows and Linux (four jobs): https://github.com/JannisRln/CANexus-Studio/actions/runs/42499919323 . PR #10 was squash-merged to main at `d3ee9e87`. This verifies the PR head; post-merge main CI should be checked separately. No Slint or hardware tests were performed.

## Risks and blockers

- Slint/pbSlintDock Windows/Linux integration needs proof of concept.
- Hardware support and recording format are later decisions, not blockers for virtual CAN.
- Confirm first CI run after workflow commit.

## Parallel work coordination

DEV-002 is merged and Done. DEV-003 is in progress on `feat/DEV-003-slint-docking-spike`; DEV-004 remains Ready on an independent branch. The GUI spike does not modify `canexus-core` interfaces. Reconcile this file against the newest main before merging; do not replace concurrent DEV-004 progress.

## Next action

1. Verify post-merge CI on main.
2. Review DEV-003 PR and obtain Windows/Linux CI evidence; implement missing native floating-window lifecycle before final feasibility decision.
3. Implement DEV-004: typed CAN/CAN FD frame and timestamp model (parallel-compatible).

Every future implementation chat must update this file with completed tasks, current commit, actual tests, failures and next action.

## DEV-003 branch verification (not yet integrated)

- Branch: `feat/DEV-003-slint-docking-spike`; owner: this implementation session (2026-11-08).
- Scope: sample Slint GUI using pbSlintDock Rust engine, workspace wiring, tests and cross-platform CI preparation.
- Local Rust builds/tests: not run (Cargo/rustc and external git clone are unavailable in the execution environment).
- CI: **passed** on Windows and Linux in PR #12 run [42501580558](https://github.com/JannisRln/CANexus-Studio/actions/runs/42501580558): project-state validation, `cargo fmt --all -- --check`, Clippy with `-D warnings`, workspace build and all-target tests. This is CI compilation/test evidence, not a native interactive GUI or floating-window test.
- Native floating-window lifecycle in CANexus: not implemented; engine-only float transition tests added.
- DEV-003 remains In Progress until integration and recorded verification, not Done merely because a PR exists.
