//! Unit tests for the `binary_time` module.
//!
//! # Test Documentation
//!
//! | Test | Validates | Traces |
//! |------|-----------|--------|
//! | `ch4_from_secondary_bytes` | Fields decode from bytes [2..8] | L3-CH4-005 |
//! | `ch4_short_buffer_rejected` | < 8 bytes rejected | L3-CH4-005 |
//! | `ch4_intra_packet_matches_secondary` | Same layout for both sources | L3-CH4-005 |
//! | `ieee1588_from_le_bytes` | Nanos/seconds decode | L3-1588-002 |
//! | `ieee1588_nanos_overflow_rejected` | nanos ≥ 1e9 rejected | L3-1588-002 |
//! | `ieee1588_short_buffer_rejected` | < 8 bytes rejected | L3-1588-002 |
//! | `ieee1588_to_nanos_since_epoch` | seconds × 1e9 + nanos | L3-1588-003 |

use super::*;

#[test]
fn ch4_from_secondary_bytes() {
    let buf = [0xAA, 0xBB, 0x34, 0x12, 0x78, 0x56, 0xE8, 0x03];
    let t = Ch4BinaryTime::from_secondary_bytes(&buf).unwrap();
    assert_eq!(t.high_order, 0x1234);
    assert_eq!(t.low_order, 0x5678);
    assert_eq!(t.microseconds, 1000);
}

#[test]
fn ch4_short_buffer_rejected() {
    assert_eq!(
        Ch4BinaryTime::from_secondary_bytes(&[0u8; 6]),
        Err(TypesError::BufferTooShort {
            expected: 8,
            actual: 6
        })
    );
}

#[test]
fn ch4_intra_packet_matches_secondary() {
    let buf = [0, 0, 1, 0, 2, 0, 3, 0];
    assert_eq!(
        Ch4BinaryTime::from_intra_packet_bytes(&buf),
        Ch4BinaryTime::from_secondary_bytes(&buf)
    );
}

#[test]
fn ieee1588_from_le_bytes() {
    // 500_000_000 ns = 0x1DCD_6500, 1000 seconds = 0x0000_03E8
    let buf: [u8; 8] = [0x00, 0x65, 0xCD, 0x1D, 0xE8, 0x03, 0x00, 0x00];
    let t = Ieee1588Time::from_le_bytes(&buf).unwrap();
    assert_eq!(t.nanoseconds, 500_000_000);
    assert_eq!(t.seconds, 1000);
}

#[test]
fn ieee1588_nanos_overflow_rejected() {
    // nanoseconds = 1_000_000_000 (too large)
    let buf: [u8; 8] = [0x00, 0xCA, 0x9A, 0x3B, 0x00, 0x00, 0x00, 0x00];
    assert_eq!(
        Ieee1588Time::from_le_bytes(&buf),
        Err(TypesError::OutOfRange {
            field: "ieee1588_nanoseconds",
            value: 1_000_000_000,
            max: 999_999_999
        })
    );
}

#[test]
fn ieee1588_short_buffer_rejected() {
    assert!(matches!(
        Ieee1588Time::from_le_bytes(&[0u8; 4]),
        Err(TypesError::BufferTooShort { .. })
    ));
}

#[test]
fn ieee1588_to_nanos_since_epoch() {
    let t = Ieee1588Time {
        nanoseconds: 500_000_000,
        seconds: 10,
    };
    assert_eq!(t.to_nanos_since_epoch(), 10_500_000_000);
}
