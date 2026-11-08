# DEV-005 — Deterministic virtual CAN fixture source

**Implementation branch:** `feat/DEV-005-virtual-can-fixtures`  
**Baseline:** `main` at `10e1bd5cdfaa14b1af189ad7891f4bf79f24ecf4`  
**Scope:** Headless CAN Classic / CAN-FD fixture source; no GUI, hardware access, or shared ingress/queue service.

## API and behavior

- `canexus_core::virtual_can::VirtualCanFixture` carries validated `CanFrame`, channel, RX/TX direction, and release tick.
- `VirtualCanAdapter::new` borrows a chronological, immutable fixture slice; descending ticks are rejected. Events sharing a timestamp retain fixture order.
- `start`, `advance_to`, `next_due`, and `stop` expose an explicit single-use lifecycle. The caller advances a logical clock; no sleeping, background threads, or hidden scheduling occurs.
- Each due event is returned as a typed `Observed<CanFrame>` by value with its original fixture tick, synthetic provenance, uncorrelated clock quality, domain, source ID, and monotonic source-local sequence.
- `fixtures::mixed_classic_fd()` provides reusable two-channel Classic, 64-byte FD/BRS, remote, and empty-payload cases with equal-tick ordering.
- The adapter holds a borrowed slice and a constant-size cursor; polling one due event is O(1) and does not allocate, but the surrounding ingestion/recording pipelines are not implemented.

## Requirement-to-test traceability

| Requirement | Implementation | Automated test | Scope result |
|---|---|---|---|
| DEV-005 | Virtual clock, script validation, fixture source | `clock_controls_visibility_and_preserves_stable_tie_order`, `single_jump_and_incremental_steps_produce_identical_observations`, `rejects_nonchronological_fixtures_without_modifying_them` | Implemented on branch, CI pending |
| REQ-HAL-002 (Proposed) | Deterministic hardware-free CAN source | `fixtures_cover_classic_fd_remote_and_two_channels`, `lifecycle_and_monotonic_clock_errors_do_not_emit_events` | Partially fulfilled: not yet a common hardware HAL implementation |
| REQ-CAN-002 (Proposed) | Two distinct logical fixture channels | `fixtures_cover_classic_fd_remote_and_two_channels` | Partially fulfilled: not multi-adapter live capture |
| REQ-CORE-004 (Proposed) | `Observed<CanFrame>` with source, channel, direction and time | `single_jump_and_incremental_steps_produce_identical_observations`, `source_identity_differs_even_with_same_fixture_and_clock_domain` | Partially fulfilled: synthetic capture only |
| REQ-ARCH-034 (Proposed) | Repeatable virtual source fixture | `single_jump_and_incremental_steps_produce_identical_observations` | Partially fulfilled: no failure injection or end-to-end capture |
| REQ-NET-003 (Proposed) | Typed, inline CAN payload and no boxed event type | `hot_path_observation_does_not_require_heap_owned_fields` | Partially fulfilled: no allocation-instrumented benchmark |

## Known limits and follow-up

- `next_due` is a pull API for deterministic tests, **not** a shared hardware abstraction or the bounded capture pipeline. DEV-006 owns batch ingress, queues, backpressure, overload and shutdown/failure tests.
- No simulated bus errors, fault injection, arbitration, scheduler, physical transmission, or real-time guarantees.
- Reopening a session requires a fresh `SourceId` from its caller. Runtime generation identity and reconnect coordination remain follow-up work; this fixture source does not claim to solve cross-session identity collisions.
- Synthetic clock domains are explicitly uncorrelated. No cross-adapter clock synchronization or wall-clock conversion is attempted.
- `needs_drop` tests establish an owned inline representation, not a measured allocation profile.
- Cargo and rustc are unavailable in the local execution environment; Windows/Linux PR CI is the compilation, Clippy, formatting and unit-test verification gate.
- PR #13 separately fixes a pre-existing Clippy failure on `main`. Until that PR integrates, a DEV-005 PR based on `main` can fail checks unrelated to DEV-005. No GUI/CI files or PR #13-owned paths are changed here.

Do not mark DEV-005 Done in the shared backlog before merge and verified main CI. Reconcile shared STATUS/BACKLOG after integration to avoid overwriting concurrent updates.
