# Accepted Product Decisions — v0.6

**Date:** 2026-11-02 · **Scope:** user-confirmed requirements, not implementation commitments.

| Decision | Confirmed outcome | Canonical requirements |
|---|---|---|
| DBC duplicate CAN IDs | Ordered default interpretation, with individually toggleable/filterable alternative interpretations | REQ-DBC-006/007 |
| DBC editing | Read-only initially; editor later | REQ-DBC-005 |
| Derived signals | Formulas and Python computations | REQ-SIG-013 |
| Plotter | Professional plotting (axes, cursors, markers, sync) | REQ-PLOT-001 |
| Trace preferences | Independent sorting/filtering and reusable presets per Trace panel | REQ-TRACE-027 |
| Recording | Direct write to interoperable standard format | REQ-LOG-001 |
| Scripting | Python primary | REQ-SCR-019 |
| Plugin SDK | Public, versioned SDK for external developers | REQ-PLG-010 |
| Tests | GUI and headless CLI | REQ-SCR-020 |
| Hardware | Portable logical channel mapping to local devices | REQ-HAL-008 |
| Preferences | Personal settings local, shared project settings in one ECU project file | REQ-PROJ-011 |
| Loading | UI first, background resource processing | REQ-UX-011 |

## Conflict and risk register

| ID | Potential conflict | Requirement-level resolution / pending design validation |
|---|---|---|
| CR-001 | Ordered DBC priority may select the wrong definition for different ECUs/channels | Priority is scoped to logical channel/network; show active source explicitly, allow alternative decoders. No raw data mutation. Need definition of per-channel mapping and fallback semantics. |
| CR-002 | Multiple DBC interpretations can multiply CPU cost and create ambiguous computed signals | Decode alternatives lazily/on demand where possible, avoid unconditional decoding of every interpretation, use stable source-qualified signal IDs. Benchmark worst-case collision fan-out. |
| CR-003 | Standard format interoperability may conflict with highest throughput and multi-network metadata | Direct standard write is mandatory; evaluate MDF4 and alternatives against independent readers, CAN FD, streaming, recovery, and replay. The precise format is **not yet decided**. Use disposable indexes if needed, not proprietary mandatory source recordings. |
| CR-004 | Python automation can block hot paths or compromise host security | Execute through bounded asynchronous subscriptions; consider isolated workers; no implicit script auto-run. OS sandboxing remains an explicit design decision. |
| CR-005 | Public native plugins can crash the process despite permissions | State in-process crash-isolation limits; consider out-of-process plugins for untrusted modules; perform ABI compatibility checks. |
| CR-006 | Local UI settings vs shared project settings can produce confusing overrides | Define a deterministic precedence scheme with explicit 'local/shared' origin in UI. Only an explicit share action writes team settings into the single project file. |
| CR-007 | Logical channels mapped per machine may be missing or ambiguous | Never auto-select a transmit-capable device ambiguously; allow explicit mapping and clear missing-device state. |
| CR-008 | Immediate UI plus background parsing can yield incomplete or stale data | Per-resource loading states; cancellation and generation tokens; dependent views show unavailable/loading state instead of stale or false decoded values. |
| CR-009 | Professional plots and formula signals with many histories are computationally expensive | Level-of-detail/downsampling for display, separate raw retained data, bounded calculation workers and measured responsiveness. |
| CR-010 | GUI/CLI automation may diverge in behavior and test outcomes | Same headless test service and script API; compare output schemas and exit statuses in automated parity tests. |
| CR-011 | Shared CAN support now and future Ethernet later can over-generalize the hot path | Shared typed event/session contracts but specialized transport records; verify generic synthetic non-CAN event integration without per-frame unnecessary allocations. |

## Verification gates to define before implementation

1. **DBC collision fixture:** two contradictory DBCs with CAN ID 0x123, controlled order, independent enable/disable and a third DBC bound to another logical channel; prove trace and signal provenance.
2. **Recording interoperability fixture:** sustained CAN FD load, direct write, independent-reader validation, bounded memory, interrupted-write behavior, replay; format choice requires evidence.
3. **Script isolation fixture:** slow Python callback, high-rate virtual CAN, CLI/GUI parity, no capture stall, denied TX without permission.
4. **Workspace portability fixture:** two Git worktrees and two machines map local hardware independently while preserving shared channel names; private presets cause no repository diff.
5. **Fast-open fixture:** large DBC+A2L project with a missing resource, UI responsive, resources marked loading/failure, no stale results after switching projects.
6. **Plot fixture:** millions of samples, synchronized multiple axes, cursors, formula source quality/error propagation, no UI freeze.
7. **Public plugin SDK fixture:** a third-party plugin compiles out of tree, is discovered, verifies compatibility, exposes capabilities, and is rejected cleanly if incompatible.

## Scope guardrails

- No GitHub Issues. Markdown in the CANexus-Studio repository is the source of truth.
- CANexus is independently installed; only its single project file is required in an ECU repository.
- No requirement to open multiple Git worktrees simultaneously.
- Ethernet integration remains a later capability; architecture must accommodate it.
- Priorities, benchmark reference hardware, file format choice, Python runtime packaging and safety model require separate technical decisions.
