//! Wire-level time representations carried in secondary headers and
//! intra-packet time stamps.
//!
//! These types hold the raw fields as transmitted. Interpretation into an
//! absolute time of day is provided by `irig106-time`.
//!
//! # Requirement Traceability
//!
//! | Requirement | Description |
//! |-------------|-------------|
//! | L3-CH4-001, L3-CH4-005 | Chapter 4 Binary Weighted Time struct and parsing |
//! | L3-1588-001..003 | IEEE-1588 time struct, parsing, epoch nanos |

use crate::error::{Result, TypesError, require_len};

// ═══════════════════════════════════════════════════════════════════════
// Chapter 4 Binary Weighted Time
// ═══════════════════════════════════════════════════════════════════════

/// IRIG 106 Chapter 4 Binary Weighted Time.
///
/// Used in secondary headers (Packet Flag bits \[3:2\] = 0b00) and intra-packet
/// timestamps.
///
/// **Traces:** L3-CH4-001 ← L2-ABS-003 ← L1-ABS-002
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ch4BinaryTime {
    /// High-order 16 bits of the binary time word.
    pub high_order: u16,
    /// Low-order 16 bits of the binary time word.
    pub low_order: u16,
    /// Microsecond component.
    pub microseconds: u16,
}

impl Ch4BinaryTime {
    /// Parse from an 8-byte little-endian buffer: `[unused(2), high(2), low(2), usec(2)]`.
    ///
    /// **Traces:** L3-CH4-005
    pub fn from_secondary_bytes(buf: &[u8]) -> Result<Self> {
        require_len(buf, 8)?;
        Ok(Self {
            high_order: u16::from_le_bytes([buf[2], buf[3]]),
            low_order: u16::from_le_bytes([buf[4], buf[5]]),
            microseconds: u16::from_le_bytes([buf[6], buf[7]]),
        })
    }

    /// Parse from an 8-byte intra-packet time stamp buffer.
    ///
    /// **Traces:** L3-CH4-005
    pub fn from_intra_packet_bytes(buf: &[u8]) -> Result<Self> {
        Self::from_secondary_bytes(buf)
    }
}

// ═══════════════════════════════════════════════════════════════════════
// IEEE-1588 Time
// ═══════════════════════════════════════════════════════════════════════

/// IEEE-1588 Precision Time Protocol time value.
///
/// **Traces:** L3-1588-001 ← L2-ABS-005 ← L1-ABS-003
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ieee1588Time {
    /// Nanoseconds within the current second (0–999_999_999).
    pub nanoseconds: u32,
    /// Seconds since the IEEE-1588 epoch.
    pub seconds: u32,
}

impl Ieee1588Time {
    /// Parse from an 8-byte little-endian buffer: `[nanoseconds(4), seconds(4)]`.
    ///
    /// **Traces:** L3-1588-002
    #[inline]
    pub fn from_le_bytes(buf: &[u8]) -> Result<Self> {
        require_len(buf, 8)?;
        let nanoseconds = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let seconds = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
        if nanoseconds >= 1_000_000_000 {
            return Err(TypesError::OutOfRange {
                field: "ieee1588_nanoseconds",
                value: nanoseconds,
                max: 999_999_999,
            });
        }
        Ok(Self {
            nanoseconds,
            seconds,
        })
    }

    /// Total nanoseconds since the IEEE-1588 epoch.
    ///
    /// **Traces:** L3-1588-003 ← L2-ABS-006
    #[inline]
    pub fn to_nanos_since_epoch(&self) -> u64 {
        (self.seconds as u64) * 1_000_000_000 + (self.nanoseconds as u64)
    }
}

#[cfg(test)]
#[path = "binary_time_tests.rs"]
mod binary_time_tests;
