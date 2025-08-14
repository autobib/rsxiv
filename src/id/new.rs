use std::{fmt, num::NonZero, str::FromStr};

use super::{
    Identifier, IdentifierError,
    parse::{self, tri},
};

/// A validated new-style arxiv identifier.
///
/// See the [module-level documentation](crate::id) for more context on this identifier.
///
/// Construct an identifier by parsing from an identifier string. To access the fields,
/// use the [`Identifier`] implementation.
/// ```
/// use std::num::NonZero;
/// use rsxiv::id::{Identifier, NewId};
///
/// // new-style identifier after 2014, with 5-digit number and version
/// let new_id = NewId::parse("1903.00015v2").unwrap();
///
/// assert_eq!(new_id.year(), 2019);
/// assert_eq!(new_id.month(), 3);
/// assert_eq!(new_id.number().get(), 15);
/// assert_eq!(new_id.version().unwrap().get(), 2);
/// ```
/// The identifier need not correspond to an actual record.
/// ```
/// # use rsxiv::id::NewId;
/// assert!(NewId::parse("0901.9999").is_ok());
/// ```
/// Construct an identifier from the raw parts.
/// ```
/// # use rsxiv::id::{Identifier, NewId};
/// use std::num::NonZero;
///
/// let new_id = NewId::new(2015, 12, NonZero::new(152).unwrap(), NonZero::new(5)).unwrap();
///
/// assert_eq!(new_id.year(), 2015);
/// assert_eq!(new_id.month(), 12);
/// assert_eq!(new_id.number(), NonZero::new(152).unwrap());
/// assert_eq!(new_id.version(), Some(NonZero::new(5).unwrap()));
/// ```
/// Errors from parsing or during construction of identifiers are reported using the [`IdentifierError`].
/// ```
/// # use std::str::FromStr;
/// # use std::num::NonZero;
/// # use rsxiv::id::NewId;
/// use rsxiv::id::IdentifierError;
///
/// // new identifiers have dates after 2007/04
/// assert_eq!(NewId::new(2001, 01, NonZero::new(1).unwrap(), None), Err(IdentifierError::DateOutOfRange));
///
/// // identifiers for years before 2014 only have 4 digits
/// assert_eq!(NewId::from_str("0902.12345"), Err(IdentifierError::NumberOutOfRange));
/// ```
///
/// [arxiv]: https://info.arxiv.org/help/arxiv_identifier.html
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct NewId {
    years_since_epoch: u8, // this is the number of years after the earliest possible year, i.e. 1991
    month: u8,
    number: NonZero<u32>,
    version: Option<NonZero<u8>>,
}

impl NewId {
    /// Return if the new identifier is 'short-style': that is, the number is at most `9999` and is
    /// padded to 4 digits. This is the case for new-style identifiers from 2014 or earlier.
    #[must_use]
    pub const fn is_short(&self) -> bool {
        // 23 = 2014 - 1991; see https://info.arxiv.org/help/arxiv_identifier.html
        self.years_since_epoch <= 23
    }

    /// Construct a new-style identifier directly from the constitutent parts. The parts are
    /// validated according to the rules for new-style identifiers.
    pub const fn new(
        year: u16,
        month: u8,
        number: NonZero<u32>,
        version: Option<NonZero<u8>>,
    ) -> Result<Self, IdentifierError> {
        if !(2007 <= year && year <= 2107)
            || (month == 0 || month > 12)
            || (year == 2007 && month < 4)
            || (year == 2107 && month >= 4)
        {
            return Err(IdentifierError::DateOutOfRange);
        }

        let threshold = if year <= 2014 { 10_000 } else { 100_000 };
        if number.get() >= threshold {
            return Err(IdentifierError::NumberOutOfRange);
        }

        Ok(Self {
            years_since_epoch: (year - 1991) as u8,
            month,
            number,
            version,
        })
    }

    pub const fn parse(id: &str) -> Result<Self, IdentifierError> {
        Self::parse_bytes(id.as_bytes())
    }

    pub const fn parse_bytes(id: &[u8]) -> Result<Self, IdentifierError> {
        match id {
            [y1, y2, m1, m2, b'.', tail @ ..] => {
                let date = [*y1, *y2, *m1, *m2];
                let number: &[u8] = tail;
                let (years_since_epoch, month) = tri!(parse::date_new(date));
                let (number, version) = if years_since_epoch <= 23 {
                    // 23 <=> 2014
                    tri!(parse::number_and_version_len_4(number))
                } else {
                    tri!(parse::number_and_version_len_5(number))
                };
                Ok(NewId {
                    years_since_epoch,
                    month,
                    number,
                    version,
                })
            }
            _ => Err(IdentifierError::IncorrectSeparator),
        }
    }
}

impl Identifier for NewId {
    /// A new-style identifier does not contain an archive.
    type Archive = ();

    #[inline]
    fn archive(&self) -> Self::Archive {}

    /// Return the year corresponding to the identifier. Guaranteed to land in the range
    /// `2007..=2106`.
    #[inline]
    fn year(&self) -> u16 {
        1991 + u16::from(self.years_since_epoch)
    }

    /// Return the month corresponding to the identifer. Guaranteed to land in the range
    /// `1..=12`.
    #[inline]
    fn month(&self) -> u8 {
        self.month
    }

    /// Return the number of the identifier. Guaranteed to land in the range `1..=99999`, and
    /// moreover land in the range `1..=9999` if `self.year() <= 2014`.
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

impl FromStr for NewId {
    type Err = IdentifierError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl fmt::Display for NewId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02}{:02}.",
            self.years_since_epoch.wrapping_add(91).rem_euclid(100),
            self.month,
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
