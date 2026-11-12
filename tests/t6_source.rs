use irig106_types::TimeSource;

#[test]
fn t6_source_table() {
    // 106-24R1 §11.2.3.2 a / Figure 11-12, SRC table.
    // Same table: 106-05 §10.6.3.2 Figure 10-16a;
    // 106-07 Figure 10-22; 106-13 and 106-15 Figure 10-19.
    // This SRC mapping starts in 106-05. The archived 106-04 full PDF,
    // §10.6.3.3 / Figure 10-15, defines EXT bit 0 and reserves bits 3–1.
    // irig106-time handles that edition-specific layout.
    assert_eq!(TimeSource::from_raw(0), TimeSource::Internal);
    assert_eq!(TimeSource::from_raw(1), TimeSource::External);
    assert_eq!(TimeSource::from_raw(2), TimeSource::InternalFromRmm);
    for raw in 3..=14 {
        assert_eq!(TimeSource::from_raw(raw), TimeSource::Reserved(raw));
    }
    assert_eq!(TimeSource::from_raw(15), TimeSource::None);
}
