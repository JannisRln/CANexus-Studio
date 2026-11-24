use std::{
    sync::{mpsc, Arc},
    thread,
    time::Duration,
};

use canexus_core::{
    can::{CanFormat, CanFrame},
    event::Observed,
    ingest::{BatchBudget, BoundedIngest, IngestConfig, IngestError, LossReason},
    session::{
        CaptureBatch, CaptureSession, FlushClock, SessionConfig, SessionError, SessionPhase,
        Subscription,
    },
    time::{ClockDomainId, TimestampResolution},
    virtual_can::{fixtures, VirtualCanAdapter, VirtualCanFixture},
    SourceId,
};

const FIRST: SourceId = SourceId::new(4);
const SECOND: SourceId = SourceId::new(5);
const WAIT: Duration = Duration::from_secs(3);
const FLUSH: Duration = Duration::from_millis(40);

fn config(batch_events: usize) -> SessionConfig {
    SessionConfig {
        ingest: IngestConfig::new(batch_events, 8, 3).unwrap(),
        flush_after: FLUSH,
        stop_grace: Duration::from_millis(30),
        clock: FlushClock::Manual,
    }
}

fn budget(config: SessionConfig, batches: usize) -> BatchBudget {
    BatchBudget::new(BoundedIngest::<CanFrame>::batch_bytes(config.ingest).unwrap() * batches)
}

fn adapter(source: SourceId, script: &[VirtualCanFixture]) -> VirtualCanAdapter<'_> {
    let mut adapter = VirtualCanAdapter::new(
        source,
        ClockDomainId::new(source.get()),
        TimestampResolution::new(1_000, 1).unwrap(),
        script,
    )
    .unwrap();
    adapter.start().unwrap();
    adapter
}

fn receive(sub: &Subscription) -> CaptureBatch {
    sub.recv_timeout(WAIT).unwrap()
}

fn assert_batch(delivery: &CaptureBatch, source: SourceId, sequences: &[u64]) {
    assert!(delivery
        .batch
        .events()
        .iter()
        .all(|event| event.id.source == source));
    let actual: Vec<_> = delivery
        .batch
        .events()
        .iter()
        .map(|e| e.id.sequence)
        .collect();
    assert_eq!(actual, sequences);
}

fn assert_empty(sub: &Subscription, error: mpsc::RecvTimeoutError) {
    assert_eq!(sub.recv_timeout(Duration::ZERO).unwrap_err(), error);
}

#[test]
fn running_service_publishes_one_fd_frame_without_follow_up_input() {
    let config = SessionConfig {
        clock: FlushClock::RealTime,
        ..config(8)
    };
    let session = CaptureSession::start(config, budget(config, 1), &[FIRST]).unwrap();
    let trace = session.subscribe(FIRST, 1).unwrap();
    let script = fixtures::mixed_classic_fd();
    let mut adapter = adapter(FIRST, &script[1..2]);
    let input = session.input();

    assert!(input
        .poll_virtual(input.generation(), &mut adapter, 10)
        .unwrap());
    let delivery = receive(&trace);
    assert_eq!(delivery.generation, input.generation());
    assert_batch(&delivery, FIRST, &[0]);
    assert_eq!(
        delivery.batch.events()[0].value.flags().format,
        CanFormat::Fd
    );
    assert_eq!(delivery.batch.events()[0].value, script[1].frame);
    assert_eq!(session.stop().sources[0].ingest.published_batches, 1);
    assert_empty(&trace, mpsc::RecvTimeoutError::Disconnected);
}

#[test]
fn manual_clock_flushes_from_the_first_frame_and_not_before_its_deadline() {
    let config = config(8);
    let session = CaptureSession::start(config, budget(config, 1), &[FIRST]).unwrap();
    let trace = session.subscribe(FIRST, 1).unwrap();
    let script = fixtures::mixed_classic_fd();
    let mut adapter = adapter(FIRST, &script);
    let input = session.input();

    assert!(input
        .poll_virtual(input.generation(), &mut adapter, 0)
        .unwrap());
    session
        .advance_clock(FLUSH - Duration::from_nanos(1))
        .unwrap();
    assert!(input
        .poll_virtual(input.generation(), &mut adapter, 10)
        .unwrap());
    assert_empty(&trace, mpsc::RecvTimeoutError::Timeout);
    assert_eq!(session.status().sources[0].ingest.published_batches, 0);

    session.advance_clock(FLUSH).unwrap();
    assert_batch(&receive(&trace), FIRST, &[0, 1]);
    assert_empty(&trace, mpsc::RecvTimeoutError::Timeout);
    assert_eq!(session.stop().sources[0].ingest.accepted_events, 2);
}

