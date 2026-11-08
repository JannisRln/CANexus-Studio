# ADR-011 — High-Performance Core Architecture and Data Flow

**Decision direction:** User requested continuation following recommended architecture (2026-11-08).  
**Detail status:** Proposed · **Revision:** 1.0  
**Related:** [HAL and Plugins](05-hal-plugins.md), [UX and Performance](02-ux-performance.md), [Measurement and Time](12-measurement-dashboard-time.md), [UI Plugins and Commands](16-ui-plugins-commands.md), [ADR-007](19-adr-007-elf-dwarf-symbol-calibration.md), [ADR-009](20-adr-009-high-performance-can-trace.md), [ADR-010](21-adr-010-measurement-calibration-workspace-ux.md).

## 1. Context and architectural direction

CANexus-Studio is a **modular in-process Rust monolith** with a headless-capable core, Slint + pbSlintDock GUI and independent bounded capture, recording, protocol, measurement and presentation paths. The architecture targets both high CAN/CAN FD throughput and excellent perceived UI responsiveness. It avoids mandatory IPC between core and GUI, but leaves room for isolated external plugins in future.

**Core rule:** A slow/closed GUI, decoder, script or query must never synchronously block hardware reception or conceal actual capture/recording loss. An overloaded system cannot guarantee losslessness; it must explicitly diagnose and account for observed loss.

Selected implementation *starting choices*:
- Dedicated capture worker per hardware-adapter context (not necessarily each channel), with independently scheduled recorder worker(s).
- Immutable owned batches, shared in-process via reference counting and bounded memory accounting.
- Hybrid scheduling: dedicated workers for latency-sensitive capture/recording; bounded CPU pools for decoding/index/search; async tasks for control-plane/I/O as appropriate; Slint event loop for UI only.
- Bounded queues and distinct per-consumer overflow policies. For **durable recorder queue overflow**, stop the recording with an explicit failure/loss status (fail closed), never silently continue as complete.
- Shared acquisition for compatible measurements; GUI rendering is coalesced and rate-limited, not proportional to captured frame count.
- Trusted native plugins in-process initially behind an explicitly versioned external ABI; future optional isolation. Trusted native code can still crash the process.
- In MVP, closing a project stops its project-bound sessions and finalizes recordings under a documented shutdown policy; closing a panel does *not* close shared sessions.
- Candidate batching threshold: 256 events, with flush at 1 ms or at a hard maximum of 1,024 events; subject to benchmarking, not universal hard guarantees.

The recording standard/file-format choice is still open per accepted `REQ-LOG-001`; this ADR does not introduce a proprietary mandatory intermediate format.

## 2. Logical data flow

```text
   Hardware / Virtual CAN HAL / future network adapters
                          |
             Capture workers / ingress
          normalization / source / clock / seq
                          |
                 Immutable raw batches
                          |
          +---------------+----------------+
          |               |                |
       Recorder       Trace History    Protocol Decoders
        queue          RAM / disk        ISO-TP, UDS,
       + writer         + indexes        XCP, DBC, ...
          |               |                |
       durable log    Trace Query      Derived Events
                          |                |
                        ViewModels     Measurement Engine
                          |          latest/history/samples
                          +--------+-------+
                                   |
                            App Services
                symbols / XCP / calibration / commands
                                   |
                           Slint + pbSlintDock
```

Arrows show logical information flow, **not** implicit thread creation or additional payload cloning. XCP transport and measurement-provider control paths have explicit HAL/session access in addition to receiving events; they are not required to pass hardware writes through Trace Query.

## 3. Event model, identity and time

Three distinct classes:
1. **Raw events:** immutable actual receive/transmit/error/bus-state observations plus explicit loss diagnostics.
2. **Derived protocol events:** decoder-produced interpretations linked to raw event sources; re-decodable on resource/version changes.
3. **Measurement samples:** typed time-stamped signal/symbol values with quality and source identity.

Example *internal Rust* shapes (not external SDK ABI):

