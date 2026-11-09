# AI-Native Integration, MCP and Agent-Driven Engineering Workflows

**Revision:** 0.8 proposal · **Status:** Proposed (no product decision or implementation accepted by this document)
**Date:** 2026-11-09

## 1. Goal and scope

CANexus-Studio should be an AI-native engineering application. An external AI agent should be able to **use CANexus as a tool** to inspect project context, debug CAN/CAN-FD and protocol interactions, operate approved measurement/test workflows, and create or edit engineering artifacts such as projects, dashboards and workspace layouts. An optional built-in AI assistant should use the **same** capabilities.

This proposal covers **both**:
- **Inbound MCP server:** expose discoverable CANexus tools, read-only resources and optional workflow prompts to external AI clients; an opt-in local stdio transport is the first target.
- **Native AI assistant:** an optional in-app conversational/agent experience calling the same versioned application services, without UI automation or privileged shortcuts; model-provider selection is a later decision.

AI is not required for normal local operation. AI suggestions are not verified engineering facts, and model output never grants itself permission to perform risky actions. Features absent from CANexus (for example, production UDS/XCP or dashboard editor capabilities) must appear as unavailable, not as functional MCP endpoints.

**Existing constraints remain authoritative:** headless Rust core and bounded queues; Slint/pbSlintDock UI; typed CAN fast path extensible to Ethernet; a single CANexus configuration file inside an ECU checkout; user-local layouts by default; explicit authorized TX/diagnostic/calibration/flash operations; Python worker isolation; reusable command dispatcher; no implicit script auto-run.

## 2. Proposed architecture

    External AI clients -- MCP protocol / opt-in server --+
                                                         |
    Native CANexus AI assistant -- provider-neutral -----+--> Versioned application service facade
                                                               |
                                  +----------------------------+--------------------------+
                                  |                            |                          |
                          Project / workspace           Queries and analysis       Controlled actions
                          CRUD and validation           Trace/measurement          capture/TX/UDS/XCP/tests
                                  |                            |                          |
                          User-local or shared           Typed core services      Permission-aware command
                          persisted configuration       / bounded snapshots      dispatcher and scheduler

The service facade is the only authoritative mutation and authorization boundary. GUI, CLI, Python scripts, native plugins and MCP call the same underlying services when they perform equivalent operations. A dashboard edit changes the workspace model and subsequently updates a Slint view; the agent does **not** inject Slint code or directly manipulate the UI thread. MCP execution, model inference and external transport must remain off the capture/recorder hot path.

An MCP transport connection must not be treated as the identity of an ECU/project session. Explicit server-issued, expiring, generation-checked project/session/operation handles are required. The selected implementation must document supported MCP specification versions and negotiate or reject incompatible clients. Protocol version and Rust SDK selection are **not yet accepted**.

### Exposure model

| MCP surface | Intended purpose | Examples (illustrative names, not a frozen API) |
|---|---|---|
| Tools | Typed actions and bounded queries | project.create, project.validate, trace.query, dashboard.create, dashboard.bind_signal, tests.run_virtual |
| Resources | Read-only snapshots and schema references | project summary, channel capabilities, signal catalog, diagnostics, trace metadata |
| Prompts | Optional reusable diagnostic workflows, never an authority source | investigate_missing_can_message, create_measurement_dashboard |

Read-only queries are distinct from commands that **send a request to a physical ECU**: a UDS ReadDataByIdentifier or XCP read may be observational in intent but still produces physical bus traffic and requires the corresponding active-operation permission.

## 3. Proposed functional requirements

All new IDs are **Proposed**. Priorities and release placement must be reviewed independently. A requirement is not Implemented merely because a similarly named service exists.

