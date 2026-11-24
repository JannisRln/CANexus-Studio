//! RAM-only, headless capture sessions with source-local typed batches.
//!
//! The session owns staging, publication and the flush worker. One lock orders
//! frame admission, timer expiry, subscription changes and stop. An observation
//! is accepted when ingestion stages it. The first staged observation starts
//! the flush interval. A new subscription starts at the next batch boundary;
//! detaching excludes that subscription from the current partial batch.
//!
//! Each start creates a fresh process-local generation, including on reconnect
//! with the same source IDs. Preserve it with asynchronous observations and
//! use it together with a batch's raw event IDs. The timer is scoped to its
//! original session. Stop fences admission, publishes each accepted tail once
//! and gives receivers a bounded interval to take queued batches.

use std::{
    collections::{HashMap, VecDeque},
    io,
    sync::{
        atomic::{AtomicU64, Ordering::Relaxed},
        mpsc::{Receiver, RecvTimeoutError},
        Arc, Condvar, Mutex, OnceLock, Weak,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use crate::{
    can::CanFrame,
    event::Observed,
    ingest::{
        BatchBudget, BatchLoss, BoundedIngest, ConsumerStats, IngestConfig, IngestError,
        IngestStats, LossReason, OverflowPolicy, SharedBatch,
    },
    virtual_can::{VirtualCanAdapter, VirtualCanError},
    SourceId,
};

static NEXT_GENERATION: AtomicU64 = AtomicU64::new(1);

/// Lifetime identity for observations, results and subscriptions.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Generation(u64);

/// Chooses the monotonic clock used for flush deadlines.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FlushClock {
    /// Monotonic host time. The worker waits for deadlines independently of input.
    RealTime,
    /// Elapsed time advanced by the owner through [CaptureSession::advance_clock].
    Manual,
}

/// Limits and scheduling policy for all sources in a session.
#[derive(Clone, Copy, Debug)]
pub struct SessionConfig {
    /// Batch size and per-source subscriber and queue limits.
    pub ingest: IngestConfig,
    /// Maximum staging interval after the first accepted observation.
    pub flush_after: Duration,
    /// Maximum receiver drain wait in monotonic host time, including in manual mode.
    pub stop_grace: Duration,
    /// Clock used to schedule partial publication.
    pub clock: FlushClock,
}

/// Failure to construct or start a capture session.
#[derive(Debug)]
pub enum StartError {
    /// At least one source is required, and each source must be unique.
    InvalidSources,
    /// A producer cannot be configured with the requested limits.
    Ingest(IngestError),
    /// No more unique generations are available in this process.
    GenerationExhausted,
    /// The operating system could not create the timer worker.
    Worker(io::Error),
}

/// A rejected session operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionError {
    /// The operation carries a different session generation.
    StaleGeneration,
    /// The session no longer accepts input or subscriptions.
    Stopped,
    /// The source is not configured in this session.
    UnknownSource,
    /// The ingest limits prevented admission or subscription.
    Ingest(IngestError),
    /// The virtual adapter rejected the requested operation.
    VirtualCan(VirtualCanError),
    /// The clock cannot be advanced manually or the requested time decreased.
    InvalidClockAdvance,
}

/// Lifecycle exposed independently of subscriber queues.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionPhase {
    /// Input and subscription changes are allowed.
    Running,
    /// Input is fenced and subscribers may still take queued batches.
    Draining,
    /// Final outcome is available and all receiver queues are empty.
    Stopped,
}

/// Identity of a subscription within a generation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SubscriptionId(u64);

/// A delivery with the generation required to interpret its raw observation IDs.
#[derive(Debug)]
pub struct CaptureBatch {
    /// Session that published the observations.
    pub generation: Generation,
    /// Immutable observations shared by every receiver of the batch.
    pub batch: SharedBatch<CanFrame>,
}

/// Producer health, separately from per-receiver delivery.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceStatus {
    /// Source of the observations.
    pub source: SourceId,
    /// Admission, publication and latest rejection. Unrouted counts refer to
    /// events published with no attached receiver.
    pub ingest: IngestStats,
    /// Subset of producer rejections caused by shared-budget exhaustion.
    pub budget_rejections: u64,
    /// Queue overflow losses for receivers detached before stop fenced input.
    pub detached_overflow_events: u64,
    /// Queued observations discarded when those receivers detached.
    pub detached_discarded_events: u64,
}

