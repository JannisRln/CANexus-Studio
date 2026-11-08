# ADR-008 — Central Measurement Engine

**Decision direction:** Recommended architecture, discussed 2026-11-08  
**Detailed requirements status:** Proposed · **Revision:** 1.0  
**Related:** [Measurement, Dashboards and Time](12-measurement-dashboard-time.md), [XCP and CANopen](14-xcp-canopen.md), [ADR-007](19-adr-007-elf-dwarf-symbol-calibration.md), [ADR-009](20-adr-009-high-performance-can-trace.md), [ADR-010](21-adr-010-measurement-calibration-workspace-ux.md), [ADR-011](22-adr-011-high-performance-core-architecture.md)

## 1. Context and architectural recommendation

The Central Measurement Engine serves XCP, CAN/DBC, CANopen, virtual and future sources, with a unified typed delivery interface for Numeric, Time Plot, recording, scripts and analysis.

**Core invariant:** The same compatible measurement must not be physically reacquired solely because it is shown in several windows.

The Measurement Engine owns acquisition planning, subscription ownership and sample routing. Slint panels do not own XCP sessions or implement transport logic; recording is independent of panel rendering.

### Recommended decisions

| Subject | Proposed decision |
|---|---|
| Acquisition | Share physical acquisition when requests are compatible |
| Rates | Separate requested, configured, observed and delivery rates |
| XCP | Prefer supported DAQ; permit polling fallback where appropriate |
| Numeric | Latest-value cache and bounded display update rate |
| Time Plot | Bounded sample history, independent LOD rendering |
| Recording | Independent, bounded, loss-accounted writer path |
| Live history | Global budget per configured project/session context plus per-signal fairness |
| Closing a project | Stop project-bound measurement sessions and finalize recordings |
| Closing a panel | Release only that panel's subscription |
| Errors | Preserve source quality, gaps, overload, actual rate and session status |

These are adopted here as a **recommended technical direction**, not proof of implementation or verification. The already accepted requirements in other documents retain their existing statuses.

## 2. Logical data flow

```text
 XCP DAQ     XCP Polling      CAN/DBC      CANopen      Offline/Replay
    |            |              |             |              |
    +------------+--------------+-------------+--------------+
                                 |
                    Measurement Provider layer
                  capabilities / timestamps / errors
                                 |
                       Subscription Planner
                     shared acquisition / rates
                                 |
                       Typed Sample Routing
                                 |
           +---------------------+---------------------+
           |                     |                     |
    LatestValueCache        LiveHistory        Recorder path
    latest / quality        bounded samples    durable, loss-accounted
           |                     |                     |
           +---------------------+---------------------+
                                 |
                          ViewModel APIs
                     Numeric / Plot / Scripts
```

The drawing is logical; no UI dependency or mandatory thread-per-component is implied. XCP read/write commands and DAQ setup use XCP session/control APIs directly rather than a Trace query path.

## 3. Identity and sample data model

A source identifies a concrete project-instance, ECU/session and artifact context, never just a human-readable symbol name.

Illustrative internal Rust model (not external plugin ABI):

```rust
pub struct MeasurementSourceId {
    pub project: ProjectInstanceId,
    pub session: SessionId,
    pub artifact: Option<ArtifactIdentity>,
    pub generation: u64,
}

pub struct MeasurementKey {
    pub source: MeasurementSourceId,
    pub object: MeasurementObjectId,
}

pub struct MeasurementSample {
    pub key: MeasurementKey,
    pub timestamp: EventTimestamp,
    pub value: MeasurementValue,
    pub quality: MeasurementQuality,
    pub provenance: SampleProvenance,
}
```

`MeasurementObjectId` may resolve to a qualified DBC signal, XCP symbol alias or another provider-specific object, subject to type/capability checks. Preserve raw and physical representation as separate typed concepts rather than overwriting source facts. Support efficient small scalar values without individual heap allocation on the fast delivery path; structs and arrays may have alternative storage.

