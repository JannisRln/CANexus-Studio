# DEV-005 — Deterministic virtual CAN input (implementation increment)

**Scope:** A headless, pull-based virtual CAN/CAN-FD adapter on top of the existing typed `CanFrame`, `Observed<T>`, and timestamp types.

## Behavior and contract

- A caller supplies an immutable slice of `VirtualCanRecord` values. Records contain validated CAN frames, channel, direction and raw synthetic clock ticks.
- `VirtualCanAdapter::new` validates non-decreasing ticks once; equal ticks model burst traffic. The adapter returns a structured error for decreasing ticks.
- Iteration yields owned `Observed<CanFrame>` with source-local sequence numbers beginning at zero, a caller-owned source ID and clock domain, explicit resolution, `TimestampOrigin::Synthetic`, and `TimestampQuality::Uncorrelated`.
- No wall-clock dependency, sleep, device SDK, GUI access, per-event heap allocation or unbounded internal queue. Fixture storage remains caller-owned; the adapter borrows it.
- Each independent replay must use a distinct source ID when its events enter one combined capture. Clock domains must not be treated as synchronized.

## Tests and traceability

| Requirement | Implementation | Evidence | Coverage |
|---|---|---|---|
| DEV-005 | `crates/canexus-core/src/virtual_can.rs` | `crates/canexus-core/tests/virtual_can.rs` | Core fixture iteration and input validation |
| REQ-HAL-002 (Proposed) | `VirtualCanAdapter` | repeatability, multi-channel, mixed CAN/FD, burst/sparse, empty and invalid fixtures | Partial: no full discover/open/close/TX HAL yet |
| REQ-CAN-002 (Proposed) | Per-record `ChannelId` | multi-channel fixture test | Partial: logical channels only |
| REQ-CORE-001 / REQ-CORE-004 (Proposed) | Headless adapter + `Observed<CanFrame>` | integration tests on core | Partial: hardware and downstream services absent |
| REQ-ARCH-034 (Proposed) | Deterministic replay data | repeated runs compared exactly | Partial: failure injection remains for subsequent tasks |

## Deferred boundaries

DEV-006 will supply bounded ingestion/batching and overload/shutdown tests. This increment does not select a stable native plugin ABI, implement a physical CAN driver, schedule TX, inject driver faults, or establish correlation to wall-clock/other adapters. No benchmark claims are made. Mark DEV-005 Done only after PR integration, CI verification, and shared development status reconciliation.

## CI and integration dependency

The repository CI workflow currently runs for pull requests targeting `main`. PR #14 therefore keeps `main` as its target while carrying the reviewed CI/acceptance corrections from PR #13 in a merge commit. Merge PR #13 first; then recheck PR #14's diff, base and Windows/Linux CI before integration. A passing branch job does not establish that post-merge `main` is green.
