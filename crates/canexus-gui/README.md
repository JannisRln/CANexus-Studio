# DEV-003 — Slint/pbSlintDock feasibility spike

This is a **prototype**, not the production GUI or public UI plugin SDK.

The Rust `DockManager` from [pbSlintDock](https://github.com/pbtechlab/pbSlintDock)
drives the `DockHost` in `ui/main.slint`. The UI includes Explorer, Trace
placeholder and Details panels, drag/tab/split callbacks, close/reopen and
in-memory layout save/restore. The independent Rust tests check layout
persistence and floating layout transitions (engine only).

## Build and run

Requires Rust 1.85.0, a working C++20 compiler and platform Slint native
dependencies. Use the **pinned** pbSlintDock checkout:

```sh
git clone https://github.com/pbtechlab/pbSlintDock third_party/pbSlintDock
git -C third_party/pbSlintDock checkout 059418b3704bbe0b4699e89341a4567a37904fc9
export PBSLINTDOCK_ROOT="$PWD/third_party/pbSlintDock"
cargo run -p canexus-gui
cargo test -p canexus-gui
```

On PowerShell:

```powershell
git clone https://github.com/pbtechlab/pbSlintDock third_party/pbSlintDock
git -C third_party/pbSlintDock checkout 059418b3704bbe0b4699e89341a4567a37904fc9
$env:PBSLINTDOCK_ROOT = (Resolve-Path third_party/pbSlintDock).Path
cargo run -p canexus-gui
cargo test -p canexus-gui
```

The pinned upstream commit is used in both the Cargo dependency and CI UI
asset checkout. The upstream code builds a C++20 layout engine through its
Rust bindings.

## Limits and verification

- The CANexus prototype is **single-window only**. Its Rust tests exercise
  floating layout transitions, but it **does not create native floating Slint
  windows** or implement float drag/close callbacks. Verify the upstream
  `pbslintdock` Rust demo for that behavior, and implement a CANexus float
  window factory before treating REQ-UX-009 or REQ-PLG-012 as verified.
- Layout save/restore is currently in-memory; no filesystem persistence or
  versioning is implemented.
- No dynamic third-party panel loading, missing-panel placeholder or stable
  plugin ABI is implemented.
- Windows/Linux CI builds and runs Rust tests; it does not provide a manual
  UI interaction or rendering test. CI results, not this README, determine
  which platforms were actually verified.
- The `WorkspaceState` global is a presentation-only sample. Real state
  shared between native floating windows must be mirrored by a Rust owner.
- No CAN data path is included; the Trace panel is a placeholder.

These limits leave REQ-UX-004, REQ-UX-009, REQ-PLG-012..014 and related
acceptance criteria **unverified**.
