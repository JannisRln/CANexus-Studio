# User Experience & Performance

**Status:** Proposed · **Revision:** 0.4

## Principle

Performance includes both high-throughput CAN processing **and perceived responsiveness**. The application must not make the user wait for whole-project scans, giant traces, or synchronous hardware timeouts before the interface becomes usable.

**GUI architecture:** Slint with pbSlintDock, with stateful application models held outside transient panel instances.

## Requirements

| ID | Priority | Status | Requirement |
|---|---|---|---|
| REQ-UX-001 | P0 | Proposed | The UI event thread does not perform blocking hardware I/O, file indexing or protocol operations. |
| REQ-UX-002 | P0 | Proposed | Main window becomes interactive before optional project scans complete. |
| REQ-UX-003 | P0 | Proposed | Trace displays use virtualization, view-level batching and demand-driven data access. |
| REQ-UX-004 | P0 | Proposed | Dock panels, split views and tabs support saved/restored layouts. |
| REQ-UX-005 | P1 | Proposed | The user can switch workspaces without restarting recordings or protocol sessions. |
| REQ-UX-006 | P1 | Proposed | Long operations expose progress, error and cancellation when safe. |
| REQ-UX-007 | P1 | Proposed | Common operations are keyboard accessible, discoverable and reversible where appropriate. |
| REQ-UX-008 | P1 | Proposed | Search, filtering and sorting large traces must not freeze unrelated panels. |
| REQ-UX-009 | P1 | Proposed | Floating/multi-monitor window behavior is validated on both supported OSes. |
| REQ-UX-010 | P1 | Proposed | UI performance is continuously measured with repeatable representative datasets. |

## Initial targets (not yet accepted)

- UI visible feedback: p95 < 50 ms.
- Main window interactive on cold start: p95 < 2 seconds, defined reference hardware.
- Switching already loaded panels: p95 < 100 ms.
- Trace scrolling: target 60 FPS under representative benchmark load.
- Background indexing and recording must not block navigation.

These are measurement hypotheses, not current guarantees. Define reference systems, dataset sizes, instrumentation and percentile windows before acceptance.

## Open decisions

Command palette, global search, keyboard-first workflow, plugin-owned UI panels, layout persistence defaults, accessibility, light/dark themes, high-DPI behavior.
