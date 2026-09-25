//! Epoch and unit newtypes.
//!
//! These wrap raw integers wherever confusing the unit (seconds vs.
//! nanoseconds) or the epoch (Unix vs. TAI) is a realistic source of bugs.
//! Each is a zero-cost `#[repr(transparent)]` wrapper.
//!
//! | Newtype | Wraps | Meaning |
//! |---------|-------|---------|
//! | [`UnixSeconds`] | `u64` | Seconds since 1970-01-01T00:00:00 UTC |
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
    /// Convert to [`UnixSeconds`] with [`TaiSeconds::to_unix`] and a
    /// [`TaiUtcOffset`].
    TaiSeconds(u64)
);

newtype!(
    /// The TAI − UTC offset in seconds, i.e. accumulated leap seconds.
    ///
    /// 37 seconds since 2017-01-01.
    TaiUtcOffset(i32)
);

newtype!(
    /// A duration measured in nanoseconds.
    NanosDuration(u64)
);

impl TaiSeconds {
    /// Convert to Unix (UTC) seconds by removing `offset` leap seconds.
    ///
    /// Saturates at `0` and `u64::MAX` rather than wrapping.
    #[inline]
    pub fn to_unix(self, offset: TaiUtcOffset) -> UnixSeconds {
        let off = offset.0;
        UnixSeconds(if off >= 0 {
            self.0.saturating_sub(off as u64)
        } else {
            self.0.saturating_add(off.unsigned_abs() as u64)
        })
    }
}

impl NanosDuration {
    /// A zero-length duration.
    pub const ZERO: NanosDuration = NanosDuration(0);

    /// Duration of `secs` seconds (saturating at `u64::MAX` nanoseconds).
    #[inline]
    pub const fn from_secs(secs: u64) -> Self {
        NanosDuration(secs.saturating_mul(1_000_000_000))
    }

    /// Duration of `millis` milliseconds (saturating at `u64::MAX` nanoseconds).
    #[inline]
    pub const fn from_millis(millis: u64) -> Self {
        NanosDuration(millis.saturating_mul(1_000_000))
    }

    /// Duration of `micros` microseconds (saturating at `u64::MAX` nanoseconds).
    #[inline]
    pub const fn from_micros(micros: u64) -> Self {
        NanosDuration(micros.saturating_mul(1_000))
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
