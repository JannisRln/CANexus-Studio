# CANexus Studio — Product UX & Design System

**Version:** 0.1 · **Datum:** 2026-11-08 · **Status:** Designvorschlag, nicht freigegeben · **Plattform:** Windows / Linux · **UI:** Slint + pbSlintDock

> Dies ist eine Designkonzeption. Die normativen Requirements bleiben die Markdown-Dateien in `docs/requirements/` des Projekt-Repositories. Anerkannte Produktentscheidungen werden hier nicht durch neue, stillschweigende Entscheidungen ersetzt.

## 1. Product principles

1. **Engineering first** — Dichte, präzise und anpassbare Arbeitsflächen statt großflächiger Marketing-Kacheln. Informationen sichtbar halten; Randbereiche ein-/ausblendbar.
2. **Capture is independent of the view** — Pause View, Stop Capture und Stop Recording sind drei unterschiedliche Aktionen mit eindeutigem Zustand. View-Filter verändern weder Capture noch Record.
3. **Fast by design** — Oberflächenreaktionen lokal und sofort, große Datenmengen virtualisiert; Suche und Indexierung asynchron mit Abbruch und Status.
4. **Context over modality** — Signal Explorer, Zeitnavigation, Trace, Inspector, Statistik und Plot referenzieren dieselben Objekte, bleiben aber eigenständige dockbare Panels.
5. **Safe by default** — XCP-Änderungen bleiben zunächst lokale Entwürfe, Schreiben erfolgt explizit. UDS-/CANopen-Aktionen führen dieselben Rechteprüfungen aus, egal ob UI, Shortcut oder Script.
6. **Non-destructive to ECU Git** — Persönliche Layouts und Filter lokal, versionierte Dashboard-Dateien separat; keine UI-Caches oder Hardwaremappings im ECU-Checkout.
7. **Protocol specialized** — Ein Signal Explorer, aber spezialisierte Oberflächen für Trace, Measurement, Calibration, UDS, CANopen und Plugins.

## 2. Workspace information architecture

**App shell (immer sichtbar):** Title/Menu bar → Project/worktree context + command search → Workspace tabs → optional status/message area → persistent status bar.

**Left activity rail:** Project, CAN, Measure, XCP, UDS, Automation, Plugins, Settings. Icons mit Tooltips und Text für Screenreader; Textlabels in erweiterter Navigation oder per Tooltip.

**Dockable work region:** Links Explorer (gewöhnlich 256 px), Mitte primärer Arbeitsbereich (min. 560 px), rechts Inspector (gewöhnlich 296 px), unten Events/Problems/Console (gewöhnlich 180 px). Alle Bereiche können über pbSlintDock verschoben/ausgekoppelt werden. Panels behalten ihre eigenen Persistenz-IDs und Query-Zustände.

**MVP-Vorlagen:** `CAN Analyze`, `Measurement`, `Calibration`, `Diagnostics`, `Blank`. Vorlagen sind Startlayouts, keine fixen Produktmodi. Der Nutzer kann mehrere Traces oder Plotter gleichzeitig öffnen.

**Scope semantics:** Oben im Arbeitsbereich immer Project+Worktree, logical channel, session mode `LIVE / REPLAY / OFFLINE` und Zeitqualität. Unterschiede zwischen Ausführungsmodus, Messquelle, Dock-Workspace und Filter-Scope deutlich ausweisen.

## 3. Core visual tokens (proposal)

### Palette

| Token | Dark | Light (proposal) | Verwendung |
|---|---|---|---|
| `color.canvas` | `#0B121A` | `#F2F5F8` | App-Hintergrund |
| `color.surface.1` | `#101C27` | `#FFFFFF` | Panels |
| `color.surface.2` | `#152532` | `#EAF0F5` | Toolbar, Tab, Hover |
| `color.surface.3` | `#1D3342` | `#DAE6EE` | aktive Header, selected row |
| `color.border` | `#2A4051` | `#C7D4DF` | Kanten, Grid separators |
| `color.text` | `#E8F2F7` | `#172A38` | Haupttext |
| `color.text.muted` | `#98AFBE` | `#516879` | Sekundärtext |
| `color.accent` | `#25D3BF` | `#087B76` | Fokus, aktive Navigation |
| `color.info` | `#69BFF5` | `#176CB0` | Info, Links, Readonly |
| `color.success` | `#4CD6A1` | `#17754E` | Online, valid, acknowledged (mit Klartext) |
| `color.warning` | `#FFC36B` | `#9C6200` | Stale, incomplete, risk |
| `color.danger` | `#FF7488` | `#B12743` | Capture/Record Stop, Fehler, Übertragungsrisiko |
| `color.signal.1` | `#38D7CC` | `#008F92` | Plotkurve A |
| `color.signal.2` | `#F4BD53` | `#895C00` | Plotkurve B |
| `color.signal.3` | `#D788F5` | `#8943B2` | Plotkurve C |
| `color.signal.4` | `#76AFF8` | `#2A61BC` | Plotkurve D |