#[test]
fn timer_and_stop_share_one_publication_and_completion_at_the_deadline() {
    let config = SessionConfig {
        stop_grace: Duration::ZERO,
        ..config(4)
    };
    for (advance_to_deadline, timer_first) in [(true, true), (true, false), (false, false)] {
        let session = CaptureSession::start(config, budget(config, 1), &[FIRST]).unwrap();
        let responsive = session.subscribe(FIRST, 1).unwrap();
        let blocked = session.subscribe(FIRST, 1).unwrap();
        let script = fixtures::mixed_classic_fd();
        let mut adapter = adapter(FIRST, &script);
        let input = session.input();
        assert!(input
            .poll_virtual(input.generation(), &mut adapter, 0)
            .unwrap());

        // Wake the timer at the exact deadline. In one order the receiver
        // confirms publication first; in the other stop can win the lock.
        if advance_to_deadline {
            session.advance_clock(FLUSH).unwrap();
        }
        let early = timer_first.then(|| receive(&responsive));
        let done = session.stop();
        assert_eq!(done.phase, SessionPhase::Stopped);
        assert_eq!(done.sources[0].ingest.accepted_events, 1);
        assert_eq!(done.sources[0].ingest.published_batches, 1);
        assert!(done.incomplete_delivery);
        assert_eq!(done.subscriptions[1].discarded_events, 1);
        assert_eq!(
            done.subscriptions[1].delivery.last_loss.unwrap().reason,
            LossReason::StopTimeout
        );

        if let Some(delivery) = early {
            assert_batch(&delivery, FIRST, &[0]);
            assert_eq!(done.subscriptions[0].received_events, 1);
        } else {
            assert_eq!(done.subscriptions[0].discarded_events, 1);
        }
        assert_empty(&responsive, mpsc::RecvTimeoutError::Disconnected);
        assert_empty(&blocked, mpsc::RecvTimeoutError::Disconnected);
        assert_eq!(
            input.poll_virtual(input.generation(), &mut adapter, 10),
            Err(SessionError::Stopped)
        );
        assert!(Arc::ptr_eq(&done, &session.stop()));
    }
}

#[test]
fn joining_and_leaving_close_the_partial_batch_at_the_documented_boundary() {
    let config = config(8);
    let session = CaptureSession::start(config, budget(config, 4), &[FIRST]).unwrap();
    let old = session.subscribe(FIRST, 2).unwrap();
    let script = fixtures::mixed_classic_fd();
    let mut adapter = adapter(FIRST, &script);
    let input = session.input();

    assert!(input
        .poll_virtual(input.generation(), &mut adapter, 0)
        .unwrap());
    let new = session.subscribe(FIRST, 2).unwrap();
    assert_batch(&receive(&old), FIRST, &[0]);
    assert_empty(&new, mpsc::RecvTimeoutError::Timeout);

    assert!(input
        .poll_virtual(input.generation(), &mut adapter, 10)
        .unwrap());
    let detached = old.unsubscribe().unwrap();
    assert_eq!(detached.received_events, 1);
    assert_eq!(detached.discarded_events, 0);
    assert_empty(&old, mpsc::RecvTimeoutError::Disconnected);
    assert_batch(&receive(&new), FIRST, &[1]);
    assert_eq!(session.stop().sources[0].ingest.published_batches, 2);
}

#[test]
fn repeated_queue_overflow_is_local_and_reports_only_known_sequence_bounds() {
    let config = config(2);
    let session = CaptureSession::start(config, budget(config, 3), &[FIRST]).unwrap();
    let blocked = session.subscribe(FIRST, 1).unwrap();
    let responsive = session.subscribe(FIRST, 1).unwrap();
    let script = fixtures::mixed_classic_fd();
    let mut adapter = adapter(FIRST, &script);
    adapter.advance_to(25).unwrap();
    let events: Vec<_> = (0..4)
        .map(|_| adapter.next_due().unwrap().unwrap())
        .collect();
    let input = session.input();

    for sequences in [[0, 1], [3, 8], [12, 16]] {
        for (event, sequence) in events[..2].iter().zip(sequences) {
            let mut event = *event;
            event.id.sequence = sequence;
            input.submit(input.generation(), event).unwrap();
        }
        assert_batch(&receive(&responsive), FIRST, &sequences);
    }
    let status = session.status();
    let loss = blocked.status().unwrap();
    assert_eq!(loss.pending_events, 2);
    assert_eq!(loss.overflow_events, 4);
    assert_eq!(loss.delivery.dropped_batches, 2);
    assert_eq!(loss.delivery.dropped_events, 4);
    let latest = loss.delivery.last_loss.unwrap();
    assert_eq!(
        (
            latest.first_sequence,
            latest.last_sequence,
            latest.event_count
        ),
        (12, 16, 2)
    );
    assert_eq!(latest.reason, LossReason::QueueFull);
    assert_eq!(status.subscriptions[1].delivery.dropped_events, 0);
    assert_eq!(status.subscriptions[1].received_events, 6);
    assert_eq!(status.sources[0].budget_rejections, 0);
    assert_batch(&receive(&blocked), FIRST, &[0, 1]);
    assert!(session.stop().incomplete_delivery);
}