/// Out-of-band health for one receiver.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SubscriptionStatus {
    /// Subscription within the snapshot's generation.
    pub id: SubscriptionId,
    /// Source of every observation in this queue.
    pub source: SourceId,
    /// Cumulative enqueue and loss counters, including discarded queued batches.
    pub delivery: ConsumerStats,
    /// Observations dropped because this receiver's queue was full.
    pub overflow_events: u64,
    /// Observations discarded from the queue on stop or detach.
    pub discarded_events: u64,
    /// Observations taken by the receiver, regardless of downstream processing.
    pub received_events: u64,
    /// Observations still waiting for the receiver.
    pub pending_events: u64,
}

/// Session snapshot, including a frozen copy at the end of stop.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionStatus {
    /// Lifetime identity for all source and receiver information.
    pub generation: Generation,
    /// Current lifecycle.
    pub phase: SessionPhase,
    /// Cumulative producer health by source.
    pub sources: Vec<SourceStatus>,
    /// Health of current subscriptions, or those present when stop fenced input.
    pub subscriptions: Vec<SubscriptionStatus>,
    /// At least one subscribed delivery was lost or discarded in this session.
    /// Source rejections and unwatched data have their own producer counters.
    pub incomplete_delivery: bool,
}

/// Owns one capture lifetime and its timer worker. Dropping it stops capture.
pub struct CaptureSession {
    shared: Arc<Shared>,
    worker: Option<JoinHandle<()>>,
}

impl CaptureSession {
    /// Starts a RAM-only session. All sources charge the supplied budget,
    /// which callers may also share with other sessions.
    pub fn start(
        config: SessionConfig,
        budget: BatchBudget,
        sources: &[SourceId],
    ) -> Result<Self, StartError> {
        if sources.is_empty() {
            return Err(StartError::InvalidSources);
        }
        let mut producers = HashMap::new();
        for &source in sources {
            if producers.contains_key(&source) {
                return Err(StartError::InvalidSources);
            }
            producers.insert(
                source,
                Producer::new(source, config.ingest, budget.clone())?,
            );
        }
        let generation = NEXT_GENERATION
            .try_update(Relaxed, Relaxed, |next| next.checked_add(1))
            .map(Generation)
            .map_err(|_| StartError::GenerationExhausted)?;
        let shared = Arc::new(Shared {
            generation,
            config,
            started: Instant::now(),
            changed: Condvar::new(),
            state: Mutex::new(State {
                producers,
                subscribers: HashMap::new(),
                next_subscription: 0,
                manual_time: Duration::ZERO,
                draining_since: None,
                completed: None,
                incomplete_delivery: false,
            }),
        });
        let timer = Arc::clone(&shared);
        let worker = thread::Builder::new()
            .name("canexus-capture-flush".into())
            .spawn(move || timer.run_timer())
            .map_err(StartError::Worker)?;
        Ok(Self {
            shared,
            worker: Some(worker),
        })
    }

    /// Creates an input handle scoped to this generation.
    #[must_use]
    pub fn input(&self) -> CaptureInput {
        CaptureInput {
            generation: self.shared.generation,
            shared: Arc::downgrade(&self.shared),
        }
    }

