//! Shared scalar packet-header observations, without a dependency on core.
//!
//! IRIG 106-24R1 §11.2.1.1 b–i and Table 11-2 define the header fields;
//! Table 11-4 names data-type codes. [`PacketFacts`] adds caller-supplied byte
//! provenance, not a wire field. The intended integration has core produce facts;
//! an orchestrator passes borrowed facts and a separate borrowed DATA slice to
//! the selected consumer. These cross-crate adapters remain planned.
//! No packet parsing, I/O, allocation, validation policy, TMATS assembly or time
//! interpretation occurs here. The presence of a value never proves trust,
//! conformance, a recording edition or a known session start.

use crate::{Rtc, SecHdrTimeFormat};

/// Small, copyable observations of one packet's primary header and offset.
///
/// The nine private fields retain their supplied values; construction performs
/// no bounds, flag-consistency, checksum, edition or semantic validation.
/// Fields use IRIG 106-24R1 §11.2.1.1 b–i / Table 11-2. Older editions use
/// Chapter 10 numbering; consumers establish edition applicability separately.
/// Offset is caller provenance in bytes; RTC is a 100 ns counter, not UTC.
///
/// No payload, CSDW, trust grade, session identity or heap storage is retained.
/// Borrow this value as `&PacketFacts` and pass DATA separately as `&[u8]`.
/// The layout budget is at most 32 bytes on x86_64, aarch64 and wasm32, enforced
/// by target-specific tests, not a stable ABI or packed wire layout guarantee.
///
/// With `serde`, the representation is a struct with the nine getter names as
/// fields. Derived deserialization is not validation; in particular, it can
/// retain an out-of-48-bit [`Rtc`] just as that type's existing serde does.
///
/// # Constructor evolution
///
/// [`Self::new_v1`] permanently requires these nine observations; its signature
/// will not grow. Future observations must be explicitly optional, with V1
/// recording absence rather than inventing a value. A later constructor can
/// supply them. Non-exhaustiveness alone does not protect constructor callers.
/// There is no `Default`: callers must supply every V1 observation explicitly.
/// Future fields also require review of the size budget and serde contract.
///
/// ```
/// use irig106_types::{KnownDataType, PacketFacts, Rtc};
/// let facts = PacketFacts::new_v1(100, Rtc::from_raw(42), 44, 8, 7, 1, 9, 255, 0x80);
/// let data = [0x0E, 0, 0, 0, b'S', b':', b'a', b';'];
/// fn observe<'a>(facts: &PacketFacts, data: &'a [u8]) -> (u64, &'a [u8]) {
///     (facts.byte_offset(), data)
/// }
/// assert_eq!(facts.known_data_type(), Some(KnownDataType::ComputerGeneratedFormat1));
/// assert!(facts.has_secondary_header());
/// let (offset, borrowed) = observe(&facts, &data);
/// assert_eq!(offset, 100);
/// assert_eq!(borrowed.as_ptr(), data.as_ptr());
/// ```
///
/// Downstream construction uses the versioned constructor, not struct literals:
///
/// ```compile_fail
/// use irig106_types::{PacketFacts, Rtc};
/// let facts = PacketFacts {
///     byte_offset: 0, rtc: Rtc::ZERO, packet_length: 24, data_length: 0,
///     channel_id: 0, data_type: 1, data_type_version: 9, sequence: 0, flags: 0,
/// };
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct PacketFacts {
    byte_offset: u64,
    rtc: Rtc,
    packet_length: u32,
    data_length: u32,
    channel_id: u16,
    data_type: u8,
    data_type_version: u8,
    sequence: u8,
    flags: u8,
}

