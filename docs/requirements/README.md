# CANexus-Studio Requirements

**Revision:** 0.4 draft · **Source of truth:** this repository's Markdown files.

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

## Topics requiring decisions

- Exact project file name and configuration schema.
- Should workspace layouts be user-local by default, or optionally project-shared?
- MVP hardware adapters and benchmark reference systems.
- Recorder native format and interchange formats.
- Required CANopen, UDS, XCP profiles for the first usable release.
- Security model for plugins and transmit permissions.

## Process

Propose in discussion → write/update Markdown → review contradictions, feasibility and tests → mark `Accepted` → implement → mark `Verified` with test evidence. 
