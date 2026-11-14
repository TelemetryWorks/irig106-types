//! # irig106-types
//!
//! Foundational, dependency-free types intended as the shared vocabulary of
//! the TelemetryWorks IRIG 106 crate ecosystem; currently used by
//! `irig106-time`.
//!
//! This crate holds only the *vocabulary* of IRIG 106: counters, wire-level
//! time representations, time-related enumerations, and unit/epoch newtypes.
//! It performs no I/O and no correlation. Interpretation of these values
//! (e.g., converting Chapter 4 binary time to an absolute time of day) lives
//! in the crates that consume them.
//!
//! ## Contents
//!
//! | Module | Types |
//! |--------|-------|
//! | [`rtc`] | [`Rtc`] (48-bit RTC), [`Ertc`] (64-bit extended RTC) |
//! | [`binary_time`] | [`Ch4BinaryTime`], [`Ieee1588Time`] |
//! | [`time_enums`] | [`TimeSource`], [`TimeFormat`], [`IrigTimeSource`], [`DateFormat`], [`SecHdrTimeFormat`] |
//! | [`units`] | [`UnixSeconds`], [`TaiSeconds`], [`TaiUtcOffset`], [`NanosDuration`], [`SignedUnixSeconds`], [`SignedNanos`], [`SignedRtcTicks`] |
//! | [`version`] | [`Irig106Version`] (separate setup/header code mappings) |
//! | [`error`] | [`TypesError`] |
//!
//! ## Features
//!
//! | Feature | Default | Effect |
//! |---------|---------|--------|
//! | `std` | yes | Implements `std::error::Error` for [`TypesError`] |
//! | `serde` | no | `Serialize`/`Deserialize` on all public value types, excluding [`TypesError`] |
//!
//! The crate is `#![no_std]` and requires neither `std` nor `alloc`.
//!
//! ## Example
//!
//! ```
//! use irig106_types::{NanosDuration, Rtc, TaiSeconds, TaiUtcOffset};
//!
//! let start = Rtc::from_le_bytes([0x80, 0x96, 0x98, 0x00, 0x00, 0x00]);
//! let end = Rtc::from_raw(start.as_raw() + 5_000_000);
//! assert_eq!(start.elapsed_nanos(end), NanosDuration::from_millis(500).unwrap().get());
//!
//! let utc = TaiSeconds::new(1_735_689_637).to_unix(TaiUtcOffset::new(37)).unwrap();
//! assert_eq!(utc.get(), 1_735_689_600); // 2025-01-01T00:00:00Z
//! ```

#![no_std]
#![deny(missing_docs)]

#[cfg(feature = "std")]
extern crate std;

pub mod binary_time;
pub mod error;
pub mod rtc;
pub mod time_enums;
pub mod units;
pub mod version;

pub use binary_time::{Ch4BinaryTime, Ieee1588Time};
pub use error::{Result, TypesError};
pub use rtc::{Ertc, Rtc};
pub use time_enums::{DateFormat, IrigTimeSource, SecHdrTimeFormat, TimeFormat, TimeSource};
pub use units::{
    NanosDuration, SignedNanos, SignedRtcTicks, SignedUnixSeconds, TaiSeconds, TaiUtcOffset,
    UnixSeconds,
};
pub use version::Irig106Version;
