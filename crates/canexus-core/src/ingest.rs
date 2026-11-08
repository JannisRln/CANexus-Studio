//! Bounded, non-blocking event batching and typed shared fan-out.
//!
//! A single producer owns staging and subscriber senders. Consumer queues have
//! finite batch capacities, and all consumers share the same immutable payload
//! allocation for each published batch. No event is sent through an unbounded
//! channel or copied once per consumer. A full queue never blocks capture.

use std::{
    num::NonZeroUsize,
    sync::{
        mpsc::{self, Receiver, SyncSender, TrySendError},
        Arc,
    },
};

use crate::{
    event::{Observed, RawEventId},
    SourceId,
};

/// Limits for a capture producer and its consumers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IngestConfig {
    batch_events: NonZeroUsize,
    max_queue_batches: NonZeroUsize,
    max_consumers: NonZeroUsize,
}

impl IngestConfig {
    /// Validates non-zero batching, queue and subscriber limits.
    pub fn new(
        batch_events: usize,
        max_queue_batches: usize,
        max_consumers: usize,
    ) -> Result<Self, IngestError> {
        Ok(Self {
            batch_events: NonZeroUsize::new(batch_events).ok_or(IngestError::InvalidLimit)?,
            max_queue_batches: NonZeroUsize::new(max_queue_batches)
                .ok_or(IngestError::InvalidLimit)?,
            max_consumers: NonZeroUsize::new(max_consumers).ok_or(IngestError::InvalidLimit)?,
        })
    }

    /// Maximum number of observations in each published batch.
    #[must_use]
    pub const fn batch_events(self) -> usize {
        self.batch_events.get()
    }

    /// Maximum allowed pending batches per consumer.
    #[must_use]
    pub const fn max_queue_batches(self) -> usize {
        self.max_queue_batches.get()
    }

    /// Maximum number of registered consumers.
    #[must_use]
    pub const fn max_consumers(self) -> usize {
        self.max_consumers.get()
    }
}

/// Capture producer state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IngestState {
    /// Consumers may be registered; no events accepted yet.
    Ready,
    /// Events may be accepted and batches published.
    Running,
    /// Final batch published and all producer senders dropped.
    Stopped,
}

/// Policy for a full consumer queue.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OverflowPolicy {
    /// Drop only the newest batch for this consumer and continue.
    DropNewest,
    /// Permanently fail this consumer; never silently resume recording.
    FailConsumer,
}

/// Observable subscriber lifecycle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConsumerState {
    /// Receives shared batches while the queue has capacity.
    Active,
    /// Queue overflow permanently stopped delivery to this consumer.
    Failed,
    /// Consumer receiver was dropped.
    Disconnected,
    /// Ingestion stopped normally, and the sender was dropped.
    Closed,
}

/// Provenance for rejected event batches.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LossReason {
    /// The bounded consumer queue was full.
    QueueFull,
    /// A previously failed consumer cannot receive new observations.
    ConsumerFailed,
    /// The consumer receiver was disconnected.
    ConsumerDisconnected,
}

/// The number and identity boundaries of an undelivered batch.
///
/// IDs delimit the rejected *observations*, not necessarily every integer
/// between them: upstream source sequences may already have gaps.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BatchLoss {
    /// Source of all observations in this batch.
    pub source: SourceId,
    /// First observed sequence rejected for this consumer.
    pub first_sequence: u64,
    /// Last observed sequence rejected for this consumer.
    pub last_sequence: u64,
    /// Number of observations actually rejected in this batch.
    pub event_count: usize,
    /// Reason this consumer did not receive them.
    pub reason: LossReason,
}

