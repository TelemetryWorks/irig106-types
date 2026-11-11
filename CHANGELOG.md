# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
