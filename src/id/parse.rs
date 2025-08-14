#[cfg(test)]
mod tests;

use std::num::NonZero;

use super::IdentifierError;

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

pub const fn number_and_version_len_3(
    number_and_version: &[u8],
) -> Result<(NonZero<u16>, Option<NonZero<u8>>), IdentifierError> {
    match number_and_version {
        [
            b1 @ b'0'..=b'9',
            b2 @ b'0'..=b'9',
            b3 @ b'0'..=b'9',
            tail @ ..,
        ] => {
            let number = 100 * (b1.saturating_sub(b'0') as u16)
                + 10 * (b2.saturating_sub(b'0') as u16)
                + (b3.saturating_sub(b'0') as u16);

            let Some(nz_number) = NonZero::new(number) else {
                return Err(IdentifierError::NumberOutOfRange);
            };

            match tail {
                [b'v', ver @ ..] => Ok((nz_number, Some(tri!(version(ver))))),
                [] => Ok((nz_number, None)),
                [b'0'..=b'9'] => Err(IdentifierError::NumberOutOfRange),
                _ => Err(IdentifierError::InvalidVersion),
            }
        }
        _ => Err(IdentifierError::InvalidNumber),
    }
}

pub const fn number_and_version_len_4(
    number_and_version: &[u8],
) -> Result<(NonZero<u32>, Option<NonZero<u8>>), IdentifierError> {
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
                return Err(IdentifierError::NumberOutOfRange);
            };

            match tail {
                [b'v', ver @ ..] => Ok((nz_number, Some(tri!(version(ver))))),
                [] => Ok((nz_number, None)),
                [b'0'..=b'9'] => Err(IdentifierError::NumberOutOfRange),
                _ => Err(IdentifierError::InvalidVersion),
            }
        }
        _ => Err(IdentifierError::InvalidNumber),
    }
}

pub const fn number_and_version_len_5(
    number_and_version: &[u8],
) -> Result<(NonZero<u32>, Option<NonZero<u8>>), IdentifierError> {
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
                return Err(IdentifierError::NumberOutOfRange);
            };

            match tail {
                [b'v', ver @ ..] => Ok((nz_number, Some(tri!(version(ver))))),
                [] => Ok((nz_number, None)),
                [b'0'..=b'9'] => Err(IdentifierError::NumberOutOfRange),
                _ => Err(IdentifierError::InvalidVersion),
            }
        }
        _ => Err(IdentifierError::InvalidNumber),
    }
}

/// Parse a new-style date block, checking length and checking for validity of dates.
///
/// Returns `(a, b)`, where the year is `a + 1991` and `b` lands in the range `[1..=12]`, indicating the month.
pub const fn date_new(date: [u8; 4]) -> Result<(u8, u8), IdentifierError> {
    match date {
        [b1 @ b'0'..=b'9', b2 @ b'0'..=b'9', b3, b4] => {
            let y1 = b1 - b'0';
            let y2 = b2 - b'0';

            // convert bytes to values and check ranges
            let m1 = b3.overflowing_sub(b'0').0;
            let m2 = b4.overflowing_sub(b'0').0;

            // month is invalid format
            if !(m1 == 0 && (1 <= m2 && m2 <= 9) || m1 == 1 && m1 <= 2) {
                return Err(IdentifierError::InvalidDate);
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
        _ => Err(IdentifierError::InvalidDate),
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DateNumber {
    pub years_since_epoch: u8,
    pub month: u8,
    pub number: NonZero<u16>,
    pub version: Option<NonZero<u8>>,
}

pub const fn date_number(datestamp: &[u8]) -> Result<DateNumber, IdentifierError> {
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
        _ => Err(IdentifierError::InvalidDate),
    }
}

/// Parse an old-style date block.
pub const fn date_old(date: [u8; 4]) -> Result<(u8, u8), IdentifierError> {
    match date {
        [b1 @ b'0'..=b'9', b2 @ b'0'..=b'9', b3, b4] => {
            // convert bytes to values and check ranges
            let y1 = b1 - b'0';
            let y2 = b2 - b'0';

            let m1 = b3.overflowing_sub(b'0').0;
            let m2 = b4.overflowing_sub(b'0').0;

            // month is invalid format
            if !(m1 == 0 && (1 <= m2 && m2 <= 9) || m1 == 1 && m2 <= 2) {
                return Err(IdentifierError::InvalidDate);
            }

            // earliest date is August 1991 and latest is March 2007
            if !(y1 == 9 && (1 <= y2 && y2 <= 9) || y1 == 0 && y2 <= 7)
                || (y1 == 9 && y2 == 1 && m2 <= 7)
                || (y1 == 0 && y2 == 7 && m2 >= 4)
            {
                return Err(IdentifierError::DateOutOfRange);
            }

            // compute distance from 1991
            let years_since_epoch = if y1 == 9 { y2 - 1 } else { y2 + 9 };

            let month = 10 * m1 + m2;

            // convert to u16
            Ok((years_since_epoch, month))
        }
        _ => Err(IdentifierError::InvalidDate),
    }
}

const fn version(version: &[u8]) -> Result<NonZero<u8>, IdentifierError> {
    match version {
        [d @ b'1'..=b'9'] => Ok(unsafe { NonZero::new_unchecked(d.saturating_sub(b'0')) }),
        [d1 @ b'1'..=b'9', d2 @ b'0'..=b'9'] => Ok(unsafe {
            NonZero::new_unchecked(10 * d1.saturating_sub(b'0') + d2.saturating_sub(b'0'))
        }),
        [d1 @ b'1'..=b'9', d2 @ b'0'..=b'9', d3 @ b'0'..=b'9'] => {
            let overflow: u16 = 100 * (d1.saturating_sub(b'0') as u16)
                + 10 * (d2.saturating_sub(b'0') as u16)
                + (d3.saturating_sub(b'0') as u16);
            if overflow > 255 {
                Err(IdentifierError::InvalidVersion)
            } else {
                // SAFETY: overflow is non-zero since d1 is non-zero, so if it fits into the u8, it
                // is still non-zero
                unsafe { Ok(NonZero::new_unchecked(overflow as u8)) }
            }
        }
        _ => Err(IdentifierError::InvalidVersion),
    }
}
