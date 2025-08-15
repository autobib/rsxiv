#[cfg(test)]
mod tests;

use std::num::NonZero;

use super::IdError;

/// Implement `?` propogation in const context.
macro_rules! tri {
    ($e:expr $(,)?) => {
        match $e {
            core::result::Result::Ok(val) => val,
            core::result::Result::Err(err) => return core::result::Result::Err(err),
        }
    };
}

pub(crate) use tri;

#[inline]
pub const fn number_and_version_len_3(
    number_and_version: &[u8],
) -> Result<(NonZero<u32>, u16), IdError> {
    match number_and_version {
        [
            b1 @ b'0'..=b'9',
            b2 @ b'0'..=b'9',
            b3 @ b'0'..=b'9',
            tail @ ..,
        ] => {
            let number = 100 * (b1.saturating_sub(b'0') as u32)
                + 10 * (b2.saturating_sub(b'0') as u32)
                + (b3.saturating_sub(b'0') as u32);

            let Some(nz_number) = NonZero::new(number) else {
                return Err(IdError::NumberOutOfRange);
            };

            match tail {
                [b'v', ver @ ..] => Ok((nz_number, tri!(version(ver)))),
                [] => Ok((nz_number, 0)),
                [b'0'..=b'9'] => Err(IdError::NumberOutOfRange),
                _ => Err(IdError::InvalidVersion),
            }
        }
        _ => Err(IdError::InvalidNumber),
    }
}

#[inline]
pub const fn number_and_version_len_4(
    number_and_version: &[u8],
) -> Result<(NonZero<u32>, u16), IdError> {
    match number_and_version {
        [
            b1 @ b'0'..=b'9',
            b2 @ b'0'..=b'9',
            b3 @ b'0'..=b'9',
            b4 @ b'0'..=b'9',
            tail @ ..,
        ] => {
            let number = 1000 * (b1.saturating_sub(b'0') as u32)
                + 100 * (b2.saturating_sub(b'0') as u32)
                + 10 * (b3.saturating_sub(b'0') as u32)
                + (b4.saturating_sub(b'0') as u32);

            let Some(nz_number) = NonZero::new(number) else {
                return Err(IdError::NumberOutOfRange);
            };

            match tail {
                [b'v', ver @ ..] => Ok((nz_number, tri!(version(ver)))),
                [] => Ok((nz_number, 0)),
                [b'0'..=b'9'] => Err(IdError::NumberOutOfRange),
                _ => Err(IdError::InvalidVersion),
            }
        }
        _ => Err(IdError::InvalidNumber),
    }
}

#[inline]
pub const fn number_and_version_len_5(
    number_and_version: &[u8],
) -> Result<(NonZero<u32>, u16), IdError> {
    match number_and_version {
        [
            b1 @ b'0'..=b'9',
            b2 @ b'0'..=b'9',
            b3 @ b'0'..=b'9',
            b4 @ b'0'..=b'9',
            b5 @ b'0'..=b'9',
            tail @ ..,
        ] => {
            let number = 10000 * (b1.saturating_sub(b'0') as u32)
                + 1000 * (b2.saturating_sub(b'0') as u32)
                + 100 * (b3.saturating_sub(b'0') as u32)
                + 10 * (b4.saturating_sub(b'0') as u32)
                + (b5.saturating_sub(b'0') as u32);

            let Some(nz_number) = NonZero::new(number) else {
                return Err(IdError::NumberOutOfRange);
            };

            match tail {
                [b'v', ver @ ..] => Ok((nz_number, tri!(version(ver)))),
                [] => Ok((nz_number, 0)),
                [b'0'..=b'9'] => Err(IdError::NumberOutOfRange),
                _ => Err(IdError::InvalidVersion),
            }
        }
        _ => Err(IdError::InvalidNumber),
    }
}