#[test]
fn reconnect_rejects_old_results_and_keeps_old_deadlines_generation_local() {
    let config = SessionConfig {
        stop_grace: Duration::ZERO,
        ..config(8)
    };
    let budget = budget(config, 3);
    let script = fixtures::mixed_classic_fd();
    let old = CaptureSession::start(config, budget.clone(), &[FIRST]).unwrap();
    let old_trace = old.subscribe(FIRST, 1).unwrap();
    let old_input = old.input();
    let mut previous_adapter = adapter(FIRST, &script[..1]);
    let stale: Observed<CanFrame> = previous_adapter.next_due().unwrap().unwrap();
    old_input.submit(old_input.generation(), stale).unwrap();
    let old_done = old.stop();

    let new = CaptureSession::start(config, budget, &[FIRST]).unwrap();
    let new_input = new.input();
    let current = new.subscribe(FIRST, 1).unwrap();
    let mut current_adapter = adapter(FIRST, &script[1..2]);
    assert!(new_input
        .poll_virtual(new_input.generation(), &mut current_adapter, 10)
        .unwrap());
    assert_ne!(new_input.generation(), old_input.generation());
    assert_eq!(
        new_input.submit(old_input.generation(), stale),
        Err(SessionError::StaleGeneration)
    );
    assert_eq!(
        old_input.submit(old_input.generation(), stale),
        Err(SessionError::Stopped)
    );
    assert_eq!(
        new_input.poll_virtual(old_input.generation(), &mut previous_adapter, 25),
        Err(SessionError::StaleGeneration)
    );
    assert_eq!(previous_adapter.now_ticks(), 0);
    assert_eq!(old.advance_clock(FLUSH), Err(SessionError::Stopped));
    assert_eq!(new.status().sources[0].ingest.accepted_events, 1);
    assert_empty(&current, mpsc::RecvTimeoutError::Timeout);
    assert_empty(&old_trace, mpsc::RecvTimeoutError::Disconnected);

    new.advance_clock(FLUSH).unwrap();
    let delivery = receive(&current);
    assert_batch(&delivery, FIRST, &[stale.id.sequence]);
    assert_eq!(delivery.batch.events()[0].value, script[1].frame);
    assert_eq!(delivery.generation, new_input.generation());
    assert_eq!(new.stop().sources[0].ingest.published_batches, 1);
    assert_eq!(old.status(), *old_done);
}

#[test]
fn shared_budget_accounts_for_two_sources_and_recovers_after_the_last_reference() {
    let config = config(2);
    let single_batch = BoundedIngest::<CanFrame>::batch_bytes(config.ingest).unwrap();
    let budget = budget(config, 2);
    let session = CaptureSession::start(config, budget.clone(), &[FIRST, SECOND]).unwrap();
    let trace = session.subscribe(FIRST, 2).unwrap();
    let peer = session.subscribe(FIRST, 2).unwrap();
    let other = session.subscribe(SECOND, 2).unwrap();
    let script = fixtures::mixed_classic_fd();
    let mut first = adapter(FIRST, &script);
    let mut second = adapter(SECOND, &script);
    let input = session.input();

    assert!(input
        .poll_virtual(input.generation(), &mut first, 10)
        .unwrap());
    assert!(input
        .poll_virtual(input.generation(), &mut first, 10)
        .unwrap());
    let pinned = receive(&trace);
    let other_owner = receive(&peer);
    assert!(pinned.batch.shares_storage_with(&other_owner.batch));
    assert_eq!(budget.live_bytes(), single_batch);
    drop(other_owner);

    assert!(input
        .poll_virtual(input.generation(), &mut second, 0)
        .unwrap());
    assert_eq!(budget.live_bytes(), budget.limit_bytes());
    for _ in 0..2 {
        assert_eq!(
            input.poll_virtual(input.generation(), &mut first, 25),
            Err(SessionError::Ingest(IngestError::BatchBudgetExhausted))
        );
    }
    let status = session.status();
    assert_eq!(status.sources[0].budget_rejections, 2);
    assert_eq!(status.sources[0].ingest.rejected_events, 2);
    assert_eq!(
        status.sources[0]
            .ingest
            .last_rejection
            .unwrap()
            .first_sequence,
        3
    );
    assert!(status
        .subscriptions
        .iter()
        .all(|entry| entry.delivery.dropped_events == 0));

    drop(pinned);
    assert_eq!(budget.live_bytes(), single_batch);
    let mut recovering = script[3];
    recovering.at_tick = 30;
    // Asynchronous observations retain the source sequence supplied by the adapter.
    let mut recovery = adapter(FIRST, std::slice::from_ref(&recovering));
    recovery.advance_to(30).unwrap();
    let mut event = recovery.next_due().unwrap().unwrap();
    event.id.sequence = 4;
    input.submit(input.generation(), event).unwrap();
    session.advance_clock(FLUSH).unwrap();
    assert_batch(&receive(&trace), FIRST, &[4]);
    assert_batch(&receive(&peer), FIRST, &[4]);
    assert_batch(&receive(&other), SECOND, &[0]);
    assert_eq!(budget.live_bytes(), 0);
    assert_eq!(budget.peak_bytes(), budget.limit_bytes());
    assert!(!session.stop().incomplete_delivery);
}