    /// Registers a receiver for future batches of one source. Any partial
    /// batch is published to existing receivers before the new receiver joins.
    pub fn subscribe(
        &self,
        source: SourceId,
        queue_batches: usize,
    ) -> Result<Subscription, SessionError> {
        let mut state = self.shared.state.lock().expect("capture state");
        state.require_running()?;
        let limits = self.shared.config.ingest;
        if queue_batches == 0 || queue_batches > limits.max_queue_batches() {
            return Err(SessionError::Ingest(IngestError::QueueLimitExceeded));
        }
        if !state.producers.contains_key(&source) {
            return Err(SessionError::UnknownSource);
        }
        let subscribed = state
            .subscribers
            .values()
            .filter(|entry| entry.status.source == source)
            .count();
        if subscribed >= limits.max_consumers() {
            return Err(SessionError::Ingest(IngestError::ConsumerLimitReached));
        }
        let id = SubscriptionId(state.next_subscription);
        state.next_subscription = state
            .next_subscription
            .checked_add(1)
            .ok_or(SessionError::Ingest(IngestError::ConsumerLimitReached))?;
        state.flush(source);
        state
            .subscribers
            .insert(id, Subscriber::new(id, source, queue_batches));
        self.shared.changed.notify_all();
        Ok(Subscription {
            id,
            shared: Arc::clone(&self.shared),
            detached: OnceLock::new(),
        })
    }

    /// Advances the manual flush clock. The running timer worker handles
    /// outstanding deadlines, including when no input follows.
    pub fn advance_clock(&self, elapsed: Duration) -> Result<(), SessionError> {
        let mut state = self.shared.state.lock().expect("capture state");
        state.require_running()?;
        if self.shared.config.clock != FlushClock::Manual || elapsed < state.manual_time {
            return Err(SessionError::InvalidClockAdvance);
        }
        state.manual_time = elapsed;
        self.shared.changed.notify_all();
        Ok(())
    }

    /// Reads lifecycle and counters even when every data queue is full.
    #[must_use]
    pub fn status(&self) -> SessionStatus {
        let mut state = self.shared.state.lock().expect("capture state");
        self.shared.complete_if_ready(&mut state).map_or_else(
            || state.snapshot(self.shared.generation),
            |completed| (*completed).clone(),
        )
    }

    /// Fences input, publishes accepted tails once and waits up to the drain
    /// grace for receivers to take queued items. At expiry, remaining items
    /// are discarded and counted. An already checked-out batch never delays
    /// stop and retains its budget charge until its final owner drops it.
    /// Calls share one drain deadline and return the same completion snapshot.
    pub fn stop(&self) -> Arc<SessionStatus> {
        let mut state = self.shared.state.lock().expect("capture state");
        if state.draining_since.is_none() {
            state.draining_since = Some(Instant::now());
            state.shutdown();
            self.shared.changed.notify_all();
        }
        loop {
            if let Some(completed) = self.shared.complete_if_ready(&mut state) {
                return completed;
            }
            let remaining = self.shared.drain_remaining(&state).expect("draining");
            state = self.shared.wait(state, remaining);
        }
    }
}

impl Drop for CaptureSession {
    fn drop(&mut self) {
        self.stop();
        if let Some(worker) = self.worker.take() {
            worker.join().expect("capture timer worker");
        }
    }
}

/// Cloneable, generation-scoped entry point for virtual adapter input.
#[derive(Clone)]
pub struct CaptureInput {
    generation: Generation,
    shared: Weak<Shared>,
}

impl CaptureInput {
    /// Captures the identity to attach to asynchronous results.
    #[must_use]
    pub const fn generation(&self) -> Generation {
        self.generation
    }

    /// Accepts one result scoped at the time the work was scheduled.
    /// Stale results are rejected before any producer state changes.
    pub fn submit(
        &self,
        generation: Generation,
        event: Observed<CanFrame>,
    ) -> Result<(), SessionError> {
        let shared = self.resolve(generation)?;
        let mut state = shared.state.lock().expect("capture state");
        state.require_running()?;
        shared.admit(&mut state, event)
    }

    /// Advances the adapter's synthetic clock and admits at most one due
    /// observation, as an ordered operation against stop. The caller owns and
    /// starts the adapter, preserves its original session generation and
    /// should poll until this returns `Ok(false)`.
    /// An admission error consumes that observation and is reported in health.
    pub fn poll_virtual(
        &self,
        generation: Generation,
        adapter: &mut VirtualCanAdapter<'_>,
        ticks: u64,
    ) -> Result<bool, SessionError> {
        let shared = self.resolve(generation)?;
        let mut state = shared.state.lock().expect("capture state");
        state.require_running()?;
        if !state.producers.contains_key(&adapter.source()) {
            return Err(SessionError::UnknownSource);
        }
        adapter
            .advance_to(ticks)
            .map_err(SessionError::VirtualCan)?;
        let Some(event) = adapter.next_due().map_err(SessionError::VirtualCan)? else {
            return Ok(false);
        };
        shared.admit(&mut state, event)?;
        Ok(true)
    }