Quality must distinguish at least `Valid`, `Stale`, `Invalid`, `Unavailable` and `Gap`. `Unsynchronized` denotes timestamp/correlation quality and is not necessarily a reason to declare a valid measured value invalid. Stale detection depends on an expected update period or policy; absence of an update is not universally a device fault.

All timestamps carry origin, clock domain, resolution and uncertainty per ADR-011 and accepted `REQ-TIME-001`. Never implicitly conflate different source generations, projects or worktrees.

## 4. Subscriptions and ownership

Each consumer owns a subscription identified independently from the physical provider acquisition.

```rust
pub struct MeasurementRequest {
    pub source: MeasurementSourceId,
    pub object: MeasurementObjectId,
    pub requested_rate: RequestedRate,
    pub delivery: DeliveryMode,
}

pub enum DeliveryMode {
    Latest,
    EverySample,
    History,
}

pub trait MeasurementService {
    async fn subscribe(
        &self,
        owner: ConsumerId,
        request: MeasurementRequest,
    ) -> Result<SubscriptionId, MeasurementError>;

    async fn update(
        &self,
        id: SubscriptionId,
        request: MeasurementRequest,
    ) -> Result<(), MeasurementError>;

    async fn unsubscribe(
        &self,
        id: SubscriptionId,
    ) -> Result<(), MeasurementError>;
}
```

This is a conceptual Rust interface; use a versioned separate contract for external plugins. Actual API form, events, structured requests, ownership drop handling and cancellation remain to be refined in implementation.

Rules:
1. Subscribe resolves the source and provider capability, validates the request and reuses a compatible acquisition when possible.
2. Update requests a replanning; the service exposes whether it was applied and what remains active.
3. Unsubscribe removes only this owner; release the physical acquisition only when no remaining consumer requires it.
4. Recorder and scripts have explicit independent owners, not a hidden attachment to a GUI panel.
5. An unavailable or disconnected provider changes subscription status and quality; it does not claim valid fresh samples.

### Example

A 1,000 Hz Time Plot and 20 Hz and 5 Hz Numeric panels may share one feasible 1,000 Hz XCP DAQ acquisition, with separate view delivery rates. This is an illustrative configuration, not a guaranteed ECU rate. If the ECU cannot supply the requested rate, the provider reports actual available/configured/observed values and rejects or negotiates the request explicitly.

## 5. DAQ/polling acquisition planner

Plan per provider/ECU session. Inputs: object identities, available DAQ events, resource and bandwidth limits, sampling requirements, transport, available address/memory information, current active plan and permitted fallback modes.

Preferred choices:
- Reuse identical compatible acquisition first.
- Prefer XCP DAQ when the ECU and target object support it.
- Poll only if supported, permitted and feasible; do not silently increase target load.
- Reject requests with explicit reason when no safe/feasible plan exists.
- Reconfiguration is an explicit state transition with applied/failed result and active-plan snapshot; never report a desired configuration as already active.

**Four distinct rates** must remain visible:

| Rate | Meaning |
|---|---|
| Requested | Consumer preference/requirement |
| Configured | Provider nominal acquisition configuration |
| Observed | Rate measured from delivered source samples |
| Delivery | Consumer-specific emitted/visual update frequency |

`EverySample` recording is not converted to `Latest` merely because an attached Numeric panel is slower. A delivery-rate cap and an acquisition-rate requirement are not interchangeable. Distinct exact-rate acquisitions may be needed for incompatible requirements; compatibility evaluation is per provider rather than a universal highest-rate rule.

DAQ eligibility needs ECU protocol capabilities/event information beyond the ELF address alone. ADR-007's symbol/address resolution and ECU access constraints continue to apply.

## 6. Three independent data paths

### LatestValueCache

Store latest sample with timestamp, quality and generation per active key. Overwriting its previous value is expected behavior; that does not imply recorder loss. Optimized for Numeric, digital/status panels and quick source queries.

