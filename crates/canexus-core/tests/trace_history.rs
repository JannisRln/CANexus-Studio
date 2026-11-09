//! Black-box tests for bounded trace retention, query and loss provenance.

use std::mem::size_of;

use canexus_core::{
    can::{CanFormat, CanFrame, CanFrameFlags, CanId},
    event::{ChannelId, EventDirection, Observed, RawEventId},
    ingest::{BatchLoss, BoundedIngest, IngestConfig, LossReason, OverflowPolicy, SharedBatch},
    time::{ClockDomainId, EventTimestamp, TimestampOrigin, TimestampQuality, TimestampResolution},
    trace_history::{
        CanIdMask, CanTraceFilter, EventFilter, HistoryError, HistoryLimits, PageError, PageRequest,
        TraceHistory,
    },
    virtual_can::{fixtures, VirtualCanAdapter},
    SourceId,
};

fn event_size() -> usize {
    size_of::<Observed<CanFrame>>()
}

fn limits(events: usize, batches: usize, page_rows: usize) -> HistoryLimits {
    HistoryLimits::new(event_size() * events, batches, page_rows).unwrap()
}

fn can_event(sequence: u64) -> Observed<CanFrame> {
    Observed {
        id: RawEventId {
            source: SourceId::new(7),
            sequence,
        },
        channel: ChannelId::new(0),
        timestamp: EventTimestamp::new(
            sequence,
            ClockDomainId::new(2),
            TimestampResolution::new(1, 1).unwrap(),
            TimestampOrigin::Synthetic,
            TimestampQuality::Uncorrelated,
        ),
        direction: EventDirection::Rx,
        value: CanFrame::new(
            CanId::standard(0x123).unwrap(),
            CanFrameFlags::CLASSIC,
            2,
            &[0xa1, 0xb2],
        )
        .unwrap(),
    }
}

fn batch_of(events: impl IntoIterator<Item = Observed<CanFrame>>) -> SharedBatch<CanFrame> {
    let events: Vec<_> = events.into_iter().collect();
    assert!(!events.is_empty());
    let config = IngestConfig::new(events.len(), 1, 1).unwrap();
    let mut ingest = BoundedIngest::new(SourceId::new(events[0].id.source.get()), config);
    let (_id, receiver) = ingest.subscribe(OverflowPolicy::DropNewest, 1).unwrap();
    ingest.start().unwrap();
    for event in events {
        ingest.push(event).unwrap();
    }
    ingest.shutdown().unwrap();
    receiver.recv().unwrap()
}

fn sequences(page: &[&Observed<CanFrame>]) -> Vec<u64> {
    page.iter().map(|event| event.id.sequence).collect()
}

#[test]
fn budgets_reject_zeros_and_evict_oldest_complete_batches() {
    assert_eq!(HistoryLimits::new(0, 1, 1), Err(HistoryError::InvalidLimit));
    assert_eq!(HistoryLimits::new(1, 0, 1), Err(HistoryError::InvalidLimit));
    assert_eq!(HistoryLimits::new(1, 1, 0), Err(HistoryError::InvalidLimit));
    let mut history = TraceHistory::new(SourceId::new(7), limits(4, 2, 3));
    assert_eq!(history.source(), SourceId::new(7));
    assert_eq!(history.oldest(), None);
    assert_eq!(history.newest(), None);
    assert_eq!(history.version(), 0);
    assert_eq!(history.append(batch_of((0..2).map(can_event))).unwrap().evicted_events, 0);
    assert_eq!(history.append(batch_of((2..4).map(can_event))).unwrap().evicted_batches, 0);
    let admitted = history.append(batch_of((4..6).map(can_event))).unwrap();
    assert_eq!(admitted.evicted_batches, 1);
    assert_eq!(admitted.evicted_events, 2);
    assert_eq!(history.oldest().unwrap().sequence, 2);
    assert_eq!(history.newest().unwrap().sequence, 5);
    assert_eq!(history.version(), 3);
    let stats = history.stats();
    assert_eq!(stats.admitted_events, 6);
    assert_eq!(stats.retained_events, 4);
    assert_eq!(stats.retained_event_bytes, 4 * event_size());
    assert_eq!(stats.retained_batches, 2);
    assert_eq!(stats.evicted_events, 2);
    assert_eq!(stats.evicted_batches, 1);
    assert_eq!(stats.reported_input_drops, 0);
    let eviction = stats.last_eviction.unwrap();
    assert_eq!(eviction.source, SourceId::new(7));
    assert_eq!((eviction.first_sequence, eviction.last_sequence), (0, 1));
    assert_eq!((eviction.events, eviction.batches), (2, 1));
}

