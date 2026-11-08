//! Allocation-free, validated CAN Classic and CAN FD frame representation.
//!
//! CAN-specific payloads are deliberately separate from the transport-neutral
//! event envelope. A frame owns at most 64 bytes and never allocates.

/// Maximum identifier for an 11-bit CAN frame.
pub const MAX_STANDARD_ID: u16 = 0x7ff;
/// Maximum identifier for a 29-bit CAN frame.
pub const MAX_EXTENDED_ID: u32 = 0x1fff_ffff;

/// An identifier whose encoding (11-bit or 29-bit) is preserved.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CanId {
    /// An 11-bit identifier.
    Standard(u16),
    /// A 29-bit identifier, including values that also fit in 11 bits.
    Extended(u32),
}

impl CanId {
    /// Creates a standard ID only when it fits in 11 bits.
    #[must_use]
    pub const fn standard(value: u16) -> Option<Self> {
        if value <= MAX_STANDARD_ID {
            Some(Self::Standard(value))
        } else {
            None
        }
    }

    /// Creates an extended ID only when it fits in 29 bits.
    #[must_use]
    pub const fn extended(value: u32) -> Option<Self> {
        if value <= MAX_EXTENDED_ID {
            Some(Self::Extended(value))
        } else {
            None
        }
    }

    /// Returns the numeric arbitration identifier.
    #[must_use]
    pub const fn raw(self) -> u32 {
        match self {
            Self::Standard(value) => value as u32,
            Self::Extended(value) => value,
        }
    }

    /// Reports whether this ID uses the 29-bit wire representation.
    #[must_use]
    pub const fn is_extended(self) -> bool {
        matches!(self, Self::Extended(_))
    }
}

/// On-wire CAN frame format.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CanFormat {
    /// Classic CAN, at most eight data bytes.
    Classic,
    /// CAN FD, at most 64 data bytes.
    Fd,
}

/// Wire-level attributes independent of the arbitration ID.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CanFrameFlags {
    /// The actual wire format.
    pub format: CanFormat,
    /// Bit rate switching; only valid for FD data frames.
    pub bit_rate_switch: bool,
    /// Error state indicator; only valid for FD data frames.
    pub error_state_indicator: bool,
    /// Remote transmission request; only valid for Classic CAN.
    pub remote: bool,
}

impl CanFrameFlags {
    /// Ordinary Classic CAN data frame flags.
    pub const CLASSIC: Self = Self {
        format: CanFormat::Classic,
        bit_rate_switch: false,
        error_state_indicator: false,
        remote: false,
    };

    /// Ordinary CAN FD data frame flags without BRS or ESI.
    pub const FD: Self = Self {
        format: CanFormat::Fd,
        bit_rate_switch: false,
        error_state_indicator: false,
        remote: false,
    };
}

/// Input validation failures for CAN frame construction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CanFrameError {
    /// DLC is outside the 4-bit CAN DLC encoding.
    InvalidDlc,
    /// BRS or ESI appeared on a Classic frame.
    ClassicFdFlags,
    /// A remote frame was specified as CAN FD.
    FdRemoteFrame,
    /// A remote frame contains data.
    RemoteFrameWithData,
    /// The data size does not match the format and encoded DLC.
    PayloadLengthMismatch,
}

/// Maps a wire DLC to an actual data length, with CAN FD's non-linear mapping.
///
/// For Classic CAN the only valid DLC values are 0 through 8. The return
/// value for remote frames denotes the requested length, not stored bytes.
#[must_use]
pub const fn payload_len_from_dlc(format: CanFormat, dlc: u8) -> Option<u8> {
    match format {
        CanFormat::Classic => {
            if dlc <= 8 { Some(dlc) } else { None }
        }
        CanFormat::Fd => match dlc {
            0..=8 => Some(dlc),
            9 => Some(12),
            10 => Some(16),
            11 => Some(20),
            12 => Some(24),
            13 => Some(32),
            14 => Some(48),
            15 => Some(64),
            _ => None,
        },
    }
}

/// Fixed-capacity CAN frame, with no per-frame allocation.
///
/// All fields are private to preserve validated invariants. Bytes beyond
/// `data_len` are zero-filled and are not part of the wire payload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CanFrame {
    id: CanId,
    flags: CanFrameFlags,
    dlc: u8,
    data_len: u8,
    data: [u8; 64],
}

impl CanFrame {
    /// Constructs a frame from exact on-wire DLC and payload bytes.
    ///
    /// FD lengths of 9, 10 or 11 bytes, for example, are invalid until
    /// explicitly padded to a representable wire length (12 bytes).
    pub fn new(
        id: CanId,
        flags: CanFrameFlags,
        dlc: u8,
        payload: &[u8],
    ) -> Result<Self, CanFrameError> {
        let expected_len =
            payload_len_from_dlc(flags.format, dlc).ok_or(CanFrameError::InvalidDlc)?;
        match flags.format {
            CanFormat::Classic => {
                if flags.bit_rate_switch || flags.error_state_indicator {
                    return Err(CanFrameError::ClassicFdFlags);
                }
            }
            CanFormat::Fd => {
                if flags.remote {
                    return Err(CanFrameError::FdRemoteFrame);
                }
            }
        }
        if flags.remote && !payload.is_empty() {
            return Err(CanFrameError::RemoteFrameWithData);
        }
        if !flags.remote && payload.len() != usize::from(expected_len) {
            return Err(CanFrameError::PayloadLengthMismatch);
        }
        let mut data = [0; 64];
        data[..payload.len()].copy_from_slice(payload);
        Ok(Self {
            id,
            flags,
            dlc,
            data_len: payload.len() as u8,
            data,
        })
    }

