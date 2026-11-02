# Trace Viewer: CANalyzer-like Analysis Workflow

**Revision:** 0.6 · **Status:** Mixed: user-confirmed decisions and proposed details

## Confirmed product decisions

- Provide **chronological** and **fixed-position / overwrite** trace modes, switchable without stopping acquisition.
- Support **sorting and filtering by columns**, with a workflow comparable to CANalyzer.
- Keep the interface responsive under heavy bus load.
- Trace is an analysis *view*, not the storage or capture engine; visualization must never control what is captured unless the user explicitly configures a capture/logging filter.

## Trace table requirements

| ID | Priority | Status | Requirement |
|---|---|---|---|
| REQ-TRACE-001 | P0 | Accepted | A trace panel offers chronological and fixed/overwrite display modes; changing mode does not restart capture. |
| REQ-TRACE-002 | P0 | Accepted | The trace permits sorting and filtering on individual visible columns. |
| REQ-TRACE-003 | P0 | Proposed | Standard CAN columns: timestamp, delta time, network, channel, direction, ID, ID format, type, DLC, actual data length, data bytes, decoded message name and flags. |
| REQ-TRACE-004 | P0 | Proposed | Columns may be shown/hidden, resized, reordered by drag & drop and reset to defaults; layout is reusable. |
| REQ-TRACE-005 | P1 | Proposed | Users can pin columns, save named column presets and apply different presets to different Trace panels. |
| REQ-TRACE-006 | P0 | Proposed | Clicking a column header sorts ascending/descending with a visible indicator and deterministic tie-break; the original event sequence is immutable. |
| REQ-TRACE-007 | P1 | Proposed | Multi-column sorting is supported; sorting large datasets runs in a background query and can be cancelled. |
| REQ-TRACE-008 | P0 | Proposed | Per-column filter popovers support equals/not-equals, ranges, contains, value lists and relevant CAN ID/mask operators. |
| REQ-TRACE-009 | P1 | Proposed | Filters across columns combine with AND by default; an advanced builder supports OR/NOT and parenthesized groups. |
| REQ-TRACE-010 | P1 | Proposed | Users can filter on physical decoded signal values and frame payload bytes, including masked byte patterns. |
| REQ-TRACE-011 | P0 | Proposed | Active filters and resulting match count are visible; clear one or all filters in one action. |
| REQ-TRACE-012 | P1 | Proposed | Right-click frame/cell actions: filter to value, exclude value, copy cell/row, copy selected frames, jump to source and open details. |
| REQ-TRACE-013 | P1 | Proposed | Full-text/hex search supports next/previous matches, match count and a scope of selected columns or all columns. |
| REQ-TRACE-014 | P0 | Proposed | A selected CAN message can expand or open an inspector showing decoded signals, raw payload, scaling, units and DBC origin. |
| REQ-TRACE-015 | P1 | Proposed | Absolute wall-clock, measurement-relative and inter-frame delta timestamps can be selected; time provenance/precision is exposed. |
| REQ-TRACE-016 | P1 | Proposed | Hexadecimal/decimal ID and byte formatting, numeric/symbolic naming, and customizable event highlighting are supported. |
| REQ-TRACE-017 | P1 | Proposed | Users can select rows/ranges and export/copy displayed selections or filtered results, with explicit view-versus-all-data scope. |
| REQ-TRACE-018 | P1 | Proposed | Multiple independent Trace panels can observe the same underlying session with independent modes, filters, sorting and column layouts. |
| REQ-TRACE-019 | P1 | Proposed | Receive, transmit, CAN error, state transition and internal event records are distinguishable; types can be selectively hidden. |
| REQ-TRACE-020 | P1 | Proposed | Bookmarks, annotations and cross-panel jump-to-time are supported in user-local state without modifying ECU source files. |

## Display modes & semantics

### Chronological

- One row per matching event in the sequence as captured; default sort is ascending event time with stable sequence number tie-break.
- Auto-follow is available at the latest event. Manual scroll/search/sort pauses auto-follow **only**, not acquisition or recording.
- User-selected non-time sort is a *sorted analysis view*; show that chronology has been overridden.
- A visible backlog/unseen-event count and 'Jump to Live' control help recover the live end of the stream.

### Fixed position / overwrite

- One row per selected **CAN message identity** (at minimum network + channel + 11/29-bit identifier + direction + frame class) rather than one row per individual event.
- New events update the same row and show last arrival time, current payload, cycle time, count and optional changed-byte highlight.
- Configurable row-order sorting stays stable unless the selected sort key changes; avoid row jumping on every payload update by batching/reordering predictably.
- Switching back to chronological restores a chronological view of the still-retained history, subject to explicit buffer limits.

## Filter scopes (never conflate them)

| Scope | Changes historical data? | Purpose |
|---|---|---|
| View filter | No | Show/hide matching rows for one Trace panel |
| Subscription/processing filter | No | Reduce processing performed by specific consumers |
| Hardware acceptance filter | Can prevent reception | Reduce traffic at device/controller |
| Recording/logging filter | Changes written content | Capture a deliberate subset of events |

The scope of every filter must be visible; changing a view filter must **not** silently change what is recorded.

## UX/performance requirements

| ID | Priority | Status | Requirement |
|---|---|---|---|
| REQ-TRACE-021 | P0 | Proposed | Trace virtualizes rows and fetches visible slices; no UI object is allocated for each stored frame. |
| REQ-TRACE-022 | P0 | Proposed | Rendering is rate-limited/batched independently from acquisition, decoding and recording. |
| REQ-TRACE-023 | P0 | Proposed | 'Pause view' freezes the view only; 'stop acquisition' and 'stop recording' are explicit separate actions. |
| REQ-TRACE-024 | P1 | Proposed | Display buffer limits and eviction/drop status are visible; overflow cannot be silently hidden. |
| REQ-TRACE-025 | P1 | Proposed | A large recording is searchable/sortable without loading all records into the GUI; queries are cancellable. |
| REQ-TRACE-026 | P1 | Proposed | A common trace UI contract allows protocol-specific columns for future Ethernet/DoIP/SOME-IP events without pretending they are CAN frames. |

## Acceptance tests (initial draft)

1. Generate 1,000,000 virtual CAN frames and open them in a virtualized Trace; memory stays bounded by the configured cache and query workload, not one view object per frame.
2. While receiving data, switch Chronological → Fixed → Chronological; recording remains uninterrupted, and retained history stays available.
3. Set filter Channel=CAN1 AND ID in 0x100..0x1FF; verify rows, match count, clear filter and unchanged recording.
4. Sort the fixed view by ID and chronological view by message name; verify the visible order without changing event timestamps or raw capture sequence.
5. Pause display while continuously writing frames; resume with live-follow and verify no capture stop was issued.
6. Reconfigure columns, close/reopen the Trace panel and verify the layout is restored without additional ECU-repository files.

## Open details

Exact default columns; row identity for protocol-level decoded events; whether regex filtering is needed; maximum view buffer policy; live sorting behavior and stable grouping details.

## v0.6 trace preset decision

| ID | Status | Requirement |
|---|---|---|
| REQ-TRACE-027 | Accepted | Each Trace panel retains independent filters, sorting, columns and reusable named presets. |
| REQ-TRACE-028 | Proposed | Local presets can be explicitly promoted into portable shared project configuration. |

Acceptance: two Trace panels show distinct filters and sorts simultaneously; saved presets survive restart; personal preset changes cause no ECU Git diff; unavailable plugin-contributed columns are explicitly reported.
