# ADR-007 — ELF/DWARF Symbols, Metadata, Dashboards and Direct XCP Access

**Decision status:** Accepted direction (2026-11-08)  
**Detailed requirements status:** Proposed · **Revision:** 1.2  
**Related:** [Project Model](01-project-model.md), [Signal Explorer](07-signal-workspace.md), [XCP and CANopen](14-xcp-canopen.md), [ADR-006 Paths](18-project-configuration-path-resolution.md)

## Context and decision

A2L-first workflows can generate noisy, build-dependent diffs. CANexus-Studio adopts an **ELF/DWARF-first, A2L-compatible** approach. The ELF provides build-specific symbol/type/location facts; user-authored files describe *logical identity*, *semantic metadata*, and *dashboard configuration* independently. All discoverable ELF/DWARF symbols remain available in the explorer even without explicit configuration. The architecture must not generate and commit extensive symbol/address lists on every build.

The ECU checkout needs only one **mandatory** CANexus file (provisionally `canexus.toml`). Additional **optional** symbol, metadata and dashboard files can be referenced from that manifest, anywhere within permitted project-relative locations; no folder layout is mandatory. These user-maintained files are individually versionable by external Git tooling. CANexus itself does not manage Git, branches, commits or worktrees.

### Files and responsibilities

| Kind | Responsibility | Must not contain |
|---|---|---|
| Main project manifest | References ELF/A2L, symbol files, metadata files, dashboard files, logical channels | Absolute build-address caches, local user state |
| Symbol definition | Stable aliases, qualified selectors, disambiguation/expected structural contracts | Display units, conversions, UI layout |
| Metadata definition | Units, conversion, format, limits, descriptions, calibration semantics | Build-specific addresses, dashboard layouts |
| Dashboard definition | One dashboard per file: stable dashboard ID, panels, bindings, layout | ELF addresses, authority to perform ECU writes |
| Runtime/cache | Artifact index, resolved addresses, transient values, XCP bindings | Never committed or generated in checkout by default |

One dashboard per file applies to *shared persisted dashboards*. Personal/unsaved dashboards and app-wide docking layout stay user-local, unless explicitly exported/shared.

Illustrative layout, not mandatory:

```text
ECU/
├── canexus.toml
├── build/firmware.elf
└── canexus/
    ├── symbols/motor.toml
    ├── metadata/motor.toml
    └── dashboards/motor-overview.toml
```

### Example manifest (schema proposed)

```toml
schema_version = 1
[project]
name = "Motor ECU"
[resources]
elf = "build/firmware.elf"
[symbols]
files = ["canexus/symbols/motor.toml"]
[metadata]
files = ["canexus/metadata/motor.toml"]
[dashboards]
files = ["canexus/dashboards/motor-overview.toml"]
```

All paths, including references in optional files, resolve relative to the **main project file** per ADR-006. No project-file path is resolved against process CWD or another worktree. Generated indexes and user-local state live outside the ECU checkout.

## Symbol identity and types

A `SymbolKey` is a qualified symbol path, optionally disambiguated by module/compile-unit/source identity. A user-defined stable alias ID may be mapped to a SymbolKey. Names/aliases are logical references; memory addresses are *not* identities. Duplicate or ambiguous matches must produce diagnostics, not arbitrary binding. Aliases may be updated explicitly on source-level rename while metadata and dashboards continue referencing the stable alias.

Illustrative symbol file:

```toml
schema_version = 1
namespace = "motor"
[symbols.pid_kp]
path = "motor_controller.pid.kp"
[symbols.pid_ki]
path = "motor_controller.pid.ki"
[symbols.active_controller]
path = "active_controller"
```

Qualified aliases in other files: `motor.pid_kp`, `motor.pid_ki`. Symbols without aliases can also be selected directly with unambiguous qualified selectors.

Internally separate:
- `SymbolKey` / optional `SymbolAliasId` (logical identity)
- `ArtifactIdentity` (ELF build-specific identity)
- `TypeFingerprint` (canonical, target-aware type layout)
- `SymbolLocation` / `AddressBinding` (build-specific)
- `XcpBinding` (connection/session-specific)

The DWARF type graph supports primitives, typedefs, enums, structs/classes, unions, bitfields, fixed and multidimensional arrays, pointers and references where described. Actual target sizes, endianness, member offsets, bit offsets, alignment, bounds and strides are used; never infer target layouts from host Rust layouts. Large arrays and nested types are expanded lazily. Optimized-away, register-only and runtime-dependent locations remain visible but may not be directly addressable.

### Pointer behavior

**Pointer type resolution is automatic; runtime dereferencing is not.** The explorer can show the statically known pointee type and member hierarchy without accessing target memory. Reading the current pointer **value** requires an explicit XCP read action; accessing the pointed-to object is a separate, explicitly requested and validated operation. Do not recursively follow pointer chains, implicitly dereference nested pointers, or automatically configure DAQ for dynamic targets.