| ID | Priority | Status | Requirement |
|---|---|---|---|
| REQ-AI-001 | P1 | Proposed | Provide an optional AI-native workflow able to analyze engineering state and request actions using versioned application services, not UI scripting. |
| REQ-AI-002 | P0 | Proposed | Define typed, versioned application service contracts reusable by GUI, CLI, scripts, plugins and AI/MCP; use one authorization path for equivalent mutations. |
| REQ-AI-003 | P1 | Proposed | Expose an opt-in MCP server for external clients with a local stdio transport and a headless execution mode. |
| REQ-AI-004 | P2 | Proposed | Optionally expose a Streamable HTTP MCP endpoint; disable it by default, bind locally by default, and require authenticated authorization for any network exposure. |
| REQ-AI-005 | P1 | Proposed | Publish discoverable tool schemas, read-only resources, capability/side-effect metadata, structured results and consistent typed errors. |
| REQ-AI-006 | P1 | Proposed | Advertise only capabilities currently installed, enabled and available for the selected project/session; missing plugins or hardware produce explicit diagnostics. |
| REQ-AI-007 | P2 | Proposed | Allow an optional embedded AI assistant with pluggable local or remote model providers, using the same services and permission policies as MCP clients. |
| REQ-AI-008 | P1 | Proposed | Offer versioned, bounded project/session introspection: project identity, active ECU/worktree, logical channels, adapter capabilities, plugin versions and connectivity. |
| REQ-AI-009 | P1 | Proposed | Allow agents to search/filter/page CAN/CAN-FD Trace events by source, channel, time range, identifier, direction and captured sequence, preserving gap/eviction information. |
| REQ-AI-010 | P1 | Proposed | Expose available DBC definitions, decoded signals, units, source/frame linkage, timestamp provenance and decoding errors without fabricating missing values. |
| REQ-AI-011 | P1 | Proposed | Expose structured logs, hardware/session errors, bus-off state, capture loss, recorder failures, protocol timeouts and health metrics for debugging. |
| REQ-AI-012 | P1 | Proposed | Expose available protocol diagnostics (ISO-TP/UDS, XCP, CANopen) with typed request/response, negative responses, timing and unsupported-feature reporting. |
| REQ-AI-013 | P1 | Proposed | Let agents inspect recording/replay/virtual-CAN data and run bounded, deterministic diagnostics or regression queries without physical hardware. |
| REQ-AI-014 | P1 | Proposed | Return evidence references for debug findings: raw-event/source sequence or bounded query cursor, clock domain/quality, decoding resource version and any data gaps. |
| REQ-AI-015 | P1 | Proposed | Create a project configuration from an empty project or an existing ECU checkout by discovering and proposing available DBC/A2L/EDS/diagnostic/script resources. |
| REQ-AI-016 | P1 | Proposed | Validate and update existing project configurations through schema-aware operations with preview/diff, explicit resource resolution and non-destructive failure handling. |
| REQ-AI-017 | P0 | Proposed | AI project creation/editing must preserve the single configuration file within an ECU repository; never reorganize, duplicate or modify ECU source/resources without separately authorized action. |
| REQ-AI-018 | P1 | Proposed | Project operations preserve relative paths, per-worktree resource isolation, machine-local hardware mapping and deterministic, Git-diff-friendly serialization. |
| REQ-AI-019 | P1 | Proposed | Let agents create, rename, update, clone and remove dashboard definitions using a typed workspace/dashboard model with stable IDs and schema validation. |
| REQ-AI-020 | P1 | Proposed | Let agents add, configure and arrange built-in dashboard widgets (numeric, plot, gauges/status) and bind available signals with units, quality and source identity. |
| REQ-AI-021 | P2 | Proposed | Expose supported third-party dashboard widget/panel types and their declarative configuration; missing/incompatible plugins produce recoverable placeholders rather than losing layout data. |
| REQ-AI-022 | P1 | Proposed | Dashboard and workspace changes support a preview, validation, explicit save/apply semantics, and correct user-local versus explicitly shared persistence. |
| REQ-AI-023 | P1 | Proposed | Support agent-driven Trace view, plot, measurement, filter and layout configuration through the same model/command services as interactive UI actions. |
| REQ-AI-024 | P1 | Proposed | Let agents define and run **authorized** virtual test scenarios, scripted tests and repeatable replay/debug sequences through the existing test and scheduling services. |
| REQ-AI-025 | P2 | Proposed | Let agents propose scripts, test functions, macros and trigger rules; filesystem writes, execution and auto-run remain separately authorized and never implicit. |
| REQ-AI-026 | P1 | Proposed | Expose capture/recording controls and active diagnostics as explicit commands with lifecycle/status, capability checks, cancellation and action-specific permissions. |
| REQ-AI-027 | P0 | Proposed | Deny side-effecting MCP/AI calls by default unless the user grants scoped capabilities; separate read-only access, config edits, file writes, capture, active diagnostics, TX, XCP/calibration writes, flashing and code execution. |
| REQ-AI-028 | P0 | Proposed | Require explicit user authorization for safety-critical effects (TX, ECU state changes, calibration writes, flash, arbitrary code); target, payload/image and consequences must be inspectable. The model cannot approve itself. |
| REQ-AI-029 | P0 | Proposed | Treat project content, trace payloads, diagnostic responses, MCP data and model-generated instructions as untrusted; never interpret them as privileged commands or as permission grants. |
| REQ-AI-030 | P1 | Proposed | Provide an audit trail for agent identity, tool, project/session generation, requested mutation, permission/approval decision and outcome, without logging secrets unnecessarily. |
| REQ-AI-031 | P0 | Proposed | Enforce project/worktree/session isolation, path restrictions, secret redaction and cancellation/expiration of stale session or resource handles. |
| REQ-AI-032 | P0 | Proposed | Bound MCP/AI query page sizes, result bytes, rate, concurrency, memory and job duration; allow cancellation and report truncation, gaps and overload explicitly. |
| REQ-AI-033 | P1 | Proposed | Support async operations with operation IDs, status, progress, timeout, cancel and final result; do not block UI/capture or assume a long-running transport session. |
| REQ-AI-034 | P1 | Proposed | Offer idempotent creation/upsert requests and explicit preview/diff/commit for configuration mutations; prevent duplicate widgets or partial project writes on retries. |
| REQ-AI-035 | P0 | Proposed | Normal CANexus use and local debugging must work with AI/MCP disabled and without Internet; remote model data sharing requires informed opt-in and privacy controls. |
| REQ-AI-036 | P1 | Proposed | Document interoperable MCP protocol versions, per-version compatibility, transport security, tool schema migrations and Windows/Linux integration tests. |

