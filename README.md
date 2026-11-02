# CANexus-Studio

CANexus-Studio is a planned cross-platform, high-performance CAN engineering application.

## Product direction

A standalone Windows/Linux desktop application combining CAN/CAN FD monitoring, analysis, diagnostics, calibration, and automation. The application is built around a Rust performance core, Slint UI and pbSlintDock workspaces, with an extensible plugin architecture.

An ECU repository needs only one CANexus project configuration file, provisionally named `canexus.toml`, referring to existing ECU resources using paths relative to that file. CANexus-Studio itself and runtime data live outside ECU repositories.

## Specifications

See [Requirements](docs/requirements/README.md). Requirements are maintained as Markdown files in this repository, **not** GitHub Issues.

This repository and its specifications are in an early design phase. Items marked `Proposed` have not yet been accepted.
