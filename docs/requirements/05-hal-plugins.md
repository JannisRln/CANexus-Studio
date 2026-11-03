# Hardware Abstraction & Plugin System

**Status:** Proposed · **Revision:** 0.7

**Architecture direction:** Separate network-agnostic lifecycle, event routing, timing and UI/query services from typed CAN-specific adapters and future Ethernet adapters. See [Network Extensibility](09-network-extensibility.md). The extension ecosystem contains two complementary layers: native plugins and a scripting runtime, specified in [Scripting & Automation](08-scripting-automation.md).

## HAL requirements

| ID | Priority | Status | Requirement |
|---|---|---|---|
| REQ-HAL-001 | P0 | Proposed | Expose a unified interface for CAN channel discovery, capabilities, lifecycle, RX/TX and errors. |
| REQ-HAL-002 | P0 | Proposed | Provide a virtual CAN adapter for repeatable tests and hardware-free use. |
| REQ-HAL-003 | P1 | Proposed | Provide Linux SocketCAN and an early Windows-compatible CAN hardware adapter (proposed: PCAN). |
| REQ-HAL-004 | P1 | Proposed | Handle device disconnects, bus-off conditions and timeouts without blocking the UI. |
| REQ-HAL-005 | P1 | Proposed | Expose channel configuration and timestamp quality where supported by hardware. |
| REQ-HAL-006 | P1 | Proposed | Manage transmit permissions, exclusive hardware configuration and safe session shutdown. |
| REQ-HAL-007 | P2 | Proposed | Extend to additional manufacturers without changing the core frame model. |

## Core data-path requirements

| ID | Priority | Status | Requirement |
|---|---|---|---|
| REQ-CORE-001 | P0 | Proposed | Core is UI independent and runs headlessly. |
| REQ-CORE-002 | P0 | Proposed | Capture, recording, decoding, protocol sessions and UI are decoupled subscribers. |
| REQ-CORE-003 | P0 | Proposed | Queues are bounded with observable overflow/backpressure behavior. |
| REQ-CORE-004 | P0 | Proposed | Frames retain source channel, direction and timestamp provenance. |
| REQ-CORE-005 | P1 | Proposed | Benchmarks cover throughput, p95/p99 latency, drops and memory consumption. |
| REQ-CORE-006 | P1 | Proposed | Failure of a UI consumer does not stall hardware receive processing. |

## Plugin requirements

| ID | Priority | Status | Requirement |
|---|---|---|---|
| REQ-PLG-001 | P0 | Proposed | Versioned SDK contracts for hardware adapters, protocols and formats. |
| REQ-PLG-002 | P0 | Proposed | Plugin manifest contains unique ID, version, compatibility and capabilities. |
| REQ-PLG-003 | P0 | Proposed | Externally loaded native plugin ABI is explicit/versioned (no assumption of stable Rust ABI). |
| REQ-PLG-004 | P1 | Proposed | Plugin permissions cover CAN reception/transmission and permitted resource access. |
| REQ-PLG-005 | P1 | Proposed | Missing/incompatible plugins are reported without crashing the application. |
| REQ-PLG-006 | P1 | Proposed | Projects declare plugin requirements in the single project file; no additional file is mandatory in ECU repositories. |
| REQ-PLG-007 | P1 | Proposed | Per-plugin logging, health status and diagnostics are available. |
| REQ-PLG-008 | P2 | Proposed | Support optional isolated plugin execution when stronger fault containment is required. |
| REQ-PLG-009 | P2 | Proposed | Expose safe UI extension and automation APIs, with explicit compatibility contracts. |

## Open decisions

Native dynamic ABI choice; plugin discovery locations; signed/verified plugin policy; external plugins' write permissions; safety of in-process extensions; module-specific scripting namespaces; network capabilities; hardware/channel arbitration and reconnect strategy.

## v0.6 accepted SDK and channel decisions

| ID | Status | Requirement |
|---|---|---|
| REQ-PLG-010 | Accepted | Provide a publicly documented, versioned plugin SDK for independent external developers. |
| REQ-HAL-008 | Accepted | Portable ECU project logical channels are mapped to concrete hardware in user/machine-local settings. |
| REQ-PLG-011 | Proposed | Publish example plugins, compatibility diagnostics, SDK migration guidance and a test harness. |

Acceptance: an external developer builds a protocol or device plugin without modifying CANexus source; incompatible plugin fails with clear diagnostics; the same project maps logical CAN1 to different physical adapters on Windows/Linux; unresolved mapping cannot silently transmit.

## v0.7 UI extension model

**Accepted REQ-PLG-012:** Public plugin SDK must allow external plugins to contribute their own dockable Slint panels. The panel lifecycle and ABI, trust model, state persistence, and limitations of dynamic Slint components require a Windows/Linux proof of concept before SDK freeze. See [Dockable UI Plugins and Commands](16-ui-plugins-commands.md).

**Accepted REQ-CMD-001:** A fully customizable command system shall expose built-in and plugin commands, shortcuts, macros, context menus, and toolbar actions through one permission-aware dispatcher. Command and macro definitions must not bypass TX/flash authorization.