### LiveHistory

A separate bounded series for plotting and interactive time-range operations. **Recommended budget**: a shared configurable global 128 MiB starting value per selected measurement/session budget domain, with per-signal fair-use caps/weights and oldest-data eviction. This is the **same** measurement-history line item as in ADR-011's proposed 640 MiB illustrative subtotal; do not double count it.

A per-signal hard fixed duration alone cannot bound memory across high rates, and a fixed independent budget per signal wastes memory for slow signals. Expose retained time span, source completeness and `HistoryEvicted` separately from capture gaps. Rendering LOD/decimation never alters recorder input or the original samples retained under applicable history policy.

### Recorder

A separate bounded durable writer path, consistent with ADR-011. Queue overflow results in **failed/incomplete recording status and controlled stop**, never silent partial success. Distinguish raw CAN recording, measurement-sample recording and future mixed recording support; the exact direct interoperable file format remains open under accepted `REQ-LOG-001`.

Historical sample retention is not equivalent to durable recording; if no disk data exists, requests for evicted time ranges report unavailability.

## 7. Project, panel and reconnect lifecycle

**Recommended MVP policy:** Project close stops its project-bound measurement sessions and finalizes dependent recording under the documented shutdown policy. Closing or moving a panel only disposes/reassigns its own subscription. Changing dashboards releases subscriptions that no remaining owner needs.

| Action | Expected result |
|---|---|
| Close Numeric or Plot panel | Release only that panel's subscription |
| Switch dashboard | Release unused panel-owned subscriptions |
| Recorder remains running | Preserve source acquisition required by recorder |
| Disconnect XCP | Mark affected subscriptions unavailable/stale; report cause |
| Reconnect ECU | New generation; validate XCP bindings and DAQ plan anew |
| Change ELF/artifact | Invalidate affected symbol/address bindings and acquisitions |
| Close project | Controlled stop of project-bound services; recording finalized or failed |
| Open two worktrees | Independent source identity, sessions and context |

A future background measurement session after closing its project is explicitly outside MVP, not silently assumed. Stale async replies from an old session/generation must not attach to a new project.

ADR-007's direct XCP reads/writes for calibration are **not automatically recurring subscriptions**. Calibration local drafts and explicit writes remain under ADR-010, with validated current artifact/session bindings. No additional special development mode is introduced.

## 8. Overload and quality handling

| Failure | Required behavior |
|---|---|
| XCP DAQ absent | Consider permitted polling or report unavailable |
| Polling exceeds ECU capacity | Reduce only via explicit negotiated plan or reject |
| Slow latest-value UI | Coalesce UI delivery, keep source/recording unaffected |
| EverySample consumer lags | Bound its queue, expose gap/failure per consumer contract |
| Missing CAN frame in decoder | Mark affected samples/transactions with decode gap quality |
| LiveHistory reaches budget | Evict per policy; emit retention status, not capture-drop |
| Recorder queue overflow | Stop recording with failed/incomplete status |
| Hardware disconnect | Mark subscription unavailable and invalidate session-dependent state |
| Timestamp correlation uncertain | Preserve clock domain and quality; no false precision |
| ELF/session changes | Invalidate stale address and DAQ bindings |

A separate bounded health channel or equivalent robust diagnostic path must remain observable when data queues themselves overflow.

## 9. API boundaries

| Module | Responsibilities |
|---|---|
| `canexus-measurement` | Subscription planner, acquisition deduplication, sample routing, Latest cache and live history |
| `canexus-xcp` | XCP session/DAQ/polling/read/write protocol, target-side errors and capabilities |
| `canexus-symbols` | Symbol identity, type facts and resolution |
| `canexus-metadata` | Scaling, units and semantics |
| `canexus-trace-data` | Immutable raw bus events, source linkage |
| `canexus-trace-analysis` | Trace-specific diff/statistics |
| `canexus-workspace` | Dashboard/panel state and persisted bindings |
| `canexus-ui` | Numeric/Plot ViewModels and Slint rendering |

