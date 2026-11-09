# CANexus-Studio — Architecture Entry Point

**Current implementation baseline:** inspected `main` at `5e801ec5e395657d4998f3ce795a1ce16393b64c` on 2026-11-09.  
**Target state:** technical proposal, not proof of product requirement acceptance or feature completion.

- [Target software architecture](TARGET-ARCHITECTURE.md): responsibility boundaries, typed data flow, runtime sessions, workers, memory ownership, application services, HAL, extensibility and implementation gates.
- [Architecture weaknesses and remediation plan](ARCHITECTURE-REMEDIATION.md): source-backed limitations, severity, planned corrections, dependencies, parallel ownership and acceptance tests.
- [Implementation status](STATUS.md) and [development backlog](BACKLOG.md): actual merged work, task state and CI evidence.
- [Requirements index](../requirements/README.md): canonical product requirements and accepted/proposed distinction.
- [ADR-011, core architecture](../requirements/22-adr-011-high-performance-core-architecture.md), [ADR-009, Trace](../requirements/20-adr-009-high-performance-can-trace.md) and [ADR-008, measurement](../requirements/23-adr-008-central-measurement-engine.md): detailed proposed architectural contracts.

## What exists on main

The Rust workspace contains:

| Crate | Actual current role |
|---|---|
| `crates/canexus-core` | Headless typed CAN/CAN-FD models, timestamp/source/event primitives, deterministic virtual CAN, bounded generic batch ingress and bounded source-local RAM Trace history/query |
| `crates/canexus-cli` | Minimal headless bootstrap binary, not yet a capture/analysis command interface |
| `crates/canexus-gui` | Slint + pbSlintDock single-window feasibility host; Trace is a placeholder, not a live/virtual production panel |

The current source code does not include a physical HAL, production recorder, protocol engines, central measurement engine, general application service facade or stable external plugin ABI. Successful Rust CI must not be confused with interactive GUI/hardware validation.

## Architectural dependency direction

`GUI / CLI / scripts / optional AI adapters` → **application services** → **headless domain/runtime services** → **typed core and HAL contracts** → vendor/device implementations.

The application remains a **modular in-process Rust monolith**. Slint is not part of the capture hot path; external IPC is optional only for isolation or integration. The current three-crate workspace need not be split into many crates before responsibilities have been prototyped and verified.

## Implementation and verification rules

Keep existing functional API contracts where sound, fix correctness and bounded-memory limitations before relying on them for hardware workflows, and benchmark before optimizing allocation layout. Document every migration with requirement → implementation → test → CI evidence and unresolved limitations.

The pinned Rust toolchain is described in `rust-toolchain.toml`. On a development machine with Rust installed, run:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace --all-targets
cargo test --workspace --all-targets
```

GitHub Actions runs the configured Windows and Linux jobs. Headless tests do not verify native floating windows, actual adapter clocks, bus behavior or full end-to-end performance. Each such claim requires its own reproducible evidence.
