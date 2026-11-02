# CANexus-Studio Requirements

**Revision:** 0.6 draft · **Source of truth:** this repository's Markdown files.

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

## Confirmed requirements since v0.5

- A CANalyzer-like Trace Viewer with chronological and fixed/overwrite modes, column sorting and filtering.
- Unified Signal Explorer with protocol/module-specific views.
- Manual, cyclic, sequence-based, signal-value-driven and triggered transmission.
- A scripting layer in addition to native plugins, with module-specific APIs.
- An extensible network architecture which can incorporate future Ethernet without implementing it in the initial CAN MVP.

## Topics requiring decisions

- Exact project file name and configuration schema.
- Should workspace layouts be user-local by default, or optionally project-shared?
- MVP hardware adapters and benchmark reference systems.
- Recorder native format and interchange formats.
- Required CANopen, UDS, XCP profiles for the first usable release.
- Security model for plugins, scripts and transmit permissions.
- Choice of scripting language/runtime (Python proposed, not yet fixed).
- Generic event representation, timestamp correlation and Ethernet integration milestones.

## Process

Propose in discussion → write/update Markdown → review contradictions, feasibility and tests → mark `Accepted` → implement → mark `Verified` with test evidence. 

12. [Accepted Decisions and Conflict Register](11-decisions-v0.6.md)

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
