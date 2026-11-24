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
| REQ-LOG-004 | Proposed | Once started, the recorder writes a continuous stream of format-native chunks to the chosen destination. A recording may exceed the configured RAM budget and span multiple GiB. |
| REQ-LOG-005 | Proposed | Users select a recording destination and may configure time- or size-based rollover. Recording storage settings are independent of the temporary Trace cache and its eviction policy. |
| REQ-LOG-006 | Accepted | Starting a measurement supports RAM-only analysis. Temporary disk-backed Trace history and durable file logging are independently optional. Either, both or neither can be enabled. |

Acceptance: record sustained synthetic CAN FD traffic directly to the selected standard; reopen in CANexus and independent compatible software; compare IDs, payloads, flags and timestamps; interrupt recording and verify explicit recovery/error status; filtering only the display does not filter recorded frames. The specific standard and recovery strategy are still open.

## Configurable resource budgets and long recordings

- Provide a machine-local resource profile and show the effective RAM limits for recent history, shared batches and writer buffers. Offer sizing controls and show estimated memory use. Validate combinations against the application-managed budget, with reserved capacity for capture and writing. Apply changes at the next measurement start initially, and show both the requested and active settings while a measurement is running.
- Send recorder batches to a dedicated writer as soon as recording starts. Bound staging by bytes, with a time-based flush for sparse traffic. Track bytes buffered and bytes written separately. Document the selected format's behavior on orderly finalization, abrupt termination and partial files. OS write completion, file-system sync and a finalized recording are distinct states.
- Keep persisted recordings at the selected destination. A separate optional Trace cache spills evicted recent data to disk in the background, under its own byte quota. Removing cache segments leaves durable recordings intact. Show where data is written and why a range is unavailable.
- Verify an 8 GiB or larger recording under a smaller RAM profile. Confirm that the destination grows before Stop, accounted pools stay within their configured limits, and an independent reader can recover every expected record from the finalized output. Exercise optional rollover and verify continuity across segment boundaries.
- Inject destination quota exhaustion, a stalled or failed writer and interrupted finalization. Show the last successfully written range and the incomplete or failed state, preserve completed segments, and keep capture operating within its own limits. Validate recovery against each supported format. Inject temporary cache exhaustion separately and retain bounded RAM history with a visible diagnostic.

## Live capture storage modes (REQ-LOG-006)

Use separate controls for **Trace history** (RAM budget, optional temporary disk quota) and **File logging** (off, manual start/stop, or configured trigger with destination and recording filter). Default a new measurement setup to RAM only with file logging off. Allow multi-GiB RAM budgets. Show the effective settings before Start. An armed trigger waits for its condition before creating output. Keep pre-trigger events in bounded history and report truncation.

| Temporary disk history | File logging | Expected measurement data on disk |
|---|---|---|
| Off | Off | None. Older events expire when bounded RAM history fills. |
| On | Off | Evicted history may spill into the temporary cache. Cache cleanup and quota eviction may discard these ranges. |
| Off | On | Only the user recording and its required metadata. Recent Trace stays in RAM. |
| On | On | Cache extends Trace retention. The recorder writes its selected stream at its own destination with its own retention policy. |

Verify all four modes using isolated cache and recording directories, a fixture larger than the RAM retention limit, and an inventory of files, captured IDs and recorded IDs. With only a trigger armed, verify zero log files before firing. With both features on, filter recording to a subset of IDs and verify the Trace still contains other retained IDs. Stop the recorder during capture, clear the cache and confirm the finalized recording remains readable. Inject a failure at each destination independently and expose the affected service's status. An explicit export of available Trace history records the selected range once and leaves ongoing file logging off if it was off.
