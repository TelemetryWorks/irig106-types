# Working with irig106-types

This guide describes the existing API. The baseline is IRIG 106-24R1;
[the review record](STANDARD-VALUES-REVIEW.md) records older-edition evidence
and drawing ambiguities. Rustdoc is the reference for each item's range,
units, invariants, errors, and applicable standard section.

## 1. Put responsibilities in the right crate

Types contains shared, stable, plain values: counters, individual time words,
field-code mappings and checked unit/epoch quantities. It does not frame or
parse packets, perform I/O, decode payloads, select validation policy, assemble
TMATS, or correlate clocks. Individual byte constructors only unpack a time
field. Their limited checks do not make a packet conforming.

| Crate | Responsibility | Relationship to types |
|---|---|---|
| `irig106-core` | Packet framing/parsing; produces header facts | Current: published 0.1.0; re-exports `Rtc` and returns it from `Header::rtc()` |
| `irig106-tmats` | Setup assembly; consumes facts supplied by an orchestrator | Planned: PacketFacts consumer without depending on core |
| `irig106-time` | Time interpretation, session/wrap policy, correlation | Current: unreleased repository dependency; counters, time words, enums and quantities |
| `irig106-decode` | Payload decoding | Intended consumer of common units/codes as needed |
| `irig106-cli` | Composition, I/O and presentation | Intended: orchestrates producers and consumers |

As of 2026-11-23, `irig106-core` is the only reverse dependency on crates.io;
`irig106-time` uses a repository dependency. The remaining rows describe planned
or intended roles, not current dependencies. The
[PacketFacts contract](PACKET-FACTS.md) now has a shared implementation here;
core adapters, TMATS assembly and orchestration remain planned. A dependency on
core is not required just to name the value.

## 2. Choose the counter and the unit

`Rtc` is a 48-bit primary-header counter at **100 ns/tick** (Chapter 11
§11.2.1.1). `Rtc::from_raw` explicitly masks to 48 bits; `from_le_bytes`
reads exactly six bytes. `Ertc` is a separate 64-bit counter at **1 ns/tick**
(§11.2.1.1 g, Figure 11-6); its byte constructor needs at least eight bytes.
Neither counter supplies a UTC/TAI epoch.

```rust
use irig106_types::{Ertc, Rtc};

assert_eq!(Rtc::from_raw(1).to_nanos(), 100);
assert_eq!(Ertc::from_raw(100).to_nanos(), 100);
assert_eq!(Rtc::MAX.elapsed_ticks(Rtc::ZERO), 1);
```

`elapsed_ticks` measures a forward difference modulo 2^48, not a signed
difference or multiple-wrap count. Numerical `Ord` is not chronological
ordering across a wrap. A time consumer chooses ordering and sessions.

For an already resolved signed displacement, use `SignedRtcTicks(i128)`
and `SignedNanos(i128)` via `new`/`get`. These quantities are not raw RTCs.
Scaling is checked and preserves sign:

```rust
use irig106_types::{NanosDuration, SignedNanos, SignedRtcTicks};

assert_eq!(SignedRtcTicks::new(-2).to_nanos(), Some(SignedNanos::new(-200)));
assert_eq!(SignedRtcTicks::new(i128::MAX).to_nanos(), None);
assert_eq!(SignedNanos::new(-1).to_unsigned(), None);
assert_eq!(SignedNanos::new(-1).magnitude(), Some(NanosDuration::new(1)));
assert_eq!(NanosDuration::new(199).as_rtc_ticks(), 1); // explicit truncation
```

All unit wrappers accept their inner integer's full domain; `Default` is
zero. Checked scaling/unsigned conversions return `None` for overflow or
sign loss. They never silently wrap or clamp. `as_rtc_ticks` explicitly
truncates a nanosecond remainder, and `Rtc::from_raw` explicitly masks.

## 3. Unpack a field, then apply the consumer's validation

`Ch4BinaryTime` reads four little-endian 16-bit words:

| Byte interval | Word | Meaning |
|---|---|---|
| 0..2 | Reserved | Ignored by the constructor |
| 2..4 | Microseconds | Conforming baseline range 0–9999; raw `u16` retained |
| 4..6 | Low-order | Low half of a full 32-bit binary count |
| 6..8 | High-order | High half of that count |

