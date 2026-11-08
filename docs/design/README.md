# CANexus Studio — UX/UI design concept v0.1

**Status:** Proposed; not an accepted product or implementation baseline. **Date:** 2026-11-08.

This folder contains the initial visual and interaction design for the Windows/Linux Slint + pbSlintDock desktop application. The normative product requirements remain in [../requirements/](../requirements/).

## Contents

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

The native Slint theme contract is ready for integration after the Rust workspace and docking proof of concept. The legacy HTML/JSON files remain design references, not runtime theme definitions.


Validate the main-shell, table and dock interactions using an actual Slint + pbSlintDock proof of concept; agree on theme, high-DPI/accessibility and keyboard conventions; decide whether to accept individual UI requirements in the canonical requirements files.