impl PacketFacts {
    /// Store all nine V1 observations exactly as supplied, with no validation.
    ///
    /// Arguments have the units/domains of their corresponding getters:
    /// byte offset (`u64`), existing [`Rtc`], two byte lengths (`u32`), channel
    /// (`u16`), then raw data type, version, sequence and flags (`u8`).
    /// Header-field authority: §11.2.1.1 b–i / Table 11-2; offset is caller
    /// provenance. Unknown codes, contradictory lengths/flags and an existing
    /// deserialized out-of-domain RTC are neither rejected nor repaired.
    ///
    /// This signature is frozen. Future optional observations are absent when
    /// constructing through V1; never append parameters or fabricated defaults.
    /// See [`PacketFacts`]'s constructor-evolution contract.
    #[allow(
        clippy::too_many_arguments,
        reason = "V1 requires every observation explicitly and its public signature is frozen"
    )]
    #[inline]
    pub const fn new_v1(
        byte_offset: u64,
        rtc: Rtc,
        packet_length: u32,
        data_length: u32,
        channel_id: u16,
        data_type: u8,
        data_type_version: u8,
        sequence: u8,
        flags: u8,
    ) -> Self {
        Self {
            byte_offset,
            rtc,
            packet_length,
            data_length,
            channel_id,
            data_type,
            data_type_version,
            sequence,
            flags,
        }
    }

    /// Packet-sync start offset in the caller's stream, `0..=u64::MAX` bytes.
    ///
    /// API provenance, not an IRIG wire field; zero does not assert a session
    /// start. Checked offset arithmetic and stream identity belong to callers.
    #[inline]
    pub const fn byte_offset(self) -> u64 {
        self.byte_offset
    }

    /// Supplied primary RTC, without masking, unwrapping or epoch conversion.
    ///
    /// §11.2.1.1 i: 48-bit count at 100 ns/tick. See [`Rtc`] for its constructor
    /// domain and the out-of-domain serde caveat; these facts add no validation.
    #[inline]
    pub const fn rtc(self) -> Rtc {
        self.rtc
    }

    /// Raw declared whole-packet length, `0..=u32::MAX` bytes (§11.2.1.1 c).
    ///
    /// Includes headers, DATA and trailer. This does not prove alignment,
    /// minimum length, available input, or an acceptable allocation size.
    #[inline]
    pub const fn packet_length(self) -> u32 {
        self.packet_length
    }

    /// Raw declared DATA length, `0..=u32::MAX` bytes (§11.2.1.1 d).
    ///
    /// Includes CSDW and any intra-packet headers/timestamps, excluding packet
    /// headers and trailer filler/checksum. Not a guarantee of complete capture.
    #[inline]
    pub const fn data_length(self) -> u32 {
        self.data_length
    }

    /// Raw channel ID, `0..=u16::MAX` (§11.2.1.1 b), without interpretation.
    ///
    /// No channel-use conformance, source identity or per-channel joining
    /// policy is inferred; those can depend on the recording edition.
    #[inline]
    pub const fn channel_id(self) -> u16 {
        self.channel_id
    }

    /// Original routing byte, `0..=255` (§11.2.1.1 h / Table 11-4).
    ///
    /// Unknown/reserved/unnamed codes remain unchanged; the optional named
    /// view is [`Self::known_data_type`], not a validity or trust check.
    #[inline]
    pub const fn data_type(self) -> u8 {
        self.data_type
    }

    /// Original data-type version byte, `0..=255` (§11.2.1.1 e).
    ///
    /// This is a layout version, not the recording edition or setup RCCVER.
    /// Consumers can explicitly use [`crate::Irig106Version::from_data_type_version`].
    #[inline]
    pub const fn data_type_version(self) -> u8 {
        self.data_type_version
    }

    /// Original per-channel sequence byte, `0..=255` (§11.2.1.1 f).
    ///
    /// Modulo-256 progression, gap detection, joining and ordering are caller
    /// responsibilities. This value is not a global packet position.
    #[inline]
    pub const fn sequence(self) -> u8 {
        self.sequence
    }

    /// Original flag byte, `0..=255`, without filtering (§11.2.1.1 g).
    ///
    /// Mechanical views below neither validate combinations nor apply an
    /// edition policy. Every bit remains available for external interpretation.
    #[inline]
    pub const fn flags(self) -> u8 {
        self.flags
    }

    /// Computed, limited Table 11-4 view of codes 0x01, 0x11 and 0x12.
    ///
    /// `None` means unnamed by this API, including other valid standard codes;
    /// it does not mean invalid or reserved. A named 0x12 is not evidence that
    /// its format applies before 106-17 or that a time marker is trusted.
    #[inline]
    pub const fn known_data_type(self) -> Option<KnownDataType> {
        match self.data_type {
            0x01 => Some(KnownDataType::ComputerGeneratedFormat1),
            0x11 => Some(KnownDataType::TimeFormat1),
            0x12 => Some(KnownDataType::TimeFormat2),
            _ => None,
        }
    }

    /// Whether flag bit 7 declares a secondary header (§11.2.1.1 g).
    ///
    /// This is a boolean bit view, not proof that twelve bytes exist or pass
    /// checksum/bounds checks. No other bits are examined or repaired.
    #[inline]
    pub const fn has_secondary_header(self) -> bool {
        self.flags & 0x80 != 0
    }

    /// Whether flag bit 6 selects secondary time for IPTS (§11.2.1.1 g).
    ///
    /// False selects the primary RTC. True with bit 7 clear is retained even
    /// though the baseline requires secondary-header presence; no repair occurs.
    #[inline]
    pub const fn intra_packet_uses_secondary_time(self) -> bool {
        self.flags & 0x40 != 0
    }

    /// Reported RTC sync-error bit 5 (§11.2.1.1 g), as a boolean.
    ///
    /// This neither computes synchronization nor establishes an overall trust
    /// grade or a time-correlation policy.
    #[inline]
    pub const fn rtc_sync_error(self) -> bool {
        self.flags & 0x20 != 0
    }

    /// Reported data-overflow bit 4 (§11.2.1.1 g), as a boolean.
    ///
    /// A reported overflow may or may not indicate payload loss; the bit view
    /// does not establish whether data was lost or which observations to adopt.
    #[inline]
    pub const fn data_overflow(self) -> bool {
        self.flags & 0x10 != 0
    }

    /// Secondary-time format from flag bits 3–2 (§11.2.1.1 g, Figures 11-4–6).
    ///
    /// Reuses [`SecHdrTimeFormat::from_packet_flags`], including reserved 0b11.
    /// Does not check edition, secondary-header presence or the IPTS selection.
    #[inline]
    pub fn secondary_time_format(self) -> SecHdrTimeFormat {
        SecHdrTimeFormat::from_packet_flags(self.flags)
    }

    /// Declared data-checksum width in bytes: 0, 1, 2 or 4 (§11.2.1.1 g).
    ///
    /// Flag bits 1–0 = 00/01/10/11 select none/8/16/32 bits respectively.
    /// This does not locate, compute or verify a checksum or enforce bounds.
    #[inline]
    pub const fn data_checksum_bytes(self) -> u8 {
        match self.flags & 0x03 {
            0 => 0,
            1 => 1,
            2 => 2,
            _ => 4,
        }
    }
}