/// Parse a new-style date block, checking length and checking for validity of dates.
///
/// Returns `(a, b)`, where the year is `a + 1991` and `b` lands in the range `[1..=12]`, indicating the month.
#[inline]
pub const fn date_new(date: [u8; 4]) -> Result<(u8, u8), IdError> {
    match date {
        [b1 @ b'0'..=b'9', b2 @ b'0'..=b'9', b3, b4] => {
            let y1 = b1 - b'0';
            let y2 = b2 - b'0';

            // convert bytes to values and check ranges
            let m1 = b3.overflowing_sub(b'0').0;
            let m2 = b4.overflowing_sub(b'0').0;

            // month is invalid format
            if !(m1 == 0 && (1 <= m2 && m2 <= 9) || m1 == 1 && m1 <= 2) {
                return Err(IdError::InvalidDate);
            }

            // the first new-style arxiv entry is April 2007; 9 is the magic number since
            // `9 + 1991 = 2000`
            let years_since_epoch = if (y1 == 0) && ((y2 <= 6) || (y2 == 7 && m1 == 0 && m2 <= 3)) {
                100 + 9 + y2
            } else {
                9 + 10 * y1 + y2
            };

            let month = 10 * m1 + m2;

            Ok((years_since_epoch, month))
        }
        _ => Err(IdError::InvalidDate),
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DateNumber {
    pub years_since_epoch: u8,
    pub month: u8,
    pub number: NonZero<u32>,
    pub version: u16,
}

#[inline]
pub const fn date_number(datestamp: &[u8]) -> Result<DateNumber, IdError> {
    match datestamp {
        [b1, b2, b3, b4, tail @ ..] => {
            let (years_since_epoch, month) = tri!(date_old([*b1, *b2, *b3, *b4]));
            let (number, version) = tri!(number_and_version_len_3(tail));
            Ok(DateNumber {
                years_since_epoch,
                month,
                number,
                version,
            })
        }
        _ => Err(IdError::InvalidDate),
    }
}

/// Parse an old-style date block.
#[inline]
pub const fn date_old(date: [u8; 4]) -> Result<(u8, u8), IdError> {
    match date {
        [b1 @ b'0'..=b'9', b2 @ b'0'..=b'9', b3, b4] => {
            // convert bytes to values and check ranges
            let y1 = b1 - b'0';
            let y2 = b2 - b'0';

            let m1 = b3.overflowing_sub(b'0').0;
            let m2 = b4.overflowing_sub(b'0').0;

            // month is invalid format
            if !(m1 == 0 && (1 <= m2 && m2 <= 9) || m1 == 1 && m2 <= 2) {
                return Err(IdError::InvalidDate);
            }

            // earliest date is August 1991 and latest is March 2007
            if !(y1 == 9 && (1 <= y2 && y2 <= 9) || y1 == 0 && y2 <= 7)
                || (y1 == 9 && y2 == 1 && m2 <= 7)
                || (y1 == 0 && y2 == 7 && m2 >= 4)
            {
                return Err(IdError::DateOutOfRange);
            }

            // compute distance from 1991
            let years_since_epoch = if y1 == 9 { y2 - 1 } else { y2 + 9 };

            let month = 10 * m1 + m2;

            // convert to u16
            Ok((years_since_epoch, month))
        }
        _ => Err(IdError::InvalidDate),
    }
}

#[inline]
const fn version(version: &[u8]) -> Result<u16, IdError> {
    // the `saturating_sub` calls will all be optimized out because of the match bounds
    match version {
        [d1 @ b'1'..=b'9'] => Ok(d1.saturating_sub(b'0') as u16),
        [d2 @ b'1'..=b'9', d1 @ b'0'..=b'9'] => {
            Ok(10 * (d2.saturating_sub(b'0') as u16) + (d1.saturating_sub(b'0') as u16))
        }
        [d3 @ b'1'..=b'9', d2 @ b'0'..=b'9', d1 @ b'0'..=b'9'] => Ok(100
            * (d3.saturating_sub(b'0') as u16)
            + 10 * (d2.saturating_sub(b'0') as u16)
            + (d1.saturating_sub(b'0') as u16)),
        [
            d4 @ b'1'..=b'9',
            d3 @ b'0'..=b'9',
            d2 @ b'0'..=b'9',
            d1 @ b'0'..=b'9',
        ] => Ok(1000 * (d4.saturating_sub(b'0') as u16)
            + 100 * (d3.saturating_sub(b'0') as u16)
            + 10 * (d2.saturating_sub(b'0') as u16)
            + (d1.saturating_sub(b'0') as u16)),
        [
            d5 @ b'1'..=b'9',
            d4 @ b'0'..=b'9',
            d3 @ b'0'..=b'9',
            d2 @ b'0'..=b'9',
            d1 @ b'0'..=b'9',
        ] => {
            // check for overflow by first fitting into a u32
            let val_u32 = 10000 * (d5.saturating_sub(b'0') as u32)
                + 1000 * (d4.saturating_sub(b'0') as u32)
                + 100 * (d3.saturating_sub(b'0') as u32)
                + 10 * (d2.saturating_sub(b'0') as u32)
                + (d1.saturating_sub(b'0') as u32);

            if val_u32 > u16::MAX as u32 {
                Err(IdError::InvalidVersion)
            } else {
                Ok(val_u32 as u16)
            }
        }
        _ => Err(IdError::InvalidVersion),
    }
}
