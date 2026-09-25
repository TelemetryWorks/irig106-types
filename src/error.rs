//! Error type for fallible constructors in `irig106-types`.

use core::fmt;

/// Result alias for `irig106-types` operations.
pub type Result<T> = core::result::Result<T, TypesError>;

/// Errors produced when constructing foundational types from raw bytes.
///
/// Downstream crates typically convert this into their own error type
/// (e.g., `irig106-time` implements `From<TypesError> for TimeError`).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TypesError {
    /// Input buffer was shorter than required.
    BufferTooShort {
        /// Number of bytes required.
        expected: usize,
        /// Number of bytes actually available.
        actual: usize,
    },

    /// A decoded field value exceeded its valid range.
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