    fn resolve(&self, generation: Generation) -> Result<Arc<Shared>, SessionError> {
        if generation != self.generation {
            return Err(SessionError::StaleGeneration);
        }
        self.shared.upgrade().ok_or(SessionError::Stopped)
    }
}

/// A bounded receiver. Explicit detachment or drop releases its queue.
pub struct Subscription {
    id: SubscriptionId,
    shared: Arc<Shared>,
    detached: OnceLock<SubscriptionStatus>,
}

impl Subscription {
    /// Reads receiver counters independently of queue capacity. After detach,
    /// returns the final counters for as long as this handle is retained.
    #[must_use]
    pub fn status(&self) -> Option<SubscriptionStatus> {
        let state = self.shared.state.lock().expect("capture state");
        state
            .subscribers
            .get(&self.id)
            .filter(|entry| entry.attached)
            .map(|entry| entry.status)
            .or_else(|| self.detached.get().copied())
    }

    /// Takes one delivery, waiting at most the supplied duration. An empty
    /// queue is disconnected after detach or when the session fences input.
    pub fn recv_timeout(&self, timeout: Duration) -> Result<CaptureBatch, RecvTimeoutError> {
        let began = Instant::now();
        let mut state = self.shared.state.lock().expect("capture state");
        loop {
            if self.shared.complete_if_ready(&mut state).is_some() {
                return Err(RecvTimeoutError::Disconnected);
            }
            let running = state.draining_since.is_none();
            let sub = state
                .subscribers
                .get_mut(&self.id)
                .ok_or(RecvTimeoutError::Disconnected)?;
            if let Some(batch) = sub.queue.pop_front() {
                let count = batch.len() as u64;
                sub.status.pending_events -= count;
                sub.status.received_events = sub.status.received_events.saturating_add(count);
                if !running {
                    self.shared.changed.notify_all();
                }
                return Ok(CaptureBatch {
                    generation: self.shared.generation,
                    batch,
                });
            }
            if !running {
                return Err(RecvTimeoutError::Disconnected);
            }
            let remaining = timeout.saturating_sub(began.elapsed());
            if remaining.is_zero() {
                return Err(RecvTimeoutError::Timeout);
            }
            state = self.shared.wait(state, remaining);
        }
    }

    /// Removes this receiver before the source's current partial batch is
    /// published to remaining receivers. Discards queued deliveries and
    /// returns the final receiver counters. Previously taken batches survive.
    pub fn unsubscribe(&self) -> Option<SubscriptionStatus> {
        let mut state = self.shared.state.lock().expect("capture state");
        self.shared.complete_if_ready(&mut state);
        let sub = state.subscribers.get_mut(&self.id)?;
        if !sub.attached {
            return None;
        }
        sub.attached = false;
        let discarded = sub.discard(LossReason::ConsumerDetached);
        let status = sub.status;
        self.detached.set(status).expect("first detach");
        state.incomplete_delivery |= discarded;
        if state.draining_since.is_none() {
            state.subscribers.remove(&self.id);
            let producer = state
                .producers
                .get_mut(&status.source)
                .expect("configured source");
            producer.detached_overflow_events = producer
                .detached_overflow_events
                .saturating_add(status.overflow_events);
            producer.detached_discarded_events = producer
                .detached_discarded_events
                .saturating_add(status.discarded_events);
            state.flush(status.source);
        }
        self.shared.changed.notify_all();
        Some(status)
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        self.unsubscribe();
    }
}

struct Shared {
    generation: Generation,
    config: SessionConfig,
    started: Instant,
    changed: Condvar,
    state: Mutex<State>,
}

impl Shared {
    fn drain_remaining(&self, state: &State) -> Option<Duration> {
        state
            .draining_since
            .map(|since| self.config.stop_grace.saturating_sub(since.elapsed()))
    }

