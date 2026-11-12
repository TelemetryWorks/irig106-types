use irig106_types::Ch4BinaryTime;

// 106-24R1 Chapter 11 Figure 11-4 (printed 11-10 / PDF page 18):
// little-endian words [reserved, microseconds, low order, high order].
// Four distinct words, including valid usec=4660, catch any word swap.
const DISTINCT_WORDS: [u8; 8] = [0x00, 0x00, 0x34, 0x12, 0x01, 0x80, 0x02, 0x00];

#[test]
fn secondary_fields_from_figure_11_4() {
    // Hand-derived: 0x0000 reserved, 0x1234 usec, 0x8001 low, 0x0002 high.
    assert_eq!(
        Ch4BinaryTime::from_secondary_bytes(&DISTINCT_WORDS).unwrap(),
        Ch4BinaryTime {
            high_order: 2,
            low_order: 32_769,
            microseconds: 4_660,
        }
    );
}

#[test]
fn intra_packet_fields_from_figure_11_4() {
    // 106-24R1 §11.2.1.1 g (bits 6, 3–2), §11.2.1.3 b:
    // a Chapter 4 IPTS uses the same selected time format, Figure 11-4.
    // Check independently against literal fields, not the secondary decoder.
    assert_eq!(
        Ch4BinaryTime::from_intra_packet_bytes(&DISTINCT_WORDS).unwrap(),
        Ch4BinaryTime {
            high_order: 2,
            low_order: 32_769,
            microseconds: 4_660,
        }
    );
}

#[test]
fn binary_weighting_preserves_all_32_bits() {
    // 106-24R1 Chapter 4 §4.7, Figures 4-4/4-5 (printed 4-10–11):
    // low LSB=0.01 s, high LSB=655.36 s => high*65536 + low ticks.
    // The drawn low-word leading zero must not truncate binary weighting.
    // Microsecond word's two leading bits are zero in these valid fixtures.
    let cases = [
        // low=0x8000 => 32768*0.01 s = 327.68 s, not zero.
        ([0, 0, 0, 0, 0x00, 0x80, 0, 0], 32_768, 327_680_000_000_u64),
        // high=1, low=0x8000 => 655.36 + 327.68 = 983.04 s.
        ([0, 0, 0, 0, 0x00, 0x80, 1, 0], 98_304, 983_040_000_000),
        // high=2, low=0x8001, usec=4660 => 1638.414660 s.
        (DISTINCT_WORDS, 163_841, 1_638_414_660_000),
        // Full 32-bit binary count; no mask on either high or low word.
        (
            [0, 0, 0, 0, 255, 255, 255, 255],
            4_294_967_295,
            42_949_672_950_000_000,
        ),
    ];
    for (bytes, expected_ticks, expected_nanos) in cases {
        for parsed in [
            Ch4BinaryTime::from_secondary_bytes(&bytes),
            Ch4BinaryTime::from_intra_packet_bytes(&bytes),
        ] {
            let time = parsed.unwrap();
            // Arithmetic here expresses §4.7; no crate encoder/converter
            // supplies the independently written expected values above.
            let ticks = (u32::from(time.high_order) << 16) | u32::from(time.low_order);
            assert_eq!(ticks, expected_ticks);
            assert_eq!(
                u64::from(ticks) * 10_000_000 + u64::from(time.microseconds) * 1_000,
                expected_nanos
            );
        }
    }
}