This is Figure 11-4, not the Chapter 4 PCM insertion order. Chapter 4 §4.7,
Figures 4-4/4-5 define `high * 65536 + low` at 10 ms/count. A low word of
0x8000 means 327.68 seconds and must not be masked to 15 bits.

```rust
use irig106_types::Ch4BinaryTime;

let time = Ch4BinaryTime::from_secondary_bytes(
    &[0, 0, 0x34, 0x12, 0x01, 0x80, 2, 0],
).unwrap();
assert_eq!((time.high_order, time.low_order, time.microseconds),
           (2, 0x8001, 0x1234));
```

The secondary and intra-packet constructors share this layout. They check
length, ignore trailing bytes and the reserved word, and retain all three
time words. Absolute interpretation and reserved/range checks remain outside
types. The word order for 106-04–106-09 is inferred from the 106-11+ figure;
older Chapter 4 drawings differ, as detailed in the review record.

`Ieee1588Time::from_le_bytes` reads `[nanoseconds(u32), seconds(u32)]`
(Figure 11-5), rejecting fractions >= 1,000,000,000. Its public fields and
serde can represent values that bypass that check. Seconds refer to the
1970 TAI/PTP epoch (§11.2.3.3), not the 1958 TAI epoch. `to_nanos_since_epoch`
only scales and adds; it does not validate or convert to UTC.

Short field buffers return `TypesError::BufferTooShort`; the PTP fraction
check returns `TypesError::OutOfRange`. These are library errors, not
standard wire codes. `TypesError` is non-exhaustive; allow a fallback when
matching it. Arithmetic quantities use `Option` for range failures instead.

## 4. Establish the edition before interpreting codes

Format 1 CSDW codes are defined in §11.2.3.2 a / Figure 11-12. Extract a
nibble before calling `TimeSource::from_raw`, `TimeFormat::from_raw`, or
`IrigTimeSource::from_raw`; those methods do not mask inputs. Unknown `u8`
values are preserved, including out-of-field values.

- SRC 0/1/2 means internal/external/internal from RMM; 3–14 is reserved,
  and 15 means none, **from 106-05 onward**.
- FMT 15 means the payload is invalid, **from 106-05 onward**.
- 106-04 instead defines EXT in bits 3–0 (only bit 0 is meaningful) and
  reserves FMT 6–15 (§10.6.3.3, Figure 10-15).
- ITS in bits 15–12 is defined from 106-17 and only for IRIG-A/B/G.
  Zero in an older recording does not mean freewheeling.
- `DateFormat` names the DATE bit's day-of-year/day-month-year alternatives;
  it does not parse or validate a calendar date.
- `SecHdrTimeFormat::from_packet_flags` extracts bits 3–2, but does not
  validate edition, secondary-header presence (bit 7), or the IPTS choice
  (bit 6). The mapping alone is not permission to interpret an IPTS.

Keep the two version-code tables separate:

```rust
use irig106_types::Irig106Version;

assert_eq!(Irig106Version::from_setup_record(7), Irig106Version::V07);
assert_eq!(Irig106Version::from_data_type_version(3), Irig106Version::V07);
assert_eq!(Irig106Version::from_setup_record(15), Irig106Version::Unknown(15));
```

The setup-record RCCVER byte comes from §11.2.7.2 / Figure 11-34. The packet
header's separate version list is §11.2.1.1 e; Table 11-4 lists each data
type's current version. A header version describes a data type's layout,
not the exact recording edition. Neither table supplies a 106-23/24/24R1
code, and setup code zero is unknown. Applicability/policy stays with consumers.

## 5. Keep epochs separate from durations

`UnixSeconds(u64)` and `SignedUnixSeconds(i128)` use the Unix epoch;
negative signed seconds precede 1970. They exclude leap seconds and do not
validate calendar ranges. `TaiSeconds(u64)` uses the **1970 PTP TAI epoch**
(§11.2.3.3). `TaiUtcOffset(i32)` is caller-supplied TAI minus UTC in seconds.