Keine semantische Aussage ausschließlich über Farbe transportieren. Bei Status immer Icon/Text ergänzen. In Plots werden Palette, Strichtyp und Legendeneintrag kombiniert, um Farbsehschwächen zu berücksichtigen.

### Typography, spacing, metrics

| Element | Vorschlag |
|---|---|
| UI font | `Inter`, falls installiert; fallback `Segoe UI` / system-ui; keine Pflicht, Font zu bündeln |
| Mono font | `JetBrains Mono` / `IBM Plex Mono`, falls installiert; fallback system monospace |
| Standard text | `13px`, Gewicht 400–500, Line-height 1.35 |
| Table mono | `12px`/`13px`, tabular numbers, right-aligned numerics |
| Heading | `15–18px`, Gewicht 600 |
| Spacing scale | `4, 8, 12, 16, 24, 32px` |
| Control height | `30px` Compact; `36px` Comfortable |
| Toolbar | `40–44px`; status bar `28px` |
| Table row height | `28px` Compact; `34px` Comfortable |
| Border radius | `4px` small, `6px` standard; keine riesigen Kartenradien |
| Divider | `1px` |
| Focus ring | `2px solid color.accent` + 2px outline-offset |
| Animation | 100–150 ms hover/change; Layout- und Dock-Aktionen höchstens 180 ms; reduce-motion respektieren |
| Scale | 100/125/150/200% HiDPI, ohne harte Pixelannahme im UI-Core |

**Dichte:** Compact ist Desktop-Default für CAN Trace; Comfortable ist ein globales, persistentes Accessibility-Setting. Ein Plugin muss dieselben Token verwenden können.

## 4. Components and interaction contracts

| Component | Varianten/Zustände | Pflichtverhalten |
|---|---|---|
| `AppShell` | project loaded/loading/error; worktree badge | Project-Kontext nie durch unsichtbare Modals verdrängen |
| `ActivityNav` | default, selected, hover, disabled | Shortcut + Tooltip + Accessible Name |
| `DockPanel` | docked, floating, hidden, missing-plugin, loading, error | Titel, Panel-ID, Min-Size, Handle, Close/Open; State lebt getrennt vom Hardware-Core |
| `WorkspaceTab` | active, dirty, running, detached | Reopening erhält Logik-/Sessionstatus |
| `CommandButton` | primary, secondary, danger; idle, busy, disabled | Immer semantische action-ID; kein Permission-Bypass |
| `StatusChip` | Live, Replay, Offline, Warning, Unavailable | Icon + Text, nicht nur Farbpünktchen |
| `VirtualTraceTable` | chronological, fixed; followed, paused; loading; partial | Virtualisiert/seitengesteuert; eigene Filter, Spalten und Sortierung pro Panel |
| `TraceFilter` | view/hardware/recording scope; inactive/active | UI zeigt explizit den Wirkungsbereich und Match Count |
| `ObjectTree` | expandable, selecting, unavailable, multi-source | Stable IDs statt Name allein; Provider/ECU/Datei anzeigbar |
| `FrameInspector` | raw, decoder interpretation(s), diff | Raw niemals durch Decode überschreiben; Provenance sichtbar |
| `NumericDisplay` | valid, stale, invalid, unavailable, loading | Unit, Quality, Source Age sichtbar; keine UI-eigenen Polling-Überraschungen |
| `TimePlot` | live, replay, offline; cursors, marker, unsynced | Ein Plotter-Modell; unbekannte Zeitsynchronisierung markieren |
| `CalibrationField` | ecu-last-read, draft, pending, ack, failed, readback | Ein lokaler Edit ist nie implizit ein ECU-Write; readback ≠ nonvolatile persistence |
| `DiagnosticServiceForm` | form/raw, pending, positive, negative, timeout | ECU, Service, TX/RX, Timeout und Antwortstatus sichtbar |
| `EventLog` | info, warning, error; filter/search | Nicht-blockierend, Quelle und Zeitstempel auswählbar |
| `Empty/ErrorState` | no device, no symbols, missing plugin, permission denied | Ursachen + nächste sichere Aktion; kein irreführendes Null-Daten-UI |
| `CommandPalette` | search, category, conflict, shortcut hint | Eine gemeinsame Registry, auch für Plugin-Commands |

