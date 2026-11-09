# CANexus-Studio — Architecture Risk Register and Remediation Plan

**Status:** Proposed implementation planning and source audit; no product requirement status change.  
**Audited source baseline:** `main` commit `5e801ec5e395657d4998f3ce795a1ce16393b64c`, 2026-11-09. **Coordination rechecked:** `main` at `f6d16e98d95f3c614b81c2b24f252e96eab56756` after PR #19 merged (documentation-only); Core source is unchanged by that merge.  
**Companion:** [Target architecture](TARGET-ARCHITECTURE.md).  
**Evidence categories:** *Observed* = explicit current code/metadata behavior; *Missing* = currently absent from inspected workspace; *Integration risk* = active, unmerged branch/PR collision; *Hypothesis* = investigate with benchmarks before optimization.

This is **not** a defect claim against the intentionally limited scope of already completed DEV tasks. It identifies what must be strengthened before their primitives support a complete application. Current evidence includes `crates/canexus-core/src/{lib,event,time,can,ingest,trace_history,virtual_can}.rs`, `crates/canexus-cli/src/main.rs`, `crates/canexus-gui/{src/main.rs,ui/main.slint}`, `docs/development/{BACKLOG,STATUS,DECISIONS}.md`, GitHub PR #19 and related ADRs. No local benchmark, hardware run or interactive GUI exercise was performed as part of this **documentation-only** review.

## 1. Prioritized findings

