use irig106_types::IrigTimeSource;

#[test]
fn t7_its_table() {
    // 106-24R1 §11.2.3.2 a / Figure 11-12, ITS bits 15–12.
    // Hand-transcribed eight entries; all remaining nibble values reserved.
    // Applies only to IRIG-A/B/G; 106-15 Figure 10-19 has reserved bits
    // 31–12 instead. The caller must decide applicability by edition/FMT.
    let expected = [
        IrigTimeSource::Freewheeling,
        IrigTimeSource::TimeCommand,
        IrigTimeSource::Rmm,
        IrigTimeSource::ExternalIrig,
        IrigTimeSource::ExternalGps,
        IrigTimeSource::ExternalNtp,
        IrigTimeSource::ExternalPtp,
        IrigTimeSource::Embedded,
    ];
    for (raw, value) in (0..8).zip(expected) {
        assert_eq!(IrigTimeSource::from_raw(raw), value);
    }
    for raw in 8..=15 {
        assert_eq!(IrigTimeSource::from_raw(raw), IrigTimeSource::Reserved(raw));
    }
}
