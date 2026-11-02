# Unified Signal Explorer and Module-Specific Views

**Revision:** 0.5 · **Status:** Mixed

## Confirmed interaction model

One **unified Signal Explorer** provides common browsing, searching and navigation across data sources. The detailed workflow depends on the module: CAN/DBC, CANopen, XCP, UDS, and future Ethernet-based protocols expose specialized views and actions. Do not force every module into an identical table or editing workflow.

## Signal identity and common semantics

A signal/object reference must include source module/plugin, network/session, source file/device and stable object identity where available. Display name alone is insufficient (duplicates and multiple worktrees occur). Separate **physical value**, **raw value**, **unit**, **quality**, **timestamp**, **update rate**, **access rights** and **data origin**. Not every object is a time-varying signal; UDS DIDs, CANopen dictionary entries and calibration parameters have different read/write semantics.

## Requirements

| ID | Priority | Status | Requirement |
|---|---|---|---|
| REQ-SIG-001 | P0 | Accepted | One Signal Explorer can discover and search data objects from all enabled modules. |
| REQ-SIG-002 | P0 | Accepted | Modules can provide their own specialized detail panes, editors and actions. |
| REQ-SIG-003 | P0 | Proposed | Explorer entries expose source, hierarchy, name, type, unit and availability, with optional live value. |
| REQ-SIG-004 | P1 | Proposed | Browse by module, network, ECU/node, message, group or user favorites; search and filter incrementally. |
| REQ-SIG-005 | P1 | Proposed | A typed capability model describes streamable, on-demand readable, writable, calibratable and diagnosable objects. |
| REQ-SIG-006 | P1 | Proposed | Drag an appropriate object into a plot, value monitor or compatible panel; incompatible actions explain why. |
| REQ-SIG-007 | P1 | Proposed | Multiple signals from different transports can share a plot or time cursor when clock correlation is known; uncertain correlation is reported. |
| REQ-SIG-008 | P1 | Proposed | Objects are linked back to their DBC/A2L/EDS/DCF or diagnostic description without changing source files. |
| REQ-SIG-009 | P1 | Proposed | Each value retains validity/quality state (fresh, stale, unavailable, invalid) and source timestamp. |
| REQ-SIG-010 | P1 | Proposed | Common explorer and plots remain functional when an optional protocol plugin is not installed; unavailable objects are signaled. |
| REQ-SIG-011 | P1 | Proposed | Module-specific contextual operations are registered through capability-based commands, with explicit permission checks for writes. |
| REQ-SIG-012 | P1 | Proposed | Favorites, recent objects and personal visualization preferences are user-local by default, not extra files in an ECU project. |
| REQ-SIG-013 | P2 | Proposed | Cross-source derived or computed signals can be defined with documented units, timestamp semantics and quality propagation. |

## Module-specific UX proposals

- **CAN / DBC:** Network → message → signals; raw/physical values, decoding layout, multiplexed signals, cyclic update frequency, send-by-signal where allowed.
- **CANopen:** Node → Object Dictionary index/subindex; NMT state, SDO read/write, PDO mappings, Heartbeat and EMCY views.
- **XCP:** ECU → measurement and calibration objects; DAQ configuration, measurement plots, calibration value grid, read/write validation.
- **UDS:** ECU → diagnostic service → DID/DTC/routines; session state, request/response details, explicit read/write operations rather than pretending all DIDs stream continuously.
- **Future Ethernet:** endpoint/service → structured signals/fields; packet detail and protocol-specific decode, shared timestamps when possible.

## Acceptance tests

1. Show DBC signals, CANopen object entries and XCP measurement objects in one explorer without collapsing their distinct capabilities.
2. Search a symbol, open the relevant module-specific editor, and jump to the original description resource.
3. Display an XCP measurement and a decoded CAN signal in one plot with provenance and timing quality.
4. Attempt to drag a read-on-demand UDS DID to a streaming plot; show a supported sampling option or a clear explanation rather than silently polling.
5. Close/recreate a dock panel without losing explorer selection or active measurement session.

## Open details

Mandatory module-specific widgets for MVP; polling rate controls; derived signal language; name collisions and aliases; rights model for writable objects.
