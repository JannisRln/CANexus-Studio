# ADR-009 — High-Performance CAN Trace Viewer

**Decision direction:** Confirmed in discussion (2026-11-08)  
**Detailed requirements:** Proposed · **Revision:** 1.0  
**Related:** [CAN Analyzer](03-can-analyzer.md), [Trace Viewer](06-trace-viewer.md), [Transmission, Triggers and Replay](10-transmission-trigger-replay.md), [UX & Performance](02-ux-performance.md), [Unified Signal Explorer](07-signal-workspace.md), [ADR-007](19-adr-007-elf-dwarf-symbol-calibration.md)

## 1. Context and decision

CANexus-Studio shall provide a professional, CANalyzer-class analysis workflow for CAN/CAN FD with high-performance navigation, filtering and replay/offline inspection. The Trace is a **view and query client**, not the capture or recorder engine.

User-confirmed choices:
- Raw frames and decoded protocol events are available in a **hierarchical** representation.
- **Chronological** and **Fixed/Overwrite** are the two main Trace modes; **Statistics** is a separate dockable panel.
- Provide **byte and decoded-signal differences**, with visual highlighting.
- Keep recent data in a bounded **RAM ring buffer**, with **optional disk-backed history**.
- Preserve previously accepted independent Trace-panel filter, sort, column and preset behavior (`REQ-TRACE-001`, `002`, `027`).

The disk-history policy (automatic temporary spill vs explicit opt-in), the default protocol-row presentation (raw-primary vs mixed list), and whether Fixed-mode live reordering is continuous remain **open**. No choice from a previously displayed but unanswered questionnaire is assumed accepted.

## 2. UX and interaction

### Main trace table

Standard CAN columns: timestamp, delta time, network/channel, RX/TX direction, 11/29-bit ID and format, message name, CAN vs CAN FD and flags, DLC and actual payload length, payload bytes, event type and quality. User can show/hide, reorder, resize and pin supported columns; columns and filters have per-panel presets. Hex/decimal, symbolic/numeric names, timestamps (measurement-relative, absolute where available, delta) and customizable highlights are selectable.

- Fast per-column filter popovers, global filter builder (AND/OR/NOT), value lists, CAN ID/mask/ranges, payload patterns and decoded signal fields.
- Search forward/back across ID, names, payload and protocol fields; matched count, cancel, scope and clear options are visible.
- Context actions: filter to/exclude value, copy row/cell/selection, compare, bookmark, open inspector, jump to source and related Plot.
- Multiple dockable Trace panels may show the same session with independent mode, filters, sorting, highlighting, column layouts and presets.
- Personal view states stay user-local. Explicitly shared presets may use portable project configuration, consistent with ADR-007 modular file model.
- Protocol Inspector, Difference View and Statistics are separate dockable panels and use shared event/time navigation; missing plugins yield recoverable unavailability.

### Chronological mode

Each raw event remains individually addressable. Derived protocol events have explicit logical IDs, time ranges and source references. Default display order is event timestamp with a deterministic captured-sequence tie-break; cross-device clock uncertainty must remain visible. Non-time sorts are clearly labeled analysis views rather than chronology. Manual scroll/search/sort disables **auto-follow only**; capture and recording continue. Show unseen event count and Jump to Live.

### Fixed/Overwrite mode

Row identity at minimum: session/source, logical network, channel, CAN ID, ID format, frame class and direction. Update each row with latest payload, last timestamp, counter, cycle-time stats, rate, optional timeout state and changed-byte highlight. Batch UI updates and maintain predictable row position; exact live-reorder policy is open. Protocol-specific derived events use provider-supplied grouping keys rather than naively collapsing UDS/ISO-TP transactions by CAN ID. Changing Trace mode does not restart capture, and returning to chronological mode exposes the still-retained history.

### Statistics (separate panel)

Show message counts, observed frequency, last/min/max/mean cycle time, jitter, configured timeout violations, per-channel bus/error statistics, measured window and timestamp source/quality. Define jitter relative to an explicit expected period when one exists; otherwise show empirical intervals without declaring a deadline violation. Missing captures and unsynchronized clocks influence quality. Statistics can follow the shared time cursor or analyze an explicitly chosen range; displayed filter vs independent-statistics scope must be labeled.

## 3. Event and protocol data model

Capture owns immutable raw events. Protocol decoders own separately materialized (or lazily computed) derived event records with stable references to input raw frames. One raw event can contribute to multiple interpretations. A decoded event may reference discontiguous frame ranges, and decoder output never overwrites a raw frame.

Conceptual Rust model:

