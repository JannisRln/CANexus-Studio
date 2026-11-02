# Product Vision & Scope

**Status:** Proposed · **Revision:** 0.4

## Vision

CANexus-Studio is a standalone engineering desktop application for observing, decoding, measuring, calibrating, diagnosing, recording, simulating and automating CAN-based ECU systems. Inspiration: CANalyzer, CANoe and CANape, without requiring feature-for-feature parity for the MVP.

## Confirmed boundaries

- Windows and Linux desktop.
- Rust performance core; Slint UI; pbSlintDock for dockable workspaces.
- Standalone installation independent of any ECU repository.
- A single CANexus project configuration file in an existing ECU repository references its existing resources.
- Project-internal paths are relative to the project file; checked-out copies/worktrees use their own resources.
- No requirement to open or manage multiple worktrees simultaneously.
- Requirements live in Markdown in the CANexus-Studio repository, without GitHub Issues.

## Proposed product qualities

| ID | Priority | Status | Requirement |
|---|---|---|---|
| REQ-SYS-001 | P0 | Proposed | The application must function without Internet access for normal local workflows. |
| REQ-SYS-002 | P0 | Proposed | A slow UI, optional plugin, or reader must not silently compromise recording or protocol processing. |
| REQ-SYS-003 | P0 | Proposed | Windows/Linux differences must be isolated behind platform/HAL services. |
| REQ-SYS-004 | P1 | Proposed | Users must be able to run common CAN tasks without writing scripts. |
| REQ-SYS-005 | P1 | Proposed | Power users must be able to automate workflows via stable application APIs. |
| REQ-SYS-006 | P0 | Proposed | Application failures must produce actionable diagnostics without modifying ECU sources. |

## Candidate product areas

- Portable project and resource management.
- High-performance UI and multi-panel workspace.
- CAN Classic / CAN FD acquisition and transmission.
- Hardware adapter management.
- CAN Trace, filtering, trigger, logging and replay.
- DBC browsing and signal decoding.
- ISO-TP and UDS diagnostics.
- XCP measurement and calibration with A2L.
- CANopen network/device exploration using EDS/DCF.
- Signal plots, automation, tests, and simulation.
- Extensible adapter, protocol, format and UI plugin system.

## Out of scope for first iterations (proposed)

Full parity with commercial suites, simultaneous worktree management, mandated cloud services, mandatory changes to ECU repository structure.

## Open decisions

MVP boundaries, minimum operating system versions, license and distribution model, minimum tool integrations.
