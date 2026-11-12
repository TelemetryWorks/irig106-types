# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

Version remains 0.1.0 (unpublished); crates.io 0.0.1 is the placeholder.

### Fixed

- T-10: `Ch4BinaryTime::from_secondary_bytes` and `from_intra_packet_bytes` now read `[reserved, microseconds, low_order, high_order]` as little-endian 16-bit words (106-24R1 Figure 11-4). Previously high-order and microseconds were swapped. Signatures and raw field types are unchanged; all 32 high/low count bits are preserved, including low `0x8000` (327.68 s). Documented Chapter 4 §4.7 weighting, microsecond leading zeros, and downstream adoption.
- T-5: `Ertc::to_nanos` counts 1 ns per tick, retaining its `u128` return type; RTC stays at 100 ns.
- T-6: source 3 decodes as `Reserved(3)`; removed the incorrect `TimeSource::Gps` variant. Renamed `InternalRtc` to `InternalFromRmm` for SRC 2, including the serde variant name. The SRC mapping applies from 106-05; 106-04 defines EXT bit 0 and reserves bits 3–1.
- T-7: FMT 0xF decodes as the new `TimeFormat::None` (payload invalid) from 106-05 onward; 106-04 reserves FMT 0x6–0xF. Edition handling remains in `irig106-time`.
- Corrected the archive evidence: 106-04 Chapter 10 is in the full PDF (§10.6.3.3, Figure 10-15); clarified that §11.2.1.1 e lists version codes and Table 11-4 gives each data type's current version.

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