```rust
pub struct RawEventId {
    pub session: SessionId,
    pub sequence: u64,
}

pub struct DecodedEvent {
    pub id: DecodedEventId,
    pub protocol: ProtocolId,
    pub kind: ProtocolEventKind,
    pub time_range: TimeRange,
    pub source_frames: FrameReferences,
    pub payload: DecodedPayload,
    pub status: DecodeStatus,
}
```

Protocol and decoder provenance includes plugin ID/version, description resource/version and decoding configuration identity. This enables cache invalidation after DBC or protocol-description changes. Derived events can be regenerated offline from available raw data.

Illustrative hierarchies:
- CAN/DBC: raw frame → message interpretation → signals; alternative DBC interpretations may coexist.
- ISO-TP: reassembled PDU → first/consecutive/related flow-control frames as applicable, with incomplete/timeout/sequence-error status.
- UDS: transaction → request/response/negative response → ISO-TP messages → frames; correlate using protocol session and timing state.
- XCP: commands/responses and DAQ events → source CAN frames with packet-level interpretation.
- CANopen: PDO, SDO, NMT, EMCY → object-specific fields.
- Future protocols register their own event types/columns rather than pretending Ethernet events are CAN frames.

**Hierarchy is a presentation relation, not a mutation of the captured event stream.** A source frame can appear in contextual inspection without being duplicated in the raw capture. Decoder events keep start/end timestamps and a clearly defined anchor for timeline ordering.

Hierarchy-aware filters offer `MatchOnly`, `PreserveAncestors` (recommended presentation default) and `IncludeRelatedFrames`; result-count semantics distinguish matching events from context-only rows.

## 4. Difference engine

Implement a reusable analysis service distinct from the UI.

- Byte diff: two compatible CAN/CAN FD frames, highlighting byte/bit changes, additions/removals due to changed payload length; compare previous occurrence, manually selected A/B or fixed baseline.
- Signal diff: compare decoded raw and physical values, units, enums, multiplexed signal presence, quality and provenance. Show absolute numerical delta and percentage only when meaningful (no division by zero).
- Never silently compare physical values decoded under incompatible DBC interpretations; report ambiguous source/decoder revisions explicitly.
- Show timestamp delta, source channel, source frame IDs and applied decoder description.
- Cross-recording comparison is a later extension, not required for initial byte/signal diff.
- Diff results cannot modify raw frames, decoding databases or the live session.

## 5. History and query architecture

```text
HAL / Capture Core (immutable raw batches)
                  |
          Trace History Service
             /           \
       RAM Ringbuffer   Optional Disk History
             \           /
        Query & Index Service
                 |
        Virtualized Trace Models
                 |
        Slint / pbSlintDock panels
```

**RAM history:** shared across Trace panels by source/session, configurable hard memory budget; batch-oriented storage and deterministic oldest-event eviction. Latest values, summary rows and virtualized visible slices are separate derived state. The UI never creates one object per stored event. Retention eviction is *not* a capture drop.

**Disk history:** optional backing for older events; segment/index organization supports seek, lazy paging and cancellable search/sort. Reuse compatible reader/writer infrastructure with the recorder where practical, while distinguishing temporary analysis history from an explicitly started, durable recording. The durable recorder's format must continue to satisfy accepted `REQ-LOG-001` (direct interoperable standard, exact format pending). No extra CANexus cache/history files are written into the ECU checkout by default.

**Indexes:** sequence → position, time range → position, channel and CAN ID → candidate ranges; richer decoded-field filters execute a bounded query plan without requiring a permanent index for every possible column. Query snapshots/generations ensure stale work never overwrites a changed panel filter or another worktree's result.

**Filters:** view filters do not affect raw capture or durable recording. Hardware acceptance, processing/subscription, recorder and view filters are visibly separate scopes.

**Overload:** capture loss, recorder drops, protocol decode incompleteness and RAM history eviction have distinct diagnostics. Expensive background analysis, disk I/O and GUI refresh must not stall acquisition or recording. Timelines from independent clock domains show correlation quality instead of pretending perfect ordering.

## 6. API boundaries

Proposed logical Rust modules/crates (exact Cargo split subject to prototype):
- `canexus-trace-data`: raw/decoded event identities and references.
- `canexus-trace-history`: RAM ring, optional disk store, retention, indexes.
- `canexus-trace-query`: typed filter AST, sort/search/paging, cancellation.
- `canexus-trace-analysis`: diff and statistics/time rules.
- `canexus-ui::trace`: Slint view models, table, hierarchy and panel state.

The acquisition HAL and recording engine retain ownership of their independent data flows. Plugins contribute structured protocol events and typed column descriptors; they do **not** mutate Slint rows. Shared search, cursor, time-navigation and source identity services provide links to Plot/Signal Explorer.