#[test]
fn stop_is_bounded_with_a_full_queue_and_a_pinned_external_batch() {
    let config = config(2);
    let budget = budget(config, 3);
    let session = Arc::new(CaptureSession::start(config, budget.clone(), &[FIRST]).unwrap());
    let blocked = session.subscribe(FIRST, 1).unwrap();
    let script = fixtures::mixed_classic_fd();
    let mut adapter = adapter(FIRST, &script);
    let input = session.input();
    for _ in 0..2 {
        assert!(input
            .poll_virtual(input.generation(), &mut adapter, 10)
            .unwrap());
    }
    let pinned = receive(&blocked);
    for _ in 0..2 {
        assert!(input
            .poll_virtual(input.generation(), &mut adapter, 25)
            .unwrap());
    }
    let mut tail = pinned.batch.events()[0];
    tail.id.sequence = 4;
    input.submit(input.generation(), tail).unwrap();

    let (completed, result) = mpsc::sync_channel(1);
    let stopping = Arc::clone(&session);
    let worker = thread::spawn(move || completed.send(stopping.stop()).unwrap());
    let done = result
        .recv_timeout(WAIT)
        .expect("stop must finish while receiver is idle");
    worker.join().unwrap();
    assert_eq!(done.sources[0].ingest.accepted_events, 5);
    assert_eq!(done.sources[0].ingest.published_batches, 3);
    assert!(done.incomplete_delivery);
    let status = done.subscriptions[0];
    assert_eq!(status.received_events, 2);
    assert_eq!(status.overflow_events, 1);
    assert_eq!(status.discarded_events, 2);
    assert_eq!(status.delivery.dropped_events, 3);
    assert_eq!(status.pending_events, 0);
    assert_eq!(budget.live_bytes(), budget.limit_bytes() / 3);
    assert_empty(&blocked, mpsc::RecvTimeoutError::Disconnected);
    drop(pinned);
    assert_eq!(budget.live_bytes(), 0);
    assert!(Arc::ptr_eq(&done, &session.stop()));
}

#[test]
fn stop_allows_a_waiting_receiver_to_drain_the_tail() {
    let config = SessionConfig {
        stop_grace: WAIT,
        ..config(8)
    };
    let session = CaptureSession::start(config, budget(config, 1), &[FIRST]).unwrap();
    let trace = session.subscribe(FIRST, 1).unwrap();
    let script = fixtures::mixed_classic_fd();
    let mut adapter = adapter(FIRST, &script);
    let input = session.input();
    assert!(input
        .poll_virtual(input.generation(), &mut adapter, 0)
        .unwrap());

    let (ready, listening) = mpsc::sync_channel(1);
    thread::scope(|scope| {
        let reader = scope.spawn(move || {
            ready.send(()).unwrap();
            let delivery = receive(&trace);
            assert_empty(&trace, mpsc::RecvTimeoutError::Disconnected);
            delivery
        });
        listening.recv_timeout(WAIT).unwrap();
        let done = session.stop();
        assert_batch(&reader.join().unwrap(), FIRST, &[0]);
        assert!(!done.incomplete_delivery);
        assert_eq!(done.subscriptions[0].received_events, 1);
        assert_eq!(done.sources[0].ingest.published_batches, 1);
    });
}

#[cfg(target_pointer_width = "64")]
#[test]
fn multi_gib_budget_is_representable_without_allocating_it() {
    let config = config(8);
    let bytes = 8_usize * 1024 * 1024 * 1024;
    let budget = BatchBudget::new(bytes);
    let session = CaptureSession::start(config, budget.clone(), &[FIRST, SECOND]).unwrap();
    assert_eq!(budget.limit_bytes(), bytes);
    assert_eq!(budget.live_bytes(), 0);
    assert_eq!(budget.peak_bytes(), 0);
    assert_eq!(session.stop().sources.len(), 2);
}
