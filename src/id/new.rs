use std::{fmt, num::NonZero, str::FromStr};

use super::{Identifier, IdentifierError, parse};

/// A validated new-style arxiv identifier.
///
/// An identifier is guaranteed to satisfy the rules for [new-style identifiers][arxiv]; that is,
/// identifiers since April 1, 2007. Note that an identifier need not correspond to an
/// actual arXiv record.
///
/// Construct an identifier by parsing from an identifier string.
/// ```
/// use std::num::NonZero;
/// use rsxiv::id::{Identifier, NewID};
///
/// // new-style identifier after 2014, with 5-digit number and version
/// let new_id: NewID = "1903.00015v2".parse().unwrap();
///
/// assert_eq!(new_id.year(), 2019);
/// assert_eq!(new_id.month(), 3);
/// assert_eq!(new_id.number(), NonZero::new(15).unwrap());
/// assert_eq!(new_id.version(), Some(NonZero::new(2).unwrap()));
/// ```
/// The identifier need not correspond to an actual record.
/// ```
/// # use rsxiv::id::NewID;
/// use std::str::FromStr;
///
/// assert!(NewID::from_str("0901.9999").is_ok());
/// ```
/// Construct an identifier from the raw parts.
/// ```
/// # use rsxiv::id::{Identifier, NewID};
/// use std::num::NonZero;
///
/// let new_id = NewID::new(2015, 12, 152, Some(5)).unwrap();
///
/// assert_eq!(new_id.year(), 2015);
/// assert_eq!(new_id.month(), 12);
/// assert_eq!(new_id.number(), NonZero::new(152).unwrap());
/// assert_eq!(new_id.version(), Some(NonZero::new(5).unwrap()));
/// ```
/// Errors from parsing or during construction of identifiers are reported using the [`IdentifierError`].
/// ```
/// # use std::str::FromStr;
/// # use rsxiv::id::NewID;
/// use rsxiv::id::IdentifierError;
///
/// assert_eq!(NewID::new(2015, 12, 152, Some(0)), Err(IdentifierError::InvalidVersion));
/// assert_eq!(NewID::new(2001, 01, 1, None), Err(IdentifierError::DateOutOfRange));
///
/// // identifiers for years before 2014 only have 4 digits
/// assert_eq!(NewID::from_str("0902.12345"), Err(IdentifierError::NumberOutOfRange));
/// ```
///
/// [arxiv]: https://info.arxiv.org/help/arxiv_identifier.html
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct NewID {
    years_since_epoch: u8, // this is the number of years after the earliest possible year, i.e. 1991
    month: u8,
    number: NonZero<u32>,
    version: Option<NonZero<u8>>,
}

impl NewID {
    /// Return if the new identifier is 'short-style': that is, the number is at most `9999` and is
    /// padded to 4 digits. This is the case for new-style identifiers from 2014 or earlier.
    pub fn is_short(&self) -> bool {
        // 23 = 2014 - 1991; see https://info.arxiv.org/help/arxiv_identifier.html
        self.years_since_epoch <= 23
    }

    /// Construct a new-style identifier directly from the constitutent parts. The parts are
    /// validated according to the rules for new-style identifiers.
    pub fn new(
        year: u16,
        month: u8,
        number: u32,
        version: Option<u8>,
    ) -> Result<Self, IdentifierError> {
        if !(2007..=2016).contains(&year)
            || (month == 0 || month > 12)
            || (year == 2007 && month < 4)
            || (year == 2016 && month >= 4)
        {
            return Err(IdentifierError::DateOutOfRange);
        }

        let threshold = if year <= 2014 { 10000 } else { 100000 };
        if number >= threshold {
            return Err(IdentifierError::NumberOutOfRange);
        }

        let number = if let Some(number) = NonZero::new(number) {
            number
        } else {
            return Err(IdentifierError::NumberOutOfRange);
        };

        let version = match version {
            Some(v) => match NonZero::new(v) {
                Some(nz) => Some(nz),
                None => return Err(IdentifierError::InvalidVersion),
            },
            None => None,
        };

        Ok(Self {
            years_since_epoch: (year - 1991) as u8,
            month,
            number,
            version,
        })
    }
}

impl Identifier for NewID {
    /// Return the year corresponding to the identifier. Guaranteed to land in the range
    /// `[2007..=2106]`.
    #[inline]
    fn year(&self) -> u16 {
        1991 + (self.years_since_epoch as u16)
    }

    /// Return the month corresponding to the identifer. Guaranteed to land in the range
    /// `[1..=12]`.
    #[inline]
    fn month(&self) -> u8 {
        self.month
    }

    /// Return the number of the identifier. Guaranteed to land in the range `[1..=9999]`, and
    /// moreover land in the range `[1..=9999]` if `self.year() <= 2014`.
    #[inline]
    fn number(&self) -> NonZero<u32> {
        self.number
    }

    /// Return the version of the identifier, if any.
    #[inline]
    fn version(&self) -> Option<NonZero<u8>> {
        self.version
    }
}

impl NewID {
    fn from_split(date: [u8; 4], number: &[u8]) -> Result<Self, IdentifierError> {
        let (years_since_epoch, month) = parse::date_new(date)?;
        let (number, version) = if years_since_epoch <= 23 {
            // 23 <=> 2014
            parse::number_and_version_len_4(number)?
        } else {
            parse::number_and_version_len_5(number)?
        };
        Ok(NewID {
            years_since_epoch,
            month,
            number,
            version,
        })
    }
}

impl FromStr for NewID {
    type Err = IdentifierError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.as_bytes() {
            [y1, y2, m1, m2, b'.', tail @ ..] => Self::from_split([*y1, *y2, *m1, *m2], tail),
            _ => Err(IdentifierError::InvalidDate),
        }
    }
}

impl fmt::Display for NewID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02}{:02}.",
            self.month,
            self.years_since_epoch.wrapping_add(91).rem_euclid(100)
        )?;

        if self.is_short() {
            write!(f, "{:04}", self.number)?;
        } else {
            write!(f, "{:05}", self.number)?;
        }

        if let Some(version) = self.version {
            write!(f, "v{version}")?;
        }

        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
// }