All providers expose capabilities, status, configured acquisition and bounded sample/error outputs. External plugin SDK is versioned independently from internal Rust traits. No Slint object or UI thread callback is required to operate or test the engine.

## 10. Proposed requirements

Preserve earlier accepted `REQ-PLOT-*`, `REQ-TIME-001`, `REQ-CORE-*` and XCP requirements. The following are new and **Proposed**, including priorities.

| ID | Priority | Status | Requirement |
|---|---|---|---|
| REQ-MEAS-001 | P0 | Proposed | Provide a central measurement engine independent of Slint and usable headlessly. |
| REQ-MEAS-002 | P0 | Proposed | Measurement source identity includes project instance, session and relevant artifact/generation scope. |
| REQ-MEAS-003 | P0 | Proposed | Samples are typed, timestamped and preserve source and quality provenance. |
| REQ-MEAS-004 | P0 | Proposed | Subscriptions have stable owner IDs and subscription IDs. |
| REQ-MEAS-005 | P0 | Proposed | Compatible consumers share physical acquisition rather than duplicating ECU requests. |
| REQ-MEAS-006 | P0 | Proposed | Acquisition rate and per-consumer delivery rate are independently represented. |
| REQ-MEAS-007 | P0 | Proposed | Expose configured and observed rates, not only the consumer-requested rate. |
| REQ-MEAS-008 | P0 | Proposed | Validate provider capabilities and feasible acquisition plans before enabling. |
| REQ-MEAS-009 | P1 | Proposed | Prefer supported XCP DAQ; use XCP polling as an allowed fallback. |
| REQ-MEAS-010 | P1 | Proposed | Apply acquisition-plan updates with explicit success/failure and actual active configuration. |
| REQ-MEAS-011 | P0 | Proposed | Maintain a bounded latest-value cache per active measurement object. |
| REQ-MEAS-012 | P0 | Proposed | Maintain a bounded live sample history independently from latest-value cache. |
| REQ-MEAS-013 | P0 | Proposed | Recording uses an independent path and is not controlled by GUI update frequency or live-history retention. |
| REQ-MEAS-014 | P0 | Proposed | Recorder sample loss is never silent; queue overflow follows ADR-011 failure policy. |
| REQ-MEAS-015 | P0 | Proposed | View/Latest delivery can coalesce intermediate UI updates without claiming recorded sample loss. |
| REQ-MEAS-016 | P1 | Proposed | Use a global live-history memory budget plus per-signal fairness/retention controls. |
| REQ-MEAS-017 | P1 | Proposed | Distinguish sample history eviction from underlying capture/recording loss. |
| REQ-MEAS-018 | P0 | Proposed | Panel lifecycle and physical acquisition/session lifetime are independently managed. |
| REQ-MEAS-019 | P0 | Proposed | Closing a project stops its project-bound measurement sessions and finalizes dependent recordings. |
| REQ-MEAS-020 | P0 | Proposed | Artifact/session generation changes invalidate stale measurement and XCP address bindings. |
| REQ-MEAS-021 | P0 | Proposed | Never blend measurements from distinct worktrees, ECUs or sessions without an explicit compatible correlation. |
| REQ-MEAS-022 | P1 | Proposed | Apply units and conversions via shared metadata service without changing raw samples. |
| REQ-MEAS-023 | P1 | Proposed | Preserve clock-domain provenance and uncertainty through measurement delivery. |
| REQ-MEAS-024 | P1 | Proposed | Expose valid, stale, invalid, unavailable and gap semantics with explicit derivation. |
| REQ-MEAS-025 | P1 | Proposed | Expose provider capacity, overload, errors and actual supported rates. |
| REQ-MEAS-026 | P1 | Proposed | Use bounded queues and explicit overload policies for measurement consumers. |
| REQ-MEAS-027 | P1 | Proposed | Expose a versioned external measurement-provider API distinct from internal Rust traits. |
| REQ-MEAS-028 | P1 | Proposed | Replay/offline sources can provide measurements via the common data model without implicit hardware I/O. |
| REQ-MEAS-029 | P1 | Proposed | Expose structured measurement throughput, latency, subscription and memory metrics. |
| REQ-MEAS-030 | P1 | Proposed | Provide deterministic virtual measurement providers and failure-injection tests. |

