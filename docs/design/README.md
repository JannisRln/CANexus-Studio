# CANexus Studio — UX/UI design concept v0.1

**Status:** Proposed; not an accepted product or implementation baseline. **Date:** 2026-11-08.

This folder contains the original visual design and the target [egui workspace and component catalog](EGUI-WORKSPACE.md). Use the historical Slint and HTML concepts as interaction references. Product requirements live in [../requirements/](../requirements/).

## Contents

- [EGUI-WORKSPACE.md](EGUI-WORKSPACE.md) — target native workspace, design tokens, component catalog and showcase acceptance.

- [slint/](slint/README.md) — Native Fluent-based Slint theme foundation with runtime System/Light/Dark switching and reusable components (prototype; not yet compiler-validated).

- [DESIGN-SYSTEM.md](DESIGN-SYSTEM.md) — UX principles, components, patterns, implementation constraints and open decisions.
- [tokens.json](tokens.json) — machine-readable proposed theme and spacing tokens.
- [PROTOTYPE.html](PROTOTYPE.html) — standalone interactive HTML/CSS/JavaScript concept with synthetic data; open directly in a modern browser.

### Mockups

Four 1600×960 prototype screenshots were produced: **CAN Analyzer**, **Measurement & Calibration**, **UDS Diagnostics**, and **Design System gallery**. They are available in the original UX design package, but the PNG binary assets have **not yet been committed** to this repository. To reproduce screenshots, run `python -m pip install playwright`, `python -m playwright install chromium`, then `python docs/design/render_mockups.py` from the repository root. The script generates all four PNGs locally from the committed prototype at 1600×960. Do not mistake concept screenshots for product implementation evidence.

## Interaction boundaries

The HTML prototype simulates UI state only. It does **not** connect to hardware, transmit frames, access ECUs, perform XCP writes, capture data or create recordings.

UX decisions in this package do not silently supersede requirements. In particular:
- Capture, recording and display pause are separate actions.
- View-level filters never implicitly change captured or recorded data.
- XCP draft editing cannot implicitly write to an ECU.
- Docking/view state is distinct from long-lived session or measurement state.
- User-local UI state must not generate extra files in ECU source repositories.

## Next

Use DEV-024/025 to verify egui/Craft behavior on Windows and Linux and build the interactive native showcase. The [GUI migration plan](../development/GUI-MIGRATION.md) defines the evidence required before cutover.
