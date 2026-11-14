//! Independent quantity definitions, not encoder/decoder round trips.
//! Unix epoch: 1970-01-01. RTC: 10 MHz = exactly 100 ns/tick
//! (IRIG 106-24R1 §11.2.1.1 i). PTP TAI epoch is 1970, not 1958.

use irig106_types::{
    NanosDuration, SignedNanos, SignedRtcTicks, SignedUnixSeconds, TaiSeconds, TaiUtcOffset,
    UnixSeconds,
};

#[test]
fn rtc_ticks_are_signed_hundred_nanoseconds() {
    // Hand-computed tick values, including the full 48-bit counter range.
    for (ticks, nanos) in [
        (0, 0),
        (1, 100),
        (-1, -100),
        (150_000, 15_000_000),
        (-150_000, -15_000_000),
        (281_474_976_710_655, 28_147_497_671_065_500),
        (-281_474_976_710_655, -28_147_497_671_065_500),
    ] {
        assert_eq!(
            SignedRtcTicks::new(ticks).to_nanos(),
            Some(SignedNanos::new(nanos))
        );
    }
}

#[test]
fn rtc_conversion_rejects_i128_overflow() {
    // floor((2^127-1)/100), computed as a decimal integer, not from crate code.
    let limit = 1_701_411_834_604_692_317_316_873_037_158_841_057;
    let nanos = 170_141_183_460_469_231_731_687_303_715_884_105_700;
    assert_eq!(
        SignedRtcTicks::new(limit).to_nanos(),
        Some(SignedNanos::new(nanos))
    );
    assert_eq!(
        SignedRtcTicks::new(-limit).to_nanos(),
        Some(SignedNanos::new(-nanos))
    );
    for ticks in [limit + 1, -limit - 1, i128::MAX, i128::MIN] {
        assert_eq!(SignedRtcTicks::new(ticks).to_nanos(), None);
    }
}

#[test]
fn nanoseconds_preserve_sign_and_check_unsigned_range() {
    for (raw, unsigned, magnitude) in [
        (0, Some(0), Some(0)),
        (1, Some(1), Some(1)),
        (-1, None, Some(1)),
        (18_446_744_073_709_551_615, Some(u64::MAX), Some(u64::MAX)),
        (-18_446_744_073_709_551_615, None, Some(u64::MAX)),
        (18_446_744_073_709_551_616, None, None),
        (-18_446_744_073_709_551_616, None, None),
        (i128::MAX, None, None),
        (i128::MIN, None, None),
    ] {
        let value = SignedNanos::new(raw);
        assert_eq!(value.get(), raw);
        assert_eq!(value.to_unsigned(), unsigned.map(NanosDuration::new));
        assert_eq!(value.magnitude(), magnitude.map(NanosDuration::new));
    }
    assert_eq!(SignedNanos::default(), SignedNanos::ZERO);
    assert_eq!(
        NanosDuration::new(u64::MAX).to_signed().get(),
        18_446_744_073_709_551_615
    );
}

#[test]
fn unix_epoch_and_earlier_calendar_origins() {
    // Gregorian day counts before the Unix epoch:
    // 1900-01-01: 70*365 + 17 leap days = 25567 days.
    // 1958-01-01: 12*365 + 3 leap days = 4383 days.
    // These are signed Unix offsets, NOT TAI/UTC clock conversions at those dates.
    assert_eq!(-25_567_i128 * 86_400, -2_208_988_800);
    assert_eq!(-4_383_i128 * 86_400, -378_691_200);
    for raw in [-2_208_988_800, -378_691_200, -1] {
        let value = SignedUnixSeconds::new(raw);
        assert_eq!(value.get(), raw);
        assert_eq!(value.to_unsigned(), None);
    }
    assert_eq!(
        SignedUnixSeconds::new(0).to_unsigned(),
        Some(UnixSeconds::new(0))
    );
    assert_eq!(
        SignedUnixSeconds::new(1).to_unsigned(),
        Some(UnixSeconds::new(1))
    );
}

#[test]
fn signed_unix_full_range_and_unsigned_boundary() {
    assert_eq!(
        UnixSeconds::new(u64::MAX).to_signed().get(),
        18_446_744_073_709_551_615
    );
    assert_eq!(
        SignedUnixSeconds::new(18_446_744_073_709_551_615).to_unsigned(),
        Some(UnixSeconds::new(u64::MAX))
    );
    for raw in [18_446_744_073_709_551_616, i128::MAX, i128::MIN] {
        let value = SignedUnixSeconds::new(raw);
        assert_eq!(value.get(), raw);
        assert_eq!(value.to_unsigned(), None);
    }
}

#[test]
fn tai_offset_subtraction_never_repairs_an_unsigned_result() {
    // Caller-supplied offsets are arithmetic inputs; no historical offset is inferred.
    for (tai, offset, signed, unsigned) in [
        (0, 0, 0, Some(0)),
        (36, 37, -1, None),
        (37, 37, 0, Some(0)),
        (38, 37, 1, Some(1)),
        (0, 37, -37, None),
        (1_735_689_637, 37, 1_735_689_600, Some(1_735_689_600)),
        (0, i32::MAX, -2_147_483_647, None),
        (0, i32::MIN, 2_147_483_648, Some(2_147_483_648)),
        (u64::MAX, 0, 18_446_744_073_709_551_615, Some(u64::MAX)),
        (u64::MAX, -1, 18_446_744_073_709_551_616, None),
        (u64::MAX, i32::MIN, 18_446_744_075_857_035_263, None),
    ] {
        let value = TaiSeconds::new(tai);
        let offset = TaiUtcOffset::new(offset);
        assert_eq!(value.to_signed_unix(offset).get(), signed);
        assert_eq!(value.to_unix(offset), unsigned.map(UnixSeconds::new));
    }
}

#[test]
fn unsigned_duration_scaling_checks_last_representable_input() {
    // Largest whole inputs whose scaled nanoseconds fit u64; next input must fail.
    assert_eq!(
        NanosDuration::from_secs(18_446_744_073),
        Some(NanosDuration::new(18_446_744_073_000_000_000))
    );
    assert_eq!(NanosDuration::from_secs(18_446_744_074), None);
    assert_eq!(
        NanosDuration::from_millis(18_446_744_073_709),
        Some(NanosDuration::new(18_446_744_073_709_000_000))
    );
    assert_eq!(NanosDuration::from_millis(18_446_744_073_710), None);
    assert_eq!(
        NanosDuration::from_micros(18_446_744_073_709_551),
        Some(NanosDuration::new(18_446_744_073_709_551_000))
    );
    assert_eq!(NanosDuration::from_micros(18_446_744_073_709_552), None);
    assert_eq!(NanosDuration::from_secs(0), Some(NanosDuration::ZERO));
    assert_eq!(NanosDuration::from_millis(0), Some(NanosDuration::ZERO));
    assert_eq!(NanosDuration::from_micros(0), Some(NanosDuration::ZERO));
    assert_eq!(NanosDuration::from_secs(u64::MAX), None);
    assert_eq!(NanosDuration::from_millis(u64::MAX), None);
    assert_eq!(NanosDuration::from_micros(u64::MAX), None);
}

#[cfg(feature = "serde")]
#[test]
fn signed_types_support_optional_serde() {
    fn supports_serde<T: serde::Serialize + for<'de> serde::Deserialize<'de>>() {}
    supports_serde::<SignedNanos>();
    supports_serde::<SignedRtcTicks>();
    supports_serde::<SignedUnixSeconds>();
}
