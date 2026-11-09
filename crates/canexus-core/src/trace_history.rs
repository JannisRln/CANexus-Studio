//! Bounded, source-local, typed RAM trace history and paginated read API.
//!
//! A history retains immutable batches from ingestion without copying frames.
//! Old batches are evicted under an event-byte budget and a batch-slot budget.
//! Eviction is **retention loss**, not a claim of capture or recorder loss.
//! Query pages borrow retained events and become stale when history mutates.

use std::{collections::VecDeque, mem::size_of, num::NonZeroUsize};

use crate::{
    can::{CanFormat, CanFrame, CanId, MAX_EXTENDED_ID, MAX_STANDARD_ID},
    event::{ChannelId, EventDirection, Observed, RawEventId},
    ingest::{BatchLoss, SharedBatch},
    SourceId,
};

/// Finite limits for retained raw data, allocation metadata and query pages.
///
/// `max_event_bytes` counts inline `Observed<T>` elements, not allocator
/// headers, `Arc` control blocks, unused allocation capacity, or external
/// retained handles. `max_batches` separately caps per-batch metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HistoryLimits {
    max_event_bytes: NonZeroUsize,
    max_batches: NonZeroUsize,
    max_page_rows: NonZeroUsize,
}

impl HistoryLimits {
    /// Builds positive history limits.
    pub fn new(
        max_event_bytes: usize,
        max_batches: usize,
        max_page_rows: usize,
    ) -> Result<Self, HistoryError> {
        Ok(Self {
            max_event_bytes: NonZeroUsize::new(max_event_bytes)
                .ok_or(HistoryError::InvalidLimit)?,
            max_batches: NonZeroUsize::new(max_batches).ok_or(HistoryError::InvalidLimit)?,
            max_page_rows: NonZeroUsize::new(max_page_rows).ok_or(HistoryError::InvalidLimit)?,
        })
    }

    /// Maximum retained inline event bytes, excluding metadata and external owners.
    #[must_use]
    pub const fn max_event_bytes(self) -> usize {
        self.max_event_bytes.get()
    }

    /// Maximum retained immutable batches.
    #[must_use]
    pub const fn max_batches(self) -> usize {
        self.max_batches.get()
    }

    /// Maximum event references returned by a page request.
    #[must_use]
    pub const fn max_page_rows(self) -> usize {
        self.max_page_rows.get()
    }
}

/// History admission or metadata errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HistoryError {
    /// One of the configuration limits was zero.
    InvalidLimit,
    /// Empty batches must not consume retention slots.
    EmptyBatch,
    /// Batch contains an observation from another source.
    SourceMismatch,
    /// A batch repeats a sequence or moves backwards.
    NonMonotonicSequence { index: usize },
    /// A single batch exceeds the retained event-byte limit or overflows size arithmetic.
    BatchTooLarge,
    /// The history's version counter cannot be advanced.
    VersionExhausted,
    /// The reported upstream loss has no events or invalid sequence boundaries.
    InvalidLossReport,
}

/// Reasons for a paginated query failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PageError {
    /// The request was empty or exceeded the configured row budget.
    PageLimitExceeded,
    /// A history mutation or source mismatch invalidated the cursor.
    StaleCursor,
}

/// Information about the oldest batches evicted by an append.
///
/// Sequence bounds refer to observed records, not a contiguous source range.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HistoryEviction {
    /// Source whose retained observations were evicted.
    pub source: SourceId,
    /// Oldest sequence removed in this eviction operation.
    pub first_sequence: u64,
    /// Newest sequence removed in this eviction operation.
    pub last_sequence: u64,
    /// Exact number of observations removed.
    pub events: usize,
    /// Number of complete batches removed.
    pub batches: usize,
}

/// Result of admitting one immutable batch.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AppendResult {
    /// Event count removed to make room for the new batch.
    pub evicted_events: usize,
    /// Complete batches removed to make room for the new batch.
    pub evicted_batches: usize,
}