#[test]
fn batch_slots_also_bound_memory_when_event_budget_is_large() {
    let mut history = TraceHistory::new(SourceId::new(7), limits(100, 2, 4));
    for seq in 0..8 {
        history.append(batch_of([can_event(seq)])).unwrap();
        assert!(history.stats().retained_batches <= 2);
        assert!(history.stats().retained_event_bytes <= 100 * event_size());
    }
    assert_eq!(history.stats().retained_events, 2);
    assert_eq!(history.stats().evicted_events, 6);
    assert_eq!(history.oldest().unwrap().sequence, 6);
    assert_eq!(history.newest().unwrap().sequence, 7);
}

#[test]
fn rejected_batches_are_transactional_and_do_not_evict_existing_history() {
    let mut history = TraceHistory::new(SourceId::new(7), limits(2, 2, 2));
    history.append(batch_of([can_event(0)])).unwrap();
    let previous = history.stats();
    let version = history.version();
    assert_eq!(
        history.append(batch_of((1..4).map(can_event))),
        Err(HistoryError::BatchTooLarge)
    );
    let mut mixed = can_event(1);
    mixed.id.source = SourceId::new(8);
    assert_eq!(
        history.append(batch_of([mixed])),
        Err(HistoryError::SourceMismatch)
    );
    assert_eq!(
        history.append(batch_of([can_event(0), can_event(1)])),
        Err(HistoryError::NonMonotonicSequence { index: 0 })
    );
    assert_eq!(
        history.append(batch_of([can_event(2), can_event(1)])),
        Err(HistoryError::NonMonotonicSequence { index: 1 })
    );
    assert_eq!(history.stats(), previous);
    assert_eq!(history.version(), version);
    assert_eq!(history.oldest().unwrap().sequence, 0);
}

#[test]
fn page_size_bounds_and_stale_cursors_are_explicit() {
    let mut history = TraceHistory::new(SourceId::new(7), limits(8, 4, 2));
    history.append(batch_of((0..2).map(can_event))).unwrap();
    history.append(batch_of((2..4).map(can_event))).unwrap();
    let filter = CanTraceFilter::default();
    assert!(matches!(
        history.can_page(&filter, PageRequest { cursor: None, limit: 0 }),
        Err(PageError::PageLimitExceeded)
    ));
    assert!(matches!(
        history.can_page(&filter, PageRequest { cursor: None, limit: 3 }),
        Err(PageError::PageLimitExceeded)
    ));
    let first = history
        .can_page(&filter, PageRequest { cursor: None, limit: 2 })
        .unwrap();
    assert_eq!(sequences(&first.rows), [0, 1]);
    let cursor = first.next.unwrap();
    let second = history
        .can_page(&filter, PageRequest { cursor: Some(cursor), limit: 2 })
        .unwrap();
    assert_eq!(sequences(&second.rows), [2, 3]);
    assert_eq!(second.next, None);
    history.append(batch_of([can_event(4)])).unwrap();
    assert!(matches!(
        history.can_page(&filter, PageRequest { cursor: Some(cursor), limit: 2 }),
        Err(PageError::StaleCursor)
    ));
    let other = TraceHistory::<CanFrame>::new(SourceId::new(9), limits(4, 2, 2));
    assert!(matches!(
        other.can_page(&filter, PageRequest { cursor: Some(cursor), limit: 1 }),
        Err(PageError::StaleCursor)
    ));
}

