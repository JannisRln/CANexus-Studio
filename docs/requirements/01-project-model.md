# Project Model & Portability

**Status:** Proposed · **Revision:** 0.4

## Confirmed product behavior

CANexus-Studio itself is installed independently from ECU projects. In an existing ECU repository, the user wants **only one additional CANexus project file**, provisionally named `canexus.toml`; existing DBC, A2L, EDS, scripts, diagnostic descriptions and source files remain in their existing locations. CANexus does not impose its own folder layout.

For example:

```text
ECU/
├── src/
├── config/network.dbc
├── calibration/ecu.a2l
├── diagnostics/uds.yaml
└── canexus.toml
```

```toml
schema_version = 1

[project]
name = "My ECU"

[resources]
dbc = ["config/network.dbc"]
a2l = ["calibration/ecu.a2l"]
diagnostics = ["diagnostics/uds.yaml"]
```

The file name and schema are **examples**, not yet approved.

## Requirements

| ID | Priority | Status | Requirement |
|---|---|---|---|
| REQ-PROJ-001 | P0 | Proposed | All internal resource paths resolve relative to the CANexus project file, never the process working directory. |
| REQ-PROJ-002 | P0 | Proposed | Moving or copying an ECU directory with its existing resources must not require path edits. |
| REQ-PROJ-003 | P0 | Proposed | An ECU Git checkout/worktree must load resources belonging to that checkout, not another copy. |
| REQ-PROJ-004 | P0 | Proposed | Only one CANexus-specific project configuration file is needed inside the ECU repository. |
| REQ-PROJ-005 | P0 | Proposed | CANexus binaries, plugins, caches, logs, trace recordings and personal settings live outside ECU repositories by default. |
| REQ-PROJ-006 | P0 | Proposed | The project file is human-readable, versioned, deterministic to serialize, and Git-diff-friendly. |
| REQ-PROJ-007 | P1 | Proposed | Missing files are reported individually with clear path and resolution actions; no fallback to files in other clones. |
| REQ-PROJ-008 | P1 | Proposed | External file dependencies, when permitted, are explicitly identified as non-portable. |
| REQ-PROJ-009 | P1 | Proposed | Project settings can be opened and edited without changing ECU source files. |
| REQ-PROJ-010 | P1 | Proposed | Optional resource discovery suggests existing resources without silently copying or modifying them. |

## Acceptance scenarios

1. Open `ECU-A/canexus.toml`; load `ECU-A/config/network.dbc`.
2. Clone to `ECU-B` and change B's DBC; opening B must use B's changed DBC.
3. Launch CANexus with process working directory outside both copies; resolution is unaffected.
4. Open and close without changes: no extra files or modifications in the ECU repository.
5. Delete a configured resource: clear validation error, no implicit lookup in A or global directories.
6. Open the same project configuration on supported Windows and Linux machines.

## Open decisions

- Is `canexus.toml` the final filename?
- Project-shared vs user-local layouts (prefer user-local default).
- Permitted references outside the project directory.
- Handling of OS-specific hardware aliases without saving machine-specific paths.