```rust
pub struct RawEventId {
    pub source: SourceId,
    pub sequence: u64,
}

pub struct CanFrame {
    pub id: CanId,
    pub flags: CanFrameFlags,
    pub dlc: u8,
    pub data_len: u8,
    pub data: [u8; 64],
}

pub enum RawEventKind {
    CanFrame(CanFrame),
    CanError(CanErrorEvent),
    BusState(BusStateEvent),
    TxResult(TxResultEvent),
    CaptureGap(CaptureGapEvent),
}

pub struct RawEvent {
    pub id: RawEventId,
    pub channel: ChannelId,
    pub timestamp: EventTimestamp,
    pub direction: EventDirection,
    pub kind: RawEventKind,
}

pub struct EventTimestamp {
    pub ticks: u64,
    pub domain: ClockDomainId,
    pub resolution: TimestampResolution,
    pub origin: TimestampOrigin,
    pub quality: TimestampQuality,
}
```

`CanId` distinguishes 11/29-bit identifiers. Preserve DLC separately from actual length, distinguish CAN Classic/CAN FD and applicable BRS/ESI/other flags. Do not allocate decoded strings per raw event. CAN-specific payload layouts must not constrain future Ethernet events: keep the event envelope/network-specific representation extensible.

**Ordering:** Assign a monotonically increasing sequence per source when ingress accepts an event. Preserve adapter-supplied hardware order information when available. Sequence is not a universal order across unsynchronized sources. Clock correlation must store offset/drift, uncertainty, reset/wrap/discontinuity and validity ranges; neither merging nor UI navigation may claim unverified nanosecond precision.

**Identity and worktrees:** Runtime `ProjectInstanceId`, `SessionId`, source/channel identity and generation are distinct from portable symbol identities. The same project manifest in two worktrees must never share mutable runtime bindings or generated caches without explicit artifact-identity checks.

**Derived events:** `DecodedEventId`, decoder ID/version, description resource version, time start/end and source-frame ranges support hierarchical DBC/ISO-TP/UDS/XCP/CANopen analysis per ADR-009. One raw frame may participate in multiple interpretations.

## 4. Ownership, batching and copy minimization

Prefer **copy minimization over dogmatic zero-copy**:
- Copy/normalize driver-owned buffers into CANexus-owned event storage when required for safe lifetime, stable layout and consistent timestamps.
- Build `EventBatch` in preallocated/bounded staging memory; freeze on publish.
- Fan out `Arc<EventBatch>` (or equivalent shared immutable reference) instead of copying full payloads for recorder, history and protocol consumers.
- Consumers may materialize their own purpose-built indexes/samples; no promise of zero copies across process boundaries or incompatible device SDKs.
- Avoid per-frame heap allocations in steady-state capture and per-frame UI objects.

Example:

```rust
pub struct EventBatch {
    pub source: SourceId,
    pub first_sequence: u64,
    pub events: Box<[RawEvent]>,
}
```

Batch flush candidate: 256 events, 1 ms maximum staging wait, hard max 1,024; validate burst and sparse-traffic behavior separately. A single application-wide `Arc` count does not enforce a memory budget: trace/recorder/decoder consumers require queue admission controls, explicit pinned-memory accounting and finite cache limits. Batches must not be retained indefinitely by unbounded plugin subscriptions.

Initial hot-path storage is AoS; columnar indexes/SoA can be used by offline Trace Query if profiling justifies the translation.

## 5. Execution domains and threading

| Domain | Work | Restriction |
|---|---|---|
| Capture worker(s) | Read adapter batches, normalize, sequence, publish, report receive errors | No disk writes, symbol resolution, decode, GUI operations or waiting indefinitely for consumers |
| Recorder worker(s) | Write direct interoperable standard format and finalize | Own bounded queue and fault status |
| Protocol workers | Stateful DBC/ISO-TP/UDS/XCP/CANopen decoding | Bounded input/output, preserve required per-context ordering |
| Measurement workers | DAQ/polling planning, shared subscriptions, sample distribution | No direct Slint dependency |
| Query/analysis workers | Trace sort/filter/search, diff, statistics, index | Cancellable, bounded, generation-checked |
| Background I/O | ELF/DWARF indexing, metadata/project loading, optional disk-history indexing | Must not block GUI/capture |
| Slint UI event loop | Input/layout/render and bounded ViewModel updates | No blocking hardware/long file operations |

No thread per panel. Avoid global mutexes that form a lock convoy across capture, query and GUI; prefer single-owner service state, message passing and short bounded locks. Async is a control/I/O mechanism, not a requirement to run latency-critical receives through the same shared executor.

## 6. Queues, overflow and data quality

All asynchronous producer/consumer edges specify a finite byte/item budget, an overflow policy, counters and health signals.

