# CANexus-Studio desktop workspace design

**Status** Proposed design for egui and Craft UI · [Architecture decision](../requirements/26-adr-012-egui-craft-ui.md)

## Workflows and information architecture

| Workspace | Default composition and primary workflow |
|---|---|
| Welcome | Open Demo, Open Recording or Configure Device. Demo uses bundled virtual CAN and labels its source. Recent projects show missing resources and a repair action. The main window becomes interactive while optional resources load. |
| Analyze | Signal Explorer, dominant Trace, on-demand Inspector, bottom Events/Problems. The toolbar exposes source, capture, record, pause view and search as separate actions. Display filters show scope and reset. Trace supports chronological/Fixed, per-column filter and sort, stable selection, raw/decoded details and differences. |
| Measure | Signal Explorer, time-series Plot and Numeric Values. Shared timeline links active views by choice. Channel/time quality, units, staleness and data gaps remain visible. Live, Replay and Offline share viewer semantics. |
| Diagnostics and Calibration | Reuse status, explorer and command patterns. Use dedicated service forms, ECU identity, progress and result views. Calibration drafts stay local until an authorized write. Failed or unsupported operations show a targeted explanation and recovery action. |
| Automation | Test discovery, run results, output/traceback, status and cancellation. Open scripts in an external editor. Present GUI and CLI results with the same test IDs. |

The shell uses one persistent project/source header and one health strip. Channel, live/replay/offline mode, capture, recorder, view pause and losses are readable independently. Layout presets initialize views and preserve stable project/service state. Put advanced actions in contextual menus or the command palette. Hide unused regions in a focused or small-screen workspace. Keep the Trace dominant at 1280×720 logical pixels and restore its keyboard path when side panels collapse.

## Design tokens and catalog

Start from the existing [design tokens](tokens.json), preserve their semantic names and map them into `egui::Visuals`, `Style`, Craft control styles and CANexus engineering views. Store one theme source for System, Light and Dark. Resolve system-theme changes in the host. Evaluate packaged or system font coverage for German/English, technical glyphs and fallback. Track all shipped font and icon licenses. Use tabular monospace digits in timestamps, identifiers, payload and numeric readings. Use scalable logical points and test readable focus, contrast and glyphs at 100%, 150% and 200% display scaling.

| Showcase family | Reuse and CANexus responsibilities |
|---|---|
| Buttons, tabs, toolbars, pane splitters, inspector sections, dialogs | Evaluate Craft components and eframe native dialogs. Style all states from tokens. Display labels for actions and supply accessible names to icons. Include hover, focus, disabled, error, long labels and keyboard-only examples. |
| Data tables and trees | Evaluate virtualized `egui_extras` table rendering. Own row identities, sticky headers, filters, sort orders, selection and payload formatting. Render only intersecting rows and a small measured prefetch window. Explorer expands and searches asynchronously. |
| Status, command palette, shortcuts and contextual menus | Build from the same registry of typed commands, labels and availability. Status uses text, icons and semantic color. Include ambiguity resolution, disabled commands with reasons and user-local shortcut overrides. |
| Property editors and dashboards | Build reusable typed fields with units, ranges, source and validation. The engine supplies availability, quality and writable capability. Layout definitions store stable widget IDs and bindings. Keep drafts, apply, progress and actual results explicitly visible. |
| Plot, cursor, raw/decoded Inspector and differences | Use viewport-bounded envelopes from the measurement API. Supply min/max and gaps, exact samples for cursor/export, consistent unit axes and keyboard navigation to one series/event. Use labels, line styles and color in plot legends and diffs. |
| Workspace docks, detached windows and plugin placeholders | Use Craft's dock model behind the host adapter. Support split, tab, drag, move via keyboard and layout reset. Show placeholder with restore/remove actions when a provider is absent. Preserve filters, selection and sessions across docking and viewport recreation. |

DEV-025 delivers an interactive showcase with these families using deterministic synthetic data. Snapshot tests cover light/dark, keyboard focus, empty/active/error states and two common scales. Native smoke covers keyboard-only start/filter/inspect, screen readers, clipboard, IME, docking and real DPI transitions. Screenshot baselines must pin fonts, backend, OS and scale. Keep user secrets and physical bus data out of fixtures.

## Interaction and rendering contracts

- A toolbar, shortcut, context menu and command palette resolve the same command ID against stable view and session context. Power commands ask permission at the same application boundary. When focus is within an editable field, normal editing keys take priority over global navigation.
- The desktop host sends typed, cancellable queries with a viewport range, sort/filter state and a generation. It converts bounded pages to view rows and caches only projected content needed by active views. Chrome uses shared caches for fonts and icons. Expensive decoding, file index and plot aggregation stay in workers.
- Apply a repaint ceiling per live panel and wake only for input, display timer or fresh results. Avoid refreshing hidden tabs from every raw frame. Detached windows have independent dirty state and reuse a shared session subscription where their requested acquisition is compatible.
- Validate layout loads transactionally before restoring. Use docking actions and keyboard alternatives with stable IDs. Test moving across monitors of different scale, switching OS theme, viewport teardown and monitor removal. Offer a recoverable in-window float mode for compositor limitations and show the active mode rather than silently relocating the panel.
- Keep the persisted layout version, identifiers and portable project fields independent of Craft serialization internals. Save settings atomically. Test invalid layout, fresh install, missing provider and migration/reset. The existing Slint spike's in-memory state can serve as an interaction reference without creating a migration dependency for real customer layout files.

See [DEV-024/025 acceptance and benchmark profiles](../development/GUI-MIGRATION.md) for the proof sequence and evidence rules.
