//! Core foundations for CANexus-Studio.
//!
//! Transport-specific event representations will live in dedicated modules;
//! the core must not assume that every future transport is CAN.

pub mod can;
pub mod time;

/// Identifies a logical data source within a running application.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SourceId(u64);

impl SourceId {
    /// Creates a source identifier from a stable process-local value.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the underlying identifier.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::SourceId;

    #[test]
    fn source_id_round_trip() {
        let source = SourceId::new(42);
        assert_eq!(source.get(), 42);
        assert_eq!(source, SourceId::new(42));
        assert_ne!(source, SourceId::new(43));
    }
}