| Consumer | Policy when overwhelmed |
|---|---|
| Hardware ingress | Drain promptly; report driver overflow (unknown exact missing frames when driver lacks sequence evidence) |
| Recorder | Bounded queue; **stop recording with explicit failed/loss status** on overflow; never silently continue as complete |
| Trace RAM ring | Evict oldest retained history, report history eviction rather than capture loss |
| Optional trace disk history | Its own bounded I/O resources; missing disk history must not be called successful recording |
| Protocol decoder | Signal decode gap; clear/re-synchronize affected stateful context deterministically |
| Fixed Trace / Numeric | Coalesce visual updates and show newest state plus counts/quality |
| Time Plot | Level-of-detail rendering, keep original samples only according to specified measurement/recording retention |
| UI/third-party panel | Bounded view snapshots; skip stale presentation updates |
| Python/plugin consumer | Explicit capability, processing budget and cancellation/failure path |

Sample gap representation:

```rust
pub struct DataGap {
    pub source: SourceId,
    pub first_missing_sequence: Option<u64>,
    pub last_missing_sequence: Option<u64>,
    pub estimated_count: Option<u64>,
    pub reason: GapReason,
}
```

Distinct categories include `DriverOverflow`, `IngressOverflow`, `RecorderQueueOverflow`, `DecoderInputGap`, `HistoryEvicted`, `PluginConsumerOverflow`. A driver drop counter must not be translated into invented sequence ranges. Overflow reporting itself must remain available when the affected data queue is full (e.g., separate bounded health path and persistent summary).

**Recording failure:** On an observed recorder queue overflow, mark recording incomplete, reject further recording writes as appropriate, stop/finalize with error metadata where format permits and expose the affected interval/count if known. Capture may continue independently. Unexpected process/power loss remains a separate recovery concern.

## 7. Memory budget and admission

Proposed starting configuration for benchmarking only:

| Budget class | Candidate |
|---|---:|
| Raw Trace RAM history | 256 MiB |
| Recorder queue | 64 MiB |
| Protocol input/output buffers | 32 MiB |
| Measurement live history | 128 MiB |
| Trace query/index caches | 128 MiB |
| GUI view caches | 32 MiB |
| **Subtotal of configured budgets** | **640 MiB** |

This is **not** a whole-process memory limit; binaries, runtime, debug indexes, drivers and plugins add to it. Account shared event batches once in global backing allocation yet enforce each consumer's retained-reference budget. Bound pool allocation, burst admission, cache eviction and pending jobs; instrument pinned batches and memory high-water marks. Reserve independent critical headroom for capture and the recorder.

## 8. Measurement integration (ADR-008 boundary)

ADR-008 remains an unsaved design draft. This ADR specifies its architectural dependency but does not pretend remaining measurement decisions are accepted.

Measurement requests identify source/session/artifact/signal, requested rate and delivery mode (latest, samples or history). A provider planner deduplicates compatible physical acquisition (e.g. DAQ or polling). A 1 kHz plot and 20 Hz / 5 Hz Numeric panels may share the same supported 1 kHz acquisition while independently rate-limiting display delivery.

**Independently bounded stores:**
- `LatestValueCache`: latest value, time, quality.
- `LiveHistory`: bounded time/sample history, optionally LOD for UI.
- `Recorder`: separate durable writer and error semantics.

A panel owns a subscription, not the XCP session. Releasing a panel subscription only stops physical acquisition when no other consumer needs it. Closing an entire project stops project-bound sessions for the MVP; an eventual "background project session" mode is deferred.

## 9. Plugin and application API boundaries

Internal Rust traits are allowed within the workspace but **not** used as an assumed stable dynamic-library Rust ABI.

External versioned native SDK:
- Manifest: unique ID, version, SDK/API compatibility, supported network/protocol/UI capabilities and permissions.
- Hardware adapter: discovery, capabilities, open/close, batch RX, TX, timestamps, errors.
- Protocol decoder: bounded batches in, typed derived events out; gap/reset callback; per-context ordering, output limits.
- Data format: bounded streaming reader/writer.
- UI plugin: stable panel types/IDs, state serialization, lifecycle hooks, bounded data delivery and Slint-thread compliance.
- Commands: all GUI/shortcuts/macros/scripts/plugin actions dispatch through permission-aware application services.