**Behavioral rules:**

- `Pause View` hält nur UI-Follow/Refresh an; `Capture` und `Recording` bleiben unverändert.
- `Stop Capture` und `Stop Recording` haben getrennte Labels und getrennte Maschinenzustände; Recording kann aus anderer Quelle weiterlaufen.
- Die Trace-Filterzeile heißt explizit **View filter**; für Hardware- oder Recording-Filter eigene Konfigurationsdialoge mit Warnhinweis.
- Nach manuellem Scrollen ist `Follow = off`; ein `Jump to Live` springt zur aktuellen Position; Unseen count wird ausgewiesen.
- Umschalten `Chronological ↔ Fixed` verändert nur die Ansicht. Nichtzeitliche Sortierung in Chronological muss als `Sorted view` gekennzeichnet sein.
- Unvollständige DBC/A2L/EDS/Diagnostic-Ressourcen werden pro Datei und Referenz sichtbar gemeldet; keine automatische Nutzung von Ressourcen aus anderem Worktree.
- Eigene Panels sind über Plugin-ID + Panel-ID identifiziert; ohne Plugin erscheint ein reparierbarer Placeholder.

## 5. Key screens / mockups

### A. CAN Analysis (`mock-01-analyzer.png`)

Links: Ressourcen, logical networks, DBC, Signal Explorer. Mitte: CAN Trace mit Filtern und `Chronological / Fixed`, Toolbar (Follow/Pause View), Event-/Frames-Table. Rechts: Frame Inspector, Decoder-Hierarchie und Byte Diff. Unten: Events/Problems; global Statusbar: `CAN1 online`, RX rate, buffer overflow/recorder status. **Statistics** ist ein eigener DockPanel-Tab, nicht ein Modus der Trace-Tabelle.

### B. Measurement & Calibration (`mock-02-measurement.png`)

Links: unified Explorer, XCP symbols / metadata. Mitte: Numeric Displays und gemeinsame Time Plot mit Cursors + Time Provenance. Rechts: Scalar Parameter Editor mit `ECU last read`, `Local draft`, `Write to ECU` und `Readback` separat. Sendeaktionen bleiben im Mockup **simuliert** und erfordern explizite Bestätigung.

### C. Diagnostics (`mock-03-diagnostics.png`)

Links: ECU Tree + Services. Mitte: UDS service form with `Form / Raw` toggle, Request/Response, DTC list. Rechts: Session Context und Kommunikationszustand. Sendeaktionen sind im Mockup nur Simulation; reale Übertragungen erfordern Permission-/Session-Prüfung.

### D. Design System (`mock-04-design-system.png`)

Tokens, semantic statuses, buttons, controls, table density and platform rules as a UI specification overview.

## 6. Accessibility & robustness

- Tastatur-Operabilität, sichtbare Focus-Indikatoren, Command-Palette, korrektes Tab-Verhalten; Enter aktiviert Action, Space Toggle, Esc schließt Popover.
- Farbkontrast: **WCAG AA als Ziel**: 4.5:1 normaler Text, 3:1 große UI-Elemente/indikativer Text; echte Messung vor Abnahme, keine pauschale Konformitätsbehauptung.
- Status nicht nur Farbcode; in Tabellen Symbol + Text, bei Plot auch Linienstil/Legende.
- Akzeptieren von Zoom/OS-Textskalierung und HiDPI; bei 1280×720 Explorer/Inspector automatisch einklappbar; Min-Size/Overflow getestet.
- Internationalisierung: UI-Strings aus zentralen Ressourcen, initial Englisch als Mockup-Standard; Deutsch möglich. Code/Doxygen bleiben Englisch.
- Fehlertoleranz: Projekt-Switch / Cancel darf keine alten Hintergrund-Ergebnisse auf neue Worktree-Views anwenden.

## 7. UX performance budget (targets, not benchmarks)

Die im Repository genannten Hypothesen werden **nicht als bewiesen** behandelt: p95 `<50ms` sichtbares Feedback, p95 `<100ms` bereits geladene Panels/Workspaces, idealisierte `60 FPS` bei definiertem Datenvolumen, Cold Start p95 `<2s` auf noch zu definierendem Referenzgerät. Vor Abnahme Datensätze, Hardware, Warm/Cold-Definitions und Instrumentation festlegen.

UI sollte nur immutable/typed sichtbare Slices, decimierte Plotmodelle und Kommandostatus erhalten. Kein CAN Frame → neues Slint Item pro Rohereignis. Subscription/Recording/Query-Services leben außerhalb der Panelinstanz. Data query generation IDs verhindern stale Ergebnisse.