    // Every path that can change a draining queue first respects the same
    // cutoff, even when the stopping thread has yet to reacquire the lock.
    fn complete_if_ready(&self, state: &mut State) -> Option<Arc<SessionStatus>> {
        if let Some(completed) = &state.completed {
            return Some(Arc::clone(completed));
        }
        let remaining = self.drain_remaining(state)?;
        if !remaining.is_zero() && state.subscribers.values().any(|sub| !sub.queue.is_empty()) {
            return None;
        }
        let mut discarded = false;
        for sub in state.subscribers.values_mut() {
            discarded |= sub.discard(LossReason::StopTimeout);
        }
        state.incomplete_delivery |= discarded;
        let mut completed = state.snapshot(self.generation);
        completed.phase = SessionPhase::Stopped;
        let completed = Arc::new(completed);
        state.completed = Some(Arc::clone(&completed));
        self.changed.notify_all();
        Some(completed)
    }

    fn now(&self, state: &State) -> Duration {
        match self.config.clock {
            FlushClock::RealTime => self.started.elapsed(),
            FlushClock::Manual => state.manual_time,
        }
    }

    fn wait<'a>(
        &self,
        state: std::sync::MutexGuard<'a, State>,
        remaining: Duration,
    ) -> std::sync::MutexGuard<'a, State> {
        // Keep relative waits representable by OS timeout APIs.
        self.changed
            .wait_timeout(state, remaining.min(Duration::from_secs(86_400)))
            .expect("capture state")
            .0
    }

    fn admit(&self, state: &mut State, event: Observed<CanFrame>) -> Result<(), SessionError> {
        let source = event.id.source;
        let now = self.now(state);
        let producer = state
            .producers
            .get_mut(&source)
            .ok_or(SessionError::UnknownSource)?;
        // Enforce this source's existing boundary even if input won the lock
        // just as the timer became runnable.
        if producer.deadline.is_some_and(|deadline| deadline <= now) {
            state.flush(source);
            self.changed.notify_all();
        }
        let producer = state.producers.get_mut(&source).expect("configured source");
        let was_empty = producer.ingest.staged_events() == 0;
        if let Err(error) = producer.ingest.push(event) {
            if error == IngestError::BatchBudgetExhausted {
                producer.budget_rejections = producer.budget_rejections.saturating_add(1);
            }
            return Err(SessionError::Ingest(error));
        }
        if producer.ingest.staged_events() == 0 {
            producer.deadline = None;
            let batch = producer.take_published();
            state.publish(source, batch);
        } else if was_empty {
            producer.deadline = Some(now.saturating_add(self.config.flush_after));
        } else {
            return Ok(());
        }
        self.changed.notify_all();
        Ok(())
    }

    fn run_timer(&self) {
        let mut state = self.state.lock().expect("capture state");
        while state.draining_since.is_none() {
            let now = self.now(&state);
            // Split borrows so due work can publish without allocating a list.
            let State {
                producers,
                subscribers,
                incomplete_delivery,
                ..
            } = &mut *state;
            for (&source, producer) in producers.iter_mut() {
                if producer.deadline.is_some_and(|deadline| deadline <= now) {
                    let batch = producer.flush().expect("deadline has staged observations");
                    *incomplete_delivery |= producer.publish(source, batch, subscribers);
                    self.changed.notify_all();
                }
            }
            let next = producers
                .values()
                .filter_map(|producer| producer.deadline)
                .min();
            state = match (self.config.clock, next) {
                (FlushClock::RealTime, Some(deadline)) => {
                    self.wait(state, deadline.saturating_sub(self.started.elapsed()))
                }
                _ => self.changed.wait(state).expect("capture state"),
            };
        }
    }
}

struct State {
    producers: HashMap<SourceId, Producer>,
    subscribers: HashMap<SubscriptionId, Subscriber>,
    next_subscription: u64,
    manual_time: Duration,
    draining_since: Option<Instant>,
    completed: Option<Arc<SessionStatus>>,
    incomplete_delivery: bool,
}

impl State {
    fn require_running(&self) -> Result<(), SessionError> {
        if self.draining_since.is_some() {
            return Err(SessionError::Stopped);
        }
        Ok(())
    }

