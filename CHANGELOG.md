# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

Version remains 0.1.0 (unpublished); crates.io 0.0.1 is the placeholder.

### Fixed

- T-5: `Ertc::to_nanos` counts 1 ns per tick, retaining its `u128` return type; RTC stays at 100 ns.
- T-6: source 3 decodes as `Reserved(3)`; removed the incorrect `TimeSource::Gps` variant. SRC 2 means internal from RMM; the `InternalRtc` name remains for compatibility.
- T-7: FMT 0xF decodes as the new `TimeFormat::None` (payload invalid).

### Added

- T-7: `IrigTimeSource` and `from_raw` for ITS values 0–7 and reserved values, with edition/format applicability documented.
- T-9: `version::Irig106Version`, re-exported at the crate root, with separate `from_setup_record` and `from_data_type_version` mappings. Reserved codes, including 0x0F and setup-record zero, remain `Unknown(raw)`; no invented 106-23 code.
- Standard-derived regression tests and a review/adoption record in `docs/STANDARD-VALUES-REVIEW.md`.

### Changed

- Edition 2024 and MSRV 1.85, the same across the IRIG 106 ecosystem (owner, 2026-09-27); CI checks 1.85. Formatting follows the 2024 style edition.

## [0.1.0] - 2026-09-24

### Added

- `Rtc` (48-bit Relative Time Counter) and `Ertc` (64-bit Extended RTC), migrated from `irig106-time`.
- `Ch4BinaryTime` and `Ieee1588Time` wire-level time representations, migrated from `irig106-time`.
- `TimeSource`, `TimeFormat`, `DateFormat`, `SecHdrTimeFormat` enums, migrated from `irig106-time`.
- Unit/epoch newtypes: `UnixSeconds`, `TaiSeconds` (with `to_unix`), `TaiUtcOffset`, `NanosDuration`.
- `TypesError` with `BufferTooShort` and `OutOfRange` variants.
- `no_std` support, optional `std` and `serde` features. MSRV 1.60.
