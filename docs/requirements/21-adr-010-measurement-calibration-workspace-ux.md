# ADR-010 — Unified Measurement & Calibration Workspace UX

**Decision direction:** Confirmed in discussion (2026-11-08)  
**Detailed requirements:** Proposed · **Revision:** 1.0  
**Related:** [UX & Performance](02-ux-performance.md), [Signal Explorer](07-signal-workspace.md), [Measurement/Dashboards/Time](12-measurement-dashboard-time.md), [XCP](14-xcp-canopen.md), [Dockable Plugin Panels](16-ui-plugins-commands.md), [ADR-007 Modular Symbol/Metadata/Dashboard Files](19-adr-007-elf-dwarf-symbol-calibration.md), [ADR-009 Trace](20-adr-009-high-performance-can-trace.md)

## 1. Context and confirmed scope

CANexus-Studio aims for a CANape-class measurement and calibration workflow while retaining a distinctive, responsive and consistent UX. CANape, CANalyzer and CANoe are *functional references*, not UI templates to copy.

The user has confirmed the following implementation priorities:

| Capability | MVP / P0 | Subsequent / P1–P2 |
|---|---|---|
| Visualization | **Numeric Display + Time Plot first** | Bar/Gauge and XY later |
| Calibration | **Scalar parameter editor first** | Curve and map editors later; optional 3D |
| Editing | **Edit a local draft and explicitly write it** | Multi-parameter editing, comparisons, interpolation later |
| Workspace | Slint + pbSlintDock and separately versioned dashboard files | Extended widgets and templates |
| Symbol actions | Capabilities and assignment to compatible views | Exact gestures and context menus remain open |

**Do not prescribe** Drag & Drop, double-click or a universal action menu for adding symbols yet. Requirements specify *outcomes*, not an unvalidated interaction gesture.

## 2. Architecture and ownership

The UI workspace contains dockable visualization and calibration panels. Panel instances own view configuration and local interaction state, **not** hardware sessions or the measurement capture engine.

```text
Slint + pbSlintDock Workspace
   ├── Numeric Panel        [P0]
   ├── Time Plot            [P0]
   ├── Parameter Editor     [P0]
   ├── Bar / Gauge          [P1]
   ├── XY Plot              [P1]
   ├── Curve Editor         [P1]
   └── Map Editor / 3D      [P1 / P2]
               |
      Common typed services
   ├── MeasurementService
   ├── SymbolResolver + MetadataRegistry
   ├── CalibrationService
   ├── TimeNavigationService
   ├── DashboardService
   └── CommandDispatcher
```

The measurement service owns active subscriptions and coordinates compatible acquisition requests; recorder subscriptions are independent of panels. Closing/moving/recreating a panel must not stop another consumer's acquisition, a running recording or an unrelated protocol session. The central command dispatcher implements common actions consistently across toolbar, shortcuts, scripts and plugins.

A dashboard is a **persisted composition** (stable dashboard ID, panel IDs, symbol references and panel-specific view settings), while an instantiated panel is a UI element. Their identities are distinct, allowing dashboard reuse without duplicating project data or acquiring hardware twice.

## 3. MVP Numeric Display

A Numeric Panel supports several entries, including scalar numbers and enum/boolean states.

Each entry may show:
- Logical symbol/reference and optional semantic display name
- Last observed raw or physical value with configured formatting, unit and precision
- Source/provenance, timestamp or age as appropriate
- Quality: valid, stale, invalid or unavailable
- A loading/error state for missing or ambiguous symbols

Display refresh must be bounded independently of the source measurement rate. No fabricated intermediate values or implicit polling by the UI widget. The subscription service chooses feasible acquisition modes according to the measurement architecture.

### Numeric workflow

1. Open an existing numeric panel or instantiate a new one.
2. Associate a compatible symbol (gesture intentionally unspecified).
3. Resolve symbol and semantic metadata; reject or diagnose invalid references.
4. Acquire or reuse a measurement subscription.
5. Show the latest value, unit, timestamp age and quality.
6. On panel removal, release *only* that panel's subscription.

## 4. MVP Time Plot

Time Plot is a common visual component for live measurement, replay and offline data, consistent with existing accepted plotting requirements.

