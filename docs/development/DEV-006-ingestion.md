# DEV-006 — Bounded typed ingress and shared event batches

**Implementation branch:** `feat/DEV-006-bounded-batch-ingestion`  
**Base:** DEV-005 squash merge `2ef21cbe87f3a03b5db57fc46efcc5cf2ba08a0f`  
**Scope:** headless, single-producer shared-batch fan-out; no GUI, HAL device driver, disk recorder or trace store.

## Design and contracts

- `BoundedIngest<T>` accepts `Observed<T>` from one `SourceId`. Generic `T` allows typed future network payloads without a CAN-only base type or boxed event.
- The producer stages at most `IngestConfig.batch_events()` events and creates one immutable `SharedBatch<T>` backing allocation per flush. Consumer handles clone only an `Arc`, not the event payload. No allocation is needed for an individual `push` while the batch has available capacity.
- `subscribe` is limited to startup and has an explicit nonzero batch-queue capacity, bounded by `max_queue_batches`. The configured maximum number of consumers is finite. Receivers use standard-library `sync_channel`; no runtime/dependency or GUI thread is required.
- `push`, `flush` and `shutdown` use `try_send` and do not block the capture producer when one subscriber is slow or disconnected.
- `OverflowPolicy::DropNewest` leaves the trace-style subscriber active; `FailConsumer` transitions recorder-style subscribers permanently to `ConsumerState::Failed` upon queue overflow, closes that subscriber's sender, and rejects further delivery to it without stopping other consumers.
- Per-consumer metrics track queued/dropped batches/events, state, and last loss provenance (source, first and last observed sequence, exact batch event count and reason). Sequence boundaries are **not** claims that every intermediate source sequence was present or dropped. Producer metrics distinguish accepted observations, published batches, and observations published without consumers.
- `shutdown` flushes a nonempty tail using the same loss policy, drops senders and lets receivers drain already queued batches. The source cannot be restarted with the same instance.
- Queue capacity is expressed in **batches**, not a fixed byte ceiling: an upper event-count bound is `batch_events × max_queue_batches` per consumer, excluding one staging batch. The full footprint depends on `size_of::<Observed<T>>()`, channel bookkeeping and separately retained receiver handles. **An external consumer can retain cloned batches indefinitely**; a future coordinated pool/admission protocol must bound globally pinned allocations.

## Requirement → implementation → test → current scope

| Requirement | Implementation | Test | Evidence/status |
|---|---|---|---|
| DEV-006 | `BoundedIngest<T>`, immutable `SharedBatch<T>`, limited queues | `subscribers_share_batch_backing_and_preserve_typed_frames`, `validates_budgets_and_admission` | Pending PR CI |
| REQ-CORE-002, REQ-CORE-003 (Proposed) | Independent bounded subscribers and nonblocking `try_send` | `full_trace_queue_drops_only_its_own_batches`, `recorder_overflow_fails_without_blocking_capture` | Partial: no scheduler/real recorder |
| REQ-CORE-006, REQ-CAN-009 (Proposed) | Recorder failure does not stall other subscribers | `recorder_overflow_fails_without_blocking_capture` | Partial: recorder-style sink, not disk I/O |
| REQ-ARCH-005, REQ-ARCH-006 (Proposed) | Per-batch `Arc` ownership; no per-consumer data clone | `subscribers_share_batch_backing_and_preserve_typed_frames`, `large_burst_keeps_queue_bounded_without_waiting_for_consumer` | Partial: no allocation instrumentation |
| REQ-ARCH-010, REQ-ARCH-011, REQ-ARCH-032 (Proposed) | Finite consumer queues, explicit consumer-local loss reporting | `full_trace_queue_drops_only_its_own_batches`, `disconnection_and_shutdown_are_observable_and_nonblocking` | Partial: no driver-overflow or decode-gap integration |
| REQ-NET-001 (Accepted), REQ-NET-003 (Proposed) | Generic typed observation payload | `generic_non_can_events_share_the_same_capture_contract` | Partial: synthetic non-CAN payload only |
| DEV-005 → DEV-006 | Virtual CAN fixture producer feeding shared batches | `virtual_can_fixture_replays_through_bounded_pipeline` | Partial headless vertical slice |
| Shutdown behavior | Final partial batch, senders closed | `final_partial_batch_drains_before_receiver_disconnect` | No timeout-based worker/drain coordination |

## Known constraints and next work

- No worker threads are owned or scheduled by this module. The caller owns source polling and consumer receive loops. No real HAL integration, disk recording, decoder gap/recovery or Trace history is claimed.
- A full queue drops **that subscriber's** batch. This is not hardware/ingress loss. The module does not fabricate missing IDs. Consumer metrics contain the latest loss record and totals; detailed persistent loss history is not implemented.
- Consumer retention outside bounded channel queues and exact global byte/pool budgets are not yet enforced. End-to-end pinned-memory admission and byte-budget accounting remain open; do not claim REQ-ARCH-007 fully fulfilled.
- One producer instance is single-source and not `Sync` by design. Additional sources need independent producers or a higher-level session router.
- No latency throughput claims without measured benchmarks and reference hardware. A deterministic 10,000-observation burst is a functional boundedness test, not a benchmark.
- Native toolchain is unavailable in the development execution environment. Actual Rustfmt, Clippy, Linux/Windows build and unit-test outcomes must be verified by GitHub Actions at the final PR head.
- DEV-007 should add a separately bounded trace history, query/read API and provenance separating history eviction from capture/consumer loss.