Conceptual query API:

```rust
pub struct TraceQuery {
    pub source: TraceSourceId,
    pub filter: FilterExpression,
    pub sort: Vec<SortKey>,
    pub hierarchy: HierarchyMode,
}

pub struct TracePageRequest {
    pub query: TraceQueryId,
    pub cursor: Option<PageCursor>,
    pub limit: usize,
}

pub trait TraceQueryService {
    async fn execute(&self, query: TraceQuery) -> Result<TraceQueryId, TraceError>;
    async fn page(&self, request: TracePageRequest) -> Result<TracePage, TraceError>;
    async fn cancel(&self, query: TraceQueryId) -> Result<(), TraceError>;
}
```

Use stable source-specific IDs and query generations. Query operations produce bounded pages, never an unbounded whole-dataset `Vec`. The UI prefetches only a bounded region around visible rows. Rust traits are internal contracts; external plugin ABI requires its own versioned interfaces.

## 7. Requirements — additions to the canonical Trace specification

Preserve all existing `REQ-TRACE-001` through `028` without reusing IDs. The following detailed requirements are **Proposed**:

| ID | Priority | Status | Requirement |
|---|---|---|---|
| REQ-TRACE-029 | P0 | Proposed | Raw CAN frames and derived protocol events are displayed hierarchically. |
| REQ-TRACE-030 | P0 | Proposed | Every decoded event references its available source frames with provenance. |
| REQ-TRACE-031 | P0 | Proposed | One raw frame may have multiple distinct decoder interpretations without modification. |
| REQ-TRACE-032 | P1 | Proposed | Protocol plugins contribute typed events, protocol fields and grouping keys. |
| REQ-TRACE-033 | P1 | Proposed | Incomplete, timed-out and malformed protocol transactions remain observable. |
| REQ-TRACE-034 | P1 | Proposed | Stored raw frames can be decoded later without reacquiring hardware traffic. |
| REQ-TRACE-035 | P0 | Proposed | Provide byte-level diff for two CAN/CAN FD frames. |
| REQ-TRACE-036 | P1 | Proposed | Highlight changed bits and differing payload lengths. |
| REQ-TRACE-037 | P1 | Proposed | Compare raw and physical decoded signal values, with decoder compatibility checks. |
| REQ-TRACE-038 | P1 | Proposed | Compare with previous occurrence, a selected event or a pinned baseline. |
| REQ-TRACE-039 | P1 | Proposed | Diff results show time delta, source and interpretation provenance. |
| REQ-TRACE-040 | P1 | Proposed | Statistics reside in an independent dockable panel, synchronized with Trace navigation. |
| REQ-TRACE-041 | P1 | Proposed | Provide message counts, rates, cycle-time statistics and windowed measurements. |
| REQ-TRACE-042 | P1 | Proposed | Support explicit periodicity, jitter and timeout rules. |
| REQ-TRACE-043 | P1 | Proposed | Statistics expose data scope, time quality and effects of missing data. |
| REQ-TRACE-044 | P1 | Proposed | Navigate between Trace, Statistics and Plot using event/time references. |
| REQ-TRACE-045 | P0 | Proposed | Provide a configurable, memory-bounded shared trace RAM ring buffer. |
| REQ-TRACE-046 | P1 | Proposed | Support optional indexed disk-backed trace history. |
| REQ-TRACE-047 | P0 | Proposed | Query trace data through virtualized, paginated slices rather than allocating GUI objects per event. |
| REQ-TRACE-048 | P1 | Proposed | Perform indexing, search and large sorts asynchronously with cancellation and stale-query suppression. |
| REQ-TRACE-049 | P0 | Proposed | View filters never implicitly modify raw capture or persisted recording. |
| REQ-TRACE-050 | P1 | Proposed | Differentiate history eviction, capture drops, decoding gaps and recorder drops. |
| REQ-TRACE-051 | P1 | Proposed | Multiple independent trace panels share a history source but retain separate view state. |
| REQ-TRACE-052 | P1 | Proposed | Hierarchy-aware filtering supports direct matches, ancestor context and related-frame inclusion. |
| REQ-TRACE-053 | P0 | Proposed | Chronological and Fixed/Overwrite are the two primary Trace modes; statistics are separate. |
| REQ-TRACE-054 | P0 | Proposed | Pause View, Follow Live and Jump to Live never stop capture or recording. |
| REQ-TRACE-055 | P1 | Proposed | Protocol Inspector and Difference View are dockable and synchronizable. |
| REQ-TRACE-056 | P1 | Proposed | Allow switching between raw and symbolic representations without mutating events. |
| REQ-TRACE-057 | P1 | Proposed | Support reusable local presets for highlights, filters and column configuration. |
| REQ-TRACE-058 | P1 | Proposed | GUI holds only visible rows plus bounded prefetch, never entire recorded histories. |

