//! Epoch and unit newtypes.
//!
//! These wrap raw integers wherever confusing the unit (seconds vs.
//! nanoseconds) or the epoch (Unix vs. TAI) is a realistic source of bugs.
//! Each is a zero-cost `#[repr(transparent)]` wrapper.
//!
//! | Newtype | Wraps | Meaning |
//! |---------|-------|---------|
//! | [`UnixSeconds`] | `u64` | Seconds since 1970-01-01T00:00:00 UTC |
//! | [`SignedUnixSeconds`] | `i128` | Signed Unix seconds; negative values precede 1970 |
//! | [`SignedRtcTicks`] | `i128` | Signed, unwrapped displacement in 100 ns RTC ticks |
//! | [`SignedNanos`] | `i128` | Signed nanosecond duration, with no epoch |
//! | [`TaiSeconds`] | `u64` | Seconds since the TAI/PTP epoch (1970-01-01T00:00:00 TAI) |
//! | [`TaiUtcOffset`] | `i32` | TAI − UTC, in seconds (accumulated leap seconds) |
//! | [`NanosDuration`] | `u64` | A duration in nanoseconds |

macro_rules! newtype {
    ($(#[$meta:meta])* $name:ident($inner:ty)) => {
        $(#[$meta])*
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "serde", serde(transparent))]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
        #[repr(transparent)]
        pub struct $name($inner);

        impl $name {
            #[doc = concat!("Wrap a raw `", stringify!($inner), "` value.")]
            #[inline]
            pub const fn new(value: $inner) -> Self {
                $name(value)
            }

            #[doc = concat!("Return the raw `", stringify!($inner), "` value.")]
            #[inline]
            pub const fn get(self) -> $inner {
                self.0
            }
        }
    };
}

newtype!(
    /// Seconds since the Unix epoch (1970-01-01T00:00:00 UTC).
    UnixSeconds(u64)
);

newtype!(
    /// Seconds since the TAI epoch used by IEEE-1588/PTP
    /// (1970-01-01T00:00:00 TAI).
    ///
    /// This is the PTP epoch, not the 1958 TAI epoch. Convert using a
    /// caller-supplied [`TaiUtcOffset`]; this crate does not choose an offset
    /// or resolve historical UTC/leap-second policy.
    TaiSeconds(u64)
);

newtype!(
    /// Caller-supplied TAI − UTC offset in whole seconds.
    ///
    /// This models the integral offsets used from 1972-01-01 onward; earlier
    /// fractional offsets cannot be represented. For example, the offset
    /// at 2017-01-01 was 37 seconds.
    TaiUtcOffset(i32)
);

newtype!(
    /// A duration measured in nanoseconds.
    NanosDuration(u64)
);

newtype!(
    /// Signed seconds relative to 1970-01-01T00:00:00 UTC (Unix epoch).
    ///
    /// Negative values represent times before 1970. This is an epoch value,
    /// not a duration; no calendar or leap-second interpretation is inferred.
    ///
    /// ```compile_fail
    /// use irig106_types::{SignedNanos, SignedUnixSeconds};
    /// let time: SignedUnixSeconds = SignedNanos::new(-1);
    /// ```
    SignedUnixSeconds(i128)
);

newtype!(
    /// Signed, unwrapped displacement in RTC ticks (100 ns per tick).
    ///
    /// This is neither a raw 48-bit counter nor an absolute timestamp.
    /// Session/wrap interpretation belongs to the caller.
    ///
    /// ```compile_fail
    /// use irig106_types::{NanosDuration, SignedRtcTicks};
    /// let nanos: NanosDuration = SignedRtcTicks::new(1);
    /// ```
    SignedRtcTicks(i128)
);

newtype!(
    /// Signed duration in nanoseconds, with no absolute epoch.
    ///
    /// Conversions to the unsigned [`NanosDuration`] explicitly distinguish
    /// preserving a nonnegative value from taking its magnitude.
    SignedNanos(i128)
);

impl UnixSeconds {
    /// Widen to signed Unix seconds without changing the epoch or value.
    #[inline]
    pub const fn to_signed(self) -> SignedUnixSeconds {
        SignedUnixSeconds(self.0 as i128)
    }
}