/// Non-overlapping categories of retention loss and explicitly reported input loss.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HistoryStats {
    /// Number of admitted observations since construction.
    pub admitted_events: u64,
    /// Number of observations currently retained.
    pub retained_events: usize,
    /// Number of immutable batches currently retained.
    pub retained_batches: usize,
    /// Estimated inline event bytes still retained.
    pub retained_event_bytes: usize,
    /// Observations evicted from RAM, not lost during capture.
    pub evicted_events: u64,
    /// Batch allocations evicted from RAM, not missing capture batches.
    pub evicted_batches: u64,
    /// Events explicitly reported missing from the input consumer queue.
    pub reported_input_drops: u64,
    /// Last retention eviction with exact retained-event identity boundaries.
    pub last_eviction: Option<HistoryEviction>,
    /// Last externally reported queue loss, with its original reason.
    pub last_input_loss: Option<BatchLoss>,
}

/// A cursor valid only for a single source-local history version.
///
/// All mutations to retained events invalidate outstanding page cursors.
/// A cursor is never a durable recording offset or a wall-clock timestamp.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PageCursor {
    source: SourceId,
    version: u64,
    batch_index: usize,
    event_index: usize,
}

/// Request a bounded page, optionally resuming from a prior page.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PageRequest {
    /// Previously returned cursor, or `None` for the first page.
    pub cursor: Option<PageCursor>,
    /// Requested maximum rows, in `1..=max_page_rows`.
    pub limit: usize,
}

/// Borrowed, zero-copy event references from a retained history version.
///
/// Each page allocates only a bounded vector of references; it does not copy
/// payloads or materialize an index for the whole retained history.
#[derive(Debug)]
pub struct TracePage<'a, T> {
    /// Captured observations matching the query, in source sequence order.
    pub rows: Vec<&'a Observed<T>>,
    /// Resume token, or `None` if no retained observations remain to scan.
    pub next: Option<PageCursor>,
    /// History version used when this page was produced.
    pub version: u64,
}

/// A simple transport-independent predicate for trace metadata.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EventFilter {
    /// Only events from this logical channel.
    pub channel: Option<ChannelId>,
    /// Only received or transmitted events.
    pub direction: Option<EventDirection>,
    /// Inclusive lower bound on source-local sequence number.
    pub first_sequence: Option<u64>,
    /// Inclusive upper bound on source-local sequence number.
    pub last_sequence: Option<u64>,
}

impl EventFilter {
    /// Checks event identity and metadata without reading a protocol payload.
    #[must_use]
    pub fn matches<T>(&self, event: &Observed<T>) -> bool {
        self.channel.is_none_or(|channel| channel == event.channel)
            && self
                .direction
                .is_none_or(|direction| direction == event.direction)
            && self
                .first_sequence
                .is_none_or(|first| event.id.sequence >= first)
            && self
                .last_sequence
                .is_none_or(|last| event.id.sequence <= last)
    }
}

/// CAN arbitration ID and mask, preserving 11-bit versus 29-bit identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CanIdMask {
    /// Arbitration ID and wire representation being matched.
    pub id: CanId,
    /// Bits considered significant by the filter.
    pub mask: u32,
}

impl CanIdMask {
    /// Validates that a mask does not reference bits outside the ID format.
    #[must_use]
    pub fn new(id: CanId, mask: u32) -> Option<Self> {
        let maximum = if id.is_extended() {
            MAX_EXTENDED_ID
        } else {
            u32::from(MAX_STANDARD_ID)
        };
        if mask > maximum {
            None
        } else {
            Some(Self { id, mask })
        }
    }

    /// Checks both wire representation and masked arbitration bits.
    #[must_use]
    pub fn matches(self, actual: CanId) -> bool {
        actual.is_extended() == self.id.is_extended()
            && actual.raw() & self.mask == self.id.raw() & self.mask
    }
}

/// CAN-specific filters layered on top of generic observation metadata.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CanTraceFilter {
    /// Transport-neutral metadata predicate.
    pub event: EventFilter,
    /// CAN identifier and mask, if any.
    pub arbitration: Option<CanIdMask>,
    /// Classic or FD frame format, if any.
    pub format: Option<CanFormat>,
}

