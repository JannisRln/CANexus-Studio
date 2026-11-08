# ADR-006 — Project Configuration & Path Resolution

**Status:** Proposed  
**Revision:** 0.1 · **Date:** 2026-11-08  
**Related:** [Project Model & Portability](01-project-model.md), [HAL & Plugins](05-hal-plugins.md), [Scripting](08-scripting-automation.md)

## Context

CANexus-Studio is installed outside ECU repositories. An existing ECU checkout should require only one CANexus-specific project configuration file (provisionally `canexus.toml`), without imposing a new directory structure. Its existing DBC, A2L, EDS, diagnostics, scripts and source files stay where they are.

Git is **not** an application feature or dependency. There are no built-in commit, branch, checkout, merge or worktree operations. Independent checkouts and Git worktrees are ordinary, independent project directories. The design must support external Git workflows through portable, deterministic text configuration.

## Decision

1. The **directory containing the opened main project file** is the sole base for all project-relative resource paths, including paths occurring in nested configuration files. Never use the process working directory.
2. Project-local input resources are resolved only within the respective opened project directory unless an explicit external-resource policy applies. Never fall back to another clone or worktree.
3. The default ECU repository footprint is one `canexus.toml` file. Do not generate layouts, lockfiles, cache, recordings, application binaries, plugin files, logs or user settings inside the ECU repository by default.
4. Resolve workspace expressions through one central, typed path-resolution service, also used by plugins via the application services. Plugins must not independently reinterpret variables.
5. Separate **ProjectDefinition** (portable shared configuration), **ProjectInstance** (runtime identity for one opened project-file location), **ProjectSession** (active measurement/diagnostic/calibration session) and **LocalProjectState** (machine/user-specific storage outside the ECU checkout).
6. Logical hardware channel names live in the project; actual adapter mappings are machine-local. An unresolved mapping must never silently choose a transmit-capable device.
7. Loading resources is independent and asynchronous where appropriate. Errors identify the affected resource; stale background results must not be applied after changing project context.

## Workspace expression syntax

| Expression | Resolution | Scope |
|---|---|---|
| `${workspaceFolder}` | Directory containing the opened project file | MVP |
| `${projectDir}` | Alias of `${workspaceFolder}` | MVP |
| `${projectFile}` | Absolute path to opened project file, at runtime | MVP |
| `${workspaceFolderBasename}` | Name of the project-file directory | MVP |
| `${userHome}` | OS user home | MVP |
| `${configDir}` | CANexus user configuration directory | MVP |
| `${dataDir}` | CANexus user data directory | MVP |
| `${env:NAME}` | Explicitly permitted environment variable, depending on field policy | MVP, restricted |
| `${project:NAME}` | Explicit project-defined variable | Future, not MVP |

Unknown/malformed variables fail with structured diagnostics. No shell evaluation, code execution, recursive unrestricted substitution or arbitrary expression language. Variable expansion does not grant filesystem access; access permissions are checked at use sites. Paths for resources inside the checkout should normally be saved as plain relative paths, rather than expanded absolute paths.

## Example (illustrative schema; file name and schema remain open)

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
name = "Engine ECU"

[resources]
dbc = ["config/network.dbc"]
a2l = ["${workspaceFolder}/calibration/ecu.a2l"]
diagnostics = ["diagnostics/uds.yaml"]
scripts = ["tests/diagnostics.py"]

[channels.CAN1]
type = "can"
bitrate = 500000
```

The `[resources]` section is consistent with the existing project-model examples. Exact channel and path-field schemas are not yet approved.

## Resolution contract

For a resource field: (1) parse and validate the configuration; (2) check that expressions and resource class are permitted; (3) expand keywords; (4) resolve any relative result against the project-file directory; (5) normalize lexically without changing the saved portable form; (6) validate existence, type, symlinks and access rules as required by operation; (7) return a typed result or structured error.

**Distinguish resolution from existence:** an output recording target may legitimately not exist yet. Symlinks, Windows reparse points and race conditions require access-time checks rather than relying on string normalization alone.

Resource classes (proposed): `ProjectInput`, `ExternalInput`, `UserOutput`. External dependencies must be visibly marked non-portable; environment variables are allowed only in approved fields. No silently broadened file access for scripts/plugins.

## API boundary (conceptual, not implementation)

```rust
pub struct ProjectContext {
    pub project_file: PathBuf,
    pub project_dir: PathBuf,
}