impl SignedUnixSeconds {
    /// Convert only if nonnegative and representable as `u64` Unix seconds.
    ///
    /// Returns `None` for pre-1970 values or values above `u64::MAX`.
    #[inline]
    pub fn to_unsigned(self) -> Option<UnixSeconds> {
        u64::try_from(self.0).ok().map(UnixSeconds)
    }
}

impl SignedRtcTicks {
    /// Convert at exactly 100 ns per tick, preserving sign.
    ///
    /// Returns `None` if multiplying by 100 overflows `i128`.
    #[inline]
    pub fn to_nanos(self) -> Option<SignedNanos> {
        self.0.checked_mul(100).map(SignedNanos)
    }
}

impl SignedNanos {
    /// A zero-length signed duration.
    pub const ZERO: Self = Self(0);

    /// Convert only if nonnegative and representable as a `u64` duration.
    ///
    /// Negative durations, including -1 ns, are rejected rather than repaired.
    #[inline]
    pub fn to_unsigned(self) -> Option<NanosDuration> {
        u64::try_from(self.0).ok().map(NanosDuration)
    }

    /// Explicit absolute magnitude, if representable as a `u64` duration.
    ///
    /// Returns `None` when the magnitude exceeds `u64::MAX`, including for
    /// `i128::MIN`. Taking the unsigned absolute value cannot overflow.
    #[inline]
    pub fn magnitude(self) -> Option<NanosDuration> {
        u64::try_from(self.0.unsigned_abs()).ok().map(NanosDuration)
    }
}

impl TaiSeconds {
    /// Convert PTP-epoch TAI seconds to unsigned Unix seconds using `offset`.
    ///
    /// Returns `None` if the result precedes 1970 or exceeds `u64::MAX`.
    /// No saturation, wrapping, or inferred offset is applied. Use
    /// [`Self::to_signed_unix`] to retain an exact integer result outside that range.
    ///
    /// Exact historical TAI/UTC conversion requires the correct whole-second
    /// TAI − UTC offset, as used from 1972-01-01 onward. Before 1972 the offset
    /// was fractional and cannot be represented by [`TaiUtcOffset`]. `Some`
    /// confirms integer range only, not historical validity of the offset.
    #[inline]
    pub fn to_unix(self, offset: TaiUtcOffset) -> Option<UnixSeconds> {
        self.to_signed_unix(offset).to_unsigned()
    }

    /// Convert to signed Unix seconds by subtracting the caller's TAI − UTC offset.
    ///
    /// Both inputs are widened to `i128` before subtraction. Every `u64`
    /// second count minus an `i32` offset fits in `i128`, so the integer
    /// subtraction is exact and infallible, including negative results and
    /// `i32::MIN`. This does not convert the 1958 TAI epoch or choose an offset.
    ///
    /// Exact historical TAI/UTC conversion requires the correct whole-second
    /// TAI − UTC offset, as used from 1972-01-01 onward. Before 1972 the offset
    /// was fractional; the caller-supplied whole-second [`TaiUtcOffset`]
    /// cannot represent it. A signed result does not remove this limitation.
    #[inline]
    pub fn to_signed_unix(self, offset: TaiUtcOffset) -> SignedUnixSeconds {
        SignedUnixSeconds(i128::from(self.0) - i128::from(offset.0))
    }
}

impl NanosDuration {
    /// A zero-length duration.
    pub const ZERO: NanosDuration = NanosDuration(0);

    /// Duration of `secs` seconds, or `None` if nanoseconds exceed `u64::MAX`.
    #[inline]
    pub const fn from_secs(secs: u64) -> Option<Self> {
        match secs.checked_mul(1_000_000_000) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    /// Duration of `millis` milliseconds, or `None` on nanosecond overflow.
    #[inline]
    pub const fn from_millis(millis: u64) -> Option<Self> {
        match millis.checked_mul(1_000_000) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    /// Duration of `micros` microseconds, or `None` on nanosecond overflow.
    #[inline]
    pub const fn from_micros(micros: u64) -> Option<Self> {
        match micros.checked_mul(1_000) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    /// Widen to a signed duration without changing the value.
    #[inline]
    pub const fn to_signed(self) -> SignedNanos {
        SignedNanos(self.0 as i128)
    }

    /// Duration in whole 100 ns RTC ticks (truncating).
    #[inline]
    pub const fn as_rtc_ticks(self) -> u64 {
        self.0 / 100
    }
}

#[cfg(test)]
#[path = "units_tests.rs"]
mod units_tests;
