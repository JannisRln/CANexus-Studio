# Transmission, Triggers, Logging and Replay

**Revision:** 0.6 · **Status:** Mixed: send modes confirmed, details proposed

## Confirmed decisions

The CAN Analyzer supports **one-shot messages**, **cyclic transmission**, **multi-step sequences**, **sending by signal values** and **triggered operations**. These actions share a controlled transmission engine across GUI, scripts and protocol plugins rather than duplicating scheduling logic.

## Requirements

| ID | Priority | Status | Requirement |
|---|---|---|---|
| REQ-TX-001 | P0 | Accepted | Send one-shot, cyclic, sequence-based, signal-value-driven and trigger-initiated CAN transmissions. |
| REQ-TX-002 | P1 | Proposed | A Send panel has an editable frame list with channel, ID, CAN/FD flags, DLC/data, enable state and descriptive name. |
| REQ-TX-003 | P1 | Proposed | Configure cyclic interval, initial delay, repetition count, start/stop conditions and scheduling status. |
| REQ-TX-004 | P1 | Proposed | Sequence steps may wait for a timer, matching RX frame, decoded signal expression or protocol response, with timeout/error branch. |
| REQ-TX-005 | P1 | Proposed | Users can edit a DBC signal's physical/raw value and see resulting encoded payload, range checks and DLC/endianness before send. |
| REQ-TX-006 | P1 | Proposed | Scheduler orders jobs predictably with documented timing limits; it never promises real-time guarantees beyond OS/adapter capability. |
| REQ-TX-007 | P0 | Proposed | Distinguish read-only observation from active bus transmission. Active jobs are visible and can be stopped centrally. |
| REQ-TX-008 | P1 | Proposed | Project open does not implicitly start transmissions; explicit enable/start required for automated sending. |
| REQ-TX-009 | P1 | Proposed | TX jobs are stopped or safely disposed on project close, adapter disconnect, script cancellation and ownership loss. |
| REQ-TX-010 | P1 | Proposed | All send operations emit observable TX request/result/error events for Trace and log; display whether timestamp reflects request or hardware confirmation. |
| REQ-TX-011 | P1 | Proposed | Trigger builder combines time, frame, signal, bus error and logical AND/OR conditions. |
| REQ-TX-012 | P1 | Proposed | Trigger actions may bookmark, notify, start/stop logging, start/stop a TX sequence or invoke permitted scripts. |
| REQ-TX-013 | P1 | Proposed | Pre-trigger ring buffers preserve configured history and reveal truncation/drop conditions. |
| REQ-TX-014 | P1 | Proposed | Logging and display filters have separately visible and independently configured scopes. |
| REQ-TX-015 | P1 | Proposed | Recorder exposes file path, written events, dropped events, rate and storage errors; writes asynchronously. |
| REQ-TX-016 | P1 | Proposed | Replay offers play/pause/seek/speed and loop over a time range; source timestamps are preserved. |
| REQ-TX-017 | P1 | Proposed | Replay targets a virtual channel by default; transmission to real hardware requires a distinct user-confirmed action. |
| REQ-TX-018 | P2 | Proposed | Users can export selected or filtered trace sections and import supported recording formats. |
| REQ-TX-019 | P2 | Proposed | Automation can inject deterministic virtual traffic into a simulation session and capture machine-readable test evidence. |

## Acceptance tests

1. Send a manual frame, start a cyclic frame job, then stop it; verify no later TX jobs execute.
2. Encode a signed/multiplexed DBC signal using defined scaling; verify resulting bytes against known test vectors.
3. Run a sequence that waits on a matching response then times out cleanly on missing response.
4. Apply a display filter; recording remains unchanged until a separately labeled recording filter is enabled.
5. Generate a trigger with 5s pre-trigger history and verify pre-trigger events are present in the file, subject to documented buffer limits.
6. Replay through Virtual CAN without transmitting on any physical interface.
7. Close a project while a script-owned sequence is active and verify its jobs are cancelled.

## Open decisions

Selection and validation of direct-write standard format, send scheduler jitter budgets, trigger expression syntax, export format order and TX authorization defaults.

## v0.6 accepted direct-to-standard recording

| ID | Status | Requirement |
|---|---|---|
| REQ-LOG-001 | Accepted | Recording writes directly to an interoperable standard format, without mandatory conversion from a proprietary native format. |
| REQ-LOG-002 | Proposed | Evaluate MDF4 against CAN/CAN FD data, stream writing, metadata, seek, replay, independent-reader compatibility and future network support. |
| REQ-LOG-003 | Proposed | Bounded asynchronous recorder buffering reports drops/disk errors and keeps memory bounded across long recordings. |

Acceptance: record sustained synthetic CAN FD traffic directly to the selected standard; reopen in CANexus and independent compatible software; compare IDs, payloads, flags and timestamps; interrupt recording and verify explicit recovery/error status; filtering only the display does not filter recorded frames. The specific standard and recovery strategy are still open.