In-process native extensions can corrupt shared memory or crash the host; `catch_unwind` is not a security boundary. For the MVP trust native plugins and provide compatibility diagnostics; explore process isolation or WASM where justified after an SDK proof of concept on Windows and Linux. A generic network SDK must not imply all networks have CAN semantics.

## 10. GUI integration

Slint + pbSlintDock consume application-owned `TraceViewModel`, `NumericViewModel`, `PlotViewModel` and `CalibrationViewModel` rather than capture queues. Each model has bounded visible data and independent local configuration.

- Trace: paginated virtual rows, filter/sort query generation, optional bounded prefetch; Chronological and Fixed per ADR-009.
- Fixed/Numeric: latest-value snapshots, batched visible updates.
- Plot: bounded sample delivery, LOD rendering with raw data retained under the relevant store's policy.
- Calibration: typed command/result state; never write solely because an input changed.
- Use a single pending coalesced UI notification per relevant model; no event-loop callback per frame.
- A stale query/symbol/ELF/session generation must never replace a newer state. Closing/recreating a panel does not terminate other consumers.

No loaded project waits synchronously for optional ELF/DBC indexing before its main UI can respond.

## 11. ELF/DWARF, XCP and project isolation (ADR-007 and ADR-010)

Resolve `SymbolAliasId`/`SymbolKey` separately from build-dependent address and target representation. A resolved `AddressBinding` and any XCP read/write eligibility are scoped to the current artifact, firmware and XCP session generation. Changes to artifact identity, target memory map, ECU configuration or connection invalidate relevant bindings.

**Required stale-write scenario:** Read/edit parameter under firmware A; load firmware B or reconnect; click Write → reject old address binding until safely re-resolved and validated. Ordinary ECU security is respected. ADR-007's unknown-identity warning and confirmed-mismatch acknowledgement stay in force, without introducing a special development authorization mode.

Project-instance IDs and cache keys prevent mixing distinct worktrees. Workspace layout, indexes, trace caches and machine-local hardware mapping live outside ECU checkout; relative project file paths stay governed by ADR-006.

## 12. Errors, lifecycle and shutdown

Representative lifecycle: `Created → Initializing → Ready → Running → Stopping → Stopped`, with a `Faulted` branch and explicit observed fault causes.

| Failure | Required consequence |
|---|---|
| Adapter disconnect/bus-off | Expose device state; stop unsafe TX; invalidate affected bindings |
| Slow/full recorder disk | Fault recording; no silent success |
| Decoder lag or invalid output | Gap/reset or reject affected derived events with diagnostics |
| Query cancelled/filter changed | Invalid old generation; drop stale results |
| XCP timeout | Result error; retain local calibration draft |
| ELF replaced | Invalidate affected symbol/XCP bindings |
| UI panel closed/plugin missing | Release panel only; preserve other services and recoverable layout placeholder |
| Untrusted/invalid plugin | Refuse incompatible plugin or isolate when supported |

Recommended project shutdown order:
1. Reject new TX/calibration commands and stop active TX schedules.
2. Quiesce/stop adapter input and publish remaining batches.
3. Flush/drain recorder within documented timeout and finalize file or explicitly report failure.
4. Stop measurement/protocol consumers; invalidate session bindings and close hardware.
5. Dispose project-bound view models and mark project stopped.

A failed device or unresponsive driver may require timeout/forced abandonment, with accurate finalization status. Do not wait indefinitely.

## 13. Performance hypotheses (not verified)

| Metric | Initial target |
|---|---|
| Steady-state capture | No unnecessary per-frame heap allocation |
| Ingress availability → batch publish | p99 < 2 ms in defined synthetic profile |
| Fan-out | Shared batch payloads, no full copy per consumer |
| GUI visible input feedback | p95 < 50 ms |
| Live Trace scrolling | Target 60 FPS |
| Live plot rendering | Target 60 FPS under specified display workload |
| Warm panel switch | p95 < 100 ms |
| Query cancellation acknowledgement | p95 < 100 ms |
| Memory | Bounded according to configured queue, cache and history budgets |
| Recording correctness | No **unreported** gaps; failed state on actual recorder overflow |
| Project open | GUI interactive before optional large ELF indexing completes |

These metrics are *targets*, not verified guarantees. Define hardware, OS, virtual load profiles, warm/cold cache, p95/p99 sample windows, memory measurement and whether hardware/driver buffering is included. Ingress-to-publish starts when the receive worker obtains the event; it is not an end-to-end on-wire hardware timing guarantee.

