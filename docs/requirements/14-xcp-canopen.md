# XCP Calibration and CANopen Network Management

**Revision:** 0.7 · **Status:** Accepted long-term product scope, proposed detail

## Accepted requirements

| ID | Proposed priority | Status | Requirement |
|---|---|---|---|
| REQ-XCP-006 | P2 | Accepted | Provide automated XCP calibration workflows in addition to a professional interactive calibration workspace. |
| REQ-XCP-007 | P1 | Accepted | Manage multiple A2L descriptions and XCP ECU sessions with automatic A2L-to-ECU version/identity compatibility checking. |
| REQ-OPEN-006 | P2 | Accepted | Provide a graphical CANopen network-management workspace for discovering, monitoring and configuring nodes. |

## Proposed requirements

| ID | Status | Requirement |
|---|---|---|
| REQ-XCP-008 | Proposed | Each XCP session is linked to a specific logical transport endpoint, A2L description and version-matching status. |
| REQ-XCP-009 | Proposed | Compatibility checking uses available ECU identification/fingerprint and A2L metadata; inconclusive results are distinguished from confirmed mismatches. |
| REQ-XCP-010 | Proposed | Automated calibration has limits, validation, progress reporting, write permissions, dry-run/simulation where feasible and rollback only when supported by the ECU. |
| REQ-XCP-011 | Proposed | Calibration objects, curves and maps expose raw and physical representation, conversion and write/compare status. |
| REQ-OPEN-007 | Proposed | CANopen nodes are discovered and shown by Node-ID with NMT state, Heartbeat, PDO/SDO activity, EMCY and available EDS/DCF mapping. |
| REQ-OPEN-008 | Proposed | Configuration changes via SDO/NMT are gated by explicit transmission authorization, device capabilities and known operational state. |
| REQ-OPEN-009 | Proposed | Network management supports save/restore configuration only when supported by the device; changed settings and errors are auditable. |

## Acceptance criteria

- [ ] Open two different A2L files and two XCP sessions; bind each to its own ECU and retain independent measurements/calibrations.
- [ ] Validate matching, mismatched and unknown ECU/A2L identities; unknown never masquerades as verified-compatible.
- [ ] Run a simulated calibration script with range checks, traceable changes and failed-write handling; a failed validation blocks unsafe writes.
- [ ] Display a simulated CANopen network with at least two nodes, NMT states, Heartbeat and EMCY details.
- [ ] Configure an Object Dictionary value through authorized SDO, observe the result, and test timeout/abort handling.
- [ ] Disconnect a node during configuration; no false success state is displayed.

## Scope and open decisions

- Which ECU identity mechanisms and A2L naming/conversion/version features are authoritative, and what to do when metadata is missing.
- Automatic optimization/calibration strategy, deterministic results, simulation support and safe write sequencing.
- CANopen CiA 301 versus future CANopen FD/profiles, node scanning policy and commissioning workflows.
