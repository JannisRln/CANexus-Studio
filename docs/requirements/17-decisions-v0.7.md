# Accepted Product Decisions — v0.7

**Decision date:** 2026-11-03  
**Status:** Accepted product direction, not verification of implementation  
**Reference:** [v0.6 decisions](11-decisions-v0.6.md)

## Decisions

| # | Decision | Requirement | Domain |
|---|---|---|---|
| 1 | One plotter works consistently with live, replay and offline data | REQ-PLOT-004 | [Measurement](12-measurement-dashboard-time.md) |
| 2 | Built-in analysis starts with basic statistics | REQ-PLOT-005 | [Measurement](12-measurement-dashboard-time.md) |
| 3 | Dashboards support programmable custom widgets | REQ-DASH-001 | [Measurement](12-measurement-dashboard-time.md) |
| 4 | Prefer hardware timestamps when available | REQ-TIME-001 | [Measurement](12-measurement-dashboard-time.md) |
| 5 | UDS service browser combines forms and raw mode | REQ-DIAG-007 | [Diagnostics](13-diagnostics-flashing-simulation.md) |
| 6 | Long-term ODX, PDX, CDD and extensible vendor format support | REQ-DIAG-008 | [Diagnostics](13-diagnostics-flashing-simulation.md) |
| 7 | Extensible, protocol-agnostic flashing framework | REQ-FLASH-001 | [Diagnostics](13-diagnostics-flashing-simulation.md) |
| 8 | Fully stateful virtual UDS ECU | REQ-SIM-001 | [Diagnostics](13-diagnostics-flashing-simulation.md) |
| 9 | Automated XCP calibration workflows | REQ-XCP-006 | [XCP/CANopen](14-xcp-canopen.md) |
| 10 | Multiple A2Ls/ECUs with compatibility/version checking | REQ-XCP-007 | [XCP/CANopen](14-xcp-canopen.md) |
| 11 | Graphical CANopen network management | REQ-OPEN-006 | [XCP/CANopen](14-xcp-canopen.md) |
| 12 | Isolated Python worker process per script session | REQ-SCR-022 | [Python/tests](15-python-tests-editor.md) |
| 13 | External editor (e.g., VS Code), no full integrated Python editor | REQ-SCR-023 | [Python/tests](15-python-tests-editor.md) |
| 14 | Python files with test functions are the test organization unit | REQ-TEST-001 | [Python/tests](15-python-tests-editor.md) |
| 15 | External plugins can provide dockable Slint panels | REQ-PLG-012 | [UI plugins/commands](16-ui-plugins-commands.md) |
| 16 | Fully customizable command system | REQ-CMD-001 | [UI plugins/commands](16-ui-plugins-commands.md) |

## Architecture conflict register

