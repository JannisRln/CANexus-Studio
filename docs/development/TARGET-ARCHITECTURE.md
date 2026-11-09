# CANexus-Studio — Target Software Architecture

**Status:** Proposed technical baseline for review; not a new Accepted product ADR.  
**Repository baseline:** `main` at `5e801ec5e395657d4998f3ce795a1ce16393b64c`, inspected 2026-11-09.  
**Companion:** [Architecture risks and remediation](ARCHITECTURE-REMEDIATION.md).  
**Primary sources:** `AGENTS.md`, `docs/requirements/README.md`, ADR-006/007/008/009/010/011, `docs/requirements/05-hal-plugins.md`, `16-ui-plugins-commands.md`, `17-decisions-v0.7.md`, `24-ai-native-mcp-integration.md` and `docs/development/STATUS.md`.

This document describes an **incremental target**, not functionality already present. Product requirement states remain unchanged: a Proposed requirement is not implicitly Accepted or Verified by this document. The v0.7 accepted product decisions and current architecture constraints take precedence over illustrative implementation details here.

## 1. Verified baseline and engineering strategy

Current Rust workspace: `canexus-core`, `canexus-cli` and `canexus-gui`. Core already provides validated typed Classic CAN/CAN FD frames and timestamp provenance, a generic `Observed<T>` envelope, deterministic virtual-CAN replay, bounded per-consumer batch ingestion, and bounded source-local RAM trace history with CAN filters and paginated reads. The CLI is a bootstrap print-only binary. Slint/pbSlintDock currently demonstrates a single-window docking host and a Trace placeholder; native floating windows, production UI view models, live capture orchestration and independent recorder workers are not implemented. See `STATUS.md` for CI evidence, which does not demonstrate native UI or hardware behavior.

**Strategy:** Preserve verified primitives and add missing contracts around them. Do not replace the working frame model, introduce a cross-process core/UI boundary or split every conceptual component into a crate immediately. First make the virtual CAN end-to-end path production-shaped, measurable and safely bounded, then add real hardware and protocols. Refactor only at a demonstrated dependency boundary, for a correctness issue, or after measurement.

## 2. Non-negotiable boundaries

1. **Headless first:** Capture, record, query, measurements and automated tests work without Slint; the GUI is a client of application-owned services.
2. **In-process hot path:** No mandatory IPC or network service between capture/core and Slint. No synchronous decode, file/index I/O or GUI callback on a capture worker.
3. **Typed data plane:** Keep CAN-specific data compact and validated (`Observed<CanFrame>` today); add other network payload families without routing every event through a CAN-only base type or per-frame `dyn` allocation.
4. **Independent consumers:** Recorder, history, decoder, measurement and view models have different bounded resources and failure policies. A slow Trace view must not silently affect recording.
5. **Observable loss and quality:** Capture/driver loss, per-consumer delivery loss, decoder gap, recorder failure, history eviction and UI coalescing have distinct meanings.
6. **Scoped identity:** An active resource is identified by project instance/worktree context, session generation and source/channel; a source-local sequence is not a globally comparable timestamp or a lifetime-independent identifier.
7. **One permission boundary:** Equivalent operations from Slint, CLI, scripts, plugins and optional AI/MCP pass through common application services and command/authorization checks. Reading an ECU through an active protocol may itself generate traffic.
8. **Portable ECU projects:** One portable CANexus config file in an existing ECU worktree; resource paths resolve relative to that file. Cache, logs, recordings, layouts and physical channel mappings live outside ECU checkouts by default.
9. **No premature plugin ABI promise:** Internal Rust traits are not a stable external dynamic-library ABI. Native plugin and Slint panel loading require explicit cross-platform proofs before SDK freeze.
10. **Offline by default:** Ordinary local operation never depends on AI services, cloud connectivity or external MCP.

## 3. Logical component and dependency map