## 14. Proposed additional requirements

Preserve existing `REQ-CORE-001..006`, `REQ-HAL-*`, `REQ-PLG-*` and the accepted product decisions. New IDs below are additive; priorities/status are proposed.

| ID | Priority | Status | Requirement |
|---|---|---|---|
| REQ-ARCH-001 | P0 | Proposed | Core can operate headlessly, without Slint or a GUI event loop. |
| REQ-ARCH-002 | P0 | Proposed | Published raw events are immutable and preserve captured bytes and metadata. |
| REQ-ARCH-003 | P0 | Proposed | Raw events have source-scoped sequence identity and runtime session generation. |
| REQ-ARCH-004 | P0 | Proposed | Timestamps preserve clock domain, origin, resolution and quality. |
| REQ-ARCH-005 | P0 | Proposed | Published event batches can be fanned out via shared immutable references without per-consumer payload copies. |
| REQ-ARCH-006 | P0 | Proposed | Stationary capture avoids unnecessary per-frame allocation and has bounded staging memory. |
| REQ-ARCH-007 | P0 | Proposed | Pinned/shared batch ownership is accounted for so consumers cannot bypass total memory budgets. |
| REQ-ARCH-008 | P1 | Proposed | Derived protocol events link to their raw sources and carry decoder provenance. |
| REQ-ARCH-009 | P0 | Proposed | Capture is scheduled independently from GUI, queries and slow secondary consumers. |
| REQ-ARCH-010 | P0 | Proposed | Asynchronous consumer queues have defined finite byte/item budgets. |
| REQ-ARCH-011 | P0 | Proposed | Overflow and actual loss are observable, attributable and never silently misrepresented. |
| REQ-ARCH-012 | P0 | Proposed | UI may coalesce visual updates without modifying underlying capture or recording. |
| REQ-ARCH-013 | P0 | Proposed | Recorder uses independent bounded queue and worker and cannot indefinitely block capture. |
| REQ-ARCH-014 | P1 | Proposed | Stateful decoders receive explicit input-gap notifications and deterministic reset/resynchronization behavior. |
| REQ-ARCH-015 | P0 | Proposed | Capture hot path does not perform blocking GUI, disk indexing, symbol resolution or protocol decoding. |
| REQ-ARCH-016 | P1 | Proposed | Asynchronous query work is cancellable and its results are protected by generations. |
| REQ-ARCH-017 | P0 | Proposed | Compatible measurement consumers share the same provider acquisition. |
| REQ-ARCH-018 | P0 | Proposed | Measurement subscriptions have explicit owners and outlive transient panels when another consumer needs them. |
| REQ-ARCH-019 | P0 | Proposed | Simultaneously opened projects and worktrees have isolated runtime/session/asset contexts. |
| REQ-ARCH-020 | P0 | Proposed | XCP address bindings are coupled to artifact, firmware/context and session generations. |
| REQ-ARCH-021 | P0 | Proposed | Stale address bindings cannot be used to perform ECU writes. |
| REQ-ARCH-022 | P1 | Proposed | Clock correlations preserve source domain, uncertainty, drift and validity intervals. |
| REQ-ARCH-023 | P0 | Proposed | External native plugin ABI is explicit and versioned rather than relying on unstable Rust ABI. |
| REQ-ARCH-024 | P0 | Proposed | Plugin manifests declare identity, compatibility, capabilities and required permissions. |
| REQ-ARCH-025 | P1 | Proposed | Plugin consumers have bounded processing/input/output resources with observable failures. |
| REQ-ARCH-026 | P1 | Proposed | Missing or incompatible plugins yield diagnostics and recoverable UI state. |
| REQ-ARCH-027 | P0 | Proposed | Slint views consume bounded application view models, not capture queues directly. |
| REQ-ARCH-028 | P0 | Proposed | GUI updates are coalesced and cannot enqueue unlimited event-loop callbacks. |
| REQ-ARCH-029 | P1 | Proposed | Third-party UI panels respect Slint thread ownership, lifecycle and subscription ownership. |
| REQ-ARCH-030 | P0 | Proposed | Critical services have typed lifecycle, shutdown and fault states. |
| REQ-ARCH-031 | P0 | Proposed | Shutdown stops unsafe TX, drains/finalizes recorder when possible and reports failed finalization. |
| REQ-ARCH-032 | P0 | Proposed | Driver/capture gaps, decoder gaps, recorder drops and history eviction have distinct statuses. |
| REQ-ARCH-033 | P1 | Proposed | Services expose machine-readable throughput, latency, queue, memory and error metrics. |
| REQ-ARCH-034 | P1 | Proposed | Virtual providers support deterministic end-to-end and failure-injection testing. |
| REQ-ARCH-035 | P1 | Proposed | Benchmarks enforce agreed performance targets under documented Windows/Linux reference conditions. |
| REQ-ARCH-036 | P1 | Proposed | Runtime caches/records and machine-specific state are outside ECU worktrees by default. |

