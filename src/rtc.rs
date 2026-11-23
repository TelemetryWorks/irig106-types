//! Relative Time Counters: the 48-bit [`Rtc`] and the 64-bit [`Ertc`].
//!
//! The RTC is a free-running 10 MHz counter providing 100 ns resolution.
//! It occupies 6 bytes (48 bits) in the primary packet header at bytes [16..22].
//!
//! Baseline: IRIG 106-24R1 §11.2.1.1 (primary-header RTC) and
//! §11.2.1.1 g / Figure 11-6 (ERTC). Earlier Chapter 10 recordings also
//! carry the primary RTC; support for a secondary format is edition-dependent.
//! These counters carry no UTC/TAI epoch or wrap/session interpretation.
//!
//! # Requirement Traceability
//!
//! | Requirement | Description |
//! |-------------|-------------|
//! | L3-RTC-001  | Newtype `Rtc(u64)` with 48-bit invariant |
//! | L3-RTC-002  | `MASK_48` constant |
//! | L3-RTC-003  | `NANOS_PER_TICK` constant |
//! | L3-RTC-004  | `Rtc::ZERO` |
//! | L3-RTC-005  | `Rtc::MAX` |
//! | L3-RTC-006  | `from_le_bytes` |
//! | L3-RTC-007  | `from_raw` with masking |
//! | L3-RTC-008  | `as_raw` |
//! | L3-RTC-009  | `elapsed_ticks` with 48-bit wrap handling |
//! | L3-RTC-010  | `elapsed_nanos` |
//! | L3-RTC-011  | `to_nanos` |
//! | L3-RTC-012  | `Ord`/`PartialOrd` implementation |
//! | L3-RTC-013  | Standard derives |
//! | L3-ERTC-001..003 | 64-bit Extended RTC |

use core::cmp::Ordering;

use crate::error::{Result, require_len};

/// Bitmask for the lower 48 bits.
///
/// **Traces:** L3-RTC-002
const MASK_48: u64 = 0x0000_FFFF_FFFF_FFFF;

/// Nanoseconds per RTC tick (10 MHz clock → 100 ns/tick).
///
/// **Traces:** L3-RTC-003
const NANOS_PER_TICK: u64 = 100;

/// A 48-bit Relative Time Counter value.
///
/// The RTC is the fundamental time reference in IRIG 106 Chapter 10 packet
/// headers. It runs at 10 MHz (100 ns per tick) and is represented as a 48-bit
/// unsigned integer stored little-endian in the 6-byte header field.
///
/// Byte construction and [`Self::from_raw`] produce values in
/// `0..=0x0000_FFFF_FFFF_FFFF` (IRIG 106-24R1 §11.2.1.1).
/// With `serde`, derived deserialization accepts any `u64` without masking;
/// callers must validate or explicitly use `from_raw` to recover that domain.
/// `Ord` compares raw counts numerically, not chronologically across a wrap.
/// Nanosecond methods assume a 48-bit value; they can overflow for an
/// out-of-domain deserialized value.
///
/// ```
/// use irig106_types::Rtc;
/// let before = Rtc::MAX;
/// assert_eq!(before.elapsed_ticks(Rtc::ZERO), 1);
/// assert_eq!(Rtc::from_raw(1_u64 << 48), Rtc::ZERO);
/// ```
///
/// **Traces:** L3-RTC-001 ← L2-RTC-001 ← L1-RTC-001
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rtc(u64);

impl Rtc {
    /// RTC value of zero.
    ///
    /// **Traces:** L3-RTC-004 ← L2-RTC-010
    pub const ZERO: Rtc = Rtc(0);

    /// Maximum 48-bit RTC value (2^48 − 1).
    ///
    /// Represents approximately 325.8 days of elapsed time at 100 ns resolution.
    ///
    /// **Traces:** L3-RTC-005 ← L2-RTC-011, L2-RTC-009
    pub const MAX: Rtc = Rtc(MASK_48);

