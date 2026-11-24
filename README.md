# CANexus-Studio

CANexus-Studio is a planned cross-platform, high-performance CAN engineering application.

## Product direction

A standalone Windows/Linux desktop application combining CAN/CAN FD monitoring, analysis, diagnostics, calibration, and automation. The target combines a headless Rust Core, egui desktop UI and Craft components behind a workspace adapter, with typed extension contracts. The [GUI migration ADR](docs/requirements/26-adr-012-egui-craft-ui.md) records the direction and the verified Slint prototype baseline.

An ECU repository needs only one CANexus project configuration file, provisionally named `canexus.toml`, referring to existing ECU resources using paths relative to that file. CANexus-Studio itself and runtime data live outside ECU repositories.

## Specifications

See [Requirements](docs/requirements/README.md). Requirements are maintained as Markdown files in this repository, **not** GitHub Issues.

This repository and its specifications are in an early design phase. Items marked `Proposed` have not yet been accepted.

## UX/UI design concept

The proposed [UX design system and interactive desktop prototype](docs/design/README.md) cover the CAN Analyzer, Measurement/Calibration, and UDS Diagnostics workflows. The [target egui workspace specification](docs/design/EGUI-WORKSPACE.md) adds the native component catalog and migration acceptance criteria.

## License

CANexus-Studio is licensed under the [MIT License](LICENSE).

## Development through ChatGPT

Implementation progress, task planning, decisions and chat-to-chat handoffs are stored in [development documentation](docs/development/README.md). Start with [current status](docs/development/STATUS.md) and [backlog](docs/development/BACKLOG.md). Substantial work uses feature branches and pull requests. CI is configured for Windows and Linux. The initial code milestone is a virtual-CAN-to-Trace vertical slice.
