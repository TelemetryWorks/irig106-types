use irig106_types::TimeSource;

#[test]
fn t6_source_table() {
    // 106-24R1 §11.2.3.2 a / Figure 11-12, SRC table.
    // Same table: 106-05 §10.6.3.2 Figure 10-16a;
    // 106-07 Figure 10-22; 106-13 and 106-15 Figure 10-19.
    // 106-04 Chapter 10 is absent from the archive: no claim about it.
    assert_eq!(TimeSource::from_raw(0), TimeSource::Internal);
    assert_eq!(TimeSource::from_raw(1), TimeSource::External);
    assert_eq!(TimeSource::from_raw(2), TimeSource::InternalRtc);
    for raw in 3..=14 {
        assert_eq!(TimeSource::from_raw(raw), TimeSource::Reserved(raw));
    }
    assert_eq!(TimeSource::from_raw(15), TimeSource::None);
}