| ID | Priority / evidence | Current state and consequence | Targeted remediation | Traceability |
|---|---|---|---|---|
| AR-01 | **Resolved coordination issue** | `main` reserves `DEV-017..021` for AI/MCP planning. PR #19 originally reused 017/018, corrected its IDs to **DEV-022/023**, and was merged to `main` as `f6d16e98`; main Windows/Linux CI run `42575721608` passed. | Keep stable IDs and prevent future collisions through project-state validation and live-branch coordination. No remaining action for the corrected PR #19 ID conflict. | DEV-017..023, project-state validation |
| AR-02 | **P0 observed** | `SourceId(u64)` and `RawEventId { source, sequence }` in `lib.rs` and `event.rs` are source-local but carry no explicit project instance, runtime session, connection generation or worktree context. This is adequate for current fixtures but insufficient to prove stale asynchronous results cannot cross sessions. | Define runtime-scoped `ProjectInstanceId`, `SessionId`, `Generation` and qualified source handles. Check context at service/query/command boundaries; retain simple typed frame metadata where possible. | REQ-ARCH-003/019, REQ-PROJ-003, REQ-AI-031 (Proposed) |
| AR-03 | **P0 observed** | `SharedBatch<T>` wraps `Arc<[Observed<T>]>`; queue batch counts are bounded, but a consumer can hold cloned batches after dequeue. `TraceHistory` explicitly counts inline event bytes, not globally pinned allocations. Process-wide memory remains unbounded by these two limits alone. | Prototype one charge per backing batch and bounded global byte credits/retained leases, with independent recorder headroom and external owner/release accounting. Coordinate with **DEV-023** once PR #19 integrates. | REQ-ARCH-007/010, REQ-TRACE-045; DEV-023 proposed in PR #19 |
| AR-04 | **P0 observed** | `BoundedIngest::push` publishes when staging fills; sparse batches only publish when caller explicitly invokes `flush()` or `shutdown()`. No current capture-worker timed flush exists; a quiet bus may show an old Trace even while events were accepted. | Add a capture-runtime flush deadline independent of UI rendering, plus explicit source shutdown drain. Keep existing `BoundedIngest` behavior deterministic. | REQ-ARCH-006/009/015, REQ-UX-001 |
| AR-05 | **P0 observed** | `BoundedIngest::subscribe` only works in `Ready`, before `start`. There is no runtime broker for attaching a new view/recorder/decoder to an already running source, nor a scoped replay/catch-up policy. | Add a serialized session-owned subscription registry (attach/detach, capacity, owner, generation, start offset, gap semantics). Do not bolt unchecked concurrent mutation onto a single-producer data structure. | REQ-CORE-002, REQ-ARCH-018, REQ-MEAS-004 (Proposed) |
| AR-06 | **P0 observed** | `ConsumerStats` and `HistoryStats` retain cumulative counters and only `last_loss` / `last_input_loss`. `TraceHistory::report_input_loss` relies on an explicit caller; no dedicated durable gap/health notification feed is implemented. Intermediate loss ranges can disappear from summaries when a new loss occurs. | Dedicated bounded health/loss event channel or durable compact loss journal plus monotonic totals, source IDs, range evidence and overflow-of-health diagnostics. Propagate decoder resets and recorder-failed state independently. | REQ-ARCH-011/014/032, REQ-TRACE-050 |
| AR-07 | **P0 missing** | CLI only prints a bootstrap line; GUI Trace is a placeholder; there is no service composition that runs virtual CAN → ingress → RAM history → query → view while handling lifecycle. | Introduce a thin headless runtime/session host and service facade; connect DEV-008's virtual Trace through bounded worker/query snapshots. Do not make Slint directly own an adapter. | DEV-008/009, REQ-ARCH-001/027/028 |
| AR-08 | **P0 missing** | No independent durable recorder with queue, format choice, overflow/finalization failure policy or crash recovery. Capture success must not be equated with file durability. | Make an explicit interoperable format decision, then prototype a bounded recorder worker with fail-closed overflow, disk-full, forced stop and finalization evidence. | DEV-013, REQ-ARCH-013/031, REQ-LOG-001 |
| AR-09 | **P1 observed** | `TraceHistory::page_with` returns borrowed refs, scans retained batches linearly and invalidates cursors on every append. Good initial RAM API, but live UI queries may repeatedly restart and borrow lifetimes complicate worker/UI handoff. | Introduce a query service with bounded owned projection or short-lived pinned snapshot leases; generation/cancellation, stable seek semantics and selective indexes after profiling. Avoid whole-trace materialization or one object per event. | DEV-008/009, REQ-TRACE-047/048 |
| AR-10 | **P1 missing** | `EventTimestamp` preserves ticks/domain/resolution/origin/quality but supplies no clock mapping, drift estimate, uncertainty interval or reset-generation model. Raw per-domain timestamps are correct; cross-adapter chronology is not yet justified. | Separate clock-correlation service; annotate unsynchronized windows and invalid correlations. Test wrap, reset and drift without rewriting raw time. | REQ-TIME-001 (Accepted), REQ-ARCH-022 (Proposed) |
| AR-11 | **P1 missing** | Virtual CAN is a fixture source, not a production HAL contract. There is no concrete SocketCAN/Windows HAL or typed bus-off/reconnect/device-capability service. | Define internal adapter discovery/config/RX/TX/errors/capabilities and hardware permission contracts; implement platform adapters incrementally with device-specific integration tests. | DEV-010/012, REQ-HAL-001..008 |
| AR-12 | **P1 missing** | Native plugin ABI, capability negotiation and crash/trust contract do not exist. Slint docking works as a single-window prototype, but external compiled UI panels and native floating windows remain unverified. | Defer public SDK freeze; validate a real third-party panel and native window lifecycle before choosing ABI/hosting. Track DEV-022 for native float in PR #19; keep untrusted code outside trust boundary when isolation is required. | REQ-PLG-003/012, DEV-022 (PR #19) |
| AR-13 | **P1 missing** | No complete portable project loader, local physical channel mapping, versioned workspace model or stale resource generation checks. Machine-local settings and ECU worktree constraints are specified but not enforced in code. | Implement project schema/path validation, per-worktree runtime identities and local mappings before physical TX and dashboard/AI mutations. No generated runtime files in ECU checkout. | REQ-PROJ-001..012, REQ-ARCH-019/036 |
| AR-14 | **P1 missing** | Common versioned command/authorization facade is not implemented. Future CLI, Python, plugins and MCP could accidentally develop independent unsafe action paths. | Design one capability-aware application command service with explicit side-effect classes, actor/session scope and audit; no implicit TX/UDS/XCP/flash approvals. Accept security decisions before exposing mutations. | REQ-CMD-001 (Accepted), REQ-AI-002/027/028 (Proposed) |
| AR-15 | **P1 missing** | Existing CI validates format, Clippy, build, tests and project-state on Windows/Linux, but does not establish actual GUI interaction, hardware support, 1M-event sustained throughput, p95/p99 latency or process memory high-water. | Add deterministic headless benchmark/failure harness with declared reference machine and load; later real GUI interaction and hardware-in-the-loop jobs. Mark targets aspirational until measured. | DEV-009, REQ-CORE-005, REQ-UX-010 |
| AR-16 | **P2 hypothesis, not a bug** | `CanFrame` stores `[u8; 64]` even for an 0–8-byte Classic frame. This simplifies allocation-free typed ingestion but may inflate memory bandwidth under large Classic-heavy loads. No performance deficit is proven. | Profile representative Classic/FD mixes and AoS vs compact/columnar candidates only after correctness and memory-accounting gates. Preserve API/wire semantics and avoid per-event allocation. | REQ-ARCH-006, ADR-011 |
| AR-17 | **P1 observed documentation drift** | Existing `docs/development/ARCHITECTURE.md` still describes the minimal DEV-002 two-crate workspace although `canexus-gui` and multiple core modules are already present. | Refresh architecture entry page to separate current implementation from proposed target. | Documentation consistency |

