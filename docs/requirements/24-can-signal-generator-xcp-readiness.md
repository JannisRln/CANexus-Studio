# CAN Signal Generator and XCP-Ready Architecture

**Revision:** 0.1 · **Status:** Proposed detailed requirements; product scope requested 2026-11-08

## Scope and release boundary

CANexus-Studio shall include a CAN Signal Generator in its product roadmap. **The generator is not part of the first release (v1).** The first release shall nevertheless retain compatible typed signal, transmission, scheduling and plugin/service interfaces so the generator can be implemented later without replacing the capture core or rewriting the GUI.

The generator must be **XCP-ready by design**, but **XCP-driven signal generation is not part of v1**. Do not interpret XCP readiness as a requirement to implement XCP sessions, DAQ, calibration or XCP stimulus in the first release. Existing XCP product requirements remain in scope for subsequent milestones and are not deleted.

## Functional direction for a later release

- Generate CAN/DBC signal values with constant, step, ramp, sine, triangle, square and user-defined sampled patterns; exact waveform subset and parameter semantics require approval.
- Use DBC scaling, offset, signedness, endianness, value bounds, multiplexing and encoding consistently with the central signal model.
- Map generated signal values to frames and transmit through the existing controlled TX engine; no independent competing scheduler.
- Support start, pause, resume, stop, duration, repetition, phase and deterministic simulation-time execution where feasible.
- Provide a preview and dry-run via Virtual CAN without requiring physical hardware.
- Make generated traffic distinguishable in Trace, recordings and test evidence; retain source, generator ID and requested-vs-confirmed TX provenance.
- Support several generators subject to explicit scheduling and ownership limits, and define conflicts when multiple generators target the same frame/signal.
- Future XCP providers may reuse waveform definitions and a typed output-target interface to write eligible ECU parameters or provide supported stimulation; actual behavior must respect ECU capabilities, session bindings, write permissions and timing limits. No assumption that ordinary XCP DAQ supports arbitrary signal stimulation.

## Requirements

| ID | Priority | Status | Requirement |
|---|---|---|---|
| REQ-GEN-001 | P2 | Proposed | Provide a configurable CAN signal generator in a post-v1 release. |
| REQ-GEN-002 | P2 | Proposed | Generate constant, step, ramp, periodic and sampled waveforms with explicit time base, value range and units. |
| REQ-GEN-003 | P2 | Proposed | Encode generated DBC signal values into CAN/CAN FD frames using the existing validated signal encoder. |
| REQ-GEN-004 | P2 | Proposed | Use the shared TX scheduler and authorization, ownership, cancellation and shutdown semantics. |
| REQ-GEN-005 | P2 | Proposed | Provide Virtual CAN preview and deterministic test execution without transmitting to physical hardware. |
| REQ-GEN-006 | P2 | Proposed | Expose generator status, requested values, actual TX results, overruns and errors through existing event/health services. |
| REQ-GEN-007 | P2 | Proposed | Define conflict arbitration for overlapping generators, manual TX and cyclic jobs on the same target. |
| REQ-GEN-008 | P1 | Proposed | Keep the first-release signal model, command and TX engine extensible for a later generator without implementing generator UI or waveform execution in v1. |
| REQ-GEN-009 | P1 | Proposed | Define a typed future output-target abstraction that can accommodate CAN-frame targets and eligible XCP parameter targets without coupling waveform logic to the transport. |
| REQ-GEN-010 | P2 | Proposed | Integrate XCP-backed generation only in a later milestone, with session-scoped binding validation, ECU capability checks and bounded scheduling. |

**Priority note:** P1 for architectural readiness is not a commitment to deliver a generator in v1. The first-release implementation should avoid premature generalized runtime machinery; establish compatible boundaries and validate with a small test double.

## Acceptance criteria

1. A v1 architecture test demonstrates that a synthetic future signal source can provide typed values to a mock target without changes to capture or Slint panel ownership.
2. In a later release, a known waveform encoded through a DBC produces expected CAN payload bytes and timestamps within documented scheduling tolerances.
3. Starting/stopping a generator through the shared TX engine obeys explicit hardware-transmit authorization and shutdown rules.
4. Virtual CAN dry-run produces no physical TX; generated output and errors remain traceable.
5. Simultaneous generator/manual/cyclic requests have a deterministic documented arbitration outcome.
6. Replacing or disconnecting an XCP session invalidates any future XCP target binding; no stale write is issued.

## Open decisions

- Exact waveform types, editor/plot UX and units/time semantics.
- Per-signal vs whole-frame generation, multiplexing and counter/checksum handling.
- Conflict policy and scheduling jitter limits on Windows/Linux.
- Generator definition persistence in optional project files vs user-local state.
- Whether the future XCP use case means periodic parameter writes, calibration test sequences or ECU-specific stimulation (these are not interchangeable).

## Related specifications

- [Transmission, Triggers and Replay](10-transmission-trigger-replay.md)
- [XCP and CANopen](14-xcp-canopen.md)
- [High-Performance Core Architecture](22-adr-011-high-performance-core-architecture.md)
- [Central Measurement Engine](23-adr-008-central-measurement-engine.md)