    fn flush(&mut self, source: SourceId) {
        let producer = self.producers.get_mut(&source).expect("configured source");
        if let Some(batch) = producer.flush() {
            self.incomplete_delivery |= producer.publish(source, batch, &mut self.subscribers);
        }
    }

    fn publish(&mut self, source: SourceId, batch: SharedBatch<CanFrame>) {
        let producer = self.producers.get_mut(&source).expect("configured source");
        self.incomplete_delivery |= producer.publish(source, batch, &mut self.subscribers);
    }

    fn shutdown(&mut self) {
        for (&source, producer) in &mut self.producers {
            let had_tail = producer.ingest.staged_events() != 0;
            producer.ingest.shutdown().expect("running producer");
            producer.deadline = None;
            if had_tail {
                let batch = producer.take_published();
                self.incomplete_delivery |= producer.publish(source, batch, &mut self.subscribers);
            }
        }
    }

    fn snapshot(&self, generation: Generation) -> SessionStatus {
        let mut sources: Vec<_> = self
            .producers
            .iter()
            .map(|(&source, producer)| SourceStatus {
                source,
                ingest: IngestStats {
                    unrouted_events: producer.unrouted_events,
                    ..producer.ingest.stats()
                },
                budget_rejections: producer.budget_rejections,
                detached_overflow_events: producer.detached_overflow_events,
                detached_discarded_events: producer.detached_discarded_events,
            })
            .collect();
        sources.sort_unstable_by_key(|entry| entry.source.get());
        let mut subscriptions: Vec<_> = self.subscribers.values().map(|sub| sub.status).collect();
        subscriptions.sort_unstable_by_key(|entry| entry.id.0);
        SessionStatus {
            generation,
            phase: if self.draining_since.is_some() {
                SessionPhase::Draining
            } else {
                SessionPhase::Running
            },
            sources,
            subscriptions,
            incomplete_delivery: self.incomplete_delivery,
        }
    }
}

struct Producer {
    ingest: BoundedIngest<CanFrame>,
    published: Receiver<SharedBatch<CanFrame>>,
    deadline: Option<Duration>,
    budget_rejections: u64,
    detached_overflow_events: u64,
    detached_discarded_events: u64,
    unrouted_events: u64,
}

impl Producer {
    fn new(
        source: SourceId,
        config: IngestConfig,
        budget: BatchBudget,
    ) -> Result<Self, StartError> {
        let mut ingest =
            BoundedIngest::with_budget(source, config, budget).map_err(StartError::Ingest)?;
        // One internal slot transfers the shared backing at the publication
        // boundary. Every operation that publishes takes it under the same lock.
        let (_, published) = ingest
            .subscribe(OverflowPolicy::DropNewest, 1)
            .map_err(StartError::Ingest)?;
        ingest.start().map_err(StartError::Ingest)?;
        Ok(Self {
            ingest,
            published,
            deadline: None,
            budget_rejections: 0,
            detached_overflow_events: 0,
            detached_discarded_events: 0,
            unrouted_events: 0,
        })
    }

    fn take_published(&self) -> SharedBatch<CanFrame> {
        self.published.try_recv().expect("one published batch")
    }

    fn flush(&mut self) -> Option<SharedBatch<CanFrame>> {
        self.deadline = None;
        self.ingest
            .flush()
            .expect("running producer")
            .then(|| self.take_published())
    }

    fn publish(
        &mut self,
        source: SourceId,
        batch: SharedBatch<CanFrame>,
        subscribers: &mut HashMap<SubscriptionId, Subscriber>,
    ) -> bool {
        let mut routed = false;
        let mut lost = false;
        for sub in subscribers
            .values_mut()
            .filter(|sub| sub.status.source == source)
        {
            routed = true;
            if sub.queue.len() == sub.capacity {
                sub.record_loss(&batch, LossReason::QueueFull);
                lost = true;
                continue;
            }
            let count = batch.len() as u64;
            sub.status.delivery.queued_batches =
                sub.status.delivery.queued_batches.saturating_add(1);
            sub.status.delivery.queued_events =
                sub.status.delivery.queued_events.saturating_add(count);
            sub.status.pending_events += count;
            sub.queue.push_back(batch.clone());
        }
        if !routed {
            self.unrouted_events = self.unrouted_events.saturating_add(batch.len() as u64);
        }
        lost
    }
}