The confirmed product choices are recorded above; granular requirements and performance thresholds await acceptance. Statistics are **not** a third Trace display mode. This supersedes any earlier suggestion of an integrated Statistics mode, not the accepted chronological/fixed decision.

## 8. Acceptance tests

1. Switch Chronological → Fixed → Chronological under load: capture and independent recording are uninterrupted.
2. Load 1,000,000 frames: GUI memory stays bounded; scrolling remains virtualized.
3. Open two Trace panels on the same source with different filters, sorts and presets: no cross-panel state leakage.
4. Decode a multi-frame ISO-TP PDU and UDS transaction: all contributing raw frames remain addressable with correct provenance.
5. Remove an ISO-TP consecutive frame: show incomplete/sequence diagnostic rather than silently completing a transaction.
6. Enable two competing DBC interpretations of the same CAN ID: both can be shown, raw input unchanged.
7. Diff two CAN FD frames with two changed bytes and different payload length: exact byte/bit positions and lengths are reported.
8. Diff compatible decoded signals: raw/physical deltas and unit handling match fixture expectations.
9. Diff incompatible decoder revisions: no unqualified physical delta.
10. Fixed view receives high-frequency updates: deterministic row identity, correct counts and stable controlled sorting.
11. Known period and timeout rules: expected violations and timestamp caveats are shown; missing events are not invented as captured frames.
12. Exhaust RAM budget: older entries are evicted with explicit retention status, not reported as bus-frame capture loss.
13. Enable disk history and query older evicted events: page/search without loading full history in GUI RAM.
14. Disable disk history: no hidden persistent recorder is created.
15. Change view filter while recording: recorded data remains unchanged.
16. Search/sort a large offline trace and cancel: unrelated GUI and acquisition remain responsive; stale search results are ignored.
17. Navigate Plot → Trace in two known clock domains: correct positions or explicit uncertain-correlation warning.
18. Open two ECU worktrees: no cross-context history, decoded data or persisted view-state contamination.
19. Verify no-op project open/close leaves the ECU Git checkout unchanged.
20. Execute 100-million-event disk-fixture seek/search benchmark with bounded memory and no full GUI materialization.

## 9. Initial performance targets (proposed, not verified)

| Metric | Target / scope |
|---|---|
| Trace navigation | 60 FPS target (p95 frame time ≤ 16.7 ms) on defined reference hardware |
| Visible UI input feedback | p95 ≤ 50 ms |
| Prepared view switch | p95 ≤ 100 ms |
| Byte diff on CAN FD frame pair | p95 ≤ 1 ms pure CPU calculation |
| Large query cancellation acknowledgement | p95 ≤ 100 ms |
| Default live UI refresh | 30 Hz target, configurable |
| Memory | Hard configurable RAM history budget, separate bounded query/UI caches |
| Datasets | Functional 100k, standard 1M, stress/disk 100M events |

These are hypotheses pending reference-machine, load-profile, warm/cold-cache and measurement-method definition. They must not be misrepresented as implementation guarantees.

## 10. Implementation order

1. **P0 / MVP:** robust raw CAN/CAN FD chronological + fixed virtual Trace, filters/sorts, shared bounded RAM history, independent view state, byte diff, pause/follow and capture isolation.
2. **P1:** DBC interpretation, ISO-TP/UDS hierarchical events, signal diff, separate Statistics panel, timing rules, time-cursor and Plot synchronization.
3. **P2 / later scaling:** optional disk-backed history, larger indexes, 100M-event workloads and cross-recording comparison.

P1 protocol concepts and optional disk APIs should be considered early so the MVP hot path need not be rewritten.

## 11. Open decisions

1. Should protocol output appear as raw-first rows with expandable related events, or as equal-status mixed rows?
2. Should Fixed-mode sorting remain stable between explicit reorder passes or continuously reorder based on live data?
3. Is disk history an automatic temporary extension or enabled explicitly per session?
4. What are the exact default columns, ID/byte formatting, grouping details, regex/search language and hierarchy filter match counts?
5. Which standard recording format/reader APIs satisfy `REQ-LOG-001`?
6. What reference hardware, load mix and RAM budgets make the proposed performance gates meaningful?

**No GUI or protocol implementation is claimed; this ADR documents the selected direction and proposed verifiable requirements.**
