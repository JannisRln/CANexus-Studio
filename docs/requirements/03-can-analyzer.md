# CAN Analyzer & Data Workflows

**Status:** Mixed · **Revision:** 0.5

## Scope

CAN Classic and CAN FD observation, decoding, transmission, filtering, trigger, recording and replay, with independent core capture and UI presentation.

**Confirmed:** The Trace provides chronological and fixed/overwrite modes and user-controlled column sorting and filtering. The Signal Explorer is common across modules, while module-specific panels retain different controls. Send functions include one-shot, cyclic, sequences, signal values and triggers.

Detailed canonical specifications: [Trace Viewer](06-trace-viewer.md), [Unified Signal Workspace](07-signal-workspace.md), [Transmission/Triggers/Replay](10-transmission-trigger-replay.md).

## Requirements

| ID | Priority | Status | Requirement |
|---|---|---|---|
| REQ-CAN-001 | P0 | Proposed | Support CAN Classic and CAN FD frames with ID, frame flags, payload, channel and timestamp. |
| REQ-CAN-002 | P0 | Proposed | Acquire frames from multiple hardware/virtual CAN channels in one session. |
| REQ-CAN-003 | P0 | Proposed | Live trace exposes timestamp, channel, direction, ID, DLC, data, flags and optional decoded signals. |
| REQ-CAN-004 | P0 | Proposed | Users can pause visual updates without stopping physical capture or recording. |
| REQ-CAN-005 | P1 | Proposed | Users can filter by channel, ID/mask, direction, data, frame type and decoded signals. |
| REQ-CAN-006 | P1 | Proposed | Users can send one-shot and cyclic frames, with visible TX state and emergency stop. |
| REQ-CAN-007 | P1 | Proposed | Users can search and navigate large past traces without loading all rows into UI memory. |
| REQ-CAN-008 | P1 | Proposed | Decoded DBC signals can be inspected alongside raw bytes and traced back to source frames. |
| REQ-CAN-009 | P1 | Proposed | Recording runs independently of trace rendering, with explicit overflow/error reporting. |
| REQ-CAN-010 | P1 | Proposed | Replay supports speed control, pause, seek and virtual-channel targets. |
| REQ-CAN-011 | P1 | Proposed | Trigger conditions can start/stop recording and retain configurable pre-trigger frames. |
| REQ-CAN-012 | P1 | Proposed | Signals support time-aligned plots, cursors and synchronized inspection with the trace. |
| REQ-CAN-013 | P1 | Proposed | Receive/transmit frame rates and bus/error statistics are visible per channel. |
| REQ-CAN-014 | P2 | Proposed | Import/export common interchange formats (scope to define: ASC, BLF, CSV, MDF4). |
| REQ-CAN-015 | P2 | Proposed | User-defined annotations/bookmarks can be associated with trace timestamps without modifying ECU files. |

## Acceptance scenarios

- Capture a full-load virtual CAN FD channel while searching a past trace; unrelated recording does not stop.
- Toggle display pause while capture continues; UI states clearly distinguish pause types.
- Open a large recording via lazy indexing; scrolling does not allocate one UI object per stored frame.
- Send cyclic frames and stop them explicitly; status and errors are visible.
- Decode one multiplexed DBC frame and inspect the raw and signal values consistently.

## Open decisions

Exact default columns, timestamp presentation, logging file format strategy, trace ring-buffer lifetime, DBC editing vs read-only for the first release, transmitter authorization defaults. Sorting and column filtering are confirmed and no longer open decisions.