| ID | Conflict / risk | Proposed mitigation or open validation |
|---|---|---|
| CR-012 | One plotter for streaming and offline datasets vs different acquisition, seek and time behavior | Unified viewer contract with explicit source modes, time domains and indexed access; no promise to overlay all modes simultaneously. |
| CR-013 | Basic built-in statistics vs earlier broad "professional plotter" concept | Keep multiple axes, cursors, markers and synced zoom as professional UX; FFT/RMS/integral are not mandatory built-ins. |
| CR-014 | Programmable Slint widgets vs stable public SDK, dynamic UI and crash safety | Test dynamic panel/widget loading and lifecycle on Windows/Linux; constrain UI-thread access and use versioned contracts. |
| CR-015 | Hardware timestamps from different devices may not share an epoch or frequency | Track clock domain, provenance, drift and confidence; annotate unsynchronized time; do not infer precision. |
| CR-016 | ODX/PDX/CDD/vendor support vs licensing and varied format versions | Modular readers and explicit compatibility matrix; support claims require real authorized fixtures. |
| CR-017 | Universal flashing vs ECU/vendor-specific safety and recovery constraints | Core provides workflows/safety contracts, while adapters provide transport, algorithm and validation; no generic rollback promise. |
| CR-018 | Stateful UDS simulation and flashing share protocol semantics but have different safety models | Reuse service/state abstractions while keeping simulated and real transport authorization strictly separate. |
| CR-019 | Automated XCP writes and uncertain A2L identity can corrupt calibration | Block unsafe writes on mismatch; unknown identity requires explicit risk decision and supported verification mechanisms. |
| CR-020 | Graphical CANopen scanning/configuration can unexpectedly modify live nodes | Discovery defaults read-only, active NMT/SDO writes require explicit authorization and status. |
| CR-021 | Per-session Python isolation vs fast startup and RAM use | Lazy worker spawning, reusable package distribution, bounded queues and lifecycle metrics; no shared interpreter state between sessions. |
| CR-022 | No embedded script editor vs user need for debugging and one-click scripting | External-editor launch, logs/tracebacks, source links and optional debug adapter, not full internal editor. |
| CR-023 | Plain Python test functions vs former idea of graphical test suites | Python files are authoritative; discover/run/report via shared GUI/CLI service, avoid mandatory graphical test authoring UI. |
| CR-024 | Fully configurable macros vs Python automation overlap | Commands own actions/permissions; macro orchestration and Python invoke the same core scheduler/service rather than duplicate TX engines. |
| CR-025 | UI panels from untrusted native plugins vs host stability | Document in-process limitations; choose trust and optional out-of-process boundaries before SDK freeze. |

## Required architecture validation

1. **Plot & time fixture:** live/replay/offline known dataset, basic statistics, known hardware and software time origins, unsynchronized clock annotation.
2. **Widget lifecycle fixture:** externally compiled plugin panel + programmable widget, docking/undocking/floating, state restoration on Windows/Linux, missing-plugin placeholder.
3. **Diagnostic format fixture:** ODX/PDX and authorized CDD/vendor samples, unsupported-feature reporting and raw-vs-form equivalence.
4. **Flashing safety fixture:** simulated target, invalid firmware, interrupted transfer and permission failure; no unverified physical-ECU recovery claim.
5. **UDS simulation fixture:** sessions, DIDs, DTCs, security states, timing and reproducible headless test.
6. **XCP fixture:** two ECU sessions and A2Ls, identity match/mismatch/unknown, automated calibration validation and write guards.
7. **CANopen fixture:** multi-node discovery, NMT/Heartbeat, Object Dictionary, authorized SDO, disconnect/abort.
8. **Python fixture:** two isolated worker processes, crash one, verify capture and second worker remain functional, GUI/CLI Python function test parity.
9. **Command fixture:** configurable shortcuts, macro and plugin commands sharing one dispatcher; no bypass of transmit/flash permission checks.

## Open implementation decisions

- Statistics definition (selected samples vs time-weighted); missing data, interpolation and units.
- Widget ABI, Slint component loading and third-party trust model.
- Timestamp correlation model and OS/adapter hardware timestamp capabilities.
- Exact ODX/PDX/CDD versions, permitted vendor formats and source-to-service normalization.
- Flash strategy contracts, hardware safety, allowed cancellation points and recovery guarantees.
- Virtual ECU state storage and script extension semantics.
- A2L-to-ECU identity methodology and what happens on unknown match.
- CANopen node scanning and active-change permission policy.
- Python interpreter delivery, dependency management, IPC and process isolation strength.
- Test function discovery convention (e.g., pytest-style vs native) and result schema.
- Command macro boundaries, context scoping, conflict precedence and persistence.
- Priorities/milestones are proposed, even when the functional direction is Accepted.

## Explicitly superseded ideas

- The earlier **optional integrated Python editor** (REQ-SCR-016) is superseded by REQ-SCR-023: external editor integration.
- A **graphical test-sequence editor** is not required; Python test functions are the accepted source of test behavior.
- Full advanced built-in FFT/RMS/integration are not mandatory; basic analysis is accepted.
- Full support for all vendor flashing variants and all diagnostic formats is **not** asserted; these require adapters, coverage definitions and validation.