pub enum ResourceKind {
    ProjectInput,
    ExternalInput,
    UserOutput,
}

pub struct ResolvedResource {
    pub path: PathBuf,
    pub kind: ResourceKind,
}

pub trait PathResolver {
    fn resolve(
        &self,
        expression: &str,
        kind: ResourceKind,
        context: &ProjectContext,
    ) -> Result<ResolvedResource, PathError>;
}
```

A dedicated `canexus-paths` crate is proposed; `canexus-project` owns loading, schema validation and project-instance lifecycle. The GUI and plugin-host APIs access the same resolver services; no direct plugin dependence on arbitrary internal structs is implied by the sketch.

## Additional requirements (proposed; preserve REQ-PROJ IDs)

| ID | Priority | Status | Requirement |
|---|---|---|---|
| REQ-PATH-001 | P0 | Proposed | Resolve all project-relative paths against the opened project-file directory, independent of process CWD. |
| REQ-PATH-002 | P0 | Proposed | `${workspaceFolder}` refers exclusively to the opened project-file directory. |
| REQ-PATH-003 | P0 | Proposed | Simultaneously opened copies/worktrees use independent project contexts and resource resolution. |
| REQ-PATH-004 | P0 | Proposed | Project loading and use require no installed Git or Git metadata. |
| REQ-PATH-005 | P0 | Proposed | Loading/saving never persists the absolute project-instance location in shared configuration. |
| REQ-PATH-006 | P0 | Proposed | Unknown or malformed variables produce structured, actionable diagnostics. |
| REQ-PATH-007 | P0 | Proposed | Project-bound input resources cannot escape the allowed root through traversal, symlinks or platform-specific links without explicit external-resource authorization. |
| REQ-PATH-008 | P1 | Proposed | Explicit external-resource fields may use permitted variables and are marked non-portable. |
| REQ-PATH-009 | P1 | Proposed | UI offers original expression, resolved path and validation status. |
| REQ-PATH-010 | P1 | Proposed | Unchanged project configurations serialize deterministically and remain byte-identical on no-op save. |
| REQ-PATH-011 | P1 | Proposed | Windows/Linux normalization and handling of platform-specific roots are covered by tests. |
| REQ-PATH-012 | P1 | Proposed | One resource's load failure does not block independent resources. |

## Acceptance tests

1. Copy `ECU-A` to `ECU-B`, change B's DBC, open B: load B's DBC only.
2. Open A and B at the same time: no shared resolution state or resource leakage.
3. Launch from an unrelated process CWD: identical project-local results.
4. Run without Git installed or a `.git` directory: normal project functionality.
5. Open/close and no-op save: no generated checkout files, and no diff to the existing project configuration.
6. Missing DBC: per-resource error; unrelated A2L remains usable.
7. Unknown variable: error names variable and configuration field.
8. Attempt `../../other/file.dbc` or symlink traversal from `ProjectInput`: denied; an explicitly authorized `ExternalInput` is handled separately.
9. Windows/Linux fixture suite: relative and platform-specific paths behave as documented.
10. Cancel/reopen a project during background loading: no stale results attach to the new `ProjectInstance`.
11. Unresolved logical hardware mappings never auto-select an adapter for TX.

## Trade-offs and risks

- Project-root-only paths maximize portability but restrict shared data outside ECU checkouts: use explicit external resources rather than implicit fallback.
- Environment variable references can be machine-dependent: validate and identify non-portable dependencies.
- Platform path semantics differ: test drive letters, UNC paths, separators, case behavior and symlinks/reparse points.
- A lexical path check alone is not a security boundary: enforce access permissions at the operation and treat untrusted scripts/plugins separately.
- Multiple project instances may reference one physical adapter: hardware ownership/TX arbitration is a separate HAL concern.

## Open decisions

- Final project filename and schema versioning/migration.
- Exact allowed `env:` fields, external-resource policy and UI warnings.
- Identity/key strategy for `LocalProjectState` across moves, copies and simultaneous worktrees.
- Local shared/personal setting precedence and explicit promotion to project configuration.
- Stable plugin-facing resolver contract, errors and capability checks.
- Whether a path expression such as `${projectFile}` is valid in file-path fields requiring a directory or only in explicitly typed contexts.

**This is a proposal, not an implemented feature or an Accepted requirement.** Adopt through the repository's existing review and status process.
