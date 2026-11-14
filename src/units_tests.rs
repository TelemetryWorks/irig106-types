//! Unit tests for the `units` module.

use super::*;

#[test]
fn new_get_round_trip() {
    assert_eq!(UnixSeconds::new(42).get(), 42);
    assert_eq!(TaiSeconds::new(42).get(), 42);
    assert_eq!(TaiUtcOffset::new(-3).get(), -3);
    assert_eq!(NanosDuration::new(7).get(), 7);
}

#[test]
fn defaults_are_zero() {
    assert_eq!(UnixSeconds::default().get(), 0);
    assert_eq!(NanosDuration::default(), NanosDuration::ZERO);
}

#[test]
fn tai_to_unix_positive_offset() {
    let tai = TaiSeconds::new(1_000_037);
    assert_eq!(
        tai.to_unix(TaiUtcOffset::new(37)),
        Some(UnixSeconds::new(1_000_000))
    );
}

#[test]
fn tai_to_unix_negative_offset() {
    let tai = TaiSeconds::new(100);
    assert_eq!(
        tai.to_unix(TaiUtcOffset::new(-5)),
        Some(UnixSeconds::new(105))
    );
}

#[test]
fn tai_to_unix_rejects_out_of_range() {
    assert_eq!(TaiSeconds::new(10).to_unix(TaiUtcOffset::new(37)), None);
    assert_eq!(
        TaiSeconds::new(u64::MAX).to_unix(TaiUtcOffset::new(i32::MIN)),
        None
    );
}

#[test]
fn nanos_duration_constructors() {
    assert_eq!(NanosDuration::from_secs(2).unwrap().get(), 2_000_000_000);
    assert_eq!(NanosDuration::from_millis(3).unwrap().get(), 3_000_000);
    assert_eq!(NanosDuration::from_micros(4).unwrap().get(), 4_000);
    assert_eq!(NanosDuration::from_secs(u64::MAX), None);
}

#[test]
fn nanos_duration_rtc_ticks() {
    assert_eq!(
        NanosDuration::from_secs(1).unwrap().as_rtc_ticks(),
        10_000_000
    );
    assert_eq!(NanosDuration::new(199).as_rtc_ticks(), 1);
}

#[test]
fn ordering() {
    assert!(NanosDuration::from_millis(1).unwrap() < NanosDuration::from_secs(1).unwrap());
    assert!(TaiUtcOffset::new(-1) < TaiUtcOffset::new(0));
}
