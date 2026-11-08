# Implementation status

**Phase:** M0 — Rust workspace completed; GUI feasibility next
**Active task:** None; DEV-003 and DEV-004 are Ready and can proceed independently
**Latest verified implementation milestone:** DEV-002 Rust workspace merged to main (`d3ee9e87`)
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

DEV-002 is merged and Done. DEV-003 (Slint/pbSlintDock spike) and DEV-004 (typed CAN frame model) are both Ready and can run in parallel on separate task branches, with shared interfaces coordinated before integration.

## Next action

1. Verify post-merge CI on main.
2. Implement DEV-003: Slint/pbSlintDock feasibility.
3. Implement DEV-004: typed CAN/CAN FD frame and timestamp model (parallel-compatible).

Every future implementation chat must update this file with completed tasks, current commit, actual tests, failures and next action.
