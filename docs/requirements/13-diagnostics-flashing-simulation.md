# Diagnostics, Flashing and ECU Simulation

**Revision:** 0.7 · **Status:** Accepted long-term product scope; detailed design proposed

## Accepted requirements

| ID | Proposed priority | Status | Requirement |
|---|---|---|---|
| REQ-DIAG-007 | P1 | Accepted | Provide a UDS service browser with structured input forms and a raw request/response mode. |
| REQ-DIAG-008 | P2 | Accepted | Long-term support for ODX, PDX, CDD and additional manufacturer-specific diagnostic description formats, subject to legal and technical feasibility. |
| REQ-FLASH-001 | P2 | Accepted | Provide an extensible flashing framework supporting UDS and additional protocol/vendor flashing strategies through modules. |
| REQ-SIM-001 | P2 | Accepted | Support a fully stateful virtual UDS ECU with configurable sessions, DIDs, DTCs, security state and script-driven behavior. |

## Proposed detailed requirements

| ID | Status | Requirement |
|---|---|---|
| REQ-DIAG-009 | Proposed | Form and raw views operate on one request/session model and display source ECU, addressing, timings, NRC and transport frames. |
| REQ-DIAG-010 | Proposed | Format importers normalize diagnostic descriptions into a typed internal diagnostic model and report unsupported features explicitly. |
| REQ-DIAG-011 | Proposed | Manufacturer-proprietary format parsers are optional plugins; no compatibility promise until validated against licensed reference files. |
| REQ-FLASH-002 | Proposed | Flash providers expose preconditions, image validation, transfer operations, timing, progress, cancellation boundaries, verification and post-flash validation. |
| REQ-FLASH-003 | Proposed | Active flashing requires explicit approval of target ECU and firmware image, supported preflight checks and exclusive ownership of critical communication resources. |
| REQ-FLASH-004 | Proposed | Interrupted flashing reports last known stage and recovery guidance; never imply that reset/cancellation is always safe. |
| REQ-FLASH-005 | Proposed | Security material and unlock algorithms remain in authorized providers and local secret stores, not Git-versioned project files. |
| REQ-SIM-002 | Proposed | Virtual ECU behavior has deterministic state transitions, service responses, timers, fault injection and traceability. |
| REQ-SIM-003 | Proposed | Simulator can reset, snapshot configuration and run against Virtual CAN/ISO-TP for headless regression tests. |

## Acceptance criteria

- [ ] A supported UDS DID is read through a form and raw mode; request/response payload and negative response semantics agree.
- [ ] Load a representative ODX/PDX description fixture and a permitted CDD/vendor fixture; unsupported constructs generate a structured diagnostic without silently corrupting interpretation.
- [ ] A simulated UDS ECU changes session, updates DID values, manages DTC state, enforces configured security states and exhibits configurable timeouts.
- [ ] A headless test drives the stateful simulator and verifies a repeatable ordered response trace.
- [ ] A simulated flashing adapter runs successful, rejected-image and interrupted-transfer tests; stages and safety state are reported.
- [ ] Flash attempts without explicit authorization, proper resource ownership or valid prerequisites are rejected before transmission.
- [ ] Unrecoverable/interrupted real-hardware scenarios have documented operator guidance; **no universal recovery guarantee** is claimed.

## Architecture conflicts / open decisions

- A universal framework is not a universal flashing algorithm: transfer rules, ECU prerequisites and recovery remain provider-specific.
- ODX/PDX/CDD support varies with profile, version, licensing and proprietary vendor semantics; exact coverage and import compliance tests need definition.
- Flashing and stateful simulator share UDS service primitives but should not couple safety-critical live TX operations to GUI state.
- Decide whether simulation state is shareable in the single project file or user-local, and define how secrets and active actions are excluded.
- Prioritize simulation and flashing **after** a stable transport/UDS core; neither is implied to be part of the initial CAN analyzer MVP.
