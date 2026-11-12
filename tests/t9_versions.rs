use irig106_types::Irig106Version;

#[test]
fn t9_two_version_tables() {
    use Irig106Version::*;
    // 106-24R1 §11.2.7.2 / Figure 11-34: RCCVER table, 0..6 reserved.
    // 106-07 §10.6.7.2 Figure 10-50 also reserves 0..6 (not Pre07).
    let setup = [V07, V09, V11, V13, V15, V17, V19, V22];
    // 106-24R1 §11.2.1.1 e / Table 11-4: header data type version.
    // These are version *codes*, not the packet data type byte.
    let header = [V04, V05, V07, V09, V11, V13, V15, V17, V19, V22];
    for raw in 0..=255_u8 {
        let setup_expected = match raw {
            7..=14 => setup[usize::from(raw - 7)],
            _ => Unknown(raw),
        };
        let header_expected = match raw {
            1..=10 => header[usize::from(raw - 1)],
            _ => Unknown(raw),
        };
        assert_eq!(Irig106Version::from_setup_record(raw), setup_expected);
        assert_eq!(Irig106Version::from_data_type_version(raw), header_expected);
    }
}
