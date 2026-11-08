# DEV-003: Slint + pbSlintDock feasibility

**Stage:** Implementation proposed in PR; no platform verification recorded yet.

## Scope

- Add an optional Rust `canexus-gui` executable in the main workspace.
- Use Slint 1.17.1 and the pinned pbSlintDock Rust binding for panel layout.
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

## Exit criteria and evidence

- [ ] Windows `cargo fmt`, Clippy, build and tests green in a specific CI run.
- [ ] Linux `cargo fmt`, Clippy, build and tests green in a specific CI run.
- [ ] Native floating-window roundtrip exercised in the CANexus GUI on both OSes.
- [ ] Persistent state across float/re-dock demonstrated in the host.
- [ ] Follow-up decision: choose supported integration strategy and document
      which parts remain prototypes.

**Known limitation:** the first CANexus UI sample is single-window; tests
cover float transitions only at the pbSlintDock engine level.
