use irig106_types::TimeFormat;

#[test]
fn t7_format_none() {
    // 106-24R1 §11.2.3.2 a / Figure 11-12: FMT 0xF = NONE
    // (time packet payload invalid). A string assertion also runs on
    // pre-fix main, where the None variant does not exist yet.
    assert_eq!(format!("{:?}", TimeFormat::from_raw(15)), "None");
    for raw in 6..=14 {
        assert_eq!(TimeFormat::from_raw(raw), TimeFormat::Reserved(raw));
    }
}
