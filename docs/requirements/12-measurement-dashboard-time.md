# Measurement, Dashboards and Time Synchronization

**Revision:** 0.7 · **Status:** Accepted product decisions, Proposed implementation details

## Accepted requirements

| ID | Priority (proposed) | Status | Requirement |
|---|---|---|---|
| REQ-PLOT-004 | P1 | Accepted | The same plotter supports live measurement, replay and offline recordings, with consistent navigation and measurement interaction. |
| REQ-PLOT-005 | P1 | Accepted | Built-in mathematical analysis provides basic minimum, maximum, arithmetic mean and difference calculations on selected signal samples or selected time ranges. |
| REQ-DASH-001 | P2 | Accepted | Dashboards support programmable custom widgets supplied by extension modules, in addition to built-in visualizations. |
| REQ-TIME-001 | P0 | Accepted | Prefer hardware timestamps where available; provide a coherent comparison strategy for multiple CAN channels and future Ethernet sources. |

## Detailed proposed requirements

| ID | Status | Requirement |
|---|---|---|
| REQ-PLOT-006 | Proposed | Live/replay/offline data sources expose source and time provenance; mode changes preserve compatible plot configuration and do not silently blend datasets. |
| REQ-PLOT-007 | Proposed | Basic statistics define treatment of gaps, invalid samples, NaN, sample versus time-weighted means and units. |
| REQ-PLOT-008 | Proposed | Plot rendering uses bounded caches and level-of-detail; analytics compute on appropriate original data rather than reduced visual samples. |
| REQ-DASH-002 | Proposed | Widget SDK exposes versioned inputs, data bindings, commands, lifecycle and supported rendering capabilities. |
| REQ-DASH-003 | Proposed | Plugins cannot silently bypass Slint UI-thread ownership, permission checks or per-frame update budgets. |
| REQ-TIME-002 | Proposed | Every timestamp carries clock domain, source (hardware/software), resolution and mapping quality; absence of synchronization is visible. |
| REQ-TIME-003 | Proposed | Clock correlation accounts for drift, offset, device reset and discontinuities without pretending unsynchronized sources have precise global ordering. |

## Acceptance criteria

- [ ] Add one signal to a plot while live, replay the capture and open it offline: zoom, cursor, axis and statistic operations have consistent semantics.
- [ ] Min/max/mean/difference calculated against a known numeric test vector match independently computed expected values; invalid sample policy is explicit.
- [ ] A custom dashboard widget supplied by an external example plugin can render bound signal data, unload/recreate, and recover without altering ECU source files.
- [ ] Under a high-rate measurement stream, a slow dashboard widget does not block recording or other panels.
- [ ] Feed streams with hardware timestamps, software timestamps and simulated drift; correlation is documented and uncertainty shown when no reliable common time base exists.
- [ ] Raw measurement series remain available when view decimation is enabled.

## Risks and open implementation decisions

- Mixed live/offline display needs explicit clock-domain and source semantics: one common plotter does not require simultaneous overlays.
- Define minimum built-in analytics carefully; FFT/RMS/integral are **not** part of the confirmed built-in baseline, though plugins may extend analysis.
- Slint custom-widget rendering ABI and pbSlintDock lifecycle need an end-to-end compatibility prototype.
- Timestamp normalization, precision, drift model, reference timing hardware, and synchronized versus estimated time must be benchmarked.