## 15. Acceptance tests

1. Inject 1,000,000 CAN Classic/CAN FD events via virtual adapter; verify event IDs, raw data, sequence and flags, including sparse and burst traffic.
2. Fan one event batch to recorder, history and decoder; instrument and verify shared backing memory/no full payload clone per consumer.
3. Block a Trace panel intentionally; capture and recorder continue, UI queue stays bounded.
4. Make recorder slower than input until queue overflows; recording transitions to **failed/incomplete** and capture remains independently operational.
5. Fill the Trace RAM budget: oldest retained events evicted; eviction does not increment capture-drop counter.
6. Remove an ISO-TP constituent frame: decoder emits incomplete/gap behavior and resynchronizes without fabricated successful reassembly.
7. Attach two Numeric panels to the same compatible simulated XCP measurement: one acquisition; removing either panel preserves the other's samples.
8. Replace an ELF or reconnect XCP after reading a parameter: old address binding is rejected for Write.
9. Open two ECU worktrees simultaneously with different DBC/ELF: no mixing of symbol resolutions, caches, events or session bindings.
10. Change Trace filter during long query: prior query result cannot overwrite newer generation.
11. Load a missing/incompatible plugin: structured diagnostic and recoverable workspace placeholder; no assertion that native crashes are isolated.
12. Close project under active recording/TX: TX stopped, recorder finalized or explicitly failed, workers disposed and bindings invalidated.
13. Search/seek in a 100M-event disk-history fixture with bounded GUI memory; no full-trace UI materialization.
14. Feed GUI updates above render capability: coalesce/dismiss redundant updates without capture/recording impairment.
15. Issue invalid XCP write using stale binding: no transport write request occurs.
16. Mix hardware/software timestamp and clock drift/reset fixtures: preserve provenance and show uncertain correlation.
17. Saturate protocol and plugin queues while recording: bounded memory, observable decoder/plugin loss, recording behavior according to its own independent policy.
18. Exercise stop/restart/disconnect/reconnect generations: no ID collision or stale asynchronous callback may attach to a new project/session.
19. Compare headless and GUI virtual-adapter runs for identical raw sequencing and recorder contents.
20. Run benchmarks on defined Windows and Linux reference systems and report throughput, p95/p99 latency, queue occupancy, peak memory, CPU and drop categories.

## 16. Implementation order

1. **Core vertical slice:** Virtual CAN → dedicated capture → immutable batches → bounded RAM history → trace query → virtualized Slint Trace; concurrently add recorder-simulating and intentionally slow consumers.
2. **Robustness:** bounded queue metrics, fault injection, shutdown, timestamp provenance, project isolation, reproducible benchmarks.
3. **Measurement / protocol:** compatible subscriptions and rate planner (finish ADR-008), DBC/ISO-TP derived events, Numeric and Plot services.
4. **XCP/calibration:** artifact-generation binding, explicit parameter read/write/readback states per ADR-007/010.
5. **SDK and scale:** versioned external plugin ABI prototypes, optional disk history, large offline query indexes, Windows/Linux certification.

## 17. Decisions deliberately still open

- Exact external ABI technology (C-compatible API, ABI framework, etc.), trust and signature policy.
- Exact interoperable direct-recording file format, recovery/finalization details.
- Numerical queue/batch budgets and benchmark reference hardware.
- Whether and when disk history automatically spills vs opt-in (ADR-009).
- ADR-008 fine points: consumer-specific acquisition-rate semantics, live-history budget and future background-session mode.
- Reconnect, CPU affinity, thread priorities and exact Rust runtime/executor choices.
- Runtime dynamically loaded Slint UI plugin proof of concept before committing SDK internals.

**Status note:** Architecture direction is recommended and user-requested for documentation. These detailed design requirements are Proposed, not implemented or Verified.
