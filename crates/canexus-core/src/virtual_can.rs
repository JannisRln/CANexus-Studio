//! Deterministic, headless CAN/CAN-FD input for fixtures and integration tests.
//!
//! The adapter borrows an immutable, pre-validated sequence of records. Polling
//! advances a cursor without sleeping, allocating, or consulting a system clock.
//! It deliberately does not define a native hardware/plugin ABI, a TX scheduler,
//! or the bounded ingestion queues planned for subsequent tasks.

use std::{fmt, iter::FusedIterator};

use crate::{
    can::CanFrame,
    event::{ChannelId, EventDirection, Observed, RawEventId},
    time::{ClockDomainId, EventTimestamp, TimestampOrigin, TimestampQuality, TimestampResolution},
    SourceId,
};

/// One reproducible observation in a virtual CAN input trace.
///
/// Ticks are relative to the clock domain supplied to the adapter. Equal
/// ticks are permitted for bursts and retain their fixture order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VirtualCanRecord {
    /// Timestamp counter value; never interpreted as wall-clock time.
    pub ticks: u64,
    /// Logical channel on which this observation appeared.
    pub channel: ChannelId,
    /// Receive or transmit direction recorded in the fixture.
    pub direction: EventDirection,
    /// Validated, owned CAN Classic or CAN FD frame.
    pub frame: CanFrame,
}

/// An invalid input fixture, rejected before iteration starts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VirtualCanError {
    /// Timestamp counters decreased between adjacent records.
    NonMonotonicTicks {
        /// Index of the first out-of-order record.
        index: usize,
        /// Previous record's counter.
        previous: u64,
        /// Out-of-order counter.
        current: u64,
    },
}

impl fmt::Display for VirtualCanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonMonotonicTicks {
                index,
                previous,
                current,
            } => write!(
                f,
                "virtual CAN timestamp decreased at index {index}: {current} < {previous}"
            ),
        }
    }
}

impl std::error::Error for VirtualCanError {}

/// Replays an immutable CAN fixture as typed observations without per-event
/// allocation, blocking I/O, wall-clock sleeps, or GUI dependencies.
///
/// A source identifies a capture instance. Consumers that combine independent
/// replays must supply distinct source IDs; each instance starts its source-local
/// sequence at zero. Timestamps remain explicitly synthetic and uncorrelated.
#[derive(Clone, Debug)]
pub struct VirtualCanAdapter<'a> {
    source: SourceId,
    clock_domain: ClockDomainId,
    resolution: TimestampResolution,
    records: &'a [VirtualCanRecord],
    cursor: usize,
}

impl<'a> VirtualCanAdapter<'a> {
    /// Validates a fixture once and creates a pull-based virtual input.
    ///
    /// Non-decreasing timestamps are required within the supplied clock domain.
    /// Equal timestamps are allowed, preserving arrival order and source sequence.
    pub fn new(
        source: SourceId,
        clock_domain: ClockDomainId,
        resolution: TimestampResolution,
        records: &'a [VirtualCanRecord],
    ) -> Result<Self, VirtualCanError> {
        for (index, pair) in records.windows(2).enumerate() {
            if pair[1].ticks < pair[0].ticks {
                return Err(VirtualCanError::NonMonotonicTicks {
                    index: index + 1,
                    previous: pair[0].ticks,
                    current: pair[1].ticks,
                });
            }
        }

        Ok(Self {
            source,
            clock_domain,
            resolution,
            records,
            cursor: 0,
        })
    }

    /// Number of observations not yet produced.
    #[must_use]
    pub fn remaining(&self) -> usize {
        self.records.len() - self.cursor
    }
}

impl Iterator for VirtualCanAdapter<'_> {
    type Item = Observed<CanFrame>;

    fn next(&mut self) -> Option<Self::Item> {
        let record = *self.records.get(self.cursor)?;
        let sequence = u64::try_from(self.cursor)
            .expect("a fixture cannot contain more than u64::MAX indexed records");
        self.cursor += 1;

        Some(Observed {
            id: RawEventId {
                source: self.source,
                sequence,
            },
            channel: record.channel,
            timestamp: EventTimestamp::new(
                record.ticks,
                self.clock_domain,
                self.resolution,
                TimestampOrigin::Synthetic,
                TimestampQuality::Uncorrelated,
            ),
            direction: record.direction,
            value: record.frame,
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.remaining();
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for VirtualCanAdapter<'_> {}

impl FusedIterator for VirtualCanAdapter<'_> {}