    /// Construct an RTC from 6 little-endian bytes.
    ///
    /// This corresponds to bytes [16..22] of an IRIG 106 Chapter 10 primary
    /// packet header.
    ///
    /// **Traces:** L3-RTC-006 ← L2-RTC-002 ← L1-RTC-002
    #[inline]
    pub fn from_le_bytes(bytes: [u8; 6]) -> Self {
        let raw = u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], 0, 0,
        ]);
        Rtc(raw)
    }

    /// Construct an RTC from a raw `u64`, masking to 48 bits.
    ///
    /// **Traces:** L3-RTC-007 ← L2-RTC-003 ← L1-RTC-001
    #[inline]
    pub fn from_raw(value: u64) -> Self {
        Rtc(value & MASK_48)
    }

    /// Return the raw 48-bit tick count.
    ///
    /// **Traces:** L3-RTC-008 ← L2-RTC-004 ← L1-RTC-001
    #[inline]
    pub fn as_raw(self) -> u64 {
        self.0
    }

    /// Compute forward elapsed ticks modulo 2^48, in `0..=2^48 - 1`.
    ///
    /// Assumes fewer than one complete counter period elapsed. This cannot
    /// distinguish a backwards observation from a forward wrap or count
    /// multiple wraps; the consumer chooses ordering and session policy.
    ///
    /// **Traces:** L3-RTC-009 ← L2-RTC-005 ← L1-RTC-003
    #[inline]
    pub fn elapsed_ticks(self, later: Rtc) -> u64 {
        later.0.wrapping_sub(self.0) & MASK_48
    }

    /// Compute forward elapsed nanoseconds modulo the 48-bit counter period.
    ///
    /// Multiplies [`Self::elapsed_ticks`] by 100, yielding
    /// `0..=28_147_497_671_065_500` ns. The same wrap/ordering caveats apply.
    ///
    /// **Traces:** L3-RTC-010 ← L2-RTC-006 ← L1-RTC-003, L1-RTC-004
    #[inline]
    pub fn elapsed_nanos(self, later: Rtc) -> u64 {
        self.elapsed_ticks(later) * NANOS_PER_TICK
    }

    /// Scale the raw count by 100 to nanoseconds relative to counter zero.
    ///
    /// For a 48-bit value the range is `0..=28_147_497_671_065_500` ns.
    /// This assigns no absolute epoch and does not unwrap the counter.
    ///
    /// **Traces:** L3-RTC-011 ← L2-RTC-007 ← L1-RTC-004
    #[inline]
    pub fn to_nanos(self) -> u64 {
        self.0 * NANOS_PER_TICK
    }

    /// Encode the 48-bit RTC value as 6 little-endian bytes.
    ///
    /// This is the inverse of `from_le_bytes`.
    ///
    /// **Traces:** GAP-11
    #[inline]
    pub fn to_le_bytes(self) -> [u8; 6] {
        let bytes = self.0.to_le_bytes();
        [bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]]
    }
}

/// **Traces:** L3-RTC-012 ← L2-RTC-008
impl PartialOrd for Rtc {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// **Traces:** L3-RTC-012 ← L2-RTC-008
impl Ord for Rtc {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

/// 64-bit Extended Relative Time Counter.
///
/// Used in secondary headers (Packet Flag bits \[3:2\] = 0b10) and
/// intra-packet time stamps. It ticks at 1 GHz (1 ns/tick), spanning all 64
/// bits. The primary-header RTC is synchronized as RTC = ERTC/100
/// (106-24R1 §11.2.1.1 g, Figure 11-6).
///
/// **Traces:** L3-ERTC-001 ← L2-ABS-007 ← L1-ABS-004
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ertc(u64);

impl Ertc {
    /// Construct from any `u64` tick count (`0..=u64::MAX`), without masking.
    #[inline]
    pub fn from_raw(value: u64) -> Self {
        Ertc(value)
    }

    /// Unpack the first eight little-endian bytes (Figure 11-6).
    ///
    /// Returns [`crate::TypesError::BufferTooShort`] for fewer than eight
    /// bytes; trailing bytes are ignored. Every 64-bit pattern is retained.
    ///
    /// **Traces:** L3-ERTC-002
    #[inline]
    pub fn from_le_bytes(buf: &[u8]) -> Result<Self> {
        require_len(buf, 8)?;
        Ok(Ertc(u64::from_le_bytes([
            buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
        ])))
    }

    /// Raw tick count in `0..=u64::MAX`, at one nanosecond per tick.
    #[inline]
    pub fn as_raw(self) -> u64 {
        self.0
    }

    /// Convert to nanoseconds (one nanosecond per tick).
    ///
    /// The result is in `0..=u64::MAX` despite the widened `u128` return type.
    /// No epoch, wrap correction, or primary RTC scaling is applied.
    ///
    /// **Traces:** L3-ERTC-003 ← L2-ABS-008
    #[inline]
    pub fn to_nanos(self) -> u128 {
        self.0 as u128
    }
}

#[cfg(test)]
#[path = "rtc_tests.rs"]
mod rtc_tests;
