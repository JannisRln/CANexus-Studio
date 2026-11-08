# CANexus-Studio Requirements

**Revision:** 0.7 draft · **Source of truth:** this repository's Markdown files.

## Rules

- **No GitHub Issues** are used to store or manage requirements.
- Use stable IDs: `REQ-<DOMAIN>-NNN`.
- Status is `Proposed`, `Accepted`, `Implemented`, or `Verified`.
- Priorities are `P0` foundation, `P1` early product, `P2` later, `P3` optional.
- Proposed priorities and targets are not binding until agreed.
- Each accepted requirement needs a rationale and measurable acceptance criteria.
- Preserve IDs when editing; explicitly document superseded requirements.
- The application's own repository contains these specifications. ECU repositories do **not** contain CANexus documentation, binaries, or runtime data: only a project configuration file is needed.

## Current specifications

1. [Product Vision & Scope](00-product-vision.md)
2. [Project Model & Portability](01-project-model.md)
3. [User Experience & Performance](02-ux-performance.md)
4. [CAN Analyzer & Data Workflows](03-can-analyzer.md)
5. [Protocols & Data Formats](04-protocols.md)
6. [HAL & Plugin Extensibility](05-hal-plugins.md)
7. [Trace Viewer & Column Filtering](06-trace-viewer.md)
8. [Unified Signal Explorer & Module Views](07-signal-workspace.md)
9. [Scripting & Automation](08-scripting-automation.md)
10. [Future Networks & Ethernet Extensibility](09-network-extensibility.md)
11. [Transmission, Triggers & Replay](10-transmission-trigger-replay.md)
12. [v0.6 Accepted Decisions & Conflict Register](11-decisions-v0.6.md)

13. [Measurement, Dashboards & Time](12-measurement-dashboard-time.md)
14. [Diagnostics, Flashing & UDS Simulation](13-diagnostics-flashing-simulation.md)
15. [XCP & CANopen Management](14-xcp-canopen.md)
16. [Python Workers & Tests](15-python-tests-editor.md)
17. [Dockable UI Plugins & Commands](16-ui-plugins-commands.md)
18. [v0.7 Decisions & Architecture Conflict Register](17-decisions-v0.7.md)
19. [ADR-006 — Project Configuration & Path Resolution (Proposed)](18-project-configuration-path-resolution.md)
20. [ADR-007 — ELF/DWARF Symbols, Metadata, Dashboards & XCP](19-adr-007-elf-dwarf-symbol-calibration.md)
21. [ADR-009 — High-Performance CAN Trace Viewer (Proposed detail)](20-adr-009-high-performance-can-trace.md)
22. [ADR-010 — Unified Measurement & Calibration Workspace UX (Proposed detail)](21-adr-010-measurement-calibration-workspace-ux.md)
23. [ADR-011 — High-Performance Core Architecture & Data Flow (Proposed detail)](22-adr-011-high-performance-core-architecture.md)

## Confirmed requirements since v0.5

- A CANalyzer-like Trace Viewer with chronological and fixed/overwrite modes, column sorting and filtering.
- Unified Signal Explorer with protocol/module-specific views.
- Manual, cyclic, sequence-based, signal-value-driven and triggered transmission.
- A scripting layer in addition to native plugins, with module-specific APIs.
- An extensible network architecture which can incorporate future Ethernet without implementing it in the initial CAN MVP.

## Topics requiring decisions

- Exact project file name and configuration schema.
- Exact project-shared vs user-local preference schemas and override precedence.
- MVP hardware adapters and benchmark reference systems.
- Select verified interoperable direct-recording standard format (MDF4 candidate).
- Required CANopen, UDS, XCP profiles for the first usable release.
- Security model for plugins, scripts and transmit permissions.
- Python runtime distribution, worker process lifecycle, dependency management, permission isolation.
- Generic event representation, timestamp correlation and Ethernet integration milestones.

## Process

Propose in discussion → write/update Markdown → review contradictions, feasibility and tests → mark `Accepted` → implement → mark `Verified` with test evidence. 

## v0.6 accepted decisions

- DBC collision resolution: ordered default interpretation plus toggleable concurrent interpretations.
- DBC sources initially read-only, editing later.
- Formula and Python-derived signals; professional signal plotting.
- Independent per-panel Trace presets.
- Direct-to-standard-format recording; standard choice TBD (MDF4 candidate).
- Python primary scripting language; external public plugin SDK; GUI and headless CLI testing.
- Logical channels mapped to local hardware; personal UI settings local, shared settings in project.
- Interactive UI first; project resources load in background.

See [v0.6 decisions and conflict register](11-decisions-v0.6.md).

## v0.7 accepted decisions

- Live, Replay and Offline use a shared professional plotter; built-in math starts with basic statistics.
- Custom dashboard widgets and dockable Slint panels may be supplied by external plugins.
- Prefer hardware timestamps, with explicit clock quality and cross-channel synchronization limits.
- UDS service browser with form and raw mode; long-term ODX/PDX/CDD/vendor import, universal extensible flashing and stateful virtual UDS ECUs.
- Automated XCP calibration with A2L/ECU compatibility checks; graphical CANopen network management.
- Isolated Python process per script session; Python files with test functions; external script editor instead of built-in editor.
- Fully customizable command system and macros shared across modules.

See [v0.7 decision and conflict register](17-decisions-v0.7.md). Accepted product direction does not imply MVP delivery or completed technical validation.
