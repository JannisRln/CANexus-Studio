# CANexus-Studio implementation roles

**Status** Proposed · **Date** 2026-11-24

These are development and review responsibilities. Product requirements, task status, dependencies and completion evidence live in the [requirements](../requirements/README.md), [backlog](BACKLOG.md) and [status](STATUS.md). [Parallel chat tasks](PARALLEL-CHATS.md) defines the immediate file boundaries and prompts for DEV-008, DEV-022 and DEV-023.

| Role | Owns | First handoff and required evidence |
|---|---|---|
| Product and integration lead | Scope, requirement IDs, ADRs, shared contracts and integration order. Maintains backlog/status and keeps product acceptance distinct from task completion. | Publish the Trace query/view contract and milestone scope, assign one owner per task, then reconcile integration branches and record requirement-to-evidence links. Escalate actual product direction decisions to the maintainer. |
| Desktop UX and Slint engineer | DEV-008 production shell, virtualized Trace and Inspector, view-state ownership, theme tokens, keyboard and accessibility. Owns production `crates/canexus-gui/src/` and `ui/`. | Demonstrate bundled virtual CAN through real ingest/history, mode/filter/sort behavior and bounded row count. Deliver display-backed keyboard and scaling evidence and the DEV-009 instrumentation hooks. |
| Docking and accessibility engineer | DEV-022 isolated native example, float/redock lifecycle, focus and monitor/DPI behavior. Owns standalone example and its tests. | Report native Windows/Linux results and the state contract for production integration. Pass platform test notes and any unsupported backend behavior to the desktop engineer. |
| Core and memory engineer | DEV-023 batch admission/lifetime accounting, capture fairness, queue overflow and overload telemetry. Owns Core ingest and budget modules and tests. | Demonstrate byte reservations remaining charged across clone and FIFO eviction until the last owner drops. Exercise cancellation, shutdown and budget exhaustion. Document telemetry and coordinate any query/history contract changes with the desktop owner. |
| Performance and verification engineer | DEV-009 fixtures, benchmark harness, headless/GUI parity checks, failure injection and reference Windows/Linux evidence. | Publish raw run data, configuration and fixture hashes for the [acceptance profiles](../requirements/25-analysis-release-baseline.md#proposed-performance-and-memory-acceptance-gates). Verify counts and budget assertions independently. Add matching hardware and accessibility evidence as features land. |
| Hardware and transport engineer | DEV-010, adapter discovery and capabilities, receive state and timing. Owns HAL/adapter integration and platform-specific fixtures. | Verify SocketCAN or another chosen Linux backend and a named Windows backend against the supported-device matrix. Provide bus-off/disconnect tests, timestamp provenance and known limitations. |
| Signal and protocol engineer | DEV-011 DBC, Signal Explorer providers, derived events and later ISO-TP/UDS, XCP/A2L and CANopen contracts. | Deliver deterministic fixtures for signedness, byte order, multiplexing and colliding interpretations before linking decoded views. Scope each later protocol profile and publish fixture provenance. |
| Recording and replay engineer | DEV-013, direct standard recorder, independent retention/index/query, offline replay and interoperability. | Provide a format decision and compatibility matrix with independently read fixtures, full-disk/interruption tests, and verification that view filters preserve recorded contents. |
| Automation and SDK engineer | Project-scoped commands, isolated Python runner, extension compatibility tests, and application service API evolution. Security-sensitive authorization contracts are reviewed with the integration lead. | Establish deterministic GUI/CLI test parity and a worker-crash fixture. Freeze public versioned contracts after implementation diversity and lifecycle evidence. DEV-017/018 retain existing prerequisites. |

## First implementation handoffs

1. **DEV-008** uses the verified DEV-003 single-window host and DEV-007 history. The desktop owner consumes bounded pages and owns panel state, while the Core owner owns event identity, order, budget and loss semantics.
2. **DEV-022** uses an isolated native example. Only its own example and tests change while DEV-008 owns the production GUI. The result transfers platform evidence and any tested panel factory contract to the desktop owner.
3. **DEV-023** stays in Core ingestion and budget tests. It reports custody of shared batch bytes, overload policy and observable failures. Negotiate changes to `TraceHistory` or a shared interface before either owner modifies it.
4. **DEV-009** follows DEV-008 for the end-to-end benchmark. Its initial UI measurements can use the current history behavior. Final global memory verification cites DEV-023 evidence or records the requirement as partial. Hardware and protocol increments follow the backlog dependencies.

## Integration evidence

- Use one task ID, owner, branch and scoped file list. Recheck live `main`, open PRs and active branches before claiming a task. Record progress using the repository workflow.
- Agree with the integration lead on shared contracts before changing signatures, data layouts or persisted formats. The owner of production code coordinates integration of changes from examples and spikes.
- For each change, link the stable requirement ID, DEV task, commit, test or benchmark command, environment, result and remaining coverage. A PR should include examples of observable failure behavior where relevant. Verification of a user-facing native interaction includes a display-backed run on Windows and Linux.
- Keep design thresholds in the requirements and measurements in benchmark output or verification reports. Mark a task Done only after its documented gate and verified integration. Keep canonical project status updates coordinated after feature branches integrate.