For dynamic target access, reject null/unmapped/out-of-range values, enforce target pointer width, endianness, alignment and memory-range rules, cap traversal depth (initial implementation: one explicit level per action), re-read/revalidate on new access and never treat a previously resolved address as permanent. Pointer-value read and pointed-object read are distinct operations.

## Metadata and dashboard model

Illustrative metadata:

```toml
schema_version = 1
[symbols."motor.pid_kp"]
display_name = "Proportional Gain"
min = 0.0
max = 100.0
[symbols."motor.temperature"]
unit = "degC"
[symbols."motor.temperature".conversion]
kind = "linear"
scale = 0.1
offset = -40.0
```

Illustrative dashboard:

```toml
schema_version = 1
[dashboard]
id = "motor.overview"
title = "Motor Overview"
[[panels]]
id = "pid"
type = "calibration"
symbols = ["motor.pid_kp", "motor.pid_ki"]
[layout]
columns = 2
```

Symbol definitions, semantic metadata and dashboard layout are *not* duplicated across files. Dashboards refer to logical IDs, inherit applicable unit/conversion metadata and can override view-only presentation options. Missing symbols persist as unresolved references, not deleted entries. App-wide docking/personal layout is user-local by default.

Configuration merge is deterministic: artifact structural facts are immutable; imported A2L may contribute protocol/semantic facts; explicit user metadata can override *semantic* fields, but cannot silently alter ELF structural facts or grant hardware permissions. Conflicting aliases/duplicate metadata definitions are diagnosed with file/field provenance, not resolved by filesystem enumeration order. No-op saves must be byte-identical; stable serialization should minimize Git diffs.

## XCP access: direct and straightforward

User workflow: **load matching ELF, pick symbol, read or write using XCP**; optionally configure DAQ when supported. There is **no special XCP development mode**, time-limited development authorization, or application-specific audit/approval workflow for ordinary reads/writes.

The symbol resolver must provide a usable static target address (or an explicitly resolved pointer target), target data representation and supported memory mapping. The XCP layer translates addresses if required, applies byte order and conversion, and uses ordinary XCP operations and ECU-enforced security/access. Read and write are separate commands and report transport/ECU errors clearly. Structs/arrays can be accessed by supported members/elements/ranges without assuming whole-object operations are universally supported.

**Compatibility:** If the ECU firmware identity is unavailable, show an unverified warning rather than requiring a special mode. If a firmware mismatch is *confirmed*, prominently warn and require one explicit acknowledgement before permitting risky writes; do not silently re-map mismatching symbols or pretend compatibility. Unknown or ambiguous addresses, unsupported types, missing access capability and malformed bounds result in rejection, not guessed access. The normal ECU XCP protection and applicable access checks are never bypassed. A warning/acknowledgement is not a guarantee that the target address is safe.

Eligibility distinguishes `SymbolResolved`, `AddressResolved`, `Readable`, `DaqEligible`, `DaqConfigured`, `Writable` and `Unavailable`; these are independent capabilities, not one Boolean. DAQ configuration also depends on target event and resource information absent from ELF alone. Runtime bindings are scoped to one ECU/session/artifact and invalidated when a relevant artifact, session, memory map or target configuration changes.

A2L remains an optional input/output interoperability source; ELF and A2L may be used independently or compared. A2L-only use is supported by existing product scope. Conflicting source facts are reported rather than silently overwritten. A2L export is limited to information genuinely available in the chosen supported subset.

## API boundaries

Proposed reusable services / crates:
- `canexus-debug-info`: ELF/DWARF parsing and location interpretation.
- `canexus-symbols`: catalog, alias/identity resolution, type graph and structural comparison.
- `canexus-metadata`: file loading, semantic overlays, deterministic merge and provenance.
- `canexus-project`: manifest and file references under ADR-006.
- `canexus-measurement`: object selection and eligibility.
- `canexus-xcp`: transport/session, XCP read/write, DAQ, address translation and ECU errors.
- `canexus-a2l`: optional A2L import/export and reconciliation.
- Dashboard UI uses typed symbol references and view models, never raw DWARF or addresses directly.

External plugins consume versioned services, not unstable internal Rust types. ELF indexing, symbol searches and dashboard loading must not block recording or the GUI; use bounded, invalidatable caches outside checkout and lazy views.

## Requirements (proposed detail)