**Priority interpretation:** P0 means fix or explicitly contain before relying on the behavior in the next production-shaped vertical slice; P1 means plan before the dependent feature family; P2 means investigate only when measurements justify it. These are architecture-review priorities, not automatically accepted product requirement priorities.

## 2. Execution waves, dependencies and parallel ownership

| Wave | Work | Dependencies / safe ownership | Exit evidence |
|---|---|---|---|
| **W0 — Repo reconciliation** | AR-01 resolved by PR #19; AR-17 updates architecture entry point | PR #19 merged and main CI passed; architecture PR edits neither `BACKLOG.md`, `STATUS.md` nor `DECISIONS.md` | Main retains unique DEV IDs; architecture docs linked after this PR merges |
| **W1 — Source/session correctness** | AR-02, AR-04, AR-05, AR-06: generation-scoped virtual capture host, timed flush, dynamic consumer broker, gap/health semantics | Core/runtime owner only; coordinate API with DEV-008 GUI owner | Fixtures for reconnect, sparse traffic, late subscriber, repeated overload and stop |
| **W1 parallel — Memory admission** | AR-03: global allocated-batch credits, retained-owner tracking, overload status | DEV-023 code owner separate from GUI; coordinate ingress API with W1 | Pin batches outside queues, saturate and release; peak accounted bytes stay within contract |
| **W1 parallel — Trace UI** | AR-07 initial, AR-09 initial: DEV-008 bounded virtual CAN Trace via existing source/history/query APIs and thin service adapter | GUI files owned by DEV-008; DEV-022 native float uses isolated example; agree on page/event contracts | Only bounded rows painted, filter/channel/ID/direction works, capture continues while panel is slow |
| **W2 — End-to-end quality** | AR-09, AR-15, AR-08: cancel-safe query snapshots, deterministic load/latency harness, recorder format decision/prototype | Requires W1 stable events, loss and memory contracts; DEV-009 after DEV-008 | Reproducible 1M-event tests, defined p95/p99 method, loss taxonomy, disk-full/finalization cases |
| **W3 — Physical capture and portability** | AR-10/11/13: HAL, actual device errors, clock correlation, portable project loader/local mappings | Stable session & permission boundaries; no UI direct adapter callbacks | Windows/Linux adapter capability matrix, disconnect/reconnect, two-worktree resource isolation |
| **W4 — Feature/SDK vertical slices** | AR-12/14 and protocols, measurement, scheduling, Python/AI extensions | Validated common app-services commands and native Slint panel proof; Accepted security/ABI decisions | Independent SDK plugin, protocol/replay fixtures and deny-by-default authorization tests |
| **W5 — Evidence-led tuning** | AR-16 or index/allocator improvements only after profiling | Stable fixture and performance baselines | Before/after throughput, allocations, memory, correctness and regression evidence |

**Parallel merge rule:** Avoid simultaneous edits to shared `STATUS.md`, `BACKLOG.md` and `DECISIONS.md`; reconcile them against latest `main` after task merges. API changes in W1 must be agreed before DEV-008 depends on them. Never merge stale PR #13/#14 blindly; previously integrated equivalent work is documented in `STATUS.md`.

## 3. Targeted acceptance tests (new tests, not claims of execution)

### Correctness and lifecycle

