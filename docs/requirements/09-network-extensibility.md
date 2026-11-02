# Multi-Network and Protocol Extensibility

**Revision:** 0.5 · **Status:** Mixed: extension goal confirmed, implementation design proposed

## Confirmed direction

CANexus-Studio starts with CAN Classic/CAN FD, but must **not be architecturally locked to CAN**. Later additions may include Ethernet and other networks/protocols. These are extension targets, **not** commitments to ship Ethernet support in the CAN MVP.

## Proposed layering

```text
Workspace / generic event tables / timeline / plots / automation
                           |
           Typed session + event + capability services
                           |
            +--------------+--------------+
            |                             |
       CAN transport                   Ethernet transport (later)
            |                             |
      CAN frames/DBC/ISO-TP        Ethernet frames/TCP/UDP/DoIP (later)
            |                             |
      CANopen / XCP / UDS            SOME/IP, XCP on Ethernet (later)
            |                             |
      CAN adapter plugins              NIC/PCAP adapter plugins
```

Core services own time correlation, session lifecycle, event routing, backpressure and projections for UI consumers. Bus-specific types remain typed; **do not force Ethernet packets into the CAN frame structure**.

## Requirements

| ID | Priority | Status | Requirement |
|---|---|---|---|
| REQ-NET-001 | P0 | Accepted | The architecture shall allow new transport/network types, including future Ethernet, without replacing the entire core or UI framework. |
| REQ-NET-002 | P0 | Proposed | Represent captured data via typed event envelopes containing source, transport type, timestamp provenance, sequence and payload type. |
| REQ-NET-003 | P0 | Proposed | Keep CAN-specific hot-path types efficient; generic abstraction must not force costly per-frame polymorphic allocation. |
| REQ-NET-004 | P0 | Proposed | Transport plugins declare capabilities: capture, transmit, timestamp, filtering, link configuration and errors. |
| REQ-NET-005 | P1 | Proposed | Protocol plugins consume transport/session capabilities via the plugin host and do not directly open OS devices. |
| REQ-NET-006 | P1 | Proposed | Trace, plots, scripts and recording support typed events and protocol-specific columns/decoders without hardcoded CAN-only APIs. |
| REQ-NET-007 | P1 | Proposed | Independent network clock domains expose synchronization state and uncertainty; no false global ordering is implied. |
| REQ-NET-008 | P1 | Proposed | Project config expresses abstract logical networks and mapping to machine-local hardware without embedding machine-specific paths. |
| REQ-NET-009 | P1 | Proposed | Optional modules register protocol-specific inspectors, signal providers, filters, TX tools and scripting namespaces. |
| REQ-NET-010 | P1 | Proposed | Core services and UI remain usable when optional network/protocol plugins are absent. |
| REQ-NET-011 | P2 | Proposed | Add Ethernet interface discovery and PCAP-based packet capture as a separate implementation milestone. |
| REQ-NET-012 | P2 | Proposed | Enable future DoIP, SOME/IP, XCP on Ethernet and TCP/UDP packet inspection via separately scoped plugins. |
| REQ-NET-013 | P2 | Proposed | Support mixed CAN/Ethernet time views when synchronization/provenance is sufficient. |

## Architecture verification

1. Implement a synthetic non-CAN event source in tests and route its events through session lifecycle, logging and a generic event view without touching the CAN frame structure.
2. Open an Ethernet-capability-disabled build and verify existing CAN views and project files still work.
3. Confirm that protocol-specific columns are available only for matching event types; unsupported fields display as unavailable, never bogus values.
4. Compare events from two unsynchronized clock domains and verify UI indicates uncertainty.

## Open decisions

Generic event storage/index format, plugin discovery security, priority/order of network types, hardware timestamp sync strategies and future Ethernet licensing dependencies.