#[test]
fn can_filter_preserves_format_mask_direction_and_capture_sequence_order() {
    let mut observations = Vec::new();
    for seq in 0..6 {
        let mut event = can_event(seq);
        event.channel = ChannelId::new(if seq == 4 { 2 } else { 1 });
        event.direction = if seq == 3 { EventDirection::Tx } else { EventDirection::Rx };
        let id = if seq == 5 {
            CanId::extended(0x123).unwrap()
        } else {
            CanId::standard(0x120 + seq as u16).unwrap()
        };
        let flags = if seq == 2 {
            CanFrameFlags::FD
        } else {
            CanFrameFlags::CLASSIC
        };
        event.value = CanFrame::new(id, flags, 2, &[seq as u8, 0x5a]).unwrap();
        observations.push(event);
    }
    let mut history = TraceHistory::new(SourceId::new(7), limits(10, 5, 2));
    history.append(batch_of(observations)).unwrap();
    assert_eq!(
        CanIdMask::new(CanId::standard(0).unwrap(), 0x800),
        None
    );
    assert_eq!(
        CanIdMask::new(CanId::extended(0).unwrap(), 0x2000_0000),
        None
    );
    let filter = CanTraceFilter {
        event: EventFilter {
            channel: Some(ChannelId::new(1)),
            direction: Some(EventDirection::Rx),
            ..EventFilter::default()
        },
        arbitration: CanIdMask::new(CanId::standard(0x120).unwrap(), 0x7f0),
        format: Some(CanFormat::Classic),
    };
    let page = history
        .can_page(&filter, PageRequest { cursor: None, limit: 2 })
        .unwrap();
    assert_eq!(sequences(&page.rows), [0, 1]);
    assert!(page.next.is_some());
    let page = history
        .can_page(
            &filter,
            PageRequest {
                cursor: page.next,
                limit: 2,
            },
        )
        .unwrap();
    assert_eq!(sequences(&page.rows), [3].into_iter().filter(|_| false).collect::<Vec<u64>>());
    // Sequence 2 is FD, 3 is TX, 4 uses another channel, 5 is extended.
    assert_eq!(history.count_matches(|event| filter.matches(event)), 2);
    let extended_mask = CanIdMask::new(CanId::extended(0x123).unwrap(), 0x1fff_ffff).unwrap();
    let extended_filter = CanTraceFilter {
        arbitration: Some(extended_mask),
        ..CanTraceFilter::default()
    };
    assert_eq!(history.count_matches(|event| extended_filter.matches(event)), 1);
    assert_eq!(
        history.count_matches(|event| EventFilter {
            first_sequence: Some(2),
            last_sequence: Some(4),
            ..EventFilter::default()
        }
        .matches(event)),
        3
    );
}

