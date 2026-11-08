# DEV-003 / DEV-004 acceptance traceability

Review date: 2026-11-08. Baseline: main `10e1bd5cdfaa14b1af189ad7891f4bf79f24ecf4`.
Review scope: DEV-004 typed CAN frame / timestamp foundation and DEV-003 Slint/pbSlintDock feasibility spike.
The requirement source statuses (Proposed/Accepted) are **not** altered by this evidence matrix.
These four acceptance statuses are **Erfüllt**, **Teilweise erfüllt**, **Offen**, **Blockiert**.
CI passing a PR never establishes end-to-end requirement fulfillment.

| Requirement | Implementation | Test evidence | Status |
|---|---|---|---|
| REQ-CAN-001 | `can.rs` CAN Classic/FD ID, DLC, flags, inline payload; `event.rs` channel, time | `identifier_boundaries_and_format_are_preserved`, `all_fd_dlc_values_map_to_exact_payload_lengths`, `reject_illegal_dlc_values_and_remote_payloads`, `fd_payload_boundaries_and_unused_capacity_are_preserved`; CI pending for acceptance branch | Teilweise erfüllt |
| REQ-CORE-004 | `event.rs::Observed<T>` channel, Rx/Tx, `EventTimestamp` | `observed_can_frame_retains_identity_and_time`; no actual capture adapter provenance test | Teilweise erfüllt |
| REQ-NET-002 | `event.rs::Observed<T>` typed payload + source/sequence/time; transport identified by payload type at compile time, no explicit runtime transport discriminator/session generation | `envelope_can_hold_non_can_payload_without_allocation`, `raw_identity_is_scoped_to_source_and_sequence`; no plugin/batch integration test | Teilweise erfüllt |
| REQ-NET-003 | `can.rs::CanFrame` inline [u8; 64]; `Observed<T>` generic owned envelope, no boxed trait object | `frame_uses_inline_storage_without_drop_glue`, `envelope_can_hold_non_can_payload_without_allocation`; no allocation-instrumented steady-state benchmark | Teilweise erfüllt |
| REQ-TIME-001 | `time.rs` hardware/host/synthetic origin, domain, rational tick resolution and quality; no clock correlation/synchronization service | `preserves_hardware_provenance_and_precise_scaling`, `refuses_cross_domain_deltas_and_discontinuities`, `same_ticks_in_different_domains_are_not_comparable`, `overflow_is_explicit_not_wrapped`; no multi-adapter sync fixture | Teilweise erfüllt |
| ADR-011 (proposed architectural detail) | Typed immutable-by-value CAN model, generic typed envelope, source-scoped identity and explicit time; no runtime session generations, raw event variants, batches, queues, overflow or shutdown | Unit tests in `can.rs`, `event.rs`, `time.rs`; full acceptance tests #1–20 not implemented | Teilweise erfüllt |
| ADR-009 (proposed Trace detail) | DEV-003 trace panel is placeholder only | No virtualized trace, filter, sort, history or GUI benchmark | Offen |
| DEV-003 docking/tabs/splitters | `crates/canexus-gui/src/main.rs` binds Slint `DockHost` to pbSlintDock engine | PR #12 Windows/Linux formatting, Clippy, compile and engine unit tests only; no manual GUI scenario evidence | Teilweise erfüllt |
| DEV-003 close/reopen and layout snapshot | `initial_dock`, toggle callbacks, `save_layout` / `restore_layout` in single-window GUI | `layout_survives_close_and_restore`; snapshot is in-memory, not persistent across process restart | Teilweise erfüllt |
| DEV-003 native float/re-dock | Engine float/dock methods, but host only uses `MAIN_WINDOW`; no second native Slint window / host lifecycle | `can_float_and_redock_without_losing_panel_registration` exercises engine only; no Windows/Linux GUI roundtrip | Blockiert |

## Limits and deferred work

- `CanFrame` is inline storage with no heap-owned members. `size_of` / `needs_drop` tests are **not** a measured allocation trace. Allocation profiles belong in DEV-006/009 performance validation.
- Raw event IDs are source-scoped. Preventing reuse after a source restart needs runtime session generation and its tests (later ingestion/HAL work), not an unsupported claim about DEV-004.
- A timestamp resolution is local to a clock domain. Cross-domain correlation, uncertainty/drift and valid intervals are not implemented; comparing unrelated raw clocks is deliberately disallowed.
- DLC legality and bytes are represented; remote frame DLC denotes requested length while the payload is empty. Error and bus-state event variants are outside DEV-004.
- No physical device, native GUI roundtrip, or performance benchmark has been executed.
- Completion evidence for the acceptance branch must refer to its own Windows/Linux GitHub Actions run; do not reuse earlier successful PR checks to assert acceptance-branch success.

## Automated CI verification

- Initial acceptance-branch head `cd014897272836e58912fd8066464c0e03ce1724` passed all four Windows/Linux GitHub Actions jobs in [run 42511275998](https://github.com/JannisRln/CANexus-Studio/actions/runs/42511275998): project-state, `cargo fmt --all -- --check`, Clippy with `-D warnings`, workspace build and all-target tests.
- That is executable test evidence for the unit cases above, **not** proof of native GUI interaction or allocation instrumentation. Final documentation-only PR head should also receive its own CI check.
