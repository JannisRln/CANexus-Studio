# Protocols & Description Formats

**Status:** Proposed · **Revision:** 0.4

## Shared principles

Transport layers, protocol state machines, configuration models, ECU resources and views are separate modules. Protocols consume one shared CAN Core / HAL rather than opening hardware directly. Protocol features are exposed through services that remain testable without the UI.

## Network descriptions (DBC)

| ID | Priority | Status | Requirement |
|---|---|---|---|
| REQ-DBC-001 | P0 | Proposed | Import DBC files by relative project reference. |
| REQ-DBC-002 | P1 | Proposed | Decode/encode signals including bit order, signedness, scaling, multiplexing and value tables. |
| REQ-DBC-003 | P1 | Proposed | Show message/signal browser, search, description and source file locations. |
| REQ-DBC-004 | P2 | Proposed | Detect/report invalid or conflicting definitions without blocking unrelated networks. |
| REQ-DBC-005 | P2 | Proposed | Offer DBC editing with deterministic formatting (separate scope decision). |

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