/// Subscriber-local delivery and loss counters.
///
/// Counters saturate rather than wrapping; these count queued observations,
/// not downstream work completed by the consumer.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ConsumerStats {
    /// Batches successfully queued for this consumer.
    pub queued_batches: u64,
    /// Observations successfully queued for this consumer.
    pub queued_events: u64,
    /// Batches not delivered to this consumer.
    pub dropped_batches: u64,
    /// Observations not delivered to this consumer.
    pub dropped_events: u64,
    /// Most recently rejected batch, if any.
    pub last_loss: Option<BatchLoss>,
}

/// Producer-local counters, independent of consumer-specific losses.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct IngestStats {
    /// Observations accepted into staging.
    pub accepted_events: u64,
    /// Non-empty immutable batches published to subscribers.
    pub published_batches: u64,
    /// Events published without any registered consumer.
    pub unrouted_events: u64,
}

/// A registered consumer's immutable snapshot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConsumerSnapshot {
    /// Whether the consumer remains eligible for delivery.
    pub state: ConsumerState,
    /// Queue delivery and loss counters.
    pub stats: ConsumerStats,
}

/// Identifier for one registered consumer.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ConsumerId(usize);

/// Errors from configuration, lifecycle or producer admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IngestError {
    /// One of the configured capacities was zero.
    InvalidLimit,
    /// Requested consumer queue size exceeded the configured bound.
    QueueLimitExceeded,
    /// The configured number of consumers was reached.
    ConsumerLimitReached,
    /// Consumers may only be registered before capture starts.
    NotReady,
    /// Capture has not started.
    NotRunning,
    /// Capture was stopped and cannot restart.
    Stopped,
    /// An event belongs to a different capture source.
    SourceMismatch,
}

/// Immutable, reference-counted batch shared by subscribers.
///
/// Cloning this handle clones only the `Arc`, not any observation. Consumers
/// must release batches after processing: retained handles outside queues
/// cannot be bounded by a producer's queue limits.
#[derive(Debug)]
pub struct SharedBatch<T>(Arc<[Observed<T>]>);

impl<T> Clone for SharedBatch<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<T> SharedBatch<T> {
    /// Returns the immutable event slice.
    #[must_use]
    pub fn events(&self) -> &[Observed<T>] {
        &self.0
    }

    /// Number of observations in the batch.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// True if this batch contains no observations.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Whether two handles refer to the same backing allocation.
    #[must_use]
    pub fn shares_storage_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

struct Consumer<T> {
    sender: Option<SyncSender<SharedBatch<T>>>,
    policy: OverflowPolicy,
    state: ConsumerState,
    stats: ConsumerStats,
}

/// Single-owner, bounded batch producer for any typed observation payload.
///
/// Each instance accepts observations from one `SourceId`, without assuming
/// that their raw sequence numbers are contiguous or their clocks comparable.
/// Consumers are registered before `start`. `push` and `shutdown` never
/// wait for a subscriber: full queues cause an explicit drop or failure.
/// Heap allocations happen at batch boundaries, not for each `push`.
pub struct BoundedIngest<T> {
    config: IngestConfig,
    source: SourceId,
    state: IngestState,
    staging: Vec<Observed<T>>,
    consumers: Vec<Consumer<T>>,
    stats: IngestStats,
}

impl<T> BoundedIngest<T> {
    /// Creates an idle capture source with preallocated staging capacity.
    #[must_use]
    pub fn new(source: SourceId, config: IngestConfig) -> Self {
        Self {
            config,
            source,
            state: IngestState::Ready,
            staging: Vec::with_capacity(config.batch_events()),
            consumers: Vec::new(),
            stats: IngestStats::default(),
        }
    }

    /// Returns the capture lifecycle state.
    #[must_use]
    pub const fn state(&self) -> IngestState {
        self.state
    }

    /// Returns producer counters.
    #[must_use]
    pub const fn stats(&self) -> IngestStats {
        self.stats
    }

    /// Number of events still waiting in the bounded staging buffer.
    #[must_use]
    pub fn staged_events(&self) -> usize {
        self.staging.len()
    }