## 4. Required agent workflows

The tool catalog below is illustrative. Exact service/tool names and output schemas must be designed, versioned and tested before implementation.

### A. CAN / ECU debugging

1. Select a **specific** open project instance, ECU/adapter session and channel or an offline/virtual fixture.
2. Inspect adapter state, trace health, gaps, timestamp domains, loaded DBCs and available protocols.
3. Execute a bounded trace query; inspect raw CAN frame evidence and decoded signal/UDS interpretation.
4. Correlate evidence conservatively: distinguish ordered source sequences from unsynchronized multi-channel timestamps; retain DBC/plugin versions.
5. Suggest a cause and optional follow-up experiment. Physical active actions require a separate permission check and possibly user approval.
6. Return a reproducible analysis summary with event/query references, evidence limitations and concrete validation steps.

### B. Create / edit a project

1. Discover an existing ECU checkout or start with an explicit empty target path.
2. Propose the project name, logical channels and found resource references, without modifying source files.
3. Validate paths against the chosen checkout and schema; display a diff of the one CANexus configuration file.
4. Obtain approval for filesystem creation/update and commit atomically; report the final project path and diagnostics.
5. Resolve machine-specific adapter bindings outside the portable file; do not silently select real transmit hardware.
6. A project copied to another worktree resolves **that** checkout's resources, not a globally cached sibling.