#[test]
fn history_eviction_is_not_source_or_consumer_capture_loss() {
    let mut ingest = BoundedIngest::new(
        SourceId::new(7),
        IngestConfig::new(2, 1, 1).unwrap(),
    );
    let (subscriber, receiver) = ingest.subscribe(OverflowPolicy::DropNewest, 1).unwrap();
    ingest.start().unwrap();
    for seq in 0..6 {
        ingest.push(can_event(seq)).unwrap();
    }
    ingest.shutdown().unwrap();
    let input_status = ingest.consumer_snapshot(subscriber).unwrap();
    assert_eq!(input_status.stats.dropped_events, 4);
    let batch = receiver.recv().unwrap();
    assert_eq!(batch.events()[0].id.sequence, 0);
    let mut history = TraceHistory::new(SourceId::new(7), limits(2, 1, 2));
    history.append(batch).unwrap();
    assert_eq!(history.stats().evicted_events, 0);
    let loss = input_status.stats.last_loss.unwrap();
    assert_eq!(loss.reason, LossReason::QueueFull);
    history.report_input_loss(loss).unwrap();
    let stats = history.stats();
    assert_eq!(stats.reported_input_drops, 2);
    assert_eq!(stats.last_input_loss, Some(loss));
    assert_eq!(stats.evicted_events, 0);
    assert_eq!(stats.retained_events, 2);
    assert_eq!(history.oldest().unwrap().sequence, 0);
    assert_eq!(
        history.report_input_loss(BatchLoss {
            source: SourceId::new(8),
            ..loss
        }),
        Err(HistoryError::SourceMismatch)
    );
    assert_eq!(
        history.report_input_loss(BatchLoss {
            event_count: 0,
            ..loss
        }),
        Err(HistoryError::InvalidLossReport)
    );
    assert_eq!(
        history.report_input_loss(BatchLoss {
            last_sequence: 1,
            first_sequence: 2,
            ..loss
        }),
        Err(HistoryError::InvalidLossReport)
    );
    history.append(batch_of((6..8).map(can_event))).unwrap();
    assert_eq!(history.stats().evicted_events, 2);
    assert_eq!(history.stats().reported_input_drops, 2);
    assert_eq!(history.newest().unwrap().sequence, 7);
}

#[test]
fn shared_batches_can_feed_recorder_and_history_without_frame_copy() {
    let mut ingest = BoundedIngest::new(
        SourceId::new(7),
        IngestConfig::new(2, 1, 2).unwrap(),
    );
    let (_trace_id, trace) = ingest.subscribe(OverflowPolicy::DropNewest, 1).unwrap();
    let (_recorder_id, recorder) = ingest.subscribe(OverflowPolicy::FailConsumer, 1).unwrap();
    ingest.start().unwrap();
    for seq in 0..2 {
        ingest.push(can_event(seq)).unwrap();
    }
    ingest.shutdown().unwrap();
    let history_batch = trace.recv().unwrap();
    let recorder_batch = recorder.recv().unwrap();
    assert!(history_batch.shares_storage_with(&recorder_batch));
    let mut history = TraceHistory::new(SourceId::new(7), limits(4, 2, 4));
    history.append(history_batch).unwrap();
    drop(recorder_batch);
    let page = history
        .can_page(&CanTraceFilter::default(), PageRequest { cursor: None, limit: 4 })
        .unwrap();
    assert_eq!(sequences(&page.rows), [0, 1]);
    assert_eq!(page.rows[0].value.data(), &[0xa1, 0xb2]);
}

#[test]
fn virtual_can_observations_preserve_time_payload_and_channel() {
    let fixtures = fixtures::mixed_classic_fd();
    let mut virtual_can = VirtualCanAdapter::new(
        SourceId::new(7),
        ClockDomainId::new(21),
        TimestampResolution::new(125, 2).unwrap(),
        &fixtures,
    )
    .unwrap();
    let mut ingest = BoundedIngest::new(
        SourceId::new(7),
        IngestConfig::new(2, 2, 1).unwrap(),
    );
    let (_id, receiver) = ingest.subscribe(OverflowPolicy::DropNewest, 2).unwrap();
    virtual_can.start().unwrap();
    ingest.start().unwrap();
    virtual_can.advance_to(25).unwrap();
    while let Some(observation) = virtual_can.next_due().unwrap() {
        ingest.push(observation).unwrap();
    }
    ingest.shutdown().unwrap();
    let mut history = TraceHistory::new(SourceId::new(7), limits(4, 2, 4));
    for batch in receiver.try_iter() {
        history.append(batch).unwrap();
    }
    let page = history
        .can_page(&CanTraceFilter::default(), PageRequest { cursor: None, limit: 4 })
        .unwrap();
    assert_eq!(sequences(&page.rows), [0, 1, 2, 3]);
    assert_eq!(page.rows[1].timestamp.ticks, 10);
    assert_eq!(page.rows[1].timestamp.domain, ClockDomainId::new(21));
    assert_eq!(page.rows[1].timestamp.elapsed_ns(), Some(625));
    assert_eq!(page.rows[1].value.data(), &[0xa5; 64]);
    assert_eq!(page.rows[2].direction, EventDirection::Tx);
    assert!(page.rows[2].value.flags().remote);
    assert_eq!(page.rows[3].timestamp.ticks, 25);
    assert_eq!(page.rows[3].channel, ChannelId::new(1));
}