```text
Slint + pbSlintDock GUI   Headless CLI   Python workers   Optional MCP / assistant
          \                   |                |                    /
                       Application Services
                  commands / authorizations / handles
                                |
                Project & Runtime Session Host
              lifecycle / generations / ownership
                                |
        +-----------------------+-----------------------+
        |                       |                       |
      Capture / HAL       Project resource          Measurement
  hardware, virtual       and symbol services       acquisition planner
        |                   DBC/A2L/EDS              latest/history
        v                       |                       ^
  Typed ingress / immutable shared batches            |
        |                       |                       |
        +---------+-------------+-----------------------+
                  |                   |
      +-----------+------------+      |
      |           |            |      |
   Recorder    RAM/disk      Protocol decoder --> derived events / samples
    writer     Trace Store      DBC/ISO-TP/UDS/XCP/CANopen
      |           |
   Durable    Trace Query / Analysis
   files      filters, indexes, diff, sort
                  |
           Bounded View Models
                  |
                Slint
```

This diagram shows responsibility and permitted data flow, not one thread per box. Recorder and trace storage consume **the same immutable raw batches** independently. XCP/UDS control and TX use explicit protocol/session/adapter APIs, never the Trace query path. Recorder errors must be observable even if its data queue is full.

### Logical modules (crate split is deliberately deferred)

| Boundary | Responsibility | Allowed dependencies | Must not depend on |
|---|---|---|---|
| `core/types` (current `canexus-core`) | Frame, event, ID, time, errors, typed batch contracts | Standard Rust foundations | Slint, vendor SDKs |
| `runtime/session` (future) | Project instance, generation, service lifecycle, cancellation, worker ownership | Core contracts | GUI widgets |
| `hal` (future) | Discovery, capabilities, RX/TX, timestamps, bus state, reconnect | Core, platform adapters | Trace views |
| `ingest/health` (evolve existing core) | Staging, bounded publication, byte accounting, loss/health | Typed core | Filesystem, GUI |
| `trace/store/query` (evolve existing core) | Bounded RAM, optional disk, queries, indexes and query generations | Raw batches, metadata | Device ownership, UI objects |
| `recording` (future) | Independent bounded writer, direct standard-format decision, finalization and fault state | Raw batches, storage I/O | View refresh |
| `protocols` (future) | Stateful reassembly/decoding with versioned provenance and gap resets | Raw event stream, protocol descriptions | Slint |
| `measurement` (future) | Shared acquisition planner, typed samples, cache, bounded history, rates/quality | Provider interfaces | Panel lifecycle |
| `project/symbols/workspace` (future) | Portable resources, local mappings, versioned dashboards/presets | Project and artifact identities | Hardware-specific hardcoded paths |
| `app-services` (future) | Typed read/query/action contracts, authorization, handles, audit | Domain services | Direct Slint internals |
| `canexus-gui` / `canexus-cli` (existing) | Presentation and command-line clients | App services, view-model adapter | Direct device driver control in panels |

Recommended dependency rule: **frontends → application services → domain runtime → typed core / HAL implementations**. Plugin API and GUI components can be separate adapters, not reverse dependencies into core. Begin with internal Rust modules; extract stable crates when compile-time dependency checks and independent tests justify it.

## 4. Event, clock and provenance contracts

- **Raw observations:** Preserve original bytes, CAN flags/DLC, channel, RX/TX direction, source-local sequence and original timestamp domain. Keep error/bus-state/Tx-result/loss observations distinct from ordinary data frames. A future Ethernet payload family must not be forced through `CanFrame`.
- **Runtime-scoped handles:** Introduce `ProjectInstanceId`, `SessionId`, monotonic `Generation` and source identity at the session service boundary. A raw event reference used outside the source-local store must carry enough session/project context to reject stale work. Migrate the existing `SourceId` and `RawEventId` API compatibly rather than replacing them without a plan.
- **Timestamp correlation:** Preserve existing raw `EventTimestamp` unchanged. A separate optional correlation service maps domains to a chosen reference with validity interval, drift, offset and uncertainty; absent correlation means no verified global order. Do not assume two adapters share an epoch.
- **Derived protocol events:** Carry decoder version, resource/description revision, raw event references, completeness status and time range. Different DBC interpretations may coexist; raw frames are never rewritten.
- **Measurement samples:** Typed value, source/session/artifact generation, units via metadata, time quality, sample quality (`Valid`, `Stale`, `Gap`, etc.). Acquisition rates are distinct from per-consumer delivery/paint rates.