struct Subscriber {
    attached: bool,
    capacity: usize,
    queue: VecDeque<SharedBatch<CanFrame>>,
    status: SubscriptionStatus,
}

impl Subscriber {
    fn new(id: SubscriptionId, source: SourceId, capacity: usize) -> Self {
        Self {
            attached: true,
            capacity,
            queue: VecDeque::new(),
            status: SubscriptionStatus {
                id,
                source,
                delivery: ConsumerStats::default(),
                overflow_events: 0,
                discarded_events: 0,
                received_events: 0,
                pending_events: 0,
            },
        }
    }

    fn discard(&mut self, reason: LossReason) -> bool {
        let discarded = !self.queue.is_empty();
        while let Some(batch) = self.queue.pop_front() {
            self.record_loss(&batch, reason);
        }
        self.status.pending_events = 0;
        discarded
    }

    fn record_loss(&mut self, batch: &SharedBatch<CanFrame>, reason: LossReason) {
        let events = batch.events();
        let first = events.first().expect("published batch is nonempty").id;
        let last = events.last().expect("published batch is nonempty").id;
        let reason_total = if reason == LossReason::QueueFull {
            &mut self.status.overflow_events
        } else {
            &mut self.status.discarded_events
        };
        *reason_total = reason_total.saturating_add(batch.len() as u64);
        let stats = &mut self.status.delivery;
        stats.dropped_batches = stats.dropped_batches.saturating_add(1);
        stats.dropped_events = stats.dropped_events.saturating_add(batch.len() as u64);
        stats.last_loss = Some(BatchLoss {
            source: first.source,
            first_sequence: first.sequence,
            last_sequence: last.sequence,
            event_count: batch.len(),
            reason,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        time::{ClockDomainId, TimestampResolution},
        virtual_can::fixtures,
    };

    #[test]
    fn receiver_winning_the_lock_after_expiry_cannot_take_the_tail() {
        let source = SourceId::new(1);
        let config = SessionConfig {
            ingest: IngestConfig::new(2, 1, 2).unwrap(),
            flush_after: Duration::from_secs(1),
            stop_grace: Duration::ZERO,
            clock: FlushClock::Manual,
        };
        let budget =
            BatchBudget::new(BoundedIngest::<CanFrame>::batch_bytes(config.ingest).unwrap());
        let session = CaptureSession::start(config, budget, &[source]).unwrap();
        let receiver = session.subscribe(source, 1).unwrap();
        let detaching = session.subscribe(source, 1).unwrap();
        let script = fixtures::mixed_classic_fd();
        let mut adapter = VirtualCanAdapter::new(
            source,
            ClockDomainId::new(1),
            TimestampResolution::new(1, 1).unwrap(),
            &script,
        )
        .unwrap();
        adapter.start().unwrap();
        let input = session.input();
        assert!(input
            .poll_virtual(input.generation(), &mut adapter, 0)
            .unwrap());

        // Simulate the stop caller yielding the state lock with expiry due,
        // so the receiver deterministically gets to the still-pending tail first.
        {
            let mut state = session.shared.state.lock().unwrap();
            state.draining_since = Some(Instant::now());
            state.shutdown();
        }

        assert_eq!(
            receiver.recv_timeout(Duration::ZERO).unwrap_err(),
            RecvTimeoutError::Disconnected
        );
        let detached = detaching.unsubscribe().unwrap();
        assert_eq!(
            detached.delivery.last_loss.unwrap().reason,
            LossReason::StopTimeout
        );
        let done = session.stop();
        assert!(done.incomplete_delivery);
        assert_eq!(done.sources[0].ingest.published_batches, 1);
        for entry in &done.subscriptions {
            assert_eq!(entry.received_events, 0);
            assert_eq!(entry.discarded_events, 1);
            assert_eq!(
                entry.delivery.last_loss.unwrap().reason,
                LossReason::StopTimeout
            );
        }
    }
}
