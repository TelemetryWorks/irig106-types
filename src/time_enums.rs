//! Time-related enumerations decoded from Chapter 10 channel-specific data
//! words and packet flags.
//!
//! # Requirement Traceability
//!
//! | Requirement | Description |
//! |-------------|-------------|
//! | L3-CSDW-008 | `TimeSource` enum |
//! | L3-CSDW-009 | `TimeFormat` enum |
//! | L3-CSDW-010 | `DateFormat` enum |
//! | L3-SEC-001  | `SecHdrTimeFormat` enum |

/// Time source applied to the recorder.
///
/// Decoded from bits \[3:0\] of the Time Data Format 1 CSDW. Version-specific
/// interpretation (value 3 is "None" in 106-04 but "GPS" from 106-05 onward)
/// is handled by `irig106-time`.
///
/// **Traces:** L3-CSDW-008 ← L2-CSDW-003 ← L1-CSDW-002
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeSource {
    /// Internal time source (0x0).
    Internal,
    /// External time source (0x1).
    External,
    /// Internal RTC (0x2).
    InternalRtc,
    /// GPS time (0x3).
    Gps,
    /// No time source (0xF).
    None,
    /// Reserved or unrecognized value.
    Reserved(u8),
}

impl TimeSource {
    /// Decode from the 4-bit raw field value using the current (106-05+) mapping.
    #[inline]
    pub fn from_raw(val: u8) -> Self {
        match val {
            0 => TimeSource::Internal,
            1 => TimeSource::External,
            2 => TimeSource::InternalRtc,
            3 => TimeSource::Gps,
            0xF => TimeSource::None,
            other => TimeSource::Reserved(other),
        }
    }
}

/// Format of the external time source.
///
/// Decoded from bits \[7:4\] of the Time Data Format 1 CSDW.
///
/// **Traces:** L3-CSDW-009 ← L2-CSDW-005 ← L1-CSDW-003
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeFormat {
    /// IRIG-B (0x0).
    IrigB,
    /// IRIG-A (0x1).
    IrigA,
    /// IRIG-G (0x2).
    IrigG,
    /// Internal RTC (0x3).
    Rtc,
    /// UTC from GPS (0x4).
    Utc,
    /// GPS time (0x5).
    Gps,
    /// Reserved or unrecognized value.
    Reserved(u8),
}

impl TimeFormat {
    /// Decode from the 4-bit raw field value.
    #[inline]
    pub fn from_raw(val: u8) -> Self {
        match val {
            0 => TimeFormat::IrigB,
            1 => TimeFormat::IrigA,
            2 => TimeFormat::IrigG,
            3 => TimeFormat::Rtc,
            4 => TimeFormat::Utc,
            5 => TimeFormat::Gps,
            other => TimeFormat::Reserved(other),
        }
    }
}

/// Date representation format in the time message.
///
/// **Traces:** L3-CSDW-010 ← L2-CSDW-007 ← L1-CSDW-005
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DateFormat {
    /// Day-of-year format (bit 9 = 0).
    DayOfYear,
    /// Day-month-year format (bit 9 = 1).
    DayMonthYear,
}

/// Time format discriminant derived from Packet Flag bits \[3:2\].
///
/// Selects the layout of the secondary header time and of intra-packet
/// time stamps.
///
/// **Traces:** L3-SEC-001
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecHdrTimeFormat {
    /// IRIG 106 Chapter 4 Binary Weighted Time.
    Ch4,
    /// IEEE-1588 Precision Time Protocol.
    Ieee1588,
    /// Extended Relative Time Counter (64-bit).
    Ertc,
    /// Reserved/unknown format.
    Reserved(u8),
}

impl SecHdrTimeFormat {
    /// Decode from Packet Flag bits \[3:2\].
    #[inline]
    pub fn from_packet_flags(flags: u8) -> Self {
        match (flags >> 2) & 0x03 {
            0 => SecHdrTimeFormat::Ch4,
            1 => SecHdrTimeFormat::Ieee1588,
            2 => SecHdrTimeFormat::Ertc,
            other => SecHdrTimeFormat::Reserved(other),
        }
    }
}

#[cfg(test)]
#[path = "time_enums_tests.rs"]
mod time_enums_tests;