## 5. Workers, delivery and memory ownership

| Execution domain | Ownership and behavior |
|---|---|
| Capture (dedicated per adapter context as needed) | Poll hardware/virtual source; normalize/order; batch and publish without indefinite waits; time-based flush for sparse traffic |
| Recorder | Independent bounded queue and writer; on overflow/fault, transition to failed/incomplete and stop finalizing as complete |
| Protocol decode | Context-ordered bounded input; deterministic reset/gap handling; derived output linked to raw IDs |
| Measurement | Provider planning, acquisition sharing, latest cache and bounded history; panel lifetime independent |
| Query / resource index | Bounded background jobs, cancellation and generation-checked results; never hold capture/GUI locks across scans |
| GUI | Slint event loop: only short snapshot application, visible-row rendering and coalesced notifications; no per-frame posted callback |

**Budgeting:** Per-consumer queue item limits are necessary but not sufficient. Introduce byte-size admission and a global per-project/session shared-allocation credit/lease model, charging each backing batch **once** and tracking all live references. Account staged buffers, queue slots, external retained pages/handles, indices and independent recorder headroom. Excess demand must produce an explicit reject/drop/failure classification rather than unbounded growth. Set numerical budgets from measurements (ADR-011 contains candidate values, not accepted guarantees).

**Subscription behavior:** Consumers may arrive/leave while a session runs, but attachment/detachment and ownership must have a well-defined serialization point, generation and per-consumer loss/replay semantics. Current `BoundedIngest::subscribe` is pre-start only; a runtime broker can initially provide dynamic registration without mutating the tested core API unsafely.

**Time/flush:** A capture runtime, not the GUI, must flush a partially full batch within a specified latency budget. `BoundedIngest::push` only publishes on a full batch unless caller explicitly calls `flush`.

## 6. Query and UI model

One `TraceHistory` per appropriate source/session is shared by multiple Trace views. UI panel state (filter, sort, chronological/fixed, columns, selection, follow-live and display rate) is independent. A query service returns bounded **owned view snapshots or short-lived typed leases**, not borrowed Rust references held across background/Slint thread boundaries. Use versioned query generations and cancellable background work; stale results are discarded. Live append must not cause permanent starvation of scrolling queries.

M1 may display virtual-source chronological rows with bounded page sizes before the full hierarchical/100M-event indexed design. Fixed mode, independent sorting, disk history, byte-diff, signal view and real UI float windows are separately tracked deliverables, never implicitly claimed by a simple Trace panel.

Use the design system under `docs/design/` as presentation guidance, not as evidence of functional UI behavior.

## 7. HAL, plugins, scripting and automated interfaces

- **HAL first:** Internal adapter API for discover → capabilities → open/configure → receive/error → authorized transmit → stop/close. State transitions and timestamp metadata are typed. Start with virtual CAN, then SocketCAN on Linux and a verified Windows device/API choice. Machine-local channel mapping is mandatory before physical TX.
- **Native plugins:** Start with trusted in-process built-ins and an independently versioned manifest/capability contract. Compare an explicit C-compatible ABI, constrained process RPC and WASM for specific extension classes. A native in-process plugin is **not crash-isolated**. UI panel plugin loading must prove Slint component/thread/window lifecycle on Windows/Linux before public SDK freeze.
- **Scripts:** Per-session isolated Python processes (accepted product direction). They call authorized services; do not share their interpreter state with capture and never auto-run project scripts.
- **Commands/AI/MCP:** Common typed service facade and permissions come first. AI/MCP requirements are Proposed; opt-in read-only introspection may follow an accepted authorization contract. Unsafe TX, ECU writes, calibration, flashing and arbitrary code require explicit scoped user approval, not model-generated permission.

