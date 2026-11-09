# DEV-007 — Bounded in-memory trace history and paginated queries

**Task branch:** `feat/DEV-007-trace-history`  
**Base:** DEV-006 merge `72dd5f2223b6071ec8c7b6eb570670c66c79660b`  
**Scope:** Headless, source-local, immutable RAM history and bounded query API. No GUI, disk history, protocol decoders, worker pool or recorder implementation.

## Implementation

- `TraceHistory<T>` consumes `SharedBatch<T>` emitted by DEV-006. It stores the same immutable `Arc` event arrays with no frame payload copy. The generic API admits future network payload types as well as `CanFrame`.
- Each instance owns one `SourceId`. Admitted event sequence numbers must increase strictly across batches; gaps are legal and **not** interpreted as verified loss. Original `EventTimestamp` (clock domain and quality) remains untouched. History iteration follows source capture sequence, not falsely synchronized wall-clock order.
- `HistoryLimits` specifies nonzero `max_event_bytes`, `max_batches` and `max_page_rows`. A whole incoming batch must fit the inline event-byte budget or is rejected transactionally. Accepted batches evict complete oldest batches until both bounds hold. The batch count also bounds history-owned `Arc`/deque metadata growth.
- `HistoryStats` distinguishes retained event counts/inline bytes, total admitted events, **RAM eviction** totals and boundaries, and explicitly **reported input consumer queue loss**. `report_input_loss(BatchLoss)` consumes an explicit DEV-006 loss notification, preserves its reason, and does not fabricate missing sequence ranges; the caller must report distinct losses once. Missing raw event sequence numbers alone do not affect drop counters.
- `TraceHistory::page_with` returns at most `max_page_rows` **borrowed event references** per page with a cursor tied to source and history version. Appending/evicting invalidates old cursors, preventing reading across inconsistent retention snapshots. `count_matches` scans without materializing per-event result objects. Both are synchronous; expensive queries must run outside the GUI event loop.
- `CanTraceFilter` adds Classic/FD format and 11/29-bit CAN ID/mask matching on top of generic channel, direction and sequence predicates. Filter evaluation does not change captured data, recording or timestamp provenance.
- The initial API is a **basic query service**, not an async/cancellable index, sort engine, full hierarchical Trace model, or durable snapshot. Chronological/fixed GUI modes belong to DEV-008 and later scoped work.

## Requirement → implementation → tests → current assessment

| Requirement | Code | Integration tests | Status |
|---|---|---|---|
| DEV-007 | `trace_history.rs` bounded history, pages, diagnostics | `budgets_reject_zeros_and_evict_oldest_complete_batches`, `page_size_bounds_and_stale_cursors_are_explicit`, `history_eviction_is_not_source_or_consumer_capture_loss` | Implemented on branch; PR CI required |
| REQ-TRACE-045 (Proposed) | Byte/event and batch-slot constrained `VecDeque<SharedBatch<T>>` | `batch_slots_also_bound_memory_when_event_budget_is_large`, `large_stream_stays_within_retained_byte_and_batch_budgets` | Partial: inline bytes only; global pinned memory not enforceable here |
| REQ-TRACE-047 (Proposed) | Bounded `TracePage` of borrowed event references | `page_size_bounds_and_stale_cursors_are_explicit`, `generic_non_can_observations_use_same_history_and_paging` | Partial: no GUI model, no index/search cancellation |
| REQ-TRACE-049 (Proposed) | Read-only event and CAN filter predicates | `can_filter_preserves_format_mask_direction_and_capture_sequence_order` | Partial: capture/recorder separation tested only in headless pipeline |
| REQ-TRACE-050, REQ-ARCH-032 (Proposed) | Independent `last_eviction`, `last_input_loss`, counters | `history_eviction_is_not_source_or_consumer_capture_loss` | Partial: driver loss and decoder gaps not yet integrated |
| REQ-NET-001 (Accepted), REQ-NET-003 (Proposed) | `TraceHistory<T>` and generic `EventFilter` | `generic_non_can_observations_use_same_history_and_paging` | Partial: synthetic non-CAN payload only |
| DEV-005 → DEV-006 → DEV-007 | Virtual CAN → shared bounded batches → history | `virtual_can_observations_preserve_time_payload_and_channel`, `shared_batches_can_feed_recorder_and_history_without_frame_copy` | Headless integration path, not hardware/GUI |
| Error handling | Transactional invalid admission, stale cursor errors | `rejected_batches_are_transactional_and_do_not_evict_existing_history` | Verified only after CI passes |

## Important resource and correctness limits

- `max_event_bytes` counts `size_of::<Observed<T>>() × retained_events`, excluding `Arc` control blocks, deque capacity/metadata and memory held by other consumers. `max_batches` bounds history-internal batch allocations, but **not global pinned heap**; external consumers may retain old `Arc` batches. Global shared-pool admission/credit accounting remains open.
- Capacity eviction removes **whole** batches (not partial events) and therefore can retain fewer bytes than the limit. History eviction is never reported as capture loss.
- An incoming single batch larger than the memory limit is explicitly rejected and does not change existing retained records. The caller must decide how to report/recover from rejected admission; the history cannot silently claim successful retention.
- Query pages borrow history; a query must not hold a mutable history borrow simultaneously. Cursor versions invalidate on every append; the caller should restart an old paginated query after a mutation.
- Filter/page operations are synchronous and may scan many retained events; run on a cancellable background worker when wired to the GUI (DEV-008+). Sorted analysis views, disk-backed history, indexes, hierarchy, fixed/overwrite projection and byte diffs are not implemented in DEV-007.
- `report_input_loss` must be fed discrete upstream drop reports at most once. A `ConsumerSnapshot` only has totals and its latest batch drop; it is not a durable feed of every earlier loss. Integration must use a dedicated loss/health channel before claiming complete gap provenance.
- Neither hardware capture nor a real recorder is included. No benchmark p95/p99, Linux/Windows GUI/hardware validation or precise cross-domain time correlation is claimed.
- Rustfmt, Clippy, build and all-target tests must be verified on Windows and Linux CI at the **final PR head**. No local Rust toolchain is available in the implementation environment.

## Next step

On successful CI, merge and verify new `main`, update `BACKLOG.md` / `STATUS.md` to mark DEV-007 Done, then implement DEV-008 Slint Trace panel only after DEV-003 native docking acceptance is resolved. Independently address shared-batch pinned-memory accounting and durable loss notification semantics.