## 11. Acceptance tests

1. Two Numeric panels subscribe to a compatible simulated XCP source: instrument provider to verify one physical acquisition.
2. Time Plot requests 1,000 Hz while Numeric clients request 20/5 Hz: common acquisition when feasible and independent display delivery.
3. Remove one consumer: another consumer/recorder keeps its acquisition.
4. Change dashboard during active recorder: no recording stop or loss induced by panel subscriptions.
5. No supported DAQ: use explicitly allowed polling or return a diagnosable failure.
6. Request a rate beyond ECU capability: no unsupported rate is falsely promised.
7. Fail a provider plan update: actual applied configuration remains correctly reported.
8. Update LatestValueCache: no full history scan and correct newest sample quality/time.
9. Fill LiveHistory budget: controlled eviction, reported retained span, no false capture-drop.
10. Overflow recorder queue: recording reports failed/incomplete and stops per ADR-011.
11. Drop CAN frames before a stateful decoder: decoded sample/transaction quality exposes gap and recovery.
12. Mix hardware/software timestamp and drift sources: preserve provenance and uncertainty.
13. Disconnect XCP: subscriptions move to defined unavailable/stale state and do not report fresh samples.
14. Replace ELF or create new XCP session: old bindings cannot be used as current.
15. Open two worktrees: no cross-context acquisition/cache/data contamination.
16. Close project under load: stop project-bound sessions, finalize recorder or surface fault; UI stays responsive.
17. Use replay/offline provider: no implicit hardware reads/writes.
18. Slow Plot consumer: unaffected recorder and independent clients; bounded queue/memory.

Measure throughput, configured/observed rates, provider load, p95/p99 delivery latency, memory and overflow counters in repeatable virtual fixtures.

## 12. Cross-ADR consistency

- **ADR-007:** Logical symbol identity stays distinct from build-specific address and XCP session binding; direct XCP Read/Write remains explicit.
- **ADR-009:** Raw CAN frames, decoded protocol events and measurement samples are different record classes; derived source references are preserved.
- **ADR-010:** Numeric/Time Plot share engine-managed acquisition and can outlive panel reorganization; calibration edits are local until explicit Write.
- **ADR-011:** Immutable batches, bounded queues, independent recording, failure isolation, clock domains, worktree separation and project-close shutdown stay authoritative.

There is no claim that durable recording always succeeds under hardware or disk overload, nor that DAQ is universally supported.

## 13. Delivery plan

Two complementary technical vertical slices:
1. **CAN Core:** Virtual CAN → Capture → immutable batches → RAM ring → Trace query → Slint Trace; inject slow consumer and simulated recording.
2. **Measurement Core:** Virtual Measurement Provider → shared subscription planner → LatestValueCache/LiveHistory → Numeric + Time Plot; inject rate negotiation, disconnect and memory-pressure cases.

Share underlying runtime source identity, timestamps, service lifecycle and quality semantics. Next specification candidate: **ADR-012 — Rust Workspace Structure & Vertical Slice Implementation Plan**, then implementation/benchmark prototypes.

## 14. Remaining details

- Numerical defaults for per-signal live-history fairness and stale thresholds.
- Provider rate compatibility algorithm, explicit exact-rate requests and update transaction semantics.
- Actual external ABI technology and provider lifecycle wire contracts.
- Reader/writer capabilities of the selected standard recording format.
- Operational observability/UI design for provider-capacity and quality diagnostics.

**Status:** Recommended architecture recorded; detailed REQ-MEAS requirements remain Proposed, not Implemented or Verified.
