//! PF-001–005: hand-specified observations, not values produced by an encoder.
//! Wire references: IRIG 106-24R1 §11.2.1.1 b–i, Tables 11-2 and 11-4.
use irig106_types::{KnownDataType, PacketFacts, Rtc, SecHdrTimeFormat};

// An API contract, not a wire layout: keep the V1 constructor callable forever.
const V1: fn(u64, Rtc, u32, u32, u16, u8, u8, u8, u8) -> PacketFacts = PacketFacts::new_v1;
const DISTINCT: PacketFacts = PacketFacts::new_v1(
    0x1234_5678_9ABC_DEF0,
    Rtc::MAX,
    0x1234_5678,
    0x9ABC_DEF0,
    0xA55A,
    0xFE,
    0xA3,
    0x5B,
    0xB7,
);

#[test]
fn every_observation_is_explicit_and_preserved() {
    // §11.2.1.1 b–i / Table 11-2. Distinct raw values catch swapped fields.
    // The offset is caller provenance, not a wire field. DATA > packet is
    // intentionally inconsistent: a facts value is not a framing validator.
    assert_eq!(DISTINCT.byte_offset(), 0x1234_5678_9ABC_DEF0);
    assert_eq!(DISTINCT.rtc(), Rtc::MAX);
    assert_eq!(DISTINCT.packet_length(), 0x1234_5678);
    assert_eq!(DISTINCT.data_length(), 0x9ABC_DEF0);
    assert_eq!(DISTINCT.channel_id(), 0xA55A);
    assert_eq!(DISTINCT.data_type(), 0xFE);
    assert_eq!(DISTINCT.data_type_version(), 0xA3);
    assert_eq!(DISTINCT.sequence(), 0x5B);
    assert_eq!(DISTINCT.flags(), 0xB7);
    fn copy_send_sync<T: Copy + Send + Sync>(value: T) -> (T, T) {
        (value, value)
    }
    let (first, second) = copy_send_sync(DISTINCT);
    assert_eq!(first, second);
    assert_eq!(V1(0, Rtc::ZERO, 0, 0, 0, 0, 0, 0, 0).byte_offset(), 0);
}

#[test]
fn raw_boundaries_have_no_hidden_defaults_or_repairs() {
    // §11.2.1.1 b–i: unsigned field widths. Length conformance, recording
    // edition and reserved-code policy are deliberately not this API's job.
    for (offset, rtc, length, channel, byte) in [
        (0, Rtc::ZERO, 0, 0, 0),
        (u64::MAX, Rtc::MAX, u32::MAX, u16::MAX, u8::MAX),
    ] {
        let facts = V1(offset, rtc, length, length, channel, byte, byte, byte, byte);
        assert_eq!(facts.byte_offset(), offset);
        assert_eq!(facts.rtc(), rtc);
        assert_eq!(facts.packet_length(), length);
        assert_eq!(facts.data_length(), length);
        assert_eq!(facts.channel_id(), channel);
        assert_eq!(facts.data_type(), byte);
        assert_eq!(facts.data_type_version(), byte);
        assert_eq!(facts.sequence(), byte);
        assert_eq!(facts.flags(), byte);
    }
    // §11.2.1.1 g says bit 6 requires bit 7; retain contradictory observations.
    let facts = V1(9, Rtc::ZERO, 1, u32::MAX, 7, 0x12, 0x01, 255, 0x40);
    assert_eq!(facts.flags(), 0x40);
    assert!(facts.intra_packet_uses_secondary_time());
    assert!(!facts.has_secondary_header());
    assert_eq!(facts.packet_length(), 1);
    assert_eq!(facts.data_length(), u32::MAX);
    // A named 106-24R1 view is NOT a declaration of edition eligibility/trust.
    assert_eq!(facts.known_data_type(), Some(KnownDataType::TimeFormat2));
    assert_eq!(facts.data_type_version(), 1);
}

#[test]
fn all_routing_bytes_keep_their_original_code_space() {
    // Table 11-4: 01 = Computer-Generated Format 1; 11/12 = Time Formats 1/2.
    // §11.2.1.1 e/f: version and sequence are separate raw bytes.
    for raw in 0..=u8::MAX {
        let facts = V1(0, Rtc::ZERO, 24, 0, 0, raw, 255 - raw, raw, 0);
        let expected = match raw {
            0x01 => Some(KnownDataType::ComputerGeneratedFormat1),
            0x11 => Some(KnownDataType::TimeFormat1),
            0x12 => Some(KnownDataType::TimeFormat2),
            _ => None,
        };
        assert_eq!(facts.data_type(), raw);
        assert_eq!(facts.known_data_type(), expected);
        assert_eq!(facts.data_type_version(), 255 - raw);
        assert_eq!(facts.sequence(), raw);
    }
    // 09 is a defined PCM Format 1 code in Table 11-4, unnamed by this view.
    assert_eq!(
        V1(0, Rtc::ZERO, 24, 0, 0, 9, 0, 0, 0).known_data_type(),
        None
    );
}

