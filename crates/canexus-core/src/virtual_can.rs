//! Deterministic, headless virtual CAN/CAN-FD event source.
//!
//! The adapter borrows an immutable, timestamp-ordered fixture slice. Virtual
//! time advances only when requested by the caller; polling does not sleep,
//! spawn threads, allocate, or perform physical bus transmission. Queueing,
//! backpressure, fault injection and a shared hardware HAL are later tasks.

use crate::{
    can::CanFrame,
    event::{ChannelId, EventDirection, Observed, RawEventId},
    time::{ClockDomainId, EventTimestamp, TimestampOrigin, TimestampQuality, TimestampResolution},
    SourceId,
};

/// One CAN observation scheduled in the virtual clock domain.
///
/// Entries with equal `at_tick` are emitted in their original slice order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VirtualCanFixture {
    /// Virtual clock tick at which the event becomes available.
    pub at_tick: u64,
    /// Logical channel on the virtual adapter.
    pub channel: ChannelId,
    /// Whether the original observation is RX or TX.
    pub direction: EventDirection,
    /// Complete, already validated CAN Classic/CAN-FD frame.
    pub frame: CanFrame,
}

/// Lifecycle of a single-use virtual capture session.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VirtualCanState {
    /// Constructed, but capture has not been started.
    Ready,
    /// Virtual time may advance and due events may be polled.
    Running,
    /// Permanently stopped; create a new adapter with a new source ID to restart.
    Stopped,
}

/// Explicit failures of virtual fixture validation or capture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VirtualCanError {
    /// Fixture timestamps decreased at this zero-based index.
    UnsortedFixture { index: usize },
    /// Capture was already started.
    AlreadyRunning,
    /// The session was permanently stopped.
    Stopped,
    /// Capture must be started before advancing/polling/stopping.
    NotRunning,
    /// Caller attempted to move the virtual clock backwards.
    ClockMovedBackwards,
    /// The source-local event sequence cannot be incremented safely.
    SequenceExhausted,
}

/// A deterministic virtual CAN source, with no internal queue or timer.
///
/// The caller owns the fixtures and determines when time advances. Events are
/// returned by value as `Observed<CanFrame>` and carry a synthetic timestamp,
/// an explicit clock domain, and source-scoped sequence numbers.
///
/// This is a capture fixture source, not a hardware TX scheduler. A new
/// capture session must use a new `SourceId` to avoid reusing event identities.
#[derive(Debug)]
pub struct VirtualCanAdapter<'a> {
    source: SourceId,
    domain: ClockDomainId,
    resolution: TimestampResolution,
    fixtures: &'a [VirtualCanFixture],
    next_index: usize,
    next_sequence: u64,
    now_ticks: u64,
    state: VirtualCanState,
}

impl<'a> VirtualCanAdapter<'a> {
    /// Validates a chronological fixture script without allocating or sorting.
    ///
    /// Duplicate timestamps are legal and retain the order of input entries.
    pub fn new(
        source: SourceId,
        domain: ClockDomainId,
        resolution: TimestampResolution,
        fixtures: &'a [VirtualCanFixture],
    ) -> Result<Self, VirtualCanError> {
        for (index, pair) in fixtures.windows(2).enumerate() {
            if pair[0].at_tick > pair[1].at_tick {
                return Err(VirtualCanError::UnsortedFixture { index: index + 1 });
            }
        }
        Ok(Self {
            source,
            domain,
            resolution,
            fixtures,
            next_index: 0,
            next_sequence: 0,
            now_ticks: 0,
            state: VirtualCanState::Ready,
        })
    }

    /// Returns the session state.
    #[must_use]
    pub const fn state(&self) -> VirtualCanState {
        self.state
    }

    /// Returns the current synthetic tick count, initially zero.
    #[must_use]
    pub const fn now_ticks(&self) -> u64 {
        self.now_ticks
    }

    /// Returns the number of fixture observations not yet emitted.
    #[must_use]
    pub fn remaining(&self) -> usize {
        self.fixtures.len() - self.next_index
    }