### C. Generate a dashboard

1. Ask for or infer an explicit purpose (for example, powertrain thermal monitoring), within an identified project and data source.
2. Discover typed signal catalog, physical units, real/virtual availability and suitable built-in or installed widget types.
3. Construct a validated draft dashboard: named plots/numerics, signal bindings, axes and a dock/layout model, with stable widget IDs.
4. Preview changes (including missing signals or unavailable plugins), without resetting live acquisitions or the recorder.
5. Apply/save to user-local workspace by default; write to the single project file only when explicitly shared and supported by the project schema.
6. Reopen the dashboard and verify bindings, layout, ownership and signal quality; edits must remain possible manually.

### D. Test and automation authoring

Agents can propose virtual CAN fixtures, Python test functions, trace assertions, trigger/measurement configuration and diagnostic workflows. Running tests uses the existing headless test service and its isolated Python workers. The user must authorize any file writes, real-hardware operations or script execution. Generated artifacts must not auto-run on project open.

## 5. Permissions, safety and data handling

| Action class | Default proposal | Important boundary |
|---|---|---|
| Existing allowed offline/virtual state and bounded Trace/Signal reads | Read-only, explicitly enabled MCP access | Project/session scope; redact secrets; no unlimited dumps |
| Create/edit project, dashboards, user-local workspace, external files | Require write capability and reviewable diff/preview | No implicit ECU source changes or extra mandatory files |
| Start capture/recording or reconfigure real hardware | Explicit active-operation permission | Hardware/session ownership, stop/cancel handling |
| Physical UDS/XCP read or other bus-request operation | Explicit diagnostic/measurement permission | Even a logical read sends frames on a physical bus |
| CAN TX, ECU reset/write, XCP calibration write, flash | Explicit action-specific authorization and human approval | Validate target, current session, binding, payload/firmware; no model self-approval |
| Execute scripts or tests with side effects | Separate execution authorization | Run untrusted Python only under the specified isolated worker model |

Security controls cannot depend solely on MCP tool annotations or on the external client honoring an approval prompt. Authorization must be enforced server-side by CANexus application services. Approval must be tied to the **actual current operation**; stale project/session generations invalidate it. Dangerous operations must not be bundled under a blanket “debug” permission.

The MCP server is **off by default**. Local stdio avoids an always-on listening socket. Any future HTTP transport must address authentication/authorization, loopback restrictions, origin validation, secret storage, rate limiting and accidental network exposure. Model providers must not automatically upload project files, captures, secrets or proprietary ECU data.

## 6. Acceptance scenarios (proposed)

- [ ] **Headless read-only MCP:** A conforming external MCP client discovers virtual CAN tools/resources, lists active capabilities and pages fixture Trace events on Windows and Linux without starting Slint.
- [ ] **No-capture regression:** A slow MCP client requests large data repeatedly; ingress, recording and GUI (if open) remain independent and limits, cancellation, pagination and truncation are observable.
- [ ] **Debug provenance:** An injected frame gap, bus-off and a deliberately wrong DBC signal interpretation are reported with raw event references, exact available quality/provenance and no invented cross-channel order.
- [ ] **Permission parity:** The same unauthorized physical TX, UDS active request, calibration write or flash action fails through MCP, CLI and GUI/shared dispatcher (where supported).
- [ ] **Approval resistance:** A malicious instruction inside CAN payload/DBC/UDS description/trace text cannot invoke privileged tools, grant permissions or approve a write.
- [ ] **Missing capability:** Without a UDS/XCP/dashboard plugin, the agent receives an actionable unavailable-capability error rather than a fabricated success.
- [ ] **Project authoring:** Create a project in an ECU checkout containing pre-existing DBC/A2L files; only the single intended CANexus project configuration is created, paths are relative and opening a second worktree resolves separate resources.
- [ ] **Project edit safety:** Invalid/missing resource, stale generation and concurrent configuration modification are rejected with diagnostics and leave the previous valid file intact.
- [ ] **Dashboard authoring:** From a virtual signal catalog, create a two-plot/three-numeric dashboard, preview and save; reopen with identical stable IDs, signal bindings, correct layout and explicitly selected local/shared storage.
- [ ] **Dashboard resilience:** A disabled third-party widget yields a recoverable placeholder; regenerating/upserting the dashboard does not duplicate widgets or terminate recording.
- [ ] **Virtual testing:** An agent launches a deterministic virtual test via MCP and gets matching result/trace references through headless CLI for the same fixture; script worker failure cannot stall acquisition.
- [ ] **Privacy/offline:** Disable AI/MCP and network; all normal CANexus functionality remains usable; remote model communication requires opt-in and sensitive project content is never transmitted implicitly.
- [ ] **Lifecycle:** Close/reopen project while an asynchronous query is pending; stale operation cannot mutate the replacement session or a sibling worktree.
- [ ] **Interoperability:** Validate schema discovery, tool inputs/outputs, cancellation, errors and permission failures with at least one independent MCP client against a documented protocol version on Windows and Linux.