impl CanTraceFilter {
    /// Checks metadata and CAN ID/format without creating a decoded event.
    #[must_use]
    pub fn matches(&self, event: &Observed<CanFrame>) -> bool {
        self.event.matches(event)
            && self
                .arbitration
                .is_none_or(|id_mask| id_mask.matches(event.value.id()))
            && self
                .format
                .is_none_or(|format| format == event.value.flags().format)
    }
}

/// Single-owner, source-local bounded raw trace history.
///
/// Retention is at complete-batch granularity. Only the last admitted source
/// sequence is kept as an ordering guard; gaps do not imply captured loss.
/// Views and recorders are separate consumers, and this service never modifies
/// capture, recorder queues or the source's timestamps.
pub struct TraceHistory<T> {
    source: SourceId,
    limits: HistoryLimits,
    batches: VecDeque<SharedBatch<T>>,
    stats: HistoryStats,
    last_sequence: Option<u64>,
    version: u64,
}

impl<T> TraceHistory<T> {
    /// Initializes an empty source-local history without allocating event storage.
    #[must_use]
    pub fn new(source: SourceId, limits: HistoryLimits) -> Self {
        Self {
            source,
            limits,
            batches: VecDeque::new(),
            stats: HistoryStats::default(),
            last_sequence: None,
            version: 0,
        }
    }

    /// Identifies the source whose events this history accepts.
    #[must_use]
    pub const fn source(&self) -> SourceId {
        self.source
    }

    /// Returns immutable counters and the most recent loss provenance.
    #[must_use]
    pub const fn stats(&self) -> HistoryStats {
        self.stats
    }

    /// Returns the current query version.
    #[must_use]
    pub const fn version(&self) -> u64 {
        self.version
    }

    /// Returns the earliest retained source-local event identity.
    #[must_use]
    pub fn oldest(&self) -> Option<RawEventId> {
        self.batches
            .front()
            .and_then(|batch| batch.events().first())
            .map(|event| event.id)
    }

    /// Returns the latest retained source-local event identity.
    #[must_use]
    pub fn newest(&self) -> Option<RawEventId> {
        self.batches
            .back()
            .and_then(|batch| batch.events().last())
            .map(|event| event.id)
    }

    /// Admits a nonempty immutable batch; evicts whole oldest batches as needed.
    ///
    /// Rejects oversize, invalid-source or nonmonotonic batches without
    /// changing the history. Retained-event bytes and batch slots stay within
    /// limits. Other owners of evicted `Arc` batches may still retain memory.
    pub fn append(&mut self, batch: SharedBatch<T>) -> Result<AppendResult, HistoryError> {
        let events = batch.events();
        let first = events.first().ok_or(HistoryError::EmptyBatch)?;
        let bytes = events
            .len()
            .checked_mul(size_of::<Observed<T>>())
            .ok_or(HistoryError::BatchTooLarge)?;
        if bytes > self.limits.max_event_bytes() {
            return Err(HistoryError::BatchTooLarge);
        }

        let mut previous = self.last_sequence;
        for (index, event) in events.iter().enumerate() {
            if event.id.source != self.source {
                return Err(HistoryError::SourceMismatch);
            }
            if previous.is_some_and(|sequence| event.id.sequence <= sequence) {
                return Err(HistoryError::NonMonotonicSequence { index });
            }
            previous = Some(event.id.sequence);
        }
        let next_version = self
            .version
            .checked_add(1)
            .ok_or(HistoryError::VersionExhausted)?;

        let mut removed = AppendResult::default();
        let mut first_evicted = None;
        let mut last_evicted = None;
        while self.batches.len() >= self.limits.max_batches()
            || self.stats.retained_event_bytes > self.limits.max_event_bytes() - bytes
        {
            let oldest = self.batches.pop_front().expect("budget requires oldest");
            let events = oldest.events();
            first_evicted.get_or_insert(events[0].id.sequence);
            last_evicted = Some(events[events.len() - 1].id.sequence);
            removed.evicted_batches += 1;
            removed.evicted_events += events.len();
            self.stats.retained_events -= events.len();
            self.stats.retained_event_bytes -= events.len() * size_of::<Observed<T>>();
        }

        if let (Some(first_sequence), Some(last_sequence)) = (first_evicted, last_evicted) {
            self.stats.last_eviction = Some(HistoryEviction {
                source: self.source,
                first_sequence,
                last_sequence,
                events: removed.evicted_events,
                batches: removed.evicted_batches,
            });
        }
        self.stats.evicted_events = self
            .stats
            .evicted_events
            .saturating_add(removed.evicted_events as u64);
        self.stats.evicted_batches = self
            .stats
            .evicted_batches
            .saturating_add(removed.evicted_batches as u64);
        self.stats.retained_events += events.len();
        self.stats.retained_event_bytes += bytes;
        self.stats.admitted_events = self
            .stats
            .admitted_events
            .saturating_add(events.len() as u64);
        self.last_sequence = previous;
        self.version = next_version;
        self.batches.push_back(batch);
        self.stats.retained_batches = self.batches.len();
        Ok(removed)
    }

