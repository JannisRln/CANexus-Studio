# Scripting and Automation

**Revision:** 0.6 · **Status:** Proposed architecture, confirmed need for a scripting system

## Purpose

A scripting system complements the native **plugin system**. Native plugins provide high-performance adapters, protocol stacks, codecs and optional UI features. Scripts provide project-level analysis, test sequencing, automation and quick experimentation **without compiling a plugin**. The two extension paths share versioned service contracts but not necessarily the same runtime or trust model.

**Confirmed:** CANexus-Studio needs scripting and plugins; module-specific APIs must be possible.

**Accepted:** Python is the primary scripting language. **Proposed implementation:** out-of-process workers, subject to isolation, startup, IPC and packaging benchmarks. A lighter embedded DSL may be added for deterministic filtering/expressions.

## Requirements

| ID | Priority | Status | Requirement |
|---|---|---|---|
| REQ-SCR-001 | P1 | Proposed | Scripts can be loaded, started, stopped, inspected and rerun from a dedicated Script Console/Manager. |
| REQ-SCR-002 | P1 | Proposed | Script execution cannot block the UI event loop or core receive/recording pipeline. |
| REQ-SCR-003 | P1 | Proposed | Scripts subscribe to typed events: CAN frame RX/TX, signal update, timer, channel state, recording state and module-specific events. |
| REQ-SCR-004 | P1 | Proposed | Core services expose versioned APIs for channel discovery, receive subscriptions, safe send, logging, time and configuration. |
| REQ-SCR-005 | P1 | Proposed | Module plugins may register additional script namespaces, types, events and commands (UDS, XCP, CANopen, future Ethernet). |
| REQ-SCR-006 | P1 | Proposed | Scripts can send one-shot/cyclic frames, manage sequences and evaluate triggers through a shared transmission service when authorized. |
| REQ-SCR-007 | P1 | Proposed | Script Manager shows running state, current tasks, outputs, error traces, cancellation and resource usage. |
| REQ-SCR-008 | P1 | Proposed | Subscriptions use bounded queues with explicit overflow/drop reporting; slow scripts do not stall capture. |
| REQ-SCR-009 | P1 | Proposed | Script timers, async operations and cancellation have documented lifecycle behavior on measurement stop, project close and device disconnect. |
| REQ-SCR-010 | P1 | Proposed | An explicit permission model covers active transmission, calibration writes, ECU diagnostic writes, filesystem and network access. |
| REQ-SCR-011 | P1 | Proposed | Script execution and auto-run are opt-in; project files never silently execute untrusted scripts on open. |
| REQ-SCR-012 | P1 | Proposed | Script paths in the ECU project configuration are relative to the single CANexus project file; no mandatory CANexus script directory. |
| REQ-SCR-013 | P1 | Proposed | Scripts use portable logical channel names, not machine-specific hardware paths. |
| REQ-SCR-014 | P2 | Proposed | Scripts can run against virtual adapters/replay for deterministic regression tests without hardware. |
| REQ-SCR-015 | P2 | Proposed | Test scripts can emit assertions, structured results and CI-compatible reports with source trace references. |
| REQ-SCR-016 | P2 | Proposed | Optional in-app editor offers syntax diagnostics, auto-completion, navigation and a documentation explorer. |
| REQ-SCR-017 | P2 | Proposed | User scripts can subscribe to decoded signal updates and create user-defined signal calculations or triggers. |
| REQ-SCR-018 | P2 | Proposed | A script API can register scoped commands/context menu actions; full custom UI widgets remain governed by the plugin SDK. |

## Two extension layers

| | Native Plugin SDK | Script Runtime |
|---|---|---|
| Primary intent | Hardware, protocols, codecs, extensible panels | Automation, tests, event handling, quick transformations |
| Packaging | Versioned plugin manifest/library or isolated service | Referenced script/source and runtime environment |
| Execution | Native in-process for trusted performance-critical modules, optional isolated process | Initially favor isolated worker for Python |
| Realtime RX | Core-defined bounded subscription / ABI | Event queue / async callback, **never** direct receive hot-path execution |
| Module API | Defines capabilities and services | Consumes advertised services and module namespaces |
| Safety | Declared capabilities/permissions; in-process crash isolation not guaranteed | Untrusted Python requires process/OS isolation; ordinary Python embedding is **not** a secure sandbox |

## Illustrative API (not a frozen interface)

```python
# Illustrative pseudocode, not an approved API.
async def on_frame(frame):
    if frame.channel == "powertrain" and frame.can_id == 0x100:
        log.info("Received powertrain frame")

subscribe_can(on_frame, channel="powertrain", can_id=0x100)
await canexus.uds.read_did("ecu_1", 0xF190)  # namespace and syntax TBD
```

## Acceptance tests

1. Receive a virtual CAN stream while running a deliberately slow event handler. Recording remains uninterrupted; any script backlog is observable.
2. Start a script that sends periodically, stop it, and prove all of its scheduled TX jobs are cancelled.
3. Try to transmit without permission and verify a clear rejection; reading trace data remains available.
4. Load a project from a different checkout and resolve its script path locally.
5. With a UDS plugin installed, its API is visible; without it, script execution produces an explicit missing-capability diagnostic.
6. Opening an unfamiliar ECU project does not automatically execute its scripts.

## Open decisions

Python version/runtime packaging, host API versioning, tracing/debugger, scheduling determinism, permission persistence and OS-level isolation.

## v0.6 accepted scripting decisions

| ID | Status | Requirement |
|---|---|---|
| REQ-SCR-019 | Accepted | Python is the primary scripting language. |
| REQ-SCR-020 | Accepted | Test automation supports both GUI execution and a headless CLI using the same core capabilities. |
| REQ-SCR-021 | Proposed | CLI returns deterministic exit codes, machine-readable results and uses virtual hardware without opening Slint. |

Acceptance: equivalent virtual-CAN script tests run in GUI and CLI with comparable outputs and matching pass/fail; slow Python callbacks cannot delay recording; opening an ECU project never executes untrusted scripts automatically. Python runtime version, dependencies, subprocess isolation and distribution remain design decisions.
