# Implementation status

**Phase:** M0/M1 — typed CAN foundation merged; CI regression and DEV-003 GUI acceptance pending
**Active task:** DEV-003 Blocked on native floating-window lifecycle/interactive verification (DEV-016); DEV-004 merged; CI/acceptance fix proposed on `fix/ci-and-dev-003-004-acceptance`
**Latest verified implementation milestone:** DEV-004 merged as `e9090748`; DEV-003 merged as `bef8513d` but not GUI-accepted; main `10e1bd5` CI failed in Clippy
**Last updated:** 2026-11-08
**Branch policy:** Hybrid: PRs for substantial/risky/parallel tasks; direct main pushes allowed for small isolated changes

## Implemented

- Product requirements and architecture documentation.
- MIT license.
- Chat-driven implementation planning and status tracking.
- Minimal Rust workspace (`canexus-core`, `canexus-cli`), pinned toolchain and cross-platform Rust CI.
- DEV-004 typed CAN/CAN-FD frame and timestamp model (PR #11).
- DEV-003 Slint/pbSlintDock feasibility spike (PR #12); production trace UI remains future work.

## Not implemented

No production trace GUI, virtual CAN, HAL, trace viewer or protocol implementation yet. The CLI is only a bootstrap entry point; DEV-003 is a feasibility spike.

## Verification evidence

DEV-002 PR #10 CI completed successfully on Windows and Linux (four jobs): https://github.com/JannisRln/CANexus-Studio/actions/runs/42499919323 . PR #10 was squash-merged to main at `d3ee9e87`. This verifies the PR head; post-merge main CI should be checked separately. No hardware tests were performed.

DEV-004 PR #11 CI succeeded at head `489f620b`, squash-merged as `e9090748`. DEV-003 PR #12 CI succeeded at head `8f42d11d`, squash-merged as `bef8513d`. Post-merge combined main CI has not yet been confirmed.

## Risks and blockers

- Slint/pbSlintDock native Windows/Linux interactive floating-window roundtrip and persisted layout remain unverified; DEV-016 records the follow-up.
- Hardware support and recording format are later decisions, not blockers for virtual CAN.
- Main CI run 42502822593 failed at Clippy on both Windows and Linux; formatting and project-state validation succeeded, build/tests skipped. Acceptance-branch CI is pending.

## Parallel work coordination

DEV-002 is merged and Done. DEV-003 and DEV-004 code are merged; DEV-003 is Blocked pending native GUI acceptance, while DEV-004 task scope is implemented but broader product requirements remain partly fulfilled. DEV-005 may proceed independently against the typed interfaces; do not modify or overwrite active DEV-005 work. Reconcile shared status files against newest main before integration.

## Next action

1. Review the CI/acceptance fix PR and confirm its Windows/Linux Clippy, formatting, build, tests and project-state checks.
2. Merge only after checks pass and verify subsequent main CI separately.
3. Complete DEV-016 native floating-window and interactive Windows/Linux feasibility checks before closing DEV-003.
4. Continue DEV-005 on its own branch without confusing typed-model unit tests with acquisition/recording validation.

Every future implementation chat must update this file with completed tasks, current commit, actual tests, failures and next action.

## Historical DEV-003 branch verification (subsequently merged)

- Branch: `feat/DEV-003-slint-docking-spike`; owner: this implementation session (2026-11-08).
- Scope: sample Slint GUI using pbSlintDock Rust engine, workspace wiring, tests and cross-platform CI preparation.
- Local Rust builds/tests: not run (Cargo/rustc and external git clone are unavailable in the execution environment).
- CI: **passed** on Windows and Linux in PR #12 run [42501580558](https://github.com/JannisRln/CANexus-Studio/actions/runs/42501580558): project-state validation, `cargo fmt --all -- --check`, Clippy with `-D warnings`, workspace build and all-target tests. This is CI compilation/test evidence, not a native interactive GUI or floating-window test.
- Native floating-window lifecycle in CANexus: not implemented; engine-only float transition tests added.
- The feature PR was subsequently merged to main. DEV-003 remains Blocked for native host floating-window and platform interaction evidence, not Done simply because its engine tests passed.

## 2026-11-08 CI and DEV-003/004 acceptance review

- Baseline main: `10e1bd5cdfaa14b1af189ad7891f4bf79f24ecf4`.
- Main run [42502822593](https://github.com/JannisRln/CANexus-Studio/actions/runs/42502822593): Rust Clippy **failed** on both Windows and Linux; formatting and project-state validation passed on both; Rust build and tests were skipped.
- Root cause: `can.rs::flags_and_payload_are_owned_and_copyable` mutates `input[0]` but never reads it again, triggering the unused-assignment warning with `-D warnings`. Fix asserts changed input independently of the frame payload. No lint suppression.
- Additional DEV-004 unit tests cover DLC and payload boundaries, zero-padding, time-domain isolation, typed generic envelopes and event identity. **Acceptance-branch verification pending** until GitHub Actions executes the new commits.
- Requirement evidence and truthful classifications: [DEV-003-004-ACCEPTANCE.md](DEV-003-004-ACCEPTANCE.md).
- There is no native Windows/Linux GUI interaction evidence; DEV-003 remains Blocked and native float lifecycle remains DEV-016.
