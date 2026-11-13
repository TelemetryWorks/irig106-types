# irig106-types

Zero-cost abstractions and foundational types for IRIG 106 telemetry.

`irig106-types` is the shared vocabulary of the TelemetryWorks IRIG 106 crate
ecosystem (`irig106-core`, `irig106-time`, `irig106-decode`, `irig106-write`,
`irig106-ch10-reader`). It defines counters, wire-level time representations,
time-related enumerations, and unit/epoch newtypes, and nothing else: no I/O,
no parsing of whole packets, no correlation.

- `#![no_std]`, no `alloc`, no required dependencies
- Edition 2024, MSRV: Rust 1.85 (the same across the IRIG 106 ecosystem)
- License: Apache-2.0

## Contents

| Module | Types | Notes |
|--------|-------|-------|
| `rtc` | `Rtc`, `Ertc` | 48-bit RTC (masked, wrap-aware elapsed time) at 100 ns/tick; 64-bit ERTC at 1 ns/tick |
| `binary_time` | `Ch4BinaryTime`, `Ieee1588Time` | Raw secondary-header / intra-packet time fields |
| `time_enums` | `TimeSource`, `TimeFormat`, `IrigTimeSource`, `DateFormat`, `SecHdrTimeFormat` | Decoded CSDW / packet-flag fields |
| `units` | `UnixSeconds`, `TaiSeconds`, `TaiUtcOffset`, `NanosDuration` | `#[repr(transparent)]` newtypes that prevent unit and epoch mix-ups |
| `version` | `Irig106Version` | Separate setup-record RCCVER and packet-header data type version mappings |
| `error` | `TypesError` | `BufferTooShort`, `OutOfRange` |

Interpretation of these values (e.g., Chapter 4 binary time → time of day,
RTC → absolute time correlation, leap-second tables) lives in
[`irig106-time`](https://github.com/TelemetryWorks/irig106-time).

## Example

```rust
use irig106_types::{NanosDuration, Rtc, TaiSeconds, TaiUtcOffset};

let start = Rtc::from_le_bytes([0x80, 0x96, 0x98, 0x00, 0x00, 0x00]);
let end = Rtc::from_raw(start.as_raw() + 5_000_000);
assert_eq!(start.elapsed_nanos(end), NanosDuration::from_millis(500).get());

let utc = TaiSeconds::new(1_735_689_637).to_unix(TaiUtcOffset::new(37));
assert_eq!(utc.get(), 1_735_689_600); // 2025-01-01T00:00:00Z
```

## Features

| Feature | Default | Effect |
|---------|---------|--------|
| `std` | yes | `impl std::error::Error for TypesError` |
| `serde` | no | `Serialize`/`Deserialize` on all public data types (newtypes are `#[serde(transparent)]`) |

## Standard values and adoption

The baseline is IRIG 106-24R1 Chapter 11:

- ERTC counts **1 ns/tick** (§11.2.1.1 g, Figure 11-6); RTC remains 100 ns/tick.
- From 106-05 onward, Format 1 SRC `0x3`–`0xE` is reserved, never GPS.
  SRC `0x2` is `InternalFromRmm`: internal from the clock in the RMM.
- From 106-05 onward, FMT `0xF` is `TimeFormat::None`: **the payload is invalid**.
  The archived 106-04 full PDF (§10.6.3.3, Figure 10-15) instead defines
  EXT bit 0 (external time present), reserves bits 3–1, and reserves FMT
  `0x6`–`0xF`. Consumers must handle the edition before using these mappings.
- `IrigTimeSource::from_raw` decodes the eight ITS values in bits 15–12
  (§11.2.3.2 a, Figure 11-12). ITS applies only to IRIG-A/B/G and is defined
  from 106-17 onward; older editions reserve those bits. This raw mapping
  does not decide whether ITS applies to a recording.
- `Irig106Version::from_setup_record` maps the extracted RCCVER byte:
  `0x07`–`0x0E` name RCC 106-07 through RCC 106-22 (§11.2.7.2, Figure 11-34).
- `Irig106Version::from_data_type_version` uses the separate packet-header
  code list in §11.2.1.1 e: `0x01`–`0x0A` name RCC 106-04 through RCC 106-22.
  Table 11-4 separately gives each data type's current version.
  This describes a data type's layout, not the recording edition.
- Reserved/unrecognized version codes become `Unknown(raw)`. In particular,
  `0x0F` is unknown in both mappings; setup-record zero is also unknown.
  The 106-22 entries are unchanged through 106-24R1; neither mapping can
  distinguish 106-22, 106-23, 106-24, and 106-24R1 recordings.

Consumers must remove `TimeSource::Gps` matches, rename `InternalRtc` to
`InternalFromRmm` (including serde consumers), handle `TimeFormat::None`,
and use the version constructor appropriate to the field. `irig106-time`
retains CSDW parsing, reserved-bit validation (31–16 and 11–10; 31–12 before
ITS), payload validity enforcement, and edition-dependent EXT/SRC, FMT, and ITS
interpretation. 106-04 uses packet data type version `0x01` (Initial Release);
that header field was called Header Version in 106-04 §10.6.1.1.5.
See [standard evidence, API changes, and adoption steps](https://github.com/TelemetryWorks/irig106-types/blob/main/docs/STANDARD-VALUES-REVIEW.md).

**0.1.0 is the planned first real release**; it is not yet published.
Crates.io currently has only the 0.0.1 placeholder. Releases use GitHub
Actions and crates.io Trusted Publishing (OIDC), with owner approval through
the `crates-io` environment. Never store a registry token or publish locally.
See the [release procedure](https://github.com/TelemetryWorks/irig106-types/blob/main/docs/RELEASING.md).

## Chapter 4 binary time layout

`Ch4BinaryTime::from_secondary_bytes` and `from_intra_packet_bytes` read
four little-endian 16-bit words in this order:
`[reserved, microseconds, low_order, high_order]` (106-24R1 Chapter 11
Figure 11-4, printed 11-10; IPTS selection per §11.2.1.1 g / §11.2.1.3 b).
The previous parser swapped the microsecond and high-order words.

Chapter 4 §4.7 and Figures 4-4/4-5 define a full 32-bit binary count,
`high_order × 65536 + low_order`, at 10 ms per tick; high-order units are
655.36 s. Low `0x8000` with high zero means 327.68 s, so the low word's top
bit must survive. In the 106-24R1 baseline, a conforming microsecond word
is 0–9999, with its two leading positions zero. The review record below
separates the inferred 106-04–106-09 packet order and older drawing differences.
These raw parsers check length and preserve fields;
reserved-bit/range validation and conversion remain with the consumer.

`irig106-time` must advance its dependency after review/merge, remove any
workaround for the old word order, and replace 17-bit/day-of-year or 15-bit
low-word interpretations with the full binary count. Update hand-written
secondary-header and IPTS fixtures; PCM stream word insertion (Chapter 4)
is distinct from this Chapter 11 byte layout. No public signatures changed.
See the [page-image evidence and failing-test record](https://github.com/TelemetryWorks/irig106-types/blob/main/docs/STANDARD-VALUES-REVIEW.md#chapter-4-binary-time-layout-t-10).

## Development checks

```sh
cargo test --all-features
cargo test --no-default-features
cargo test --features serde
cargo +1.85.0 check --no-default-features
cargo +1.85.0 check
cargo +1.85.0 check --features serde
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
cargo build --target wasm32-unknown-unknown --no-default-features
```
