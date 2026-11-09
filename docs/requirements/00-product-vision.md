# Product Vision & Scope

**Status:** Mixed · **Revision:** 0.7

## Vision

CANexus-Studio is a standalone engineering desktop application for observing, decoding, measuring, calibrating, diagnosing, recording, simulating and automating ECU network systems. CAN Classic and CAN FD are the first focus; the architecture must support future Ethernet and other network technologies. Inspiration: CANalyzer, CANoe and CANape, without requiring feature-for-feature parity for the MVP.

## Confirmed boundaries

- Windows and Linux desktop.
- Rust performance core; Slint UI; pbSlintDock for dockable workspaces.
- Standalone installation independent of any ECU repository.
- A single CANexus project configuration file in an existing ECU repository references its existing resources.
- Project-internal paths are relative to the project file; checked-out copies/worktrees use their own resources.
- No requirement to open or manage multiple worktrees simultaneously.
- Requirements live in Markdown in the CANexus-Studio repository, without GitHub Issues.
- The analyzer offers chronological and fixed-position Trace modes, with column sorting and filtering.
- The Signal Explorer is shared, while detailed UI panels are module-specific.
- Sending covers one-shot, cyclic, sequences, signals and triggers.
- A scripting layer complements native plugins for automation and module-specific APIs.
- New networks (including potential Ethernet) must be extensible without rebuilding the core.

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
- An independent scripting runtime for users, tests and module automation.
- Proposed AI-native debugging and artifact authoring via optional MCP tools and a built-in assistant, using the same guarded core services as GUI/CLI; see [AI-Native Integration & MCP](24-ai-native-mcp-integration.md).
- Future Ethernet/DoIP/SOME-IP integrations (not in the initial CAN MVP).

## Out of scope for first iterations (proposed)

Full parity with commercial suites, simultaneous worktree management, mandated cloud services, mandatory changes to ECU repository structure.

## Open decisions

MVP boundaries, minimum operating system versions, license and distribution model, minimum tool integrations; scripting runtime; packet/event abstractions. See [Trace](06-trace-viewer.md), [Signals](07-signal-workspace.md), [Scripting](08-scripting-automation.md), [Networks](09-network-extensibility.md), [TX and Triggers](10-transmission-trigger-replay.md).

## v0.7 scope decisions

Long-term features now explicitly accepted: unified live/replay/offline plotter with basic statistics, programmable custom dashboard widgets, hardware timestamp preference, UDS forms plus raw mode, ODX/PDX/CDD/vendor description import, extensible flashing, stateful virtual UDS ECU, automated XCP calibration with A2L/ECU checks, graphical CANopen management, per-session isolated Python workers, external Python editor, Python test files, public dockable Slint UI panels and fully customizable command system.

These are **product-scope commitments**, not guarantees of all features in the initial MVP. See [v0.7 decision register](17-decisions-v0.7.md).
