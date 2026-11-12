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
/// Decoded from bits \[3:0\] of the Time Data Format 1 CSDW
/// (106-24R1 §11.2.3.2 a, Figure 11-12). This SRC mapping applies from
/// 106-05 onward; values 3–14 are reserved. In 106-04 §10.6.3.3,
/// Figure 10-15, these bits are EXT: bit 0 indicates external time present,
/// and bits 3–1 are reserved. Edition handling belongs to `irig106-time`.
///
/// **Traces:** L3-CSDW-008 ← L2-CSDW-003 ← L1-CSDW-002
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeSource {
    /// Internal time source (0x0).
    Internal,
    /// External time source (0x1).
    External,
    /// Internal from RMM: time derived from the clock in the RMM (0x2).
    InternalFromRmm,
    /// No time source (0xF).
    None,
    /// Reserved or unrecognized value.
    Reserved(u8),
}

impl TimeSource {
    /// Decode the extracted 106-05-and-later SRC field, preserving reserved values.
    #[inline]
    pub fn from_raw(val: u8) -> Self {
        match val {
            0 => TimeSource::Internal,
            1 => TimeSource::External,
            2 => TimeSource::InternalFromRmm,
            0xF => TimeSource::None,
            other => TimeSource::Reserved(other),
        }
    }
}

/// Format of the external time source.
///
/// Decoded from bits \[7:4\] of the Time Data Format 1 CSDW using the
/// 106-05-and-later mapping. In 106-04 §10.6.3.3, Figure 10-15, FMT
/// 0x6–0xF is reserved; NONE at 0xF applies from 106-05 onward.
/// Edition handling belongs to `irig106-time`.
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
    /// Native GPS time (0x5).
    Gps,
    /// NONE (0xF) from 106-05 onward: the time packet payload is invalid.
    ///
    /// 106-24R1 §11.2.3.2 a, Figure 11-12. Consumers must not interpret
    /// the payload as a valid time reference.
    None,
    /// Reserved or unrecognized value.
    Reserved(u8),
}

impl TimeFormat {
    /// Decode the extracted FMT field using the 106-05-and-later mapping.
    #[inline]
    pub fn from_raw(val: u8) -> Self {
        match val {
            0 => TimeFormat::IrigB,
            1 => TimeFormat::IrigA,
            2 => TimeFormat::IrigG,
            3 => TimeFormat::Rtc,
            4 => TimeFormat::Utc,
            5 => TimeFormat::Gps,
            0xF => TimeFormat::None,
            other => TimeFormat::Reserved(other),
        }
    }
}

/// Source of an internal IRIG time code generator (ITS).
///
/// Decoded from the extracted bits 15–12 of the Format 1 CSDW, per
/// 106-24R1 §11.2.3.2 a / Figure 11-12. Defined from 106-17 onward,
/// only when FMT is IRIG-A, B, or G. Earlier editions reserve these bits;
/// callers must establish edition and format before interpreting them.
/// In particular, zero in an older recording does not report freewheeling.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IrigTimeSource {
    /// Freewheeling with no time source or loss of source (0x0).
    Freewheeling,
    /// Freewheeling from the .TIME command (0x1).
    TimeCommand,
    /// Freewheeling from RMM time (0x2).
    Rmm,
    /// Locked to an external IRIG time signal (0x3).
    ExternalIrig,
    /// Locked to external GPS (0x4).
    ExternalGps,
    /// Locked to external NTP (0x5).
    ExternalNtp,
    /// Locked to external PTP (0x6).
    ExternalPtp,
    /// Locked to embedded time from an input track/channel (0x7).
    Embedded,
    /// Reserved (0x8–0xF) or unrecognized value, preserved unchanged.
    Reserved(u8),
}

impl IrigTimeSource {
    /// Decode an extracted ITS field, without deciding its applicability.
    #[inline]
    pub fn from_raw(val: u8) -> Self {
        match val {
            0 => Self::Freewheeling,
            1 => Self::TimeCommand,
            2 => Self::Rmm,
            3 => Self::ExternalIrig,
            4 => Self::ExternalGps,
            5 => Self::ExternalNtp,
            6 => Self::ExternalPtp,
            7 => Self::Embedded,
            other => Self::Reserved(other),
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
