# Python Workers, Test Functions and External Editors

**Revision:** 0.7 · **Status:** Accepted product decisions and proposed execution details

## Accepted requirements

| ID | Proposed priority | Status | Requirement |
|---|---|---|---|
| REQ-SCR-022 | P1 | Accepted | Execute Python scripts in isolated worker processes per script session, rather than one shared in-process interpreter. |
| REQ-SCR-023 | P1 | Accepted | Use external editors such as VS Code for Python editing; a full in-app Python editor is not required. |
| REQ-TEST-001 | P1 | Accepted | Organize automated tests as Python files containing test functions, callable from the GUI or headless CLI. |

## Proposed detailed requirements

| ID | Status | Requirement |
|---|---|---|
| REQ-SCR-024 | Proposed | Each worker has its own process lifecycle, bounded event/command queues, monitored resource use, cancellation and crash reporting. |
| REQ-SCR-025 | Proposed | Permission boundaries cover transmit, diagnostics, calibration, flashing, filesystem and network capabilities; worker separation alone does not provide an OS security sandbox. |
| REQ-SCR-026 | Proposed | An Open-in-External-Editor command opens existing script files by their resolved path, without generating a mandatory project-side editor configuration. |
| REQ-TEST-002 | Proposed | Discover Python test functions by a documented naming/metadata convention; avoid a mandatory CANexus-specific test directory. |
| REQ-TEST-003 | Proposed | Tests receive declared fixtures for virtual transport, protocols, sessions and assertions and yield structured pass/fail/skip/error results. |
| REQ-TEST-004 | Proposed | GUI and CLI use the same discovery/execution engine and return consistent test identities, logs and outcomes. |
| REQ-TEST-005 | Proposed | Timers and resource cleanup are deterministic for virtual-device tests; operations on real hardware use explicit capability guards. |

## Acceptance criteria

- [ ] Run two separate Python script sessions; crash one interpreter and confirm the other worker and CAN recorder remain functional.
- [ ] A deliberately slow handler causes bounded backpressure visibility without stopping frame acquisition.
- [ ] Start Python tests from GUI and CLI; identical fixtures produce matching test case IDs and pass/fail results.
- [ ] Open an ECU-local Python script in the system's configured external editor and resolve its path from the CANexus project file correctly.
- [ ] Stop a worker with outstanding cyclic send jobs and confirm the core revokes its jobs.
- [ ] Deny a script unauthorised access to a protected TX/flash capability; result is explicitly reported.
- [ ] Opening a cloned ECU repository does not automatically execute Python or install packages.

## Conflicts and open decisions

- Worker-per-session increases memory and startup cost. Define pooled interpreter startup, lazy worker launch, crash isolation and performance budgets without converting to shared interpreter state.
- Python version and dependency environment: bundled distribution vs user-managed environment; lockfile policy must respect one required CANexus configuration file per ECU repository.
- External editor integration: file associations, VS Code URI/CLI handling and remote containers are platform-specific and optional.
- Existing requirement REQ-SCR-016 (optional in-app editor) is **superseded by REQ-SCR-023** and should not be implemented unless explicitly reconsidered.
- Prior idea of GUI test-sequence editor is **not accepted**. Core test data model may support future extensions, but Python functions are authoritative.
