# CANexus-Studio implementation roles

These roles own work items and review boundaries. [Backlog](BACKLOG.md), [status](STATUS.md) and [verified integration](README.md) remain the source of task state. The current desktop scope follows [ADR-012](../requirements/26-adr-012-egui-craft-ui.md) and the [migration plan](GUI-MIGRATION.md).

| Role | Ownership and interfaces | Handoff and acceptance evidence |
|---|---|---|
| Lead software architect and integrator | ADRs, dependencies, Core/application/desktop and IPC boundaries, integration branch. | Approve shared contracts before cross-owned edits. Reconcile requirements and verified main state after merge. |
| Requirements engineer | Canonical requirement IDs, release profiles, traceability and acceptance coverage. | Link requirement, DEV task, commit and test. Document supersessions and unverified coverage without changing unrelated statuses. |
| UX/UI designer | Workspace flows, tokens, accessibility and component catalog under `docs/design/`. | Specify first-start and keyboard journeys, layout, missing/error states and native UI acceptance. Review implementation against the workspace spec. |
| egui and Craft UI developer | DEV-025 shell and DEV-008 production Trace under `crates/canexus-gui`, typed view models and audited widget reuse. | Produce native component showcase and trace interaction evidence. Consume scoped pages and commands without owning acquisition sessions. |
| Craft ecosystem and native docking engineer | DEV-024 isolated dependency proof and DEV-022 isolated viewport example, exact source/feature selection and candidate compatibility. | Deliver license and source graph, native Windows/Linux float/redock, focus, monitor and Wayland matrix. Handoff validated adapters and limitations to the GUI owner. |
| Core and performance engineer | DEV-023 shared-batch accounting, ingest, fairness, and cache/worker budgets. | Verify lifetime charges across clones and eviction, no capture wait due to view work, clear overload counters and reproducible resource data. |
| Hardware and protocol engineer | DEV-010 HAL, DEV-011 DBC and protocol-specific providers for ISO-TP/UDS, XCP/A2L and CANopen. | Supply compatibility matrix, authorized fixtures, time provenance, disconnect/bus-off evidence and protocol-specific coverage. |
| Plugin architecture engineer | DEV-026 manifests, versioned panels, second provider and compatibility/IPC contracts. | Provide separately built provider, stable panel lifecycle and missing/incompatible recovery. Review native ownership and isolation before releasing an SDK. |
| MCP and AI integration engineer | DEV-017/018 typed facade/optional local stdio, bounded read-only runtime tools and separate fixture-only development tools. | Prove schema/role/timeout, independent client parity and no alternate path around user permissions. Keep offline base deployable. |
| Test and quality assurance engineer | DEV-009 deterministic fixtures, GUI/CLI parity, benchmark harness, physical/native test matrices and coverage ledger. | Publish exact run parameters, fixture hashes and raw results. Independently reproduce desktop timing, resource caps and recording integrity. |
| Security and safety engineer | Capability gates, permission/audit review, failure modes, dependency provenance and controlled device actions. | Verify denied/stale device actions create no physical TX/write and untrusted or missing extensions fail visibly. Review recording partial-state semantics. |
| CI/CD and release engineer | DEV-025 toolchain, lockfile, platform packages, cross-platform Actions, reproducible releases and dependency notices. | Supply locked Windows/Linux build/test and actual build IDs. Separate native acceptance and performance from shared CI. |
| Documentation engineer | Build/run documentation, verified status, role handoffs, user workflow/limits and release evidence. | Keep legacy evidence identifiable, link active specs and update public support matrix with tested states. Maintain English API and Doxygen comments. |
| Open-source community and upstream contribution engineer | DEV-027 standalone contribution/readiness, generic Craft feedback and governance research. | Prepare local reproducible upstream candidates and contribution/asset inventory. Formal association or external submission follows owner authorization. |
| Recording and replay engineer | DEV-013 streaming durable writer, independent disk/history retention and offline query/replay. | Prove direct-format interoperability, growing destination files, constrained RAM and visible complete/incomplete state in multi-GiB and storage-failure fixtures. |

## First implementation handoffs

1. **DEV-024** owns an isolated example/worktree and candidate pins. Complete dependency, license, native smoke and platform evidence before production setup. **DEV-023** can proceed independently in Core ingestion and budget tests.
2. **DEV-025** owns target GUI manifests, host, showcase and shared CI cutover. **DEV-022** owns a standalone floating-window validation example and coordinates before touching production source.
3. **DEV-008** integrates the verified target shell with DEV-007 Trace history. Document the bounded query/page contract with the Core and QA owners. **DEV-009** consumes the instrumentation and verifies the acceptance profiles on both platforms.
4. For each task, choose one branch and owner, name the owned files and recheck main, open PRs and status before claiming work. Shared APIs and schemas need an agreed contract before either owner edits them.
5. Attach requirement IDs, commit/PR, fixture hashes, actual commands and results. Mark Done after verified integration according to repository rules. Keep actual timing and hardware evidence distinct from build/test CI.