    /// Returns the maximum number of queued events allowed per consumer.
    ///
    /// This excludes the producer's one staging batch, channel bookkeeping,
    /// and batches a consumer has removed from its queue and retained.
    #[must_use]
    pub fn max_queued_events(&self) -> Option<usize> {
        self.config
            .batch_events()
            .checked_mul(self.config.max_queue_batches())
    }

    /// Adds a bounded consumer before capture starts.
    ///
    /// Queue capacity must be in `1..=config.max_queue_batches()`.
    /// Receiving is independent of other consumers and of the producer.
    pub fn subscribe(
        &mut self,
        policy: OverflowPolicy,
        queue_batches: usize,
    ) -> Result<(ConsumerId, Receiver<SharedBatch<T>>), IngestError> {
        if self.state != IngestState::Ready {
            return Err(IngestError::NotReady);
        }
        if queue_batches == 0 || queue_batches > self.config.max_queue_batches() {
            return Err(IngestError::QueueLimitExceeded);
        }
        if self.consumers.len() >= self.config.max_consumers() {
            return Err(IngestError::ConsumerLimitReached);
        }
        let (sender, receiver) = mpsc::sync_channel(queue_batches);
        let id = ConsumerId(self.consumers.len());
        self.consumers.push(Consumer {
            sender: Some(sender),
            policy,
            state: ConsumerState::Active,
            stats: ConsumerStats::default(),
        });
        Ok((id, receiver))
    }

    /// Returns delivery status even after shutdown.
    #[must_use]
    pub fn consumer_snapshot(&self, id: ConsumerId) -> Option<ConsumerSnapshot> {
        self.consumers.get(id.0).map(|consumer| ConsumerSnapshot {
            state: consumer.state,
            stats: consumer.stats,
        })
    }

    /// Starts a one-shot capture session.
    pub fn start(&mut self) -> Result<(), IngestError> {
        match self.state {
            IngestState::Ready => {
                self.state = IngestState::Running;
                Ok(())
            }
            IngestState::Running => Err(IngestError::NotReady),
            IngestState::Stopped => Err(IngestError::Stopped),
        }
    }

    /// Accepts one typed observation, publishing upon a full staging batch.
    ///
    /// An invalid source or lifecycle returns an error and does not update
    /// counters. On an accepted event, individual subscriber overflow is
    /// recorded in that subscriber's snapshot rather than blocking capture.
    pub fn push(&mut self, event: Observed<T>) -> Result<(), IngestError> {
        self.require_running()?;
        if event.id.source != self.source {
            return Err(IngestError::SourceMismatch);
        }
        self.staging.push(event);
        self.stats.accepted_events = self.stats.accepted_events.saturating_add(1);
        if self.staging.len() == self.config.batch_events() {
            self.publish_staged();
        }
        Ok(())
    }

    /// Publishes a partially filled final batch without blocking consumers.
    ///
    /// Returns `true` only when a non-empty batch was published.
    pub fn flush(&mut self) -> Result<bool, IngestError> {
        self.require_running()?;
        if self.staging.is_empty() {
            return Ok(false);
        }
        self.publish_staged();
        Ok(true)
    }

    /// Flushes the tail and closes all senders, allowing receivers to drain.
    ///
    /// The final batch follows the same overflow policy as every other batch.
    /// Existing consumer snapshots remain readable after shutdown.
    pub fn shutdown(&mut self) -> Result<(), IngestError> {
        self.require_running()?;
        if !self.staging.is_empty() {
            self.publish_staged();
        }
        self.state = IngestState::Stopped;
        for consumer in &mut self.consumers {
            consumer.sender.take();
            if consumer.state == ConsumerState::Active {
                consumer.state = ConsumerState::Closed;
            }
        }
        Ok(())
    }

    fn require_running(&self) -> Result<(), IngestError> {
        match self.state {
            IngestState::Ready => Err(IngestError::NotRunning),
            IngestState::Running => Ok(()),
            IngestState::Stopped => Err(IngestError::Stopped),
        }
    }

