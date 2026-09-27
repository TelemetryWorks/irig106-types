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
| `rtc` | `Rtc`, `Ertc` | 48-bit RTC (masked, wrap-aware elapsed time) and 64-bit extended RTC, 100 ns/tick |
| `binary_time` | `Ch4BinaryTime`, `Ieee1588Time` | Raw secondary-header / intra-packet time fields |
| `time_enums` | `TimeSource`, `TimeFormat`, `DateFormat`, `SecHdrTimeFormat` | Decoded CSDW / packet-flag fields |
| `units` | `UnixSeconds`, `TaiSeconds`, `TaiUtcOffset`, `NanosDuration` | `#[repr(transparent)]` newtypes that prevent unit and epoch mix-ups |
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
