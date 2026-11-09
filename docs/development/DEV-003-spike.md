# DEV-003: Slint + pbSlintDock feasibility

**Stage:** Feasibility spike integrated in main; task DEV-003 Done (scope-only closure, 2026-11-09).

## Scope

- Add an optional Rust `canexus-gui` executable in the main workspace.
- Use Slint 1.17.1 and the pinned pbSlintDock Rust binding for panel layout.
- Raise pinned Rust toolchain from 1.85 to 1.92: the Slint 1.17.1 dependency explicitly requires rustc 1.92; the prior toolchain fails dependency resolution.
- Demonstrate docking, tab and splitter input forwarding, close/reopen and
  snapshot-based layout restore with a sample three-panel workspace.
- Add engine-level persistence/floating tests and Windows/Linux CI builds.

## Architecture implications

- Dock panel lifecycle is independent from recording/protocol session state.
- Slint cannot reparent live components across native windows; future native
  floats need Rust-owned state, synchronization and window lifecycle logic.
- Vendor UI is resolved through `PBSLINTDOCK_ROOT`; the native layout
  library is built from the pinned Git dependency. No vendor sources are
  committed to the CANexus repository.
- This spike is not evidence of third-party compiled Slint plugin loading.
  Do not freeze the plugin SDK or claim REQ-PLG-012 verified.

## Verified DEV-003 exit criteria (minimal integration only)

- [x] Windows `cargo fmt`, Clippy, build and tests green in GitHub Actions run [42501580558](https://github.com/JannisRln/CANexus-Studio/actions/runs/42501580558).
- [x] Linux `cargo fmt`, Clippy, build and tests green in GitHub Actions run [42501580558](https://github.com/JannisRln/CANexus-Studio/actions/runs/42501580558).
- [x] GitHub PR [#12](https://github.com/JannisRln/CANexus-Studio/pull/12) squash-merged to main as `bef8513d8aeb56d4bd1144be21fdfd4f9a73cb17`.
- [x] Single Slint window contains a pbSlintDock host with three fixed built-in panel types, callback forwarding and in-memory layout snapshot restore (`crates/canexus-gui/src/main.rs`, `ui/main.slint`).
- [x] Engine-level Rust unit tests cover layout restore and the `DockManager` float/redock state transition (`layout_survives_close_and_restore`, `can_float_and_redock_without_losing_panel_registration`).
- [x] Follow-up architecture decision: native floating windows require a separate host-owned window lifecycle and shared state and are tracked under DEV-017, not silently accepted as tested by this spike (DEV-DEC-009).

## Explicitly not accepted by DEV-003 closure

- [ ] A real native floating Slint window is created, moved and docked back inside CANexus (DEV-017). The current GUI is **single-window only** and has no native float window factory.
- [ ] Window close/restore, focus, DPI, geometry, multi-monitor behavior and retained panel state are interaction-tested on both Windows and Linux (DEV-017).
- [ ] Persistent layout on disk, versioned workspaces and third-party dynamically compiled Slint dock panels / plugin placeholders work (later tasks; REQ-PLG-012..014 not verified).
- [ ] A production trace UI is present (DEV-008); the current Trace panel is a placeholder.

DEV-003's **original minimal-integration backlog acceptance** is supported by verified build/test evidence. Earlier broader checkboxes in this document are **deferred, not passed**. This is an explicit task split requested on 2026-11-09, not a claim that floating behavior was secretly tested. The spike demonstrates toolkit integration and engine-level lifecycle, not native graphical UX. REQ-UX-004/009 and REQ-PLG-012..014 retain their separate product-level statuses.