Required features:
- Multiple signals with supported independent Y axes, units and color legends
- Zoom, pan, auto-scale and manual axis ranges
- Live follow and navigation of retained history without stopping acquisition
- One or two cursors, time/value delta and marker support
- Explicit timestamp source/clock-domain quality
- Level-of-detail rendering for large sample histories; analytics operate on source data, not displayed decimated pixels
- Shared time-navigation integration with Trace when cross-source correlation is known

### Plot workflow

1. Open a plot; assign one or more suitable measurement objects.
2. The plot subscribes to shared measurement streams or uses a replay/offline data source.
3. User navigates to a time range and activates two cursors.
4. Display delta-time and values at the selected positions; label interpolated/estimated values.
5. Synchronize with CAN Trace if supported, otherwise indicate uncertain time alignment.

Plot interaction, sample delivery and rendering do not block raw acquisition or recording.

## 5. MVP Scalar Parameter Calibration

### Model and states

A parameter editor must always distinguish:

| Concept | Meaning |
|---|---|
| Last observed ECU value | Result of the latest successful XCP read, with source/time |
| Local draft | User-edited value not yet written |
| Write pending | XCP operation submitted, response not yet known |
| Write acknowledged | ECU/protocol acknowledged write, not proof of readback or persistence |
| Write failed | ECU/protocol returned a failure; keep local draft |
| Readback verified / differing | Optional subsequent read agrees or differs according to defined raw/physical comparison |
| Persistent storage | Separate ECU-specific state; never inferred solely from write acknowledgement |

### Editing workflow

1. User selects an addressable, supported XCP parameter, then explicitly **reads** it.
2. The view shows last-read ECU value and independent local draft.
3. Editing the draft must not cause an XCP write.
4. Before an explicit **Write** command, validate data type, raw/physical conversion, relevant bounds, target binding and available ECU permission/capability. Incomplete or invalid information produces a clear error; never guess an address.
5. Issue the requested XCP write asynchronously; report ECU/protocol response and distinguish pending, acknowledged and failed states.
6. On failure retain the draft and show error details. Do not falsely update the last-read ECU value.
7. Optionally request readback and compare the target representation. Do not claim nonvolatile persistence without explicit ECU evidence.

For the first iteration, allow at most one pending write per parameter/session. More complicated concurrent editing/versioning is deferred.

**No implicit ECU writes** occur upon editing, saving dashboard files, switching views, opening sessions or loading offline data. The existing ADR-007 ordinary XCP read/write access policy applies; ADR-010 does not introduce a separate development mode.

### Offline behavior

Live, Replay and Offline should use shared visualization models but expose their distinct capabilities. Replay/offline data can be viewed and compared but cannot silently invoke ECU I/O. An offline edit changes only its local draft/dataset until a separate, explicit export or supported transfer operation is chosen.

## 6. Deferred visualization and calibration editors

- **Bar/Gauge [P1]:** scale, threshold ranges, state display.
- **XY Plot [P1]:** pair differently timed signals using a declared pairing/interpolation strategy, refuse misleading time correlation.
- **Curve Editor [P1]:** 1D axis + values, graphical/table editing, local undo/redo and explicit write.
- **Map Editor [P1]:** axis-aware table and heatmap, cell/region editing, reference comparison; actual axis semantics require metadata, not merely DWARF array dimensions.
- **3D map rendering [P2]:** optional display of supported calibrated surfaces, without becoming a prerequisite for table/heatmap use.

These features are in product scope but are not MVP prerequisites.

## 7. Dashboard persistence and workspace state

ADR-007 defines **one shared dashboard per dedicated file**. Store logical symbol references, stable dashboard and panel IDs, visual settings and the dashboard's own panel arrangement. Do not store resolved ELF memory addresses, live pointer values, last measured samples or hardware transmit/write authorization in dashboard files.

The overall application's personal docking arrangement stays user-local by default. Project-relative files follow ADR-006, with no Git integration. Save only the intentionally edited dashboard file; no-op save produces no unexpected Git diff. Missing symbol references remain visible with diagnostics and are not removed silently.

Illustrative dashboard schema (not yet ratified):

```toml
schema_version = 1

[dashboard]
id = "motor.monitoring"
title = "Motor Monitoring"

[[panels]]
id = "motor_values"
type = "numeric"
symbols = ["motor.speed", "motor.temperature"]

[[panels]]
id = "motor_plot"
type = "time_plot"
symbols = ["motor.speed", "motor.temperature"]
```

## 8. API and implementation boundaries