    /// Starts capture once. A stopped session cannot be restarted.
    pub fn start(&mut self) -> Result<(), VirtualCanError> {
        match self.state {
            VirtualCanState::Ready => {
                self.state = VirtualCanState::Running;
                Ok(())
            }
            VirtualCanState::Running => Err(VirtualCanError::AlreadyRunning),
            VirtualCanState::Stopped => Err(VirtualCanError::Stopped),
        }
    }

    /// Advances virtual time monotonically; no events are emitted implicitly.
    pub fn advance_to(&mut self, ticks: u64) -> Result<(), VirtualCanError> {
        self.require_running()?;
        if ticks < self.now_ticks {
            return Err(VirtualCanError::ClockMovedBackwards);
        }
        self.now_ticks = ticks;
        Ok(())
    }

    /// Polls one due observation, returning `None` when no event is due.
    ///
    /// Polling is O(1) and allocation-free. The caller can drain all currently
    /// due events using a loop, or control scheduling one event at a time.
    pub fn next_due(&mut self) -> Result<Option<Observed<CanFrame>>, VirtualCanError> {
        self.require_running()?;
        let Some(fixture) = self.fixtures.get(self.next_index).copied() else {
            return Ok(None);
        };
        if fixture.at_tick > self.now_ticks {
            return Ok(None);
        }
        let following_sequence = self
            .next_sequence
            .checked_add(1)
            .ok_or(VirtualCanError::SequenceExhausted)?;
        let observation = Observed {
            id: RawEventId {
                source: self.source,
                sequence: self.next_sequence,
            },
            channel: fixture.channel,
            timestamp: EventTimestamp::new(
                fixture.at_tick,
                self.domain,
                self.resolution,
                TimestampOrigin::Synthetic,
                TimestampQuality::Uncorrelated,
            ),
            direction: fixture.direction,
            value: fixture.frame,
        };
        self.next_index += 1;
        self.next_sequence = following_sequence;
        Ok(Some(observation))
    }

    /// Stops capture immediately, discarding any unconsumed fixture entries.
    pub fn stop(&mut self) -> Result<(), VirtualCanError> {
        self.require_running()?;
        self.state = VirtualCanState::Stopped;
        Ok(())
    }

    fn require_running(&self) -> Result<(), VirtualCanError> {
        match self.state {
            VirtualCanState::Ready => Err(VirtualCanError::NotRunning),
            VirtualCanState::Running => Ok(()),
            VirtualCanState::Stopped => Err(VirtualCanError::Stopped),
        }
    }
}

/// Reusable, hardware-independent fixture scenarios for integration tests.
pub mod fixtures {
    use super::VirtualCanFixture;
    use crate::{
        can::{CanFrame, CanFrameFlags, CanId},
        event::{ChannelId, EventDirection},
    };

