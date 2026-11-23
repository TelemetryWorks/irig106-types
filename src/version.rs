//! Separate setup-record and packet-header version-code mappings.
//!
//! These map the codes listed in 106-24R1, not an inferred recording edition.
//! A packet header's data type version identifies that data type's layout;
//! it must not be used as the setup record's declared recording version.

/// RCC release named by a setup-record or packet-header version code.
///
/// The two fields use different code tables: use the constructor named for
/// the field. Neither table assigns a code to 106-23, 106-24, or 106-24R1.
/// Their tables retain the 106-22 entry (unchanged through 106-24R1).
/// Unknown/reserved codes are preserved, never guessed as a later release.
/// Codes are dimensionless bytes (`0..=255`), not edition numbers themselves.
///
/// The setup table is 106-24R1 §11.2.7.2 / Figure 11-34; the separate
/// header list is §11.2.1.1 e (Table 11-4 lists current per-type versions).
/// The variants below name entries in those lists, not proof of the recording
/// edition. The same variant can correspond to different raw bytes in the
/// two fields. No reverse mapping or policy decision is implied.
///
/// ```
/// use irig106_types::Irig106Version;
/// assert_eq!(Irig106Version::from_setup_record(7), Irig106Version::V07);
/// assert_eq!(Irig106Version::from_data_type_version(3), Irig106Version::V07);
/// assert_eq!(Irig106Version::from_setup_record(15), Irig106Version::Unknown(15));
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Irig106Version {
    /// RCC 106-04: no setup code; header code `0x01`.
    V04,
    /// RCC 106-05: no setup code; header code `0x02`.
    V05,
    /// RCC 106-07: setup code `0x07`, header code `0x03`.
    V07,
    /// RCC 106-09: setup code `0x08`, header code `0x04`.
    V09,
    /// RCC 106-11: setup code `0x09`, header code `0x05`.
    V11,
    /// RCC 106-13: setup code `0x0A`, header code `0x06`.
    V13,
    /// RCC 106-15: setup code `0x0B`, header code `0x07`.
    V15,
    /// RCC 106-17: setup code `0x0C`, header code `0x08`.
    V17,
    /// RCC 106-19: setup code `0x0D`, header code `0x09`.
    V19,
    /// RCC 106-22: setup code `0x0E`, header code `0x0A`.
    /// The table entry remains unchanged through 106-24R1.
    V22,
    /// Reserved or unrecognized byte (`0..=255`), preserved unchanged.
    ///
    /// Direct construction also permits codes named by a table; it does not
    /// retain which of the two fields originally supplied the byte.
    Unknown(u8),
}

impl Irig106Version {
    /// Decode RCCVER from bits 7–0 of a setup-record (TMATS) CSDW.
    ///
    /// 106-24R1 §11.2.7.2, Figure 11-34: 0x07–0x0E name 106-07
    /// through 106-22; 0x00–0x06 and 0x0F–0xFF are reserved.
    /// A zero value is unknown, not evidence of a pre-07 recording.
    /// This is the Format 1 setup record, not Format 4 streaming configuration.
    #[inline]
    pub fn from_setup_record(val: u8) -> Self {
        match val {
            0x07 => Self::V07,
            0x08 => Self::V09,
            0x09 => Self::V11,
            0x0A => Self::V13,
            0x0B => Self::V15,
            0x0C => Self::V17,
            0x0D => Self::V19,
            0x0E => Self::V22,
            other => Self::Unknown(other),
        }
    }

    /// Decode the packet primary header's data type version byte.
    ///
    /// 106-24R1 §11.2.1.1 e lists the version codes: 0x01–0x0A name
    /// 106-04 through 106-22; 0x00 and 0x0B–0xFF are reserved.
    /// Table 11-4 separately gives each data type's current version.
    /// This is not the header's data type byte or the setup-record RCCVER.
    #[inline]
    pub fn from_data_type_version(val: u8) -> Self {
        match val {
            0x01 => Self::V04,
            0x02 => Self::V05,
            0x03 => Self::V07,
            0x04 => Self::V09,
            0x05 => Self::V11,
            0x06 => Self::V13,
            0x07 => Self::V15,
            0x08 => Self::V17,
            0x09 => Self::V19,
            0x0A => Self::V22,
            other => Self::Unknown(other),
        }
    }
}