Proposed logical Rust modules, not necessarily separate Cargo crates:

- `canexus-workspace`: dashboard models, persistence, panel IDs, view navigation and panel registration.
- `canexus-measurement`: shared subscriptions, sample delivery and time provenance.
- `canexus-calibration`: parameter sessions, local draft buffers, validation, write coordinator and readback comparison.
- `canexus-ui`: Slint numeric/plot/parameter panels, later bars/XY/curves/maps.
- `canexus-symbols` + `canexus-metadata`: stable references, target type facts and semantic formatting.
- `canexus-xcp`: actual read/write transport, target binding, protocol errors.
- Existing `CommandDispatcher`: shared actions with capability checks.

UI panel code must not own XCP sessions or call blocking hardware I/O on the UI thread. A provider can be reused in GUI and headless contexts. Native plugin ABI is not the internal Rust trait ABI.

## 9. Requirements (proposed detail)

Earlier accepted requirements retain their original IDs and status. The following items are **Proposed**; all priority levels are also proposed until ratified.

| ID | Priority | Status | Requirement |
|---|---|---|---|
| REQ-WORK-001 | P0 | Proposed | Measurement and calibration panels can be independently docked, moved and reopened. |
| REQ-WORK-002 | P0 | Proposed | Measurement/calibration session state is independent of transient panel lifecycle. |
| REQ-WORK-003 | P0 | Proposed | Each shared dashboard is persisted in one dedicated file, per ADR-007. |
| REQ-WORK-004 | P0 | Proposed | Compatible consumers of a measurement share an acquisition through the central measurement service. |
| REQ-WORK-005 | P1 | Proposed | Live, replay and offline modes use common visualization models with source-specific capabilities. |
| REQ-WORK-006 | P1 | Proposed | Panels can synchronize time cursors and markers with declared time-correlation quality. |
| REQ-WORK-007 | P0 | Proposed | Closing a panel cannot stop an acquisition still needed by another panel, script or recorder. |
| REQ-WORK-008 | P1 | Proposed | Saving one dashboard changes only that file; shared and personal state have distinct storage. |
| REQ-WORK-009 | P1 | Proposed | Missing symbol references remain present and show actionable diagnostics. |
| REQ-VIEW-001 | P0 | Proposed | Numeric Display supports multiple scalar/enum values, units, formats and data-quality states. |
| REQ-VIEW-002 | P1 | Proposed | Bar/Gauge Display supports scaling and visible thresholds. |
| REQ-VIEW-003 | P0 | Proposed | Time Plot provides multiple axes, zoom/pan, auto-follow, cursors and markers. |
| REQ-VIEW-004 | P1 | Proposed | XY Plot defines sample pairing and uncertainty for differently timed input signals. |
| REQ-VIEW-005 | P0 | Proposed | Displays distinguish fresh/stale/invalid/unavailable values and show age when relevant. |
| REQ-VIEW-006 | P0 | Proposed | Visualization refresh is decoupled from capture and recording rates. |
| REQ-VIEW-007 | P1 | Proposed | Plot cursors display value/time and interpolation or source quality at the chosen position. |
| REQ-VIEW-008 | P1 | Proposed | Plot time navigation can synchronize with CAN Trace and other compatible views. |
| REQ-CAL-001 | P0 | Proposed | Read, locally edit and explicitly write supported XCP scalar calibration parameters. |
| REQ-CAL-002 | P0 | Proposed | Preserve separate last-read ECU value and user-edited draft value. |
| REQ-CAL-003 | P1 | Proposed | Provide a 1D characteristic curve editor. |
| REQ-CAL-004 | P1 | Proposed | Provide a map editor with value table and heatmap. |
| REQ-CAL-005 | P2 | Proposed | Provide optional 3D map rendering. |
| REQ-CAL-006 | P1 | Proposed | Support undo/redo of local calibration edits. |
| REQ-CAL-007 | P1 | Proposed | Compare calibration datasets and highlight changed values. |
| REQ-CAL-008 | P0 | Proposed | Distinguish XCP write response, error, optional readback and persistence state. |
| REQ-CAL-009 | P1 | Proposed | Curve/map semantics derive axes, layout and conversions from explicitly validated metadata. |
| REQ-CAL-010 | P0 | Proposed | No implicit ECU writes occur during editing, dashboard loading or source switching. |
| REQ-CAL-011 | P0 | Proposed | Validate target data type, conversion, bounds and session binding before each write. |
| REQ-CAL-012 | P0 | Proposed | Failed writes retain local draft values and display errors without false success. |
| REQ-CAL-013 | P1 | Proposed | Optional readback compares observed ECU value with the desired target representation. |
| REQ-CAL-014 | P1 | Proposed | Parameter tables can edit multiple parameters locally and issue explicit writes. |
| REQ-CAL-015 | P1 | Proposed | Successful protocol write is not displayed as verified NVM persistence. |
| REQ-UX-SYM-001 | P0 | Proposed | Provide fast search and hierarchical navigation across discoverable symbols. |
| REQ-UX-SYM-002 | P1 | Proposed | Assign symbols to compatible measurement/calibration panels; precise gesture remains undecided. |
| REQ-UX-SYM-003 | P1 | Proposed | Available actions derive from object capabilities such as readable, writable or streamable. |
| REQ-UX-SYM-004 | P1 | Proposed | Common operations are accessible without unnecessary intermediate dialogs. |