## 8. Versioned project and application service contracts

Before broad feature implementation define versioned request/response and error shapes for:

1. `ProjectSession`: open/close, current generation, resource identity, local mapping, cancellation.
2. `TraceQuery`: source/session, filter, bounded page/cursor, retention/gap status, cancel, query generation.
3. `CaptureControl` and `Health`: start/stop, capabilities, backlog, loss counters/events and lifecycle.
4. `RecordingControl`: start/stop/status/finalize with explicit incomplete/failure state.
5. `CommandDispatch`: actor, scope, target, side effects, approval and audit record.
6. `MeasurementSubscription`: owner, provider/source generation, requested/configured/observed/delivery rates, status, unsubscribe.

An internal Rust service API may evolve initially; **external plugin/MCP wire contracts require explicit compatibility/version handling**. No specific async runtime or stable Rust trait-object ABI is imposed by this architecture document.

## 9. Open decisions to resolve before freezing interfaces

| Decision | When required | Options / evidence needed |
|---|---|---|
| Stable runtime identity + session generation representation | Before production GUI, recorder and multi-worktree services | Connection/reconnect/stale-query fixtures |
| Global batch accounting and admission policy | Before untrusted/high-throughput consumers | Pinning, overload, short-lived lease benchmarks |
| Capture flush and lifecycle scheduling | Before live end-to-end Trace acceptance | Sparse traffic latency + shutdown fixtures |
| Record format and crash/finalization contract | Before recorder prototype and hardware capture | Standards compatibility, corruption tests, Windows/Linux readers |
| Hardware choice and CAN timing/config capabilities | Before DEV-010 | SocketCAN and Windows adapter capability matrix |
| Timestamp drift/correlation model | Before cross-adapter timeline claims | Reset/wrap/offset/drift fixtures |
| Plugin trust, ABI and dynamic Slint panel loading | Before public SDK freeze | Independent third-party plugin + native-window proofs |
| Project config name/schema and local mapping | Before portable project persistence | Two-worktree and cross-platform path tests |
| Authorization, scripting, AI/MCP exposure | Before agent/device mutations | Scoped permissions, stale handles, deny-by-default tests |
| Benchmark reference hardware and targets | Before accepting performance objectives | Repeatable Windows/Linux load and percentile methodology |

## 10. Implementation sequence and gates

Use [Architecture remediation](ARCHITECTURE-REMEDIATION.md) for risk-to-work-item mapping. Suggested integration order:

- **Gate A — reconcile and baseline:** Integrate/reconcile active PRs without duplicate DEV IDs; preserve accepted v0.7 direction; verify current main CI and document architecture ownership.
- **Gate B — virtual vertical slice:** Runtime/session scope, capture worker flush/stop, robust loss accounting, global batch admission prototype, and a small headless app/trace service. In parallel DEV-008 may deliver a single-window bounded Slint Trace using existing core; it must not claim missing contracts as complete.
- **Gate C — measurement evidence:** 1,000,000-event deterministic end-to-end fixture with overload/failures, Windows/Linux benchmarking, recorder design/prototype and cancellable Trace queries. Performance targets are hypotheses until reference machines and tests establish them.
- **Gate D — usable adapters and decode:** Accepted HAL interface + at least one proven adapter per OS, DBC decoding with provenance, project path resolution, and safe scheduled TX after authorization.
- **Gate E — professional workflows:** ISO-TP/UDS, XCP, CANopen, measurement/plotting, extensible recording and plugin SDK in independently testable vertical slices.
- **Gate F — advanced extensions:** External Slint panels, isolated Python worker workflows, optional guarded MCP/AI and future networks, dependent on accepted contracts.

Every gate needs requirement → implementation → test → evidence traceability, actual CI at the merged SHA, reproducible fault tests and explicit unresolved limitations. A design proposal, green build or a merged PR alone does not make an entire product requirement Verified.
