# Protocols & Description Formats

**Status:** Proposed · **Revision:** 0.7

## Shared principles

Transport layers, protocol state machines, configuration models, ECU resources and views are separate modules. Protocols consume shared capability-based transport/session services rather than opening hardware directly. CAN protocol plugins use the CAN transport; future Ethernet-based protocols use an Ethernet transport without being represented as CAN frames. Protocol features are exposed through services that remain testable without the UI.

See [Multi-network Extensibility](09-network-extensibility.md) for future Ethernet and [Unified Signal Explorer](07-signal-workspace.md) for module-specific object semantics. Script APIs per module are specified in [Scripting](08-scripting-automation.md).

## Network descriptions (DBC)

| ID | Priority | Status | Requirement |
|---|---|---|---|
| REQ-DBC-001 | P0 | Proposed | Import DBC files by relative project reference. |
| REQ-DBC-002 | P1 | Proposed | Decode/encode signals including bit order, signedness, scaling, multiplexing and value tables. |
| REQ-DBC-003 | P1 | Proposed | Show message/signal browser, search, description and source file locations. |
| REQ-DBC-004 | P2 | Proposed | Detect/report invalid or conflicting definitions without blocking unrelated networks. |
| REQ-DBC-005 | P2 | Accepted | DBC files are read-only initially; a DBC editor is planned for a later milestone. |

## Diagnostics (ISO-TP and UDS)

| ID | Priority | Status | Requirement |
|---|---|---|---|
| REQ-DIAG-001 | P1 | Proposed | Provide standalone ISO-TP transport handling with configurable addressing and flow control. |
| REQ-DIAG-002 | P1 | Proposed | UDS service framework with request/response handling and configurable timeouts. |
| REQ-DIAG-003 | P1 | Proposed | Diagnostic sessions, Tester Present, Read/WriteDataByIdentifier and DTC operations. |
| REQ-DIAG-004 | P1 | Proposed | Show raw UDS/ISO-TP frames and interpreted service results together. |
| REQ-DIAG-005 | P2 | Proposed | Security Access through a configured authorized provider; sensitive secrets do not enter versioned ECU project files. |
| REQ-DIAG-006 | P2 | Proposed | Reusable diagnostic sequences and automation with explicit bus-transmission authorization. |

## XCP measurement and calibration

| ID | Priority | Status | Requirement |
|---|---|---|---|
| REQ-XCP-001 | P1 | Proposed | XCP on CAN with an independent transport abstraction. |
| REQ-XCP-002 | P1 | Proposed | Parse referenced A2L files and expose measurement and calibration objects. |
| REQ-XCP-003 | P1 | Proposed | DAQ measurement and synchronized visualization with other CAN signals. |
| REQ-XCP-004 | P1 | Proposed | Calibration read/write workflows with visible state, validation and operator safeguards. |
| REQ-XCP-005 | P2 | Proposed | XCP on Ethernet via an additional transport plugin. |

## CANopen

| ID | Priority | Status | Requirement |
|---|---|---|---|
| REQ-OPEN-001 | P1 | Proposed | Provide CiA 301 CANopen node state, NMT, Heartbeat and error monitoring. |
| REQ-OPEN-002 | P1 | Proposed | SDO upload/download with explicit timeout and abort-code diagnostics. |
| REQ-OPEN-003 | P1 | Proposed | PDO and EMCY decoding; expose device Object Dictionary. |
| REQ-OPEN-004 | P1 | Proposed | Parse EDS/DCF resource files referenced from the ECU project. |
| REQ-OPEN-005 | P2 | Proposed | Support active device configuration/commissioning and CANopen profiles to be selected. |

## Open decisions

Precise supported UDS service list, addressing modes and diagnostic database formats; A2L coverage and calibration safety; CANopen devices/profiles and CANopen FD; write-access defaults for all active operations.

## v0.6 DBC requirements

| ID | Status | Requirement |
|---|---|---|
| REQ-DBC-005 | Accepted | Initial DBC support is read-only; a DBC editor is planned for a later milestone. |
| REQ-DBC-006 | Accepted | Default interpretation of colliding CAN identifiers follows configurable DBC priority order. |
| REQ-DBC-007 | Accepted | Additional applicable DBC interpretations can be enabled/disabled and filtered independently. |
| REQ-DBC-008 | Proposed | Decoding tracks channel/network, standard/extended ID, DBC source and definition, avoiding ambiguous identity. |

Acceptance: load two DBCs defining CAN 0x123 differently; first wins as default and both interpretations remain available; disable the first to select the next enabled one; reverse order to change default; neither activity alters original frames or DBC files.



## v0.7 protocol feature extensions

- **UDS:** structured service browser with raw mode, long-term ODX/PDX/CDD/vendor descriptions, extensible flashing framework and stateful virtual UDS ECU.
- **XCP:** automated calibration workflows; multiple A2Ls and ECU sessions with explicit identity/version compatibility results.
- **CANopen:** graphical node discovery and network-management workspace.

These product features are Accepted; their details and milestone allocations are defined in [Diagnostics/Flashing/Simulation](13-diagnostics-flashing-simulation.md) and [XCP/CANopen](14-xcp-canopen.md). The existing base protocol requirements remain applicable. No unvalidated claim of complete support for arbitrary vendor file versions or ECU flashing recovery is made.