## 8. Slint / pbSlintDock mapping

```text
CANexusWindow [AppShell]
 ├─ Menu / ProjectBar / CommandBar
 ├─ ActivityNav
 └─ DockRoot [pbSlintDock]
     ├─ SignalExplorerPanel [provider tree + capabilities]
     ├─ TracePanel [virtual table / view query model]
     ├─ InspectorPanel [protocol variants / raw source]
     ├─ NumericPanel [bound measurement consumers]
     ├─ TimePlotPanel [bounded LOD drawable data]
     ├─ ScalarCalibrationPanel [draft + write coordinator]
     ├─ UdsServicePanel [service command dispatcher]
     └─ EventLogPanel [bounded log model]
 └─ GlobalStatusBar
```

**Logical component model (proposal):** `ThemeTokens`, `AppShell`, `DockPanelChrome`, `CommandBar`, `StatusIndicator`, `VirtualDataGrid`, `FilterBar`, `ObjectTree`, `ProtocolInspector`, `PlotCanvas`, `CalDraftEditor`, `LogTable`. Eine Slint-Komponente soll gebundene View-Model-Slices konsumieren, keine eigenen Hardwarehandles verwalten.

## 9. Fluent theme implementation direction (2026-11-08)

The selected design direction is Slint **Fluent** with a runtime theme preference **System / Light / Dark**, implemented via `Palette.color-scheme` and the Fluent-backed global `CxTheme` in [`slint/theme.slint`](slint/theme.slint). In System mode use `ColorScheme.unknown`; Light and Dark apply explicit overrides. Slint `Palette` / `StyleMetrics` are the base for custom components, not an independent full replacement palette.

User preference persistence belongs in application-local settings outside ECU repositories and remains a Rust host integration task. The stand-alone theme preview demonstrates the intended API, not a tested production implementation. Former JSON/HTML prototypes are visual references, not the native Slint source of truth.

## 10. Decisions needed for v0.2

1. Theme: Fluent + System/Light/Dark selected; host persistence, contrast tests and OS change handling pending.
2. Layout: Mindestauflösung und gewünschte Compact/Comfortable-Voreinstellung.
3. Globales Command-System: Palette `Ctrl+K` vs `Ctrl+Shift+P`; Priorität von Tastaturkonflikten.
4. Trace: Raw primary vs mixed hierarchical Event-Row; Storage policy für Disk-Backed History (separates ADR-Thema).
5. Inspector Default: rechts verankert oder unter der Trace-Tabelle.
6. Calibration: Transmit confirmation UX, Readback-Defaults, recovery von lokalen Drafts.
7. Signalzuweisung: Drag & Drop, Context Command, Double Click — explizit noch offen.
8. Window/Multi-Monitor/High-DPI pbSlintDock Proof of Concept auf Windows/Linux.
9. Light theme Kontraste und Farbsinn-Farbpaletten mit automatisiertem Screenshot-/A11y-Test.
10. Plugin-Leitplanken: Host-Vertrauen, Versionierung, Theme-Token-Namensräume, Missing Panel und Permission UI.

## 11. Traceability (ausgewählte Quellen)

**Accepted product requirements:** `REQ-TRACE-001/002/027`, `REQ-SIG-001/002`, `REQ-UX-011`, `REQ-PROJ-011/012`, `REQ-PLG-012`, `REQ-CMD-001`, v0.7 decisions. **Detailed but Proposed:** ADR-009 and ADR-010, majority of performance targets and token choices in this document. This document does not alter requirement status.

- [`docs/requirements/02-ux-performance.md`](https://github.com/JannisRln/CANexus-Studio/blob/main/docs/requirements/02-ux-performance.md)
- [`docs/requirements/06-trace-viewer.md`](https://github.com/JannisRln/CANexus-Studio/blob/main/docs/requirements/06-trace-viewer.md)
- [`docs/requirements/07-signal-workspace.md`](https://github.com/JannisRln/CANexus-Studio/blob/main/docs/requirements/07-signal-workspace.md)
- [`docs/requirements/16-ui-plugins-commands.md`](https://github.com/JannisRln/CANexus-Studio/blob/main/docs/requirements/16-ui-plugins-commands.md)
- [`docs/requirements/20-adr-009-high-performance-can-trace.md`](https://github.com/JannisRln/CANexus-Studio/blob/main/docs/requirements/20-adr-009-high-performance-can-trace.md)
- [`docs/requirements/21-adr-010-measurement-calibration-workspace-ux.md`](https://github.com/JannisRln/CANexus-Studio/blob/main/docs/requirements/21-adr-010-measurement-calibration-workspace-ux.md)