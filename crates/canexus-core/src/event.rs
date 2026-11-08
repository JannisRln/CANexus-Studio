//! Transport-neutral observation metadata and identity.
//!
//! The generic payload permits CAN now and other transport event kinds later.

use crate::{time::EventTimestamp, SourceId};

/// A logical channel within a data source.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ChannelId(u32);

impl ChannelId {
    /// Creates a channel identifier.
    #[must_use]
    pub const fn new(value: u32) -> Self { Self(value) }
    /// Returns the channel identifier.
    #[must_use]
    pub const fn get(self) -> u32 { self.0 }
}

/// Captured ordering is local to a source, not a universal clock order.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RawEventId {
    /// Source that assigned the sequence.
    pub source: SourceId,
    /// Monotonic sequence within that source.
    pub sequence: u64,
}

/// Whether an event was received or transmitted.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EventDirection {
    /// Traffic observed arriving on the channel.
    Rx,
    /// Traffic transmitted on the channel.
    Tx,
}

/// An observation with provenance and an arbitrary typed event payload.
///
/// Events are owned values; no heap allocation is performed by this envelope.
/// A complete network event vocabulary (errors, bus states, gaps) is deferred
/// to subsequent HAL and ingestion tasks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Observed<T> {
    /// Source and capture sequence.
    pub id: RawEventId,
    /// Logical input/output channel.
    pub channel: ChannelId,
    /// Original clock ticks and provenance.
    pub timestamp: EventTimestamp,
    /// Receive/transmit direction.
    pub direction: EventDirection,
    /// Transport-specific typed observation.
    pub value: T,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        can::{CanFrame, CanFrameFlags, CanId},
        time::{ClockDomainId, TimestampOrigin, TimestampQuality, TimestampResolution},
    };

    #[test]
    fn observed_can_frame_retains_identity_and_time() {
        let frame = CanFrame::new(
            CanId::standard(0x123).unwrap(), CanFrameFlags::CLASSIC, 2, &[0xab, 0xcd]
        ).unwrap();
        let observation = Observed {
            id: RawEventId { source: SourceId::new(7), sequence: 21 },
            channel: ChannelId::new(2),
            timestamp: EventTimestamp::new(
                1000, ClockDomainId::new(9),
                TimestampResolution::new(1, 1).unwrap(),
                TimestampOrigin::Hardware, TimestampQuality::Uncorrelated
            ),
            direction: EventDirection::Rx,
            value: frame,
        };
        assert_eq!(observation.id.source.get(), 7);
        assert_eq!(observation.id.sequence, 21);
        assert_eq!(observation.channel.get(), 2);
        assert_eq!(observation.direction, EventDirection::Rx);
        assert_eq!(observation.timestamp.ticks, 1000);
        assert_eq!(observation.value.data(), &[0xab, 0xcd]);
    }
}
