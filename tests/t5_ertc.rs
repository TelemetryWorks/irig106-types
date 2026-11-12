use irig106_types::{Ertc, Rtc};

#[test]
fn t5_ertc_nanoseconds() {
    // 106-24R1 §11.2.1.1 g, Figure 11-6: LSLW followed by MSLW,
    // 1 GHz => 1 ns/tick; RTC = ERTC/100. Hand-written LE fixtures.
    for (bytes, nanos) in [
        ([0; 8], 0_u128),
        ([1, 0, 0, 0, 0, 0, 0, 0], 1),
        ([100, 0, 0, 0, 0, 0, 0, 0], 100),
        ([0, 0, 0, 0, 1, 0, 0, 0], 4_294_967_296),
        ([255; 8], 18_446_744_073_709_551_615),
    ] {
        assert_eq!(Ertc::from_le_bytes(&bytes).unwrap().to_nanos(), nanos);
    }
    assert_eq!(Ertc::from_raw(100).to_nanos(), 100);
    assert_eq!(Rtc::from_raw(1).to_nanos(), 100);
}
