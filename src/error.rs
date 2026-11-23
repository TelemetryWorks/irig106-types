//! Errors from unpacking individual time fields, not whole-packet validation.
//!
//! These are Rust API errors, not IRIG wire codes. Buffer sizes come from
//! 106-24R1 Figures 11-4–11-6; the PTP fraction range accompanies Figure 11-5.
//! Numeric conversion overflow in [`crate::units`] uses `Option`, not this enum.
//! No edition selection or validation policy is encoded here.

use core::fmt;

/// Result of an individual field constructor, with [`TypesError`] on failure.
pub type Result<T> = core::result::Result<T, TypesError>;

/// Length/range failures reported by individual byte constructors.
///
/// This non-exhaustive enum can gain variants; consumers must allow a fallback.
/// Error payloads describe the offending input without allocating. With `std`,
/// it implements `std::error::Error`; it has no serde implementation.
///
/// Downstream crates typically convert this into their own error type
/// (e.g., `irig106-time` implements `From<TypesError> for TimeError`).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TypesError {
    /// Fewer bytes than the field layout requires; `actual < expected`
    /// when returned by this crate's constructors (both counts are bytes).
    /// Public construction does not enforce that relation.
    BufferTooShort {
        /// Number of bytes required.
        expected: usize,
        /// Number of bytes actually available.
        actual: usize,
    },

    /// A decoded unsigned field exceeded its inclusive maximum.
    ///
    /// The PTP constructor reports nanoseconds ≥ 1_000_000_000 with
    /// `max = 999_999_999`; units follow `field`, not the error type itself.
    /// Public construction need not satisfy `value > max`.
    OutOfRange {
        /// Name of the field (e.g., "ieee1588_nanoseconds").
        field: &'static str,
        /// The actual decoded value.
        value: u32,
        /// The maximum allowed value (inclusive).
        max: u32,
    },
}

impl fmt::Display for TypesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypesError::BufferTooShort { expected, actual } => {
                write!(
                    f,
                    "buffer too short: expected {expected} bytes, got {actual}"
                )
            }
            TypesError::OutOfRange { field, value, max } => {
                write!(f, "{field} value {value} exceeds maximum {max}")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for TypesError {}

/// Return `BufferTooShort` unless `buf` holds at least `expected` bytes.
#[inline]
pub(crate) fn require_len(buf: &[u8], expected: usize) -> Result<()> {
    if buf.len() < expected {
        Err(TypesError::BufferTooShort {
            expected,
            actual: buf.len(),
        })
    } else {
        Ok(())
    }
}
