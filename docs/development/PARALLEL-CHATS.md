# Parallel chat tasks — CANexus-Studio

**Updated:** 2026-11-09. **Status source of truth:** `docs/development/BACKLOG.md`, `STATUS.md`, `AGENTS.md`; each chat MUST re-check current main, active PRs and branches. The prompts below are independent candidate tasks, not proof they are currently in progress.

## Shared rules for all three chats

- Use `JannisRln/CANexus-Studio`. Before editing, read `AGENTS.md`, `docs/development/README.md`, `STATUS.md`, `BACKLOG.md`, `DECISIONS.md`, related requirements and latest `main`. Check open PRs to prevent duplicate task ownership.
- Create one feature branch and PR per DEV task, never force-push `main`, do not create GitHub Issues, do not silently promote Proposed product requirements, and keep Rust/Doxygen documentation in **English**.
- Keep the repo's pinned Slint, pbSlintDock and Rust versions unless a documented reason and verified migration are in scope.
- Run available checks; require real final-head Windows and Linux CI (project-state, rustfmt, Clippy `-D warnings`, build, all-target tests). State what **was not run**. Do not merge without passing checks and do not mark Done until verified integration.
- **Parallel protection:** avoid editing `docs/development/STATUS.md`, `BACKLOG.md`, `DECISIONS.md` on feature branches while other tasks run. Reconcile these shared files against live main *after* integration in a small follow-up change. Do not overwrite another chat's progress.

## Chat A: DEV-008 — Virtualized Slint Trace MVP

**Branch:** `feat/DEV-008-virtual-trace`

**Ownership:** `crates/canexus-gui/src/main.rs`, `crates/canexus-gui/ui/main.slint`, additional GUI-owned modules, GUI-specific tests and `docs/development/DEV-008-*.md`. Do not edit Core ingestion/history or DEV-017's standalone native-window example.

**Copy/paste prompt:**

> Implementiere DEV-008 für CANexus-Studio. Lies zuerst den aktuellen GitHub-Stand sowie AGENTS.md, Entwicklungsdokumentation und Anforderungen für Trace/UX. DEV-003 ist nur als Single-Window-Docking-Spike abgeschlossen; native Floating-Fenster sind DEV-017 und **keine** Voraussetzung für DEV-008. Arbeite auf `feat/DEV-008-virtual-trace`. Implementiere in den GUI-Dateien einen responsiven Slint-Trace-MVP mit echter virtueller CAN-Datenquelle über DEV-005 → DEV-006 → DEV-007, begrenzter sichtbarer Row-Menge, Page-API, Filter (Kanal/ID/Richtung) und klarer Trennung von Capture und UI-Refresh; keine Synchronarbeit des Capture-Pfads auf dem UI-Thread. Bewahre typisierte CAN/CAN-FD-Daten und Zeitstempel-Provenienz. Nutze pbSlintDock für das bestehende Single-Window-Layout. Berücksichtige die akzeptierten chronologischen/fixed Trace-Modi; nicht implementierbare Details explizit dokumentieren statt behaupten. Keine Änderungen an `crates/canexus-core/src/ingest.rs`, `trace_history.rs` oder an `crates/canexus-gui/examples/native_docking.rs`. Ergänze automatisierte Tests, prüfe Windows/Linux-CI am finalen PR-Head, erstelle PR und melde Commit SHA, echte Tests, Limitierungen und nächsten Schritt. Gemeinsame STATUS/BACKLOG-Dateien erst nach Merge gegen main abgleichen.

## Chat B: DEV-017 — Native floating window proof

**Branch:** `feat/DEV-017-native-docking-proof`

**Ownership:** Isolated `crates/canexus-gui/examples/native_docking.rs` (or an equally isolated `examples/native_docking/` directory), `docs/development/DEV-017-*.md` and self-contained non-production tests. Coordinate before touching `crates/canexus-gui/src/main.rs`, `ui/main.slint`, workspace configuration or shared CI; these are owned by Chat A/main integration.

**Copy/paste prompt:**