## 10. Acceptance tests

1. Numeric shows an XCP scalar value with unit and quality; invalid/stale/unavailable cases show distinct states.
2. Two Numeric panels referencing the same compatible source share one physical acquisition; use provider instrumentation to verify.
3. Closing one panel while another panel or recorder consumes the value does not stop acquisition.
4. A Time Plot displays multiple signals/axes and handles cursor time/value deltas with known fixture values.
5. Millions of samples are navigable using bounded GUI data and LOD without blocking recording.
6. Pausing plot follow and manually scrolling does not change acquisition state.
7. Two clock domains with uncertain correlation do not produce falsely precise Plot↔Trace alignment.
8. Reading an XCP parameter yields correct raw/physical representation and source timestamp.
9. Editing a draft emits **zero** XCP write requests until the explicit Write action.
10. Explicit write sends the expected target representation; pre-write bounds/type/conversion validation handles invalid input.
11. A failed XCP write keeps the local draft and displays the ECU/protocol failure without reporting success.
12. An acknowledged write is not automatically labeled readback-verified or persisted in NVM.
13. Optional readback detects agreement or mismatch without conflating the protocol acknowledgement.
14. Saving one shared dashboard modifies only its own file; personal dock layout changes do not modify ECU files.
15. Missing symbols remain configured and visible as resolution errors rather than disappearing.
16. Replay/offline interaction invokes no implicit ECU reads or writes.
17. Slow panel rendering cannot block unrelated CAN capture and recording.
18. Opening two worktrees keeps dashboard files, symbol resolution and local project context independent.

Tests use deterministic virtual XCP and measurement fixtures, GUI/headless parity where practical, and defined timestamp/quality scenarios.

## 11. Performance targets (proposed, not verified)

| Interaction | Target |
|---|---|
| Open an already-loaded Numeric Panel | p95 < 100 ms |
| Local parameter edit visual feedback | p95 < 50 ms |
| Switch an already-loaded dashboard | p95 < 100 ms |
| Live plot rendering | Target 60 FPS at defined load |
| Large histories | No GUI object per raw sample |
| XCP read/write | Non-blocking UI; actual duration depends on ECU, transport and session |

Reference hardware, input load, warm/cold conditions and percentile windows remain to be set before these can be verification gates.

## 12. Open decisions

- Exact gesture(s) for assigning symbols to panels: Drag & Drop, double-click, context action or other.
- Default plot colors, axes, cursor controls and view presets.
- Whether unsaved local calibration drafts are recovered after an unexpected disconnect/restart.
- How comparison calibration datasets are versioned and exchanged.
- Minimum external visualization plugin API and lifecycle requirements.
- Exact schema for shared dashboard panel layout and migration.

## 13. Consequences and scope

**MVP:** Numeric Display, Time Plot, scalar Parameter Editor, local-draft editing with explicit XCP write and modular dashboard persistence.  
**P1:** Bar/Gauge, XY, Curve/Map Editors, comparison and advanced navigation.  
**P2:** Optional 3D maps and advanced plugin visualizations.

This ADR complements `REQ-PLOT-001`, `REQ-PLOT-004`, `REQ-DASH-001`, ADR-007 and the central measurement-engine design (ADR-008 remains a discussion draft, not yet persisted). It does not claim implementation or validation.

**User-confirmed direction; detailed requirements remain Proposed.**