## 7. Delivery sequence and dependencies (proposal)

1. **Service contract and safety foundation:** inventory existing project/workspace/trace/measurement/command services; agree on typed DTOs, capability discovery, project-instance handles, permission scopes, error schema and audit semantics. No new capture-path work.
2. **Read-only local MCP vertical slice:** stdio server + headless virtual CAN + bounded trace/health queries + independent client integration tests. Depends on the core Trace query APIs and test harness; does not depend on a production GUI.
3. **Controlled creation/editing:** first project create/validate/upsert, then dashboard/workspace CRUD and typed signal/widget bindings after their respective persistence and UI model contracts are mature.
4. **Advanced control and native assistant:** permissioned active capture/diagnostics/test operations and a model-provider-neutral in-app assistant after shared service and approval boundaries are verified.
5. **Optional remote transport and future protocol plugins:** only with a reviewed authentication and network exposure model.

No item in this sequence is currently implemented by this document. A new agent API must not bypass or silently reimplement existing schedulers, persistence, diagnostics or user authorization.

## 8. Open decisions

- Should the in-app assistant ship alongside local MCP or follow the externally usable MCP server?
- Exact MCP protocol version(s), Rust SDK, tool naming/versioning, schema generation, stdio process startup and host lifecycle.
- Is an in-app model runtime required in v1 or are optional configured model providers sufficient?
- Minimum authorization UX, consent persistence, non-interactive/headless policy and safety-critical approval mechanism.
- Which project/dashboard settings are serializable in the single ECU configuration versus user-local workspace state?
- How to preview/apply or rollback multiple interdependent project/dashboard edits; exact atomic persistence strategy.
- Which dashboards/widgets/plot types exist in the initial product and their stable schema/panel lifecycle contract.
- Scope of physical UDS/XCP active reads, CAN TX and remote-access permissions for early MCP milestones.
- Capture/recorder performance budgets and supported maximum result/window sizes measured with real workloads.
- Whether MCP resources/prompts and optional outbound MCP client integration are needed in the first release.

## 9. Relationship to existing requirements

This proposal is additive. Preserve the accepted single-project-file and local/shared settings model (REQ-PROJ-004/011/012), existing SDK/commands requirements (REQ-PLG-012, REQ-CMD-001/006), Python scripting permissions and isolation (REQ-SCR-010/011/022), central measurement architecture (REQ-MEAS-*), Trace provenance and bounds (REQ-TRACE-*, REQ-ARCH-*), and diagnostic/flash/XCP preconditions. Future network protocols must register services and typed capabilities rather than force CAN-only semantics onto every agent tool.

**Normative upstream reference for interoperability planning:** Model Context Protocol specification, https://modelcontextprotocol.io/specification/ (verify and pin the implemented protocol revision during design; no SDK selected here).
