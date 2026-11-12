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
/// timestamps. The binary high/low words form a full 32-bit count in 10 ms
/// units: `high_order × 65536 + low_order` (106-24R1 Chapter 4 §4.7).
/// No bit of either word is masked, including the top bit of the low word.
///
/// This is a raw representation: parsing checks buffer length, ignores the
/// reserved word, and preserves the three time words without validating them.
/// Consumers handle reserved-bit checks and conversion to absolute time.
///
/// **Traces:** L3-CH4-001 ← L2-ABS-003 ← L1-ABS-002
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ch4BinaryTime {
    /// High-order 16 bits of the binary count; one unit is 655.36 seconds.
    ///
    /// 106-24R1 Chapter 4 §4.7, Figures 4-4/4-5.
    pub high_order: u16,
    /// Low-order 16 bits of the binary count; one unit is 10 milliseconds.
    ///
    /// All 16 bits are retained, including bit 15 (327.68 seconds).
    pub low_order: u16,
    /// Raw microsecond word; a conforming value is 0–9999.
    ///
    /// Figures 4-4/4-5 show its two leading positions zero; §4.7 defines
    /// 1 microsecond resolution and reset at 10 ms. Parsing does not mask
    /// or validate this word.
    pub microseconds: u16,
}

impl Ch4BinaryTime {
    /// Parse `[reserved(2), microseconds(2), low_order(2), high_order(2)]`.
    ///
    /// Each word is little-endian, per 106-24R1 §11.1 and Figure 11-4
    /// (printed page 11-10), with the low 16-bit half of each 32-bit row
    /// appearing first in the byte stream. The reserved word is ignored;
    /// all bits of the time words are preserved. Only length is validated.
    ///
    /// **Traces:** L3-CH4-005
    pub fn from_secondary_bytes(buf: &[u8]) -> Result<Self> {
        require_len(buf, 8)?;
        Ok(Self {
            high_order: u16::from_le_bytes([buf[6], buf[7]]),
            low_order: u16::from_le_bytes([buf[4], buf[5]]),
            microseconds: u16::from_le_bytes([buf[2], buf[3]]),
        })
    }

    /// Parse an 8-byte intra-packet time stamp in the same layout as
    /// [`Self::from_secondary_bytes`].
    ///
    /// Applies when the packet flags select Chapter 4 secondary-header time
    /// for IPTS (106-24R1 §11.2.1.1 g and §11.2.1.3 b).
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
