# DEV-003: Slint + pbSlintDock feasibility

**Stage:** Spike merged to main as `bef8513d`; feasibility acceptance remains blocked pending native float/re-dock and interactive Windows/Linux proof (DEV-016).

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

## Exit criteria and evidence

- [x] Windows `cargo fmt`, Clippy, build and tests green in GitHub Actions run [42501580558](https://github.com/JannisRln/CANexus-Studio/actions/runs/42501580558).
- [x] Linux `cargo fmt`, Clippy, build and tests green in GitHub Actions run [42501580558](https://github.com/JannisRln/CANexus-Studio/actions/runs/42501580558).
- [ ] Native floating-window roundtrip exercised in the CANexus GUI on both OSes.
- [ ] Persistent state across float/re-dock demonstrated in the host.
- [ ] Follow-up decision: choose supported integration strategy and document
      which parts remain prototypes.
- [ ] Disk-backed layout persistence across process restart verified on both OSes.
- [ ] Interactive tabs, docking, splitters and close/reopen verified on both OSes.

**Known limitation:** the first CANexus UI sample is single-window; tests
cover float transitions only at the pbSlintDock engine level. Layout snapshots
are process-local memory, not persisted to disk. Prior successful PR CI confirms
compilation and automated unit tests only, not a GUI interaction scenario.

**Follow-up DEV-016:** implement native Slint float-window creation/closure,
transfer the logical panel between MAIN_WINDOW and floating windows and back,
restore layout and panel state after process restart, and demonstrate docking,
tabs, splitter drag, close/reopen and float/re-dock on both Windows and Linux.
Document a reproducible test procedure with results, screenshots or recordings,
and any platform-specific caveats. Choose and record the integration strategy
before marking DEV-003 Done. See `DEV-003-004-ACCEPTANCE.md`.