#[test]
fn generic_non_can_observations_use_same_history_and_paging() {
    #[derive(Debug, Eq, PartialEq)]
    struct EthernetFixture([u8; 4]);
    let mut ingest = BoundedIngest::new(
        SourceId::new(7),
        IngestConfig::new(2, 1, 1).unwrap(),
    );
    let (_id, rx) = ingest.subscribe(OverflowPolicy::DropNewest, 1).unwrap();
    ingest.start().unwrap();
    for seq in 0..2 {
        let base = can_event(seq);
        ingest
            .push(Observed {
                id: base.id,
                timestamp: base.timestamp,
                channel: base.channel,
                direction: base.direction,
                value: EthernetFixture([seq as u8, 1, 2, 3]),
            })
            .unwrap();
    }
    ingest.shutdown().unwrap();
    let item_bytes = size_of::<Observed<EthernetFixture>>();
    let mut history = TraceHistory::new(
        SourceId::new(7),
        HistoryLimits::new(2 * item_bytes, 1, 1).unwrap(),
    );
    history.append(rx.recv().unwrap()).unwrap();
    assert_eq!(history.count_matches(|event| event.value.0[0] == 1), 1);
    let first = history
        .page_with(PageRequest { cursor: None, limit: 1 }, |_| true)
        .unwrap();
    assert_eq!(first.rows[0].value.0, [0, 1, 2, 3]);
    let second = history
        .page_with(PageRequest { cursor: first.next, limit: 1 }, |_| true)
        .unwrap();
    assert_eq!(second.rows[0].value.0, [1, 1, 2, 3]);
    assert_eq!(second.next, None);
}

#[test]
fn large_stream_stays_within_retained_byte_and_batch_budgets() {
    let mut ingest = BoundedIngest::new(
        SourceId::new(7),
        IngestConfig::new(32, 1, 1).unwrap(),
    );
    let (_id, rx) = ingest.subscribe(OverflowPolicy::DropNewest, 1).unwrap();
    let mut history = TraceHistory::new(SourceId::new(7), limits(128, 4, 16));
    ingest.start().unwrap();
    for seq in 0..10_000 {
        ingest.push(can_event(seq)).unwrap();
        if seq % 32 == 31 {
            history.append(rx.recv().unwrap()).unwrap();
        }
        let stats = history.stats();
        assert!(stats.retained_event_bytes <= 128 * event_size());
        assert!(stats.retained_batches <= 4);
    }
    ingest.shutdown().unwrap();
    history.append(rx.recv().unwrap()).unwrap();
    assert_eq!(history.stats().admitted_events, 10_000);
    assert_eq!(history.stats().retained_events, 112);
    assert_eq!(history.stats().evicted_events, 9_888);
    assert_eq!(history.oldest().unwrap().sequence, 9_888);
    assert_eq!(history.newest().unwrap().sequence, 9_999);
    let page = history
        .can_page(
            &CanTraceFilter::default(),
            PageRequest { cursor: None, limit: 16 },
        )
        .unwrap();
    assert_eq!(page.rows.len(), 16);
    assert!(page.next.is_some());
    assert_eq!(page.rows[0].id.sequence, 9_888);
}
