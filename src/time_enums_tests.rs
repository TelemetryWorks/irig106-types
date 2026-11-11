//! Unit tests for the `time_enums` module.
//!
//! # Test Documentation
//!
//! | Test | Validates | Traces |
//! |------|-----------|--------|
//! | `time_source_from_raw` | All defined values and reserved fallback | L3-CSDW-008 |
//! | `time_format_from_raw` | All defined values and reserved fallback | L3-CSDW-009 |
//! | `sec_hdr_time_format_from_flags` | Bits \[3:2\] select format, other bits ignored | L3-SEC-001 |

use super::*;

#[test]
fn time_source_from_raw() {
    assert_eq!(TimeSource::from_raw(0), TimeSource::Internal);
    assert_eq!(TimeSource::from_raw(1), TimeSource::External);
    assert_eq!(TimeSource::from_raw(2), TimeSource::InternalRtc);
    assert_eq!(TimeSource::from_raw(3), TimeSource::Gps);
    assert_eq!(TimeSource::from_raw(0xF), TimeSource::None);
    assert_eq!(TimeSource::from_raw(7), TimeSource::Reserved(7));
}

#[test]
fn time_format_from_raw() {
    assert_eq!(TimeFormat::from_raw(0), TimeFormat::IrigB);
    assert_eq!(TimeFormat::from_raw(1), TimeFormat::IrigA);
    assert_eq!(TimeFormat::from_raw(2), TimeFormat::IrigG);
    assert_eq!(TimeFormat::from_raw(3), TimeFormat::Rtc);
    assert_eq!(TimeFormat::from_raw(4), TimeFormat::Utc);
    assert_eq!(TimeFormat::from_raw(5), TimeFormat::Gps);
    assert_eq!(TimeFormat::from_raw(9), TimeFormat::Reserved(9));
}

#[test]
fn sec_hdr_time_format_from_flags() {
    assert_eq!(
        SecHdrTimeFormat::from_packet_flags(0b0000_0000),
        SecHdrTimeFormat::Ch4
    );
    assert_eq!(
        SecHdrTimeFormat::from_packet_flags(0b0000_0100),
        SecHdrTimeFormat::Ieee1588
    );
    assert_eq!(
        SecHdrTimeFormat::from_packet_flags(0b0000_1000),
        SecHdrTimeFormat::Ertc
    );
    assert_eq!(
        SecHdrTimeFormat::from_packet_flags(0b0000_1100),
        SecHdrTimeFormat::Reserved(3)
    );
    // Bits outside [3:2] are ignored.
    assert_eq!(
        SecHdrTimeFormat::from_packet_flags(0b1111_0011),
        SecHdrTimeFormat::Ch4
    );
}
