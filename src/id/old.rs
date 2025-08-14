mod archive;

use std::{fmt, num::NonZero, str::FromStr};

use super::{Identifier, IdentifierError, parse};
pub use archive::Archive;

/// A validated old-style arxiv identifier.
///
/// An identifier is the [preferred external identifier][preferred] corresponding to an [old-style identifiers][arxiv]; that is,
/// identifiers before March 31, 2007. Note that an identifier need not correspond to an
/// actual arXiv record.
///
/// The subject class information not stored within this identifier.
///
/// [arxiv]: https://info.arxiv.org/help/arxiv_identifier.html
/// [preferred]: https://info.arxiv.org/help/arxiv_identifier_for_services.html
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct OldID {
    archive: Archive,
    years_since_epoch: u8, // this is the number of years after the earliest possible year, i.e. 1991
    month: u8,
    number: NonZero<u16>,
    version: Option<NonZero<u8>>,
}

impl OldID {
    /// Construct an old-style identifier from its constitutent parts.
    ///
    /// See the [module-level documentation](crate::id) for syntax.
    pub const fn new(
        archive: Archive,
        year: u16,
        month: u8,
        number: NonZero<u16>,
        version: Option<NonZero<u8>>,
    ) -> Result<Self, IdentifierError> {
        if !(1991 <= year && year <= 2007)
            || (month == 0 || month > 12)
            || (year == 1991 && month <= 7)
            || (year == 2007 && month >= 4)
        {
            return Err(IdentifierError::DateOutOfRange);
        }

        if number.get() >= 1000 {
            return Err(IdentifierError::NumberOutOfRange);
        }

        Ok(Self {
            archive,
            years_since_epoch: (year - 1991) as u8,
            month,
            number,
            version,
        })
    }

    /// Parse an old-style identifier from raw bytes.
    ///
    /// See the [module-level documentation](crate::id) for syntax.
    pub const fn parse_bytes(id: &[u8]) -> Result<Self, IdentifierError> {
        match archive::strip_prefix(id) {
            Some((archive, tail)) => {
                let date_number = match tail {
                    [b'/', tail @ ..]
                    | [b'.', b'A'..=b'Z', b'A'..=b'Z', b'/', tail @ ..]
                    | tail => tail,
                };
                let parse::DateNumber {
                    years_since_epoch,
                    month,
                    number,
                    version,
                } = match parse::date_number(date_number) {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                };

                Ok(Self {
                    archive,
                    years_since_epoch,
                    month,
                    number,
                    version,
                })
            }
            None => Err(IdentifierError::InvalidArchive),
        }
    }

    /// Parse an old-style identifier from a string slice.
    ///
    /// This is identical to the [`FromStr`] implementation, but can be used in const contexts.
    ///
    /// See the [module-level documentation](crate::id) for syntax.
    pub const fn parse(id: &str) -> Result<Self, IdentifierError> {
        Self::parse_bytes(id.as_bytes())
    }
}

impl Identifier for OldID {
    type Archive = Archive;

    fn archive(&self) -> Self::Archive {
        self.archive
    }

    /// Return the year corresponding to the identifier. Guaranteed to land in the range
    /// `1991..=2007`.
    fn year(&self) -> u16 {
        1991 + u16::from(self.years_since_epoch)
    }

    /// Return the month corresponding to the identifer. Guaranteed to land in the range
    /// `1..=12`, and in the range `[8..=12]` if `self.year() == 1991` and in the range `1..=3` if
    /// `self.year() == 2007`.
    fn month(&self) -> u8 {
        self.month
    }

    /// Return the number of the identifier. Guaranteed to land in the range `1..=999`.
    fn number(&self) -> NonZero<u32> {
        // SAFETY: the number is initially non-zero
        unsafe { NonZero::new_unchecked(u32::from(self.number.get())) }
    }

    /// Return the version of the identifier. The version may not be present.
    fn version(&self) -> Option<NonZero<u8>> {
        self.version
    }
}

impl FromStr for OldID {
    type Err = IdentifierError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl fmt::Display for OldID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.archive.to_id())?;
        f.write_str("/")?;
        write!(
            f,
            "{:02}{:02}{:03}",
            self.years_since_epoch.wrapping_add(91).rem_euclid(100),
            self.month,
            self.number
        )?;

        if let Some(version) = self.version {
            write!(f, "v{version}")?;
        }

        Ok(())
    }
}