    fn publish_staged(&mut self) {
        let events = std::mem::replace(
            &mut self.staging,
            Vec::with_capacity(self.config.batch_events()),
        );
        let count = events.len();
        debug_assert!(count > 0);
        let first = events.first().expect("nonempty batch").id;
        let last = events.last().expect("nonempty batch").id;
        let batch = SharedBatch(Arc::from(events));
        self.stats.published_batches = self.stats.published_batches.saturating_add(1);
        if self.consumers.is_empty() {
            self.stats.unrouted_events = self.stats.unrouted_events.saturating_add(count as u64);
        }

        for consumer in &mut self.consumers {
            let reason = match consumer.state {
                ConsumerState::Active => {
                    match consumer
                        .sender
                        .as_ref()
                        .expect("active consumer owns sender")
                        .try_send(batch.clone())
                    {
                        Ok(()) => {
                            consumer.stats.queued_batches =
                                consumer.stats.queued_batches.saturating_add(1);
                            consumer.stats.queued_events =
                                consumer.stats.queued_events.saturating_add(count as u64);
                            None
                        }
                        Err(TrySendError::Full(_)) => {
                            if consumer.policy == OverflowPolicy::FailConsumer {
                                consumer.state = ConsumerState::Failed;
                                consumer.sender.take();
                            }
                            Some(LossReason::QueueFull)
                        }
                        Err(TrySendError::Disconnected(_)) => {
                            consumer.state = ConsumerState::Disconnected;
                            consumer.sender.take();
                            Some(LossReason::ConsumerDisconnected)
                        }
                    }
                }
                ConsumerState::Failed => Some(LossReason::ConsumerFailed),
                ConsumerState::Disconnected => Some(LossReason::ConsumerDisconnected),
                ConsumerState::Closed => None,
            };
            if let Some(reason) = reason {
                consumer.stats.dropped_batches =
                    consumer.stats.dropped_batches.saturating_add(1);
                consumer.stats.dropped_events =
                    consumer.stats.dropped_events.saturating_add(count as u64);
                consumer.stats.last_loss = Some(BatchLoss {
                    source: first.source,
                    first_sequence: first.sequence,
                    last_sequence: last.sequence,
                    event_count: count,
                    reason,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        can::{CanFrame, CanFrameFlags, CanId},
        event::{ChannelId, EventDirection},
        time::{ClockDomainId, EventTimestamp, TimestampOrigin, TimestampQuality, TimestampResolution},
        virtual_can::{fixtures, VirtualCanAdapter},
    };

    fn config(batch: usize, queue: usize, consumers: usize) -> IngestConfig {
        IngestConfig::new(batch, queue, consumers).unwrap()
    }

    fn frame_event(sequence: u64) -> Observed<CanFrame> {
        Observed {
            id: RawEventId {
                source: SourceId::new(7),
                sequence,
            },
            channel: ChannelId::new(1),
            timestamp: EventTimestamp::new(
                sequence,
                ClockDomainId::new(3),
                TimestampResolution::new(1, 1).unwrap(),
                TimestampOrigin::Synthetic,
                TimestampQuality::Uncorrelated,
            ),
            direction: EventDirection::Rx,
            value: CanFrame::new(
                CanId::standard(0x123).unwrap(),
                CanFrameFlags::CLASSIC,
                2,
                &[0x12, 0x34],
            )
            .unwrap(),
        }
    }

    #[test]
    fn validates_budgets_and_admission() {
        assert_eq!(IngestConfig::new(0, 1, 1), Err(IngestError::InvalidLimit));
        assert_eq!(IngestConfig::new(1, 0, 1), Err(IngestError::InvalidLimit));
        assert_eq!(IngestConfig::new(1, 1, 0), Err(IngestError::InvalidLimit));
        let mut ingest = BoundedIngest::<CanFrame>::new(SourceId::new(7), config(2, 3, 1));
        assert_eq!(ingest.max_queued_events(), Some(6));
        assert_eq!(ingest.push(frame_event(0)), Err(IngestError::NotRunning));
        assert_eq!(ingest.flush(), Err(IngestError::NotRunning));
        assert_eq!(
            ingest.subscribe(OverflowPolicy::DropNewest, 0).err(),
            Some(IngestError::QueueLimitExceeded)
        );
        assert_eq!(
            ingest.subscribe(OverflowPolicy::DropNewest, 4).err(),
            Some(IngestError::QueueLimitExceeded)
        );
        let _first = ingest.subscribe(OverflowPolicy::DropNewest, 1).unwrap();
        assert_eq!(
            ingest.subscribe(OverflowPolicy::DropNewest, 1).err(),
            Some(IngestError::ConsumerLimitReached)
        );
        ingest.start().unwrap();
        assert_eq!(
            ingest.subscribe(OverflowPolicy::DropNewest, 1).err(),
            Some(IngestError::NotReady)
        );
        assert_eq!(ingest.start(), Err(IngestError::NotReady));
        let mut wrong = frame_event(0);
        wrong.id.source = SourceId::new(8);
        assert_eq!(ingest.push(wrong), Err(IngestError::SourceMismatch));
        assert_eq!(ingest.stats().accepted_events, 0);
        ingest.shutdown().unwrap();
        assert_eq!(ingest.start(), Err(IngestError::Stopped));
        assert_eq!(ingest.flush(), Err(IngestError::Stopped));
        assert_eq!(ingest.shutdown(), Err(IngestError::Stopped));
        assert_eq!(ingest.push(frame_event(0)), Err(IngestError::Stopped));
    }

    #[test]
    fn subscribers_share_batch_backing_and_preserve_typed_frames() {
        let mut ingest = BoundedIngest::new(SourceId::new(7), config(2, 2, 2));
        let (a_id, a) = ingest.subscribe(OverflowPolicy::DropNewest, 2).unwrap();
        let (b_id, b) = ingest.subscribe(OverflowPolicy::FailConsumer, 2).unwrap();
        ingest.start().unwrap();
        ingest.push(frame_event(10)).unwrap();
        assert_eq!(ingest.staged_events(), 1);
        assert!(a.try_recv().is_err());
        ingest.push(frame_event(12)).unwrap();
        let first = a.try_recv().unwrap();
        let second = b.try_recv().unwrap();
        assert!(first.shares_storage_with(&second));
        assert!(first.shares_storage_with(&first.clone()));
        assert_eq!(first.len(), 2);
        assert!(!first.is_empty());
        assert_eq!(first.events()[0].id.sequence, 10);
        assert_eq!(first.events()[1].id.sequence, 12);
        assert_eq!(first.events()[1].value.data(), &[0x12, 0x34]);
        assert_eq!(ingest.consumer_snapshot(a_id).unwrap().stats.queued_events, 2);
        assert_eq!(ingest.consumer_snapshot(b_id).unwrap().stats.queued_events, 2);
        assert_eq!(ingest.stats().published_batches, 1);
    }

    #[test]
    fn full_trace_queue_drops_only_its_own_batches() {
        let mut ingest = BoundedIngest::new(SourceId::new(7), config(2, 4, 2));
        let (trace_id, trace) = ingest.subscribe(OverflowPolicy::DropNewest, 1).unwrap();
        let (rec_id, recorder) = ingest.subscribe(OverflowPolicy::FailConsumer, 4).unwrap();
        ingest.start().unwrap();
        for sequence in 0..6 {
            ingest.push(frame_event(sequence)).unwrap();
        }
        assert_eq!(trace.try_recv().unwrap().events()[0].id.sequence, 0);
        assert!(trace.try_recv().is_err());
        for expected in [0, 2, 4] {
            assert_eq!(
                recorder.try_recv().unwrap().events()[0].id.sequence,
                expected
            );
        }
        let trace_info = ingest.consumer_snapshot(trace_id).unwrap();
        assert_eq!(trace_info.state, ConsumerState::Active);
        assert_eq!(trace_info.stats.queued_events, 2);
        assert_eq!(trace_info.stats.dropped_events, 4);
        assert_eq!(trace_info.stats.dropped_batches, 2);
        assert_eq!(
            trace_info.stats.last_loss,
            Some(BatchLoss {
                source: SourceId::new(7),
                first_sequence: 4,
                last_sequence: 5,
                event_count: 2,
                reason: LossReason::QueueFull,
            })
        );
        let recorder_info = ingest.consumer_snapshot(rec_id).unwrap();
        assert_eq!(recorder_info.state, ConsumerState::Active);
        assert_eq!(recorder_info.stats.dropped_events, 0);
        assert_eq!(recorder_info.stats.queued_events, 6);
    }

    #[test]
    fn recorder_overflow_fails_without_blocking_capture() {
        let mut ingest = BoundedIngest::new(SourceId::new(7), config(2, 4, 2));
        let (rec_id, recording) = ingest.subscribe(OverflowPolicy::FailConsumer, 1).unwrap();
        let (trace_id, trace) = ingest.subscribe(OverflowPolicy::DropNewest, 4).unwrap();
        ingest.start().unwrap();
        for sequence in 0..6 {
            ingest.push(frame_event(sequence)).unwrap();
        }
        let rec_info = ingest.consumer_snapshot(rec_id).unwrap();
        assert_eq!(rec_info.state, ConsumerState::Failed);
        assert_eq!(rec_info.stats.queued_events, 2);
        assert_eq!(rec_info.stats.dropped_events, 4);
        assert_eq!(rec_info.stats.last_loss.unwrap().reason, LossReason::ConsumerFailed);
        assert_eq!(recording.recv().unwrap().events()[0].id.sequence, 0);
        assert!(recording.recv().is_err());
        assert_eq!(ingest.consumer_snapshot(trace_id).unwrap().stats.queued_events, 6);
        assert_eq!(trace.try_iter().map(|batch| batch.len()).sum::<usize>(), 6);
        assert_eq!(ingest.stats().accepted_events, 6);
    }

    #[test]
    fn disconnection_and_shutdown_are_observable_and_nonblocking() {
        let mut ingest = BoundedIngest::new(SourceId::new(7), config(3, 1, 1));
        let (id, receiver) = ingest.subscribe(OverflowPolicy::DropNewest, 1).unwrap();
        ingest.start().unwrap();
        ingest.push(frame_event(2)).unwrap();
        ingest.push(frame_event(4)).unwrap();
        drop(receiver);
        assert!(ingest.flush().unwrap());
        assert_eq!(ingest.consumer_snapshot(id).unwrap().state, ConsumerState::Disconnected);
        assert_eq!(
            ingest.consumer_snapshot(id).unwrap().stats.last_loss.unwrap().reason,
            LossReason::ConsumerDisconnected
        );
        assert!(!ingest.flush().unwrap());
        ingest.push(frame_event(6)).unwrap();
        ingest.shutdown().unwrap();
        assert_eq!(ingest.state(), IngestState::Stopped);
        assert_eq!(ingest.stats().accepted_events, 3);
        assert_eq!(ingest.consumer_snapshot(id).unwrap().stats.dropped_events, 3);
    }

    #[test]
    fn final_partial_batch_drains_before_receiver_disconnect() {
        let mut ingest = BoundedIngest::new(SourceId::new(7), config(4, 1, 1));
        let (id, receiver) = ingest.subscribe(OverflowPolicy::DropNewest, 1).unwrap();
        ingest.start().unwrap();
        ingest.push(frame_event(1)).unwrap();
        ingest.push(frame_event(2)).unwrap();
        ingest.shutdown().unwrap();
        assert_eq!(ingest.consumer_snapshot(id).unwrap().state, ConsumerState::Closed);
        assert_eq!(receiver.recv().unwrap().len(), 2);
        assert!(receiver.recv().is_err());
        assert_eq!(ingest.staged_events(), 0);
    }

    #[test]
    fn virtual_can_fixture_replays_through_bounded_pipeline() {
        let fixtures = fixtures::mixed_classic_fd();
        let mut source = VirtualCanAdapter::new(
            SourceId::new(7),
            ClockDomainId::new(9),
            TimestampResolution::new(1, 1).unwrap(),
            &fixtures,
        )
        .unwrap();
        let mut ingest = BoundedIngest::new(SourceId::new(7), config(3, 2, 1));
        let (_id, rx) = ingest.subscribe(OverflowPolicy::DropNewest, 2).unwrap();
        ingest.start().unwrap();
        source.start().unwrap();
        source.advance_to(25).unwrap();
        while let Some(event) = source.next_due().unwrap() {
            ingest.push(event).unwrap();
        }
        ingest.shutdown().unwrap();
        let first = rx.recv().unwrap();
        let tail = rx.recv().unwrap();
        assert_eq!(first.len(), 3);
        assert_eq!(tail.len(), 1);
        assert_eq!(first.events()[1].value.data(), &[0xa5; 64]);
        assert_eq!(first.events()[2].direction, EventDirection::Tx);
        assert_eq!(tail.events()[0].timestamp.ticks, 25);
        assert!(rx.recv().is_err());
    }

    #[test]
    fn generic_non_can_events_share_the_same_capture_contract() {
        #[derive(Debug, Eq, PartialEq)]
        struct EthernetFixture {
            bytes: [u8; 3],
        }
        let mut ingest = BoundedIngest::new(SourceId::new(7), config(1, 1, 1));
        let (_id, rx) = ingest.subscribe(OverflowPolicy::DropNewest, 1).unwrap();
        ingest.start().unwrap();
        let base = frame_event(0);
        ingest
            .push(Observed {
                id: base.id,
                channel: base.channel,
                timestamp: base.timestamp,
                direction: base.direction,
                value: EthernetFixture { bytes: [1, 2, 3] },
            })
            .unwrap();
        ingest.shutdown().unwrap();
        assert_eq!(rx.recv().unwrap().events()[0].value.bytes, [1, 2, 3]);
    }

    #[test]
    fn large_burst_keeps_queue_bounded_without_waiting_for_consumer() {
        let mut ingest = BoundedIngest::new(SourceId::new(7), config(32, 1, 1));
        let (id, receiver) = ingest.subscribe(OverflowPolicy::DropNewest, 1).unwrap();
        ingest.start().unwrap();
        for sequence in 0..10_000 {
            ingest.push(frame_event(sequence)).unwrap();
            assert!(ingest.staged_events() < 32);
        }
        ingest.shutdown().unwrap();
        let snapshot = ingest.consumer_snapshot(id).unwrap();
        assert_eq!(snapshot.stats.queued_events, 32);
        assert_eq!(snapshot.stats.dropped_events, 9_968);
        assert_eq!(ingest.stats().accepted_events, 10_000);
        assert_eq!(receiver.recv().unwrap().len(), 32);
        assert!(receiver.recv().is_err());
    }

    #[test]
    fn events_without_subscribers_are_accounted_separately() {
        let mut ingest = BoundedIngest::new(SourceId::new(7), config(2, 1, 1));
        ingest.start().unwrap();
        ingest.push(frame_event(0)).unwrap();
        ingest.push(frame_event(1)).unwrap();
        ingest.shutdown().unwrap();
        assert_eq!(ingest.stats().unrouted_events, 2);
        assert_eq!(ingest.stats().published_batches, 1);
    }
}