```rust
use irig106_types::{TaiSeconds, TaiUtcOffset};

let unix = TaiSeconds::new(1_735_689_637)
    .to_unix(TaiUtcOffset::new(37)).unwrap();
assert_eq!(unix.get(), 1_735_689_600); // 2025-01-01T00:00:00Z
```

`to_signed_unix` widens before subtraction; `to_unix` additionally checks the
unsigned range. Historical conversion is exact only with the correct
whole-second offset, as used from **1972-01-01 onward**. Before 1972 the
offset was fractional; a whole-second parameter cannot represent it.
A signed Unix type does not remove that limitation. No method selects a
leap-second table; `Some` confirms integer range, not historical validity.

## 6. Features and ongoing documentation checks

The library is `no_std` and uses no `alloc`; no dependency is required.
`std` (default) adds `std::error::Error` for `TypesError`. `serde` adds
serialization/deserialization to all public value types, excluding
`TypesError`. Unit wrappers use transparent serde representation.

Derived serde is **not validation**: notably an `Rtc` can deserialize a
`u64` wider than 48 bits. Validate untrusted input, or explicitly mask with
`Rtc::from_raw` when that is the caller's intended operation. Likewise,
public raw fields and enum payloads can bypass byte-constructor checks.

The crate denies missing documentation and broken intra-doc links. CI builds
rustdoc with warnings denied and runs doctests as part of the feature tests.
Useful focused checks:

```sh
RUSTDOCFLAGS="-D warnings -D rustdoc::broken_intra_doc_links" cargo doc --no-deps --all-features
cargo test --doc --all-features
cargo test --doc --no-default-features
```

Guide Rust examples are also compiled during the CI Documentation job.

## 7. Pass header facts and borrow DATA separately

`PacketFacts` contains only primary-header observations and byte provenance
(Chapter 11 §11.2.1.1 / Table 11-2). Construct it from the producer's existing
observations; this is not packet parsing or proof of trust. All nine values are
required by the frozen V1 constructor. The call below is an independent example,
not a released core adapter or TMATS assembler API:

```rust
use irig106_types::{KnownDataType, PacketFacts, Rtc};

let facts = PacketFacts::new_v1(100, Rtc::from_raw(42), 44, 8, 7, 1, 9, 255, 0x80);
let data = [0x0E, 0, 0, 0, b'S', b':', b'a', b';'];
fn consume<'a>(facts: &PacketFacts, data: &'a [u8]) -> (u64, &'a [u8]) {
    (facts.byte_offset(), data)
}
assert_eq!(facts.known_data_type(), Some(KnownDataType::ComputerGeneratedFormat1));
assert!(facts.has_secondary_header());
let (offset, borrowed) = consume(&facts, &data);
assert_eq!(offset, 100);
assert_eq!(borrowed.as_ptr(), data.as_ptr());
```

`data_type()` always preserves the raw byte. The named view initially covers
only 0x01/0x11/0x12; `None` includes valid codes not named by this API. Flag views
are mechanical: bit 7 secondary presence, bit 6 IPTS source, bit 5 RTC sync error,
bit 4 overflow, bits 3–2 secondary format, bits 1–0 checksum width (0/1/2/4 bytes).
Contradictory flags and lengths remain observable; nothing chooses an edition,
verifies a checksum, recognises a trusted closure marker, or repairs input.

There is no `Default`; non-exhaustiveness prevents external struct literals.
`new_v1` will not grow parameters: future fields must represent absence explicitly
for old callers, with a new constructor to supply them. All size changes need
review. See [constructor evolution](PACKET-FACTS.md#constructor-evolution-implementation-decision-2026-11-23).

DATA must exclude headers and packet trailer filler/checksum and include its
CSDW where applicable. Check capture bounds and external integrity evidence;
`data_length()` is a declaration, not proof all bytes were captured. Keep the
borrow within the backing buffer's lifetime. Context such as session start,
trust/gaps, edition, joining phases and closure/completeness belongs outside
facts. Borrowing this value allocates nothing, but it does not make core 0.1's
owned DATA capture allocation-free or establish end-to-end performance.