    /// Records a known consumer-input loss without treating it as RAM eviction.
    ///
    /// Use only an explicit upstream `BatchLoss`. A discontinuous raw
    /// sequence by itself does not prove why an event was absent.
    pub fn report_input_loss(&mut self, loss: BatchLoss) -> Result<(), HistoryError> {
        if loss.source != self.source {
            return Err(HistoryError::SourceMismatch);
        }
        if loss.event_count == 0 || loss.last_sequence < loss.first_sequence {
            return Err(HistoryError::InvalidLossReport);
        }
        self.stats.reported_input_drops = self
            .stats
            .reported_input_drops
            .saturating_add(loss.event_count as u64);
        self.stats.last_input_loss = Some(loss);
        Ok(())
    }

    /// Returns a bounded, version-checked page using any typed event predicate.
    ///
    /// A query scans retained batches in capture sequence order, not in an
    /// invented global wall-clock order. Filtering may scan many rows without
    /// allocating them. Callers must run expensive scans off the GUI thread.
    pub fn page_with<F>(
        &self,
        request: PageRequest,
        mut matches: F,
    ) -> Result<TracePage<'_, T>, PageError>
    where
        F: FnMut(&Observed<T>) -> bool,
    {
        if request.limit == 0 || request.limit > self.limits.max_page_rows() {
            return Err(PageError::PageLimitExceeded);
        }
        let (first_batch, first_event) = match request.cursor {
            None => (0, 0),
            Some(cursor) => {
                if cursor.source != self.source || cursor.version != self.version {
                    return Err(PageError::StaleCursor);
                }
                (cursor.batch_index, cursor.event_index)
            }
        };
        let mut rows = Vec::with_capacity(request.limit);
        for (batch_index, batch) in self.batches.iter().enumerate().skip(first_batch) {
            let begin = if batch_index == first_batch {
                first_event
            } else {
                0
            };
            for (event_index, event) in batch.events().iter().enumerate().skip(begin) {
                if matches(event) {
                    rows.push(event);
                    if rows.len() == request.limit {
                        let (next_batch, next_event) = if event_index + 1 == batch.len() {
                            (batch_index + 1, 0)
                        } else {
                            (batch_index, event_index + 1)
                        };
                        let next = (next_batch < self.batches.len()).then_some(PageCursor {
                            source: self.source,
                            version: self.version,
                            batch_index: next_batch,
                            event_index: next_event,
                        });
                        return Ok(TracePage {
                            rows,
                            next,
                            version: self.version,
                        });
                    }
                }
            }
        }
        Ok(TracePage {
            rows,
            next: None,
            version: self.version,
        })
    }

    /// Counts matching retained events without materializing results.
    ///
    /// This is O(retained events), so it belongs on a query worker for large
    /// datasets. It does not affect capture or retention.
    #[must_use]
    pub fn count_matches<F>(&self, mut matches: F) -> usize
    where
        F: FnMut(&Observed<T>) -> bool,
    {
        self.batches
            .iter()
            .flat_map(|batch| batch.events())
            .filter(|event| matches(event))
            .count()
    }
}

impl TraceHistory<CanFrame> {
    /// Returns a bounded CAN-specific page, retaining the original frame bytes.
    pub fn can_page(
        &self,
        filter: &CanTraceFilter,
        request: PageRequest,
    ) -> Result<TracePage<'_, CanFrame>, PageError> {
        self.page_with(request, |event| filter.matches(event))
    }
}