> Bearbeite DEV-017 parallel zu DEV-008 für CANexus-Studio. Lies zuerst aktuellen GitHub-Stand, AGENTS.md, DEV-003-spike.md, GUI-README, pbSlintDock-API und REQ-UX-004/009 sowie REQ-PLG-012..014. Erstelle auf `feat/DEV-017-native-docking-proof` einen eigenständigen, kompilierbaren nativen Slint/pbSlintDock-Floating-Window-Prototyp unter `crates/canexus-gui/examples/`, ohne die produktiven `src/main.rs` oder `ui/main.slint` zu verändern. Zeige Window-Erstellung/Schließen, Float → Redock, stabile Panel-Identität und Rust-eigene Panel-/Session-State-Verwaltung. Ergänze deterministische Lifecycle-Tests und ein reproduzierbares Windows/Linux-Smoke-Verfahren; bei möglicher nativer automatisierter GUI-Ausführung liefere die echten Nachweise, sonst markiere native Interaktion als unverified/Blocked. Keine falsche Verifikation durch reine DockManager-Engine-Tests; keine Behauptung eines implementierten externen Plugin-SDKs. PR mit tatsächlich bestandenen Checks, Risiken und konkretem Integrationsvertrag für DEV-008 erstellen. Editiere keine Core-Dateien und keine gemeinsamen Statusdateien auf dem Feature-Branch.

## Chat C: DEV-018 — Shared-batch memory admission

**Branch:** `feat/DEV-018-pinned-batch-budget`

**Ownership:** `crates/canexus-core/src/ingest.rs`, new Core-owned budget modules/tests and `docs/development/DEV-018-*.md`. No GUI/Slint files, no changes to the `TraceHistory` public API without explicit integration agreement.

**Copy/paste prompt:**

> Implementiere DEV-018 in CANexus-Studio parallel zu DEV-008/017. Lies zuerst GitHub-main, AGENTS.md, DEV-006/007-Dokumentation, ADR-011 und die akzeptierten Netzwerk-/Zeit-Requirements. Untersuche das Problem, dass `Arc`-SharedBatch-Referenzen außerhalb der bounded queues gehalten werden und damit der globale RAM-Verbrauch nicht durch Queue- oder Trace-Limits begrenzt ist. Implementiere einen typisierten, testbaren Budget-/Credit-Admission-Prototyp für gemeinsam gehaltene Event-Batches im Core, mit expliziter Verweigerung bei ausgeschöpftem Budget und korrekt freigegebenem Budget nach dem letzten `Arc`; keine Kopie pro Consumer und kein blockierendes Warten des Capture-Threads. Berücksichtige Shutdown, langsame Consumer, Recorder-Fail-Closed und nachvollziehbare Überlast-Statistiken. Der bisherige `BoundedIngest`-/`SharedBatch`-API-Vertrag soll nach Möglichkeit kompatibel bleiben; potenzielle Breaking Changes als separaten Vorschlag dokumentieren. Ändere keine GUI-Dateien oder `trace_history.rs`. Verwende `feat/DEV-018-pinned-batch-budget`, füge Unit- und Overload-Tests hinzu, verifiziere finalen PR-Head per Windows/Linux CI und dokumentiere tatsächliche Grenzen, Commit SHA und Merge-Status. Gemeinsame Status-/Backlog-Dokumente nur nach Merge gegen aktuelles main abgleichen.

## Integration order

- **First:** merge the DEV-003 scope-split PR that registers DEV-008/017/018 and unblocks the tasks.
- **Independent parallel work:** A owns productive GUI files, B owns an isolated native-window example, C owns Core ingestion-budget implementation. They may proceed concurrently once each has checked the latest main/PR ownership.
- **Do not prematurely combine B into A:** after both A and B pass CI, merge the isolated B example; integrate its native window factory into the production GUI in a **separate** PR rebased on the newest GUI main to avoid competing `main.rs` edits.
- **Integration gate:** final-head CI and main CI evidence for each merge, and a fresh STATUS/BACKLOG reconciliation; DEV-008's success does not automatically verify REQ-UX-009 or third-party Slint plugin loading.