/// Limited names for three raw data-type bytes in 106-24R1 Table 11-4.
///
/// Obtained through [`PacketFacts::known_data_type`]; all original bytes stay
/// in [`PacketFacts::data_type`]. The named subset may grow. Absence of a name
/// does not mean the standard reserves a code; no variant proves applicable
/// edition, body validity, trust or eligibility for setup closure. Variants are
/// names, not a wire encoding; retain/use the original data-type getter rather
/// than enum casts. Serde uses the derived enum representation, not numeric wire codes.
///
/// Exhaustive matching is intentionally unavailable to downstream crates:
///
/// ```compile_fail
/// use irig106_types::KnownDataType;
/// fn closed(code: KnownDataType) {
///     match code {
///         KnownDataType::ComputerGeneratedFormat1 => (),
///         KnownDataType::TimeFormat1 => (),
///         KnownDataType::TimeFormat2 => (),
///     }
/// }
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum KnownDataType {
    /// Code 0x01: Computer-Generated Data, Format 1 (Setup Record), Table 11-4.
    ///
    /// No CSDW interpretation, assembly, completeness or adoption is implied.
    ComputerGeneratedFormat1,
    /// Code 0x11: Time Data, Format 1, Table 11-4 / §11.2.3.2.
    ///
    /// Names the packet format; does not decode time or establish marker trust.
    TimeFormat1,
    /// Code 0x12: Time Data, Format 2 (Network Time), Table 11-4 / §11.2.3.3.
    ///
    /// Defined from 106-17; a baseline name does not make it eligible in older
    /// recordings. Edition and closure policy remain with consumers.
    TimeFormat2,
}