    /// Mixed standard Classic, extended CAN FD, and Classic remote traffic.
    ///
    /// Two events share tick 10 deliberately, to test stable ordering.
    #[must_use]
    pub fn mixed_classic_fd() -> [VirtualCanFixture; 4] {
        let fd_payload = [0xa5; 64];
        [
            VirtualCanFixture {
                at_tick: 0,
                channel: ChannelId::new(0),
                direction: EventDirection::Rx,
                frame: CanFrame::new(
                    CanId::standard(0x123).unwrap(),
                    CanFrameFlags::CLASSIC,
                    2,
                    &[0x12, 0x34],
                )
                .unwrap(),
            },
            VirtualCanFixture {
                at_tick: 10,
                channel: ChannelId::new(1),
                direction: EventDirection::Rx,
                frame: CanFrame::new(
                    CanId::extended(0x1abcde).unwrap(),
                    CanFrameFlags {
                        bit_rate_switch: true,
                        ..CanFrameFlags::FD
                    },
                    15,
                    &fd_payload,
                )
                .unwrap(),
            },
            VirtualCanFixture {
                at_tick: 10,
                channel: ChannelId::new(0),
                direction: EventDirection::Tx,
                frame: CanFrame::new(
                    CanId::standard(0x456).unwrap(),
                    CanFrameFlags {
                        remote: true,
                        ..CanFrameFlags::CLASSIC
                    },
                    8,
                    &[],
                )
                .unwrap(),
            },
            VirtualCanFixture {
                at_tick: 25,
                channel: ChannelId::new(1),
                direction: EventDirection::Rx,
                frame: CanFrame::new(CanId::standard(0).unwrap(), CanFrameFlags::CLASSIC, 0, &[])
                    .unwrap(),
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::can::{CanFormat, CanFrameFlags};

    fn adapter(fixtures: &[VirtualCanFixture]) -> VirtualCanAdapter<'_> {
        VirtualCanAdapter::new(
            SourceId::new(7),
            ClockDomainId::new(9),
            TimestampResolution::new(125, 2).unwrap(),
            fixtures,
        )
        .unwrap()
    }

    #[test]
    fn fixtures_cover_classic_fd_remote_and_two_channels() {
        let fixtures = fixtures::mixed_classic_fd();
        assert_eq!(fixtures[0].frame.data(), &[0x12, 0x34]);
        assert_eq!(fixtures[1].frame.flags().format, CanFormat::Fd);
        assert!(fixtures[1].frame.flags().bit_rate_switch);
        assert!(fixtures[1].frame.id().is_extended());
        assert_eq!(fixtures[1].frame.dlc(), 15);
        assert_eq!(fixtures[1].frame.data(), &[0xa5; 64]);
        assert!(fixtures[2].frame.flags().remote);
        assert_eq!(fixtures[2].frame.dlc(), 8);
        assert_eq!(fixtures[2].frame.data(), &[]);
        assert_eq!(fixtures[3].frame.flags(), CanFrameFlags::CLASSIC);
        assert_eq!(fixtures[3].frame.data_len(), 0);
        assert_ne!(fixtures[0].channel, fixtures[1].channel);
    }

    #[test]
    fn clock_controls_visibility_and_preserves_stable_tie_order() {
        let fixtures = fixtures::mixed_classic_fd();
        let mut adapter = adapter(&fixtures);
        adapter.start().unwrap();
        assert_eq!(adapter.remaining(), 4);
        let first = adapter.next_due().unwrap().unwrap();
        assert_eq!(first.id.sequence, 0);
        assert_eq!(first.timestamp.ticks, 0);
        assert_eq!(adapter.next_due(), Ok(None));
        adapter.advance_to(9).unwrap();
        assert_eq!(adapter.next_due(), Ok(None));
        adapter.advance_to(10).unwrap();
        let second = adapter.next_due().unwrap().unwrap();
        let third = adapter.next_due().unwrap().unwrap();
        assert_eq!(second.id.sequence, 1);
        assert_eq!(third.id.sequence, 2);
        assert_eq!(second.value, fixtures[1].frame);
        assert_eq!(third.value, fixtures[2].frame);
        assert_eq!(second.timestamp.ticks, third.timestamp.ticks);
        assert_eq!(adapter.remaining(), 1);
        assert_eq!(adapter.next_due(), Ok(None));
        adapter.advance_to(25).unwrap();
        assert_eq!(adapter.next_due().unwrap().unwrap().id.sequence, 3);
        assert_eq!(adapter.remaining(), 0);
        assert_eq!(adapter.next_due(), Ok(None));
    }

    #[test]
    fn single_jump_and_incremental_steps_produce_identical_observations() {
        let fixtures = fixtures::mixed_classic_fd();
        let mut stepped = adapter(&fixtures);
        stepped.start().unwrap();
        let mut stepped_events = Vec::new();
        for tick in [0, 1, 9, 10, 20, 25] {
            stepped.advance_to(tick).unwrap();
            while let Some(event) = stepped.next_due().unwrap() {
                stepped_events.push(event);
            }
        }

        let mut jumped = adapter(&fixtures);
        jumped.start().unwrap();
        jumped.advance_to(25).unwrap();
        let mut jumped_events = Vec::new();
        while let Some(event) = jumped.next_due().unwrap() {
            jumped_events.push(event);
        }
        assert_eq!(stepped_events, jumped_events);
        assert_eq!(jumped_events.len(), 4);
        assert_eq!(jumped_events[1].timestamp.domain, ClockDomainId::new(9));
        assert_eq!(
            jumped_events[1].timestamp.origin,
            TimestampOrigin::Synthetic
        );
        assert_eq!(
            jumped_events[1].timestamp.quality,
            TimestampQuality::Uncorrelated
        );
        assert_eq!(jumped_events[1].timestamp.elapsed_ns(), Some(625));
        assert_eq!(jumped_events[2].direction, EventDirection::Tx);
    }

    #[test]
    fn source_identity_differs_even_with_same_fixture_and_clock_domain() {
        let fixtures = fixtures::mixed_classic_fd();
        let resolution = TimestampResolution::new(1, 1).unwrap();
        let mut first = VirtualCanAdapter::new(
            SourceId::new(1),
            ClockDomainId::new(3),
            resolution,
            &fixtures,
        )
        .unwrap();
        let mut second = VirtualCanAdapter::new(
            SourceId::new(2),
            ClockDomainId::new(3),
            resolution,
            &fixtures,
        )
        .unwrap();
        first.start().unwrap();
        second.start().unwrap();
        let a = first.next_due().unwrap().unwrap();
        let b = second.next_due().unwrap().unwrap();
        assert_ne!(a.id, b.id);
        assert_eq!(a.id.sequence, b.id.sequence);
        assert_eq!(a.value, b.value);
    }

    #[test]
    fn rejects_nonchronological_fixtures_without_modifying_them() {
        let mut fixtures = fixtures::mixed_classic_fd();
        fixtures.swap(0, 1);
        assert_eq!(
            VirtualCanAdapter::new(
                SourceId::new(1),
                ClockDomainId::new(1),
                TimestampResolution::new(1, 1).unwrap(),
                &fixtures
            )
            .err(),
            Some(VirtualCanError::UnsortedFixture { index: 1 })
        );
    }

    #[test]
    fn lifecycle_and_monotonic_clock_errors_do_not_emit_events() {
        let fixtures = fixtures::mixed_classic_fd();
        let mut adapter = adapter(&fixtures);
        assert_eq!(adapter.state(), VirtualCanState::Ready);
        assert_eq!(adapter.next_due(), Err(VirtualCanError::NotRunning));
        assert_eq!(adapter.advance_to(1), Err(VirtualCanError::NotRunning));
        assert_eq!(adapter.stop(), Err(VirtualCanError::NotRunning));
        adapter.start().unwrap();
        assert_eq!(adapter.start(), Err(VirtualCanError::AlreadyRunning));
        adapter.advance_to(10).unwrap();
        assert_eq!(
            adapter.advance_to(9),
            Err(VirtualCanError::ClockMovedBackwards)
        );
        assert_eq!(adapter.now_ticks(), 10);
        assert_eq!(adapter.remaining(), 4);
        adapter.advance_to(10).unwrap();
        adapter.stop().unwrap();
        assert_eq!(adapter.state(), VirtualCanState::Stopped);
        assert_eq!(adapter.next_due(), Err(VirtualCanError::Stopped));
        assert_eq!(adapter.advance_to(100), Err(VirtualCanError::Stopped));
        assert_eq!(adapter.start(), Err(VirtualCanError::Stopped));
        assert_eq!(adapter.stop(), Err(VirtualCanError::Stopped));
    }

    #[test]
    fn empty_fixture_script_and_exhausted_sequence_are_explicit() {
        let mut empty = adapter(&[]);
        empty.start().unwrap();
        empty.advance_to(u64::MAX).unwrap();
        assert_eq!(empty.next_due(), Ok(None));

        let fixtures = fixtures::mixed_classic_fd();
        let mut adapter = adapter(&fixtures);
        adapter.start().unwrap();
        adapter.next_sequence = u64::MAX;
        assert_eq!(adapter.next_due(), Err(VirtualCanError::SequenceExhausted));
        assert_eq!(adapter.remaining(), 4);
    }

    #[test]
    fn hot_path_observation_does_not_require_heap_owned_fields() {
        assert!(!std::mem::needs_drop::<VirtualCanFixture>());
        assert!(!std::mem::needs_drop::<Observed<CanFrame>>());
        let fixtures = fixtures::mixed_classic_fd();
        let mut adapter = adapter(&fixtures);
        adapter.start().unwrap();
        assert_eq!(
            adapter.next_due().unwrap().unwrap().value,
            fixtures[0].frame
        );
    }
}