1. **Session generations:** Start virtual session A, issue query, stop/reconnect as B, inject late A reply; B must reject A's result. Identical source-local sequence values in different worktrees remain distinct.
2. **Sparse traffic:** Single frame and long idle gap must be published within documented runtime flush deadline without GUI polling or a full batch.
3. **Dynamic consumers:** Subscribe/unsubscribe during traffic; opening/closing a Trace panel must not reset active capture/recorder or create an unbounded replay backlog.
4. **Deterministic loss:** Cause at least two different consumer queue overflows; both remain queryable with exact known batch counts and reasons. Separate driver drop counts from unknowable sequence ranges.
5. **Decoder gap:** Drop a constituent ISO-TP frame in a virtual fixture; never emit a falsely successful reassembly; explicit reset/recovery.
6. **RAM retention:** Evict oldest Trace batch and keep recorder's independent input; increment retention-eviction counts, not capture-drop counts.
7. **Recorder:** Simulate queue overflow, disk-full and stop timeout; no recording may report successful completeness after confirmed failure.
8. **Project:** Two checkouts with different DBC/A2L and working directories do not cross-resolve; opening/editing project does not add runtime files to ECU checkout.
9. **Permissions:** Unknown hardware mapping or stale session denies TX; derived AI/model commands cannot grant themselves approval for physical actions.

### Memory and throughput

10. **Pinned credits:** Publish and remove history batches while a slow query/plugin retains a clone; global accounting remains nonzero until final release. Admission fails/evicts per policy without deadlock; independent critical recorder reserve is honored.
11. **Burst and idle:** A deterministic million-event CAN Classic/FD fixture checks every accepted event's identity/bytes and reports captured, delivered, intentionally dropped and evicted counts separately.
12. **Measurement:** Record allocations per frame after warm-up, allocated bytes by batch/consumer, peak RSS, CPU usage, ingress→publish p50/p95/p99 and query latency with hardware/OS/load fixtures declared. Do not equate synthetic receive-worker latency to on-wire latency.
13. **Slow GUI:** Starve GUI updates while capture and recorder run; UI pending jobs and visible row count stay bounded, capture continues and any consumer loss is explicit.
14. **Queries:** Change filter/scroll generation under append load; previous result never overwrites new panel state. Cursor recovery/cancellation is bounded, and sorting/filtering does not block capture.

### Cross-platform evidence

15. All feature PRs: final-head Windows and Linux project-state, rustfmt, Clippy `-D warnings`, build and all-target tests as applicable; after merge, verify main independently.
16. GUI native window, focus, DPI and multi-monitor behavior: **manual/integration evidence**, not inferred from compile and headless Rust tests.
17. Real adapters, hardware timestamps, high bus load and TX permissions: **hardware-in-the-loop evidence**, separate from virtual CAN success.

ADR-011 suggests p99 ingress-to-publish under 2 ms, p95 UI visible feedback under 50 ms and 60 FPS scrolling as **performance hypotheses**. The team must accept hardware profiles, exact measurement methodology and acceptable failure thresholds before using these as release gates.

## 4. Required decision checkpoints

Before increasing scope, write an explicit ADR or accepted technical decision covering each: (a) session/identity and stale-work rules, (b) shared byte/lease budget semantics, (c) which loss/health records survive queue overflow, (d) runtime subscription and flush contract, (e) project schema/local mapping, (f) recorder format/finalization, (g) native plugin trust/ABI/window lifecycle, and (h) command authorization. Existing ADR-008/009/011 details remain Proposed unless separately accepted. A finished DEV task does not change a REQ status automatically.

## 5. Recommended immediate action

1. PR #19 is already merged with corrected DEV-022/023 IDs and successful post-merge main CI. Preserve these stable IDs in all subsequent task branches.
2. Review this documentation-only architecture proposal; **do not** treat it as acceptance of unresolved technical choices.
3. Deliver DEV-008 in its GUI-owned branch and DEV-023 in a separate core-owned branch, coordinating their shared batch/query contracts.
4. Add scoped W1 runtime/session + gap + timer task(s) to the canonical backlog in a separate coordinated change, selecting the next unused IDs on current `main`.
5. Use DEV-009 to establish reproducible failure/performance evidence before real device support or more complex protocols.

The recommended next **core implementation** is the generation-scoped virtual capture/session host with timed batch flush and explicit loss/health delivery. It unlocks reliable GUI consumption and later hardware adapters without forcing a rewrite of the existing frame, ingestion or history primitives.