| ID | Priority | Requirement |
|---|---|---|
| REQ-ELF-001 | P0 | Load ELF with/without DWARF, reporting available capabilities. |
| REQ-ELF-002 | P0 | Expose all discoverable symbols through a searchable, hierarchical catalog. |
| REQ-ELF-003 | P0 | Keep logical symbol identity separate from build-specific address. |
| REQ-ELF-004 | P0 | Interpret supported DWARF struct/array/member/enum/bitfield layouts using target ABI. |
| REQ-ELF-005 | P0 | Detect ambiguous/missing symbols and incompatible structural changes. |
| REQ-ELF-006 | P0 | Resolve pointee types statically without implicit runtime dereferencing. |
| REQ-ELF-007 | P1 | Support explicit, validated pointer-value and pointed-object reads as separate operations. |
| REQ-ELF-008 | P1 | Compare two builds semantically, including address-only vs type/layout changes. |
| REQ-SYM-001 | P0 | Support optional symbol definition files containing stable aliases/selectors only. |
| REQ-SYM-002 | P0 | Allow raw ELF/DWARF symbols to be used without configured aliases if unambiguous. |
| REQ-SYM-003 | P0 | Preserve alias references across rebuilds and explicit source renames. |
| REQ-META-009 | P0 | Support optional separate, project-relative semantic metadata files. |
| REQ-META-010 | P0 | Metadata references stable symbol identity; no generated address copies. |
| REQ-META-011 | P0 | Resolve conflicts deterministically with diagnostic provenance; no silent structural overrides. |
| REQ-DASH-004 | P1 | Each shared dashboard is stored in its own file with stable ID and symbol references. |
| REQ-DASH-005 | P1 | Dashboards store no build-specific addresses or access grants. |
| REQ-DASH-006 | P1 | Personal docking layout changes do not modify shared dashboard/project files. |
| REQ-XCP-ELF-001 | P0 | Bind resolved ELF symbols to specific XCP sessions and invalidate stale bindings. |
| REQ-XCP-ELF-002 | P0 | Support direct XCP read of supported addressable symbols. |
| REQ-XCP-ELF-003 | P0 | Support direct XCP write of supported addressable symbols, respecting ECU protection and normal access checks. |
| REQ-XCP-ELF-004 | P1 | Read/write supported struct members and array elements using actual target layout. |
| REQ-XCP-ELF-005 | P1 | Convert raw/physical values using declared metadata with bounds/type validation. |
| REQ-XCP-ELF-006 | P1 | Configure DAQ only for targets/events supported by the ECU. |
| REQ-XCP-ELF-007 | P0 | Report unresolvable addresses, unsupported types and XCP/ECU errors clearly. |
| REQ-XCP-ELF-008 | P0 | Show unavailable/mismatched firmware identity; confirmed mismatch requires explicit acknowledgement before risky writes, no special development mode. |
| REQ-XCP-ELF-009 | P1 | Never automatically dereference pointers or DAQ-subscribe dynamic pointer targets. |
| REQ-PROJ-013 | P0 | One manifest remains mandatory; optional symbol/metadata/dashboard files follow ADR-006 relative-path rules. |
| REQ-PROJ-014 | P1 | No-op project load/save produces no generated files or unexpected Git diffs. |

IDs above are new for this ADR; preserve already accepted IDs in earlier specifications. The previously discussed development-authorization requirements are **not adopted**. Existing A2L-only and advanced calibration requirements remain in scope; they are not implicitly deleted.

## Acceptance tests

1. Rebuild with address changes only: the same logical references resolve to new addresses without editing metadata or dashboards.
2. Two identical simple names in different compile units: no arbitrary match.
3. Rename source symbol and update one alias mapping: metadata and dashboards need no changes.
4. Verify target struct padding, bitfields and multidimensional array element offsets using compiled DWARF fixtures.
5. Show pointee type hierarchy without issuing any XCP request.
6. Explicit pointer-value read does not read pointed memory; separate target read requires valid address, range and access.
7. Pointer value changes: no reuse of stale dynamic target address; no implicit recursive traversal or DAQ.
8. Read and write a supported scalar through a simulated XCP target; validate wire representation and ECU error reporting.
9. Unknown firmware identity: show a warning, no special mode; confirmed mismatch: require an explicit acknowledgement for risky writes.
10. An unknown location, ambiguous symbol or unsupported target representation cannot result in a guessed XCP operation.
11. Rebuild changes an array length/type: show compatibility issue, invalidate affected bindings.
12. Two independent worktrees resolve separate ELF and metadata resources, even when open simultaneously.
13. Two dashboards stored in separate files can be edited independently without unrelated file rewrites.
14. Opening/closing without edits leaves the ECU checkout unchanged; cache and personal layouts remain outside.
15. Invalid or duplicate alias/metadata definitions emit source-located diagnostics.
16. A2L-only session continues to work; ELF/A2L conflicts are shown rather than silently merged.
17. Large symbol fixtures and malformed DWARF do not freeze UI/acquisition or cause unbounded indexing.

## Risks and non-goals

ELF/DWARF does not guarantee that optimized or dynamic variables are addressable; XCP-specific event/configuration and target mapping may still be needed. Compiler-specific debug information needs coverage tests. Static pointer type exploration is not permission to read arbitrary addresses. Firmware mismatch warnings reduce but do not eliminate address risk.

No mandatory generated A2L, generated per-symbol Git changes, custom Git implementation, automatic pointer graph traversal, special XCP development mode or guarantee of ECU rollback is required.

## Follow-up decisions

- Exact TOML schema and qualified symbol/alias escaping.
- Symbol/metadata imports and conflict precedence for multi-file projects.
- XCP transport defaults, identity discovery and safety behavior for known mismatch.
- Dashboard data bindings, layout versioning and shared/personal presets.
- ELF indexing performance targets and supported DWARF versions/compilers.

**Architecture direction approved in conversation; this document does not claim implementation or verification.**
