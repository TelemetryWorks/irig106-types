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
//! ## Scope and consumers
//!
//! Shared, stable, plain values belong here when several crates need the same
//! units, field codes, or wire representation. Packet framing/parsing, I/O,
//! decoding, validation policy, TMATS assembly, calendar interpretation, and
//! time correlation belong to consumers. The small byte constructors here
//! only unpack individual time fields; they do not parse whole packets.
//!
//! | Crate | Role in the intended architecture | Shared vocabulary |
//! |---|---|---|
//! | `irig106-core` | Produces packet-header facts after framing/parsing | Counters and header field values |
//! | `irig106-tmats` | Consumes setup facts passed by an orchestrator; assembles setup records | Version codes and shared header values |
//! | `irig106-time` | Interprets and correlates time | Counters, time words, CSDW enums, signed quantities |
//! | `irig106-decode` | Decodes payloads using the shared vocabulary | Counters, units, field codes as needed |
//! | `irig106-cli` | Orchestrates the libraries and presents results | Values passed between libraries |
//!
//! This is a responsibility map, not a claim that all five crates currently
//! depend on this crate. The established consumer is `irig106-time`; the
//! packet-facts API for core/TMATS is a separate design and is not defined here.
//! TMATS and time must not need a dependency on core merely to exchange values.
//!
//! ## Reading contracts
//!
//! Citations use IRIG 106-24R1 unless another edition is named. Chapter 11
//! §11.2.1.1 defines primary-header fields; Figures 11-4–11-6 define secondary
//! time layouts; §11.2.3.2 a / Figure 11-12 defines Format 1 CSDW codes;
//! §11.2.7.2 / Figure 11-34 defines setup-record version codes. Unit wrappers,
//! checked arithmetic, and Rust errors are library abstractions, not additional
//! standard wire fields. Their docs specify their integer domains explicitly.
//!
//! Raw values are not evidence of conformance. Public time-word fields allow
//! arbitrary integers, enum constructors preserve unknown codes, and optional
//! derived serde deserialization does not run byte-constructor checks or RTC
//! masking. Consumers validate untrusted values before applying their policy.
//! No method chooses a recording edition or leap-second table.
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
#![deny(rustdoc::broken_intra_doc_links)]

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
