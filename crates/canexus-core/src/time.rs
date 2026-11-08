//! Explicit time-domain and provenance metadata for captured observations.
//!
//! Raw clock ticks are never assumed to be globally synchronized or UTC.

/// Distinguishes independent clock counters within a capture.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ClockDomainId(u64);

impl ClockDomainId {
    /// Creates a clock domain identifier.
    #[must_use]
    pub const fn new(value: u64) -> Self { Self(value) }
    /// Returns the underlying identifier.
    #[must_use]
    pub const fn get(self) -> u64 { self.0 }
}

/// Tick resolution expressed as an exact rational number of nanoseconds.
///
/// A single tick represents `numerator_ns / denominator` nanoseconds.
/// Conversion to wall-clock time and synchronization are separate operations.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TimestampResolution {
    numerator_ns: u64,
    denominator: u64,
}

impl TimestampResolution {
    /// Creates a strictly positive rational resolution.
    #[must_use]
    pub const fn new(numerator_ns: u64, denominator: u64) -> Option<Self> {
        if numerator_ns == 0 || denominator == 0 {
            None
        } else {
            Some(Self { numerator_ns, denominator })
        }
    }

    /// Number of nanoseconds in the numerator.
    #[must_use]
    pub const fn numerator_ns(self) -> u64 { self.numerator_ns }

    /// Tick resolution denominator.
    #[must_use]
    pub const fn denominator(self) -> u64 { self.denominator }
}

/// Where a timestamp was measured or generated.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TimestampOrigin {
    /// Timestamp supplied by the adapter/device hardware.
    Hardware,
    /// Timestamp assigned by host software.
    Host,
    /// Deterministic or reconstructed timestamp (e.g. replay).
    Synthetic,
}

/// Describes whether clock correlation has been established.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TimestampQuality {
    /// Correlation to other clocks has not been established.
    Uncorrelated,
    /// Mapped to a reference clock with an estimated error bound.
    Estimated,
    /// Verified reference-clock mapping exists; uncertainty is tracked separately.
    Synchronized,
    /// Clock reset, wrap, or otherwise invalidated mapping is known.
    Discontinuous,
}

/// Original clock ticks and their metadata, without implicit conversion.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct EventTimestamp {
    /// Raw counter value within the clock domain.
    pub ticks: u64,
    /// Clock domain for interpreting the counter.
    pub domain: ClockDomainId,
    /// Duration of one counter tick.
    pub resolution: TimestampResolution,
    /// Capture origin.
    pub origin: TimestampOrigin,
    /// Clock correlation status.
    pub quality: TimestampQuality,
}

impl EventTimestamp {
    /// Creates a timestamp with explicit provenance.
    #[must_use]
    pub const fn new(
        ticks: u64,
        domain: ClockDomainId,
        resolution: TimestampResolution,
        origin: TimestampOrigin,
        quality: TimestampQuality,
    ) -> Self {
        Self { ticks, domain, resolution, origin, quality }
    }

    /// Computes elapsed nanoseconds from the domain's zero tick, if representable.
    ///
    /// This is not an absolute UTC time and must not be compared across
    /// independent clock domains without a separate clock-correlation model.
    #[must_use]
    pub fn elapsed_ns(self) -> Option<u64> {
        let product = u128::from(self.ticks)
            .checked_mul(u128::from(self.resolution.numerator_ns))?;
        u64::try_from(product / u128::from(self.resolution.denominator)).ok()
    }

    /// Returns a signed tick delta only within the same clock domain and resolution.
    #[must_use]
    pub fn delta_ticks(self, earlier: Self) -> Option<i128> {
        if self.domain != earlier.domain || self.resolution != earlier.resolution
            || self.quality == TimestampQuality::Discontinuous
            || earlier.quality == TimestampQuality::Discontinuous
        {
            return None;
        }
        Some(i128::from(self.ticks) - i128::from(earlier.ticks))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_zero_resolution() {
        assert!(TimestampResolution::new(0, 1).is_none());
        assert!(TimestampResolution::new(1, 0).is_none());
        assert_eq!(TimestampResolution::new(1, 1_000_000).unwrap().denominator(), 1_000_000);
    }

    #[test]
    fn preserves_hardware_provenance_and_precise_scaling() {
        let resolution = TimestampResolution::new(125, 2).unwrap();
        let timestamp = EventTimestamp::new(
            4, ClockDomainId::new(3), resolution,
            TimestampOrigin::Hardware, TimestampQuality::Uncorrelated
        );
        assert_eq!(timestamp.elapsed_ns(), Some(250));
        assert_eq!(timestamp.domain.get(), 3);
        assert_eq!(timestamp.origin, TimestampOrigin::Hardware);
        assert_eq!(timestamp.quality, TimestampQuality::Uncorrelated);
        assert_eq!(resolution.numerator_ns(), 125);
    }

    #[test]
    fn refuses_cross_domain_deltas_and_discontinuities() {
        let res = TimestampResolution::new(1, 1).unwrap();
        let base = EventTimestamp::new(
            10, ClockDomainId::new(1), res,
            TimestampOrigin::Host, TimestampQuality::Uncorrelated
        );
        let later = EventTimestamp::new(20, ClockDomainId::new(1), res, base.origin, base.quality);
        assert_eq!(later.delta_ticks(base), Some(10));
        assert_eq!(base.delta_ticks(later), Some(-10));
        let other = EventTimestamp { domain: ClockDomainId::new(2), ..later };
        assert_eq!(other.delta_ticks(base), None);
        let reset = EventTimestamp { quality: TimestampQuality::Discontinuous, ..later };
        assert_eq!(reset.delta_ticks(base), None);
        let incompatible = EventTimestamp {
            resolution: TimestampResolution::new(2, 1).unwrap(), ..later
        };
        assert_eq!(incompatible.delta_ticks(base), None);
    }

    #[test]
    fn overflow_is_explicit_not_wrapped() {
        let res = TimestampResolution::new(u64::MAX, 1).unwrap();
        let stamp = EventTimestamp::new(
            u64::MAX, ClockDomainId::new(0), res,
            TimestampOrigin::Synthetic, TimestampQuality::Estimated
        );
        assert_eq!(stamp.elapsed_ns(), None);
    }
}