    /// Returns the ID including standard/extended representation.
    #[must_use]
    pub const fn id(&self) -> CanId { self.id }

    /// Returns frame format and wire flags.
    #[must_use]
    pub const fn flags(&self) -> CanFrameFlags { self.flags }

    /// Returns the unmodified on-wire DLC.
    #[must_use]
    pub const fn dlc(&self) -> u8 { self.dlc }

    /// Returns the number of stored data bytes (zero for remote frames).
    #[must_use]
    pub const fn data_len(&self) -> u8 { self.data_len }

    /// Returns only the meaningful payload bytes.
    #[must_use]
    pub fn data(&self) -> &[u8] { &self.data[..usize::from(self.data_len)] }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifier_boundaries_and_format_are_preserved() {
        assert_eq!(CanId::standard(0), Some(CanId::Standard(0)));
        assert_eq!(CanId::standard(0x7ff).unwrap().raw(), 0x7ff);
        assert_eq!(CanId::standard(0x800), None);
        assert_eq!(CanId::extended(0x1fff_ffff), Some(CanId::Extended(0x1fff_ffff)));
        assert_eq!(CanId::extended(0x2000_0000), None);
        assert_ne!(CanId::Standard(42), CanId::Extended(42));
        assert!(CanId::Extended(42).is_extended());
    }

    #[test]
    fn all_fd_dlc_values_map_to_exact_payload_lengths() {
        let lengths = [0, 1, 2, 3, 4, 5, 6, 7, 8, 12, 16, 20, 24, 32, 48, 64];
        for (dlc, length) in lengths.iter().enumerate() {
            assert_eq!(payload_len_from_dlc(CanFormat::Fd, dlc as u8), Some(*length));
            let data = vec![0x5a; usize::from(*length)];
            let frame = CanFrame::new(CanId::Standard(1), CanFrameFlags::FD, dlc as u8, &data)
                .expect("valid DLC and payload");
            assert_eq!(frame.dlc(), dlc as u8);
            assert_eq!(frame.data_len(), *length);
            assert_eq!(frame.data(), data);
        }
        assert_eq!(payload_len_from_dlc(CanFormat::Fd, 16), None);
    }

    #[test]
    fn classic_lengths_and_empty_payload() {
        for dlc in 0..=8 {
            let data = vec![0xaa; dlc as usize];
            assert!(CanFrame::new(CanId::Standard(0), CanFrameFlags::CLASSIC, dlc, &data).is_ok());
        }
        assert_eq!(payload_len_from_dlc(CanFormat::Classic, 9), None);
    }

    #[test]
    fn reject_mismatched_dlc_and_illegal_flag_combinations() {
        let id = CanId::Standard(0);
        assert_eq!(
            CanFrame::new(id, CanFrameFlags::FD, 9, &[0; 9]),
            Err(CanFrameError::PayloadLengthMismatch)
        );
        assert_eq!(
            CanFrame::new(id, CanFrameFlags::CLASSIC, 9, &[]),
            Err(CanFrameError::InvalidDlc)
        );
        assert_eq!(
            CanFrame::new(id, CanFrameFlags { bit_rate_switch: true, ..CanFrameFlags::CLASSIC }, 0, &[]),
            Err(CanFrameError::ClassicFdFlags)
        );
        assert_eq!(
            CanFrame::new(id, CanFrameFlags { error_state_indicator: true, ..CanFrameFlags::CLASSIC }, 0, &[]),
            Err(CanFrameError::ClassicFdFlags)
        );
        assert_eq!(
            CanFrame::new(id, CanFrameFlags { remote: true, ..CanFrameFlags::FD }, 0, &[]),
            Err(CanFrameError::FdRemoteFrame)
        );
    }

    #[test]
    fn remote_classic_frame_preserves_requested_dlc_without_data() {
        let flags = CanFrameFlags { remote: true, ..CanFrameFlags::CLASSIC };
        let frame = CanFrame::new(CanId::Standard(5), flags, 8, &[]).unwrap();
        assert_eq!(frame.dlc(), 8);
        assert_eq!(frame.data_len(), 0);
        assert!(frame.data().is_empty());
        assert_eq!(
            CanFrame::new(CanId::Standard(5), flags, 1, &[1]),
            Err(CanFrameError::RemoteFrameWithData)
        );
    }

    #[test]
    fn flags_and_payload_are_owned_and_copyable() {
        let flags = CanFrameFlags {
            bit_rate_switch: true,
            error_state_indicator: true,
            ..CanFrameFlags::FD
        };
        let mut input = [1, 2, 3, 4];
        let frame = CanFrame::new(CanId::Extended(0x123), flags, 4, &input).unwrap();
        input[0] = 99;
        assert_eq!(frame.data(), &[1, 2, 3, 4]);
        assert_eq!(frame.flags(), flags);
        assert_eq!(frame.id(), CanId::Extended(0x123));
    }
}