#[test]
fn flag_views_match_the_standard_for_every_byte() {
    // §11.2.1.1 g, pp. 11-8–9. Hand-transcribed truth table for high nibbles
    // 0..F: secondary present, secondary IPTS source, RTC error, overflow.
    let high = [
        (false, false, false, false),
        (false, false, false, true),
        (false, false, true, false),
        (false, false, true, true),
        (false, true, false, false),
        (false, true, false, true),
        (false, true, true, false),
        (false, true, true, true),
        (true, false, false, false),
        (true, false, false, true),
        (true, false, true, false),
        (true, false, true, true),
        (true, true, false, false),
        (true, true, false, true),
        (true, true, true, false),
        (true, true, true, true),
    ];
    // Bits 3–2 name Figures 11-4/5/6, then reserved. Bits 1–0 name
    // none/8/16/32 BIT checksums -> literal widths 0/1/2/4 BYTES.
    let formats = [
        SecHdrTimeFormat::Ch4,
        SecHdrTimeFormat::Ieee1588,
        SecHdrTimeFormat::Ertc,
        SecHdrTimeFormat::Reserved(3),
    ];
    let checksum_bytes = [0, 1, 2, 4];
    for flags in 0..=u8::MAX {
        let facts = V1(0, Rtc::ZERO, 24, 0, 0, 1, 0, 0, flags);
        let (secondary, ipts, rtc_error, overflow) = high[usize::from(flags / 16)];
        assert_eq!(facts.has_secondary_header(), secondary, "flags={flags:#x}");
        assert_eq!(facts.intra_packet_uses_secondary_time(), ipts);
        assert_eq!(facts.rtc_sync_error(), rtc_error);
        assert_eq!(facts.data_overflow(), overflow);
        assert_eq!(
            facts.secondary_time_format(),
            formats[usize::from(flags / 4 % 4)]
        );
        assert_eq!(
            facts.data_checksum_bytes(),
            checksum_bytes[usize::from(flags % 4)]
        );
        assert_eq!(facts.flags(), flags);
    }
}

#[test]
fn borrowed_payload_is_separate_from_the_copy_value() {
    // PF-005 is an API ownership contract, not another standard wire field.
    // §11.2.7.2 / Figure 11-34 puts the setup CSDW before setup text in DATA.
    fn observe<'a>(facts: &PacketFacts, data: &'a [u8]) -> (u64, &'a [u8]) {
        (facts.byte_offset(), data)
    }
    let payload = [0x0E, 0, 0, 0, b'A', b';'];
    let (offset, borrowed) = observe(&DISTINCT, &payload);
    assert_eq!(offset, DISTINCT.byte_offset());
    assert_eq!(borrowed.as_ptr(), payload.as_ptr());
    assert_eq!(borrowed, payload);
}

#[cfg(feature = "serde")]
#[test]
fn serde_traits_and_raw_deserialization_do_not_add_validation() {
    // PF-001/002: serde is an API representation, not IRIG validation.
    // §11.2.1.1 g contradictory flags and an out-of-48-bit RTC are retained,
    // matching the existing Rtc serde caveat rather than silently masking.
    use serde::de::{self, Deserializer, Visitor, value::MapDeserializer};
    struct Raw(u64);
    impl<'de> Deserializer<'de> for Raw {
        type Error = de::value::Error;
        fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
            visitor.visit_u64(self.0)
        }
        fn deserialize_newtype_struct<V: Visitor<'de>>(
            self,
            _: &'static str,
            visitor: V,
        ) -> Result<V::Value, Self::Error> {
            visitor.visit_newtype_struct(self)
        }
        serde::forward_to_deserialize_any! {
            bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
            byte_buf option unit unit_struct seq tuple tuple_struct map struct
            enum identifier ignored_any
        }
    }
    fn serde_value<T: serde::Serialize + for<'de> serde::Deserialize<'de>>() {}
    serde_value::<PacketFacts>();
    serde_value::<KnownDataType>();
    let fields = [
        ("byte_offset", u64::MAX),
        ("rtc", u64::MAX),
        ("packet_length", 1),
        ("data_length", u64::from(u32::MAX)),
        ("channel_id", u64::from(u16::MAX)),
        ("data_type", 0xFE),
        ("data_type_version", 0xFF),
        ("sequence", 0x80),
        ("flags", 0x40),
    ];
    let de = MapDeserializer::<_, de::value::Error>::new(
        fields.into_iter().map(|(key, value)| (key, Raw(value))),
    );
    let facts = <PacketFacts as serde::Deserialize>::deserialize(de).unwrap();
    assert_eq!(facts.byte_offset(), u64::MAX);
    assert_eq!(facts.rtc().as_raw(), u64::MAX);
    assert_eq!(facts.packet_length(), 1);
    assert_eq!(facts.data_length(), u32::MAX);
    assert_eq!(facts.channel_id(), u16::MAX);
    assert_eq!(facts.data_type(), 0xFE);
    assert_eq!(facts.data_type_version(), 0xFF);
    assert_eq!(facts.sequence(), 0x80);
    assert_eq!(facts.flags(), 0x40);
}
