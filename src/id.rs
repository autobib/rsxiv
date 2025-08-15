//! # Typed representation of arXiv identifiers
//!
//! This module implements a typed representation of [arXiv identifiers][arxivid]; that is, the
//! (alpha)numerical string such as `1501.00001`, `0706.0001`, or `math/0309136`.
//!
//! The primary entrypoint in this module is the [`ArticleId`] type, which represents a validated article identifier.
//!
//! This module *only validates the format*: an identifier may or may not correspond to an actual
//! record in the arXiv database. A convenience [`is_valid`] function can be used to check if a
//! given string corresponds to a valid arXiv identifier.
//!
//! ## Using the [`ArticleId`]
//! ### Parsing and displaying
//! An [`ArticleId`] can be obtained from a raw string using the [`ArticleId::parse`] method. The
//! string can be obtained again using its [`Display`] implementation. The [`ArticleId::parse`] method is
//! equivalent to the [`FromStr`] implementation, with the added feature that [`ArticleId::parse`] is a
//! `const fn`.
//!
//! For example,
//! ```
//! use rsxiv::id::ArticleId;
//!
//! let id_str = "math/0309136v2";
//! let id = ArticleId::parse(id_str).unwrap();
//! assert_eq!(id_str, id.to_string());
//! ```
//!
//! ### Accessing fields.
//! A variety of fields can be accessed using [`ArticleId`] methods.
//! ```
//! use rsxiv::id::{Archive, ArticleId};
//!
//! let id = ArticleId::parse("hep-th/0309013v1").unwrap();
//! assert_eq!(id.year(), 2003);
//! assert_eq!(id.month(), 9);
//! assert_eq!(id.number().get(), 13);
//! assert_eq!(id.version().unwrap().get(), 1);
//! assert_eq!(id.archive(), Some(Archive::HepTh));
//!
//! // a new-style identifier does not contain an archive
//! let id = ArticleId::parse("1204.0012").unwrap();
//! assert!(id.archive().is_none());
//! ```
//!
//! ### No subject class
//! The subject class in old-style identifiers is not stored. ArXiv does not check
//! validity of the subject class in their API, and the [official recommendation][arxivscheme] is to drop the subject class
//! from old-style identifiers when present.
//! ```
//! # use rsxiv::id::ArticleId;
//!
//! // the identifier is automatically trimmed and the subject class is dropped
//! let id = ArticleId::parse("math.PR/0002012").unwrap();
//! assert_eq!(id.to_string(), "math/0002012");
//!
//! // the subject class need not be valid as long as it is in the format `.[A-Z][A-Z]`:
//! assert_eq!(ArticleId::parse("math.ZZ/0002012"), Ok(id));
//! ```
//!
//! ### Ordering
//! The [`ArticleId`]s implement [`Ord`] and are sorted according in order of the following
//! parameters:
//! 1. Year
//! 2. Month
//! 3. Archive (if old-style ID)
//! 4. Number
//! 5. Version (no version, followed by `v1`, `v2`, etc.)
//!
//! This is different than the lexicographic order of the identifier (as a string), which sorts by Archive first (if present) before the other parameters and
//! only takes into account the last two digits of the year.
//! ```
//! use rsxiv::id::ArticleId;
//! // sorts by year before archive
//! assert!(
//!     ArticleId::parse("hep-th/0502001").unwrap() <= ArticleId::parse("astro-ph/0703999").unwrap()
//! );
//!
//! // a new-style identifier date before 07/04 corresponds to a date in 2100:
//! // `0903...` is 2009/03
//! // `0407...` is 2104/07
//! assert!(
//!     ArticleId::parse("0903.0001").unwrap() <= ArticleId::parse("0407.00001").unwrap()
//! );
//! ```
//!
//! ### Maximum version
//! In principle, the version could be any positive integer. In practice, the version is required
//! to fit in a `u16`; that is, it can be at most `65535`. Since an arXiv version can only be
//! incremented at most once per day, this gives about 179 years worth of version labels. Currently
//! (August 15, 2025), the largest valid version of any article on arXiv is `0901.2093v152`.
//!
//! ### (De)serialization
//! Serialization and deserialization can be done with the [`ArticleId::deserialize`] and [`ArticleId::serialize`] methods.
//! ```
//! use rsxiv::id::ArticleId;
//!
//! let id = ArticleId::parse("0903.0001").unwrap();
//! let n = 1297881117612900352;
//!
//! assert_eq!(id.serialize(), n);
//! assert_eq!(ArticleId::deserialize(n), Some(id));
//! ```
//! Internally, an [`ArticleId`] is actually a `u64`, so serialization is free and deserialization
//! amounts to verifying that the `u64` corresponds to an actual identifier.
//!
//! The deserialization format is guaranteed to remain unchanged for major versions of this crate. See the [in-memory representation](#in-memory-representation) section for
//! more detail.
//!
//! ## Detailed format description
//! This is a reproduction of the [arXiv identifier documentation][arxivid], and gives a complete
//! description of the parsing formt used in this module.
//!
//! - [Old-style](#old-style-august-1991-to-march-2007)
//! - [New-style, short](#new-style-short-april-2007-to-december-2014)
//! - [New-style, long](#new-style-long-january-2015-to-march-2107)
//!
//! ### Old-style (August 1991 to March 2007)
//! These are identifiers of the form `archive/YYMMNNNvV`, where:
//!
//! 1. `archive` is valid archive as enumerated in the [`Archive`] enum, in the format returned by
//!    [`Archive::to_id`]
//! 2. `YY` is the last two digits of the year.
//! 3. `MM` is the month, in the range `1..=12`.
//! 4. `NNN` is a 3-digit number in the range `001..=999`, zero-padded to length 3.
//! 5. The year must lie in the range `1991..=2007`.
//! 6. The version is optional, and if present is of the form `vV` where `V` is an unpadded
//!    integer in the range `1..=u16::MAX`.
//! 7. If `year == 1991`, then `month >= 8`.
//! 8. If `year == 2007`, then `month <= 3`.
//!
//! ### New-style, short (April 2007 to December 2014)
//! These are identifiers of the form `YYMM.NNNNvV`, where:
//!
//! 1. `YY` is the last two digits of the year.
//! 2. `MM` is the month, in the range `1..=12`.
//! 4. `NNNN` is a 4-digit number in the range `0001..=9999`, zero-padded to length 4.
//! 6. The version is optional, and if present is of the form `vV` where `V` is an unpadded
//!    integer in the range `1..=u16::MAX`.
//! 7. The year must lie in the range `2007..=2014`.
//! 8. If `year == 2007`, then `month >= 4`.
//!
//! ### New-style, long (January 2015 to March 2107)
//! These are identifiers of the form `YYMM.NNNNNvV`, where:
//!
//! 1. `YY` is the last two digits of the year.
//! 2. `MM` is the month, in the range `1..=12`.
//! 4. `NNNNN` is a 5-digit number in the range `00001..=99999`, zero-padded to length 5.
//! 6. The version is optional, and if present is of the form `vV` where `V` is an unpadded
//!    integer in the range `1..=u16::MAX`.
//! 7. The year must lie in the range `2014..=2107`.
//! 8. If `year == 2107`, then `month <= 3`.
//!
//!
//! ## In-memory representation
//! Internally, an [`ArticleId`] is just a [`u64`]. The big-endian memory layout is as follows:
//! ```txt
//! years_since_epoch(u8) month(u8) archive(u8) number(u24) version(u16)
//! ```
//! The various parameters are defined as follows:
//!
//! - `years_since_epoch`: the value is the number of years since the arXiv epoch (`1991`, which is the constant [`ARXIV_EPOCH`]). For example,
//!   `2` is equivalent to `1993`.
//! - `month`: the month in the range `1..=12` starting with `Jan = 1`, etc.
//! - `archive`: the `#[repr(u8)]` value of [`Archive`], with the special value `0` used
//!   to indicate that the archive is not present (as is the case for new-style identifiers).
//! - `number`: the article number, which fits in the range since `2^24 - 1 = 16_777_215` gives
//!   sufficient space to store up to 7 digits.
//! - `version`: the version, as a `u16`, with the value `0` indicating that the version is not
//!   present.
//!
//! In particular, the ordering and equality checks for an [`ArticleId`] are equivalent to the ordering and
//! equality checks of the underlying `u64`.
//!
//! ### Unused bits
//! There are 14 bits which are always set to `0`. Note that the implementation *assumes* for
//! correctness that these bits are set to `0`, and therefore cannot be used to pack additional
//! information.
//!
//! - `years_since_epoch`: 1 highest bit (max value is `116`)
//! - `month`: 4 highest bits (max value is `12`)
//! - `archive`: 2 highest bits (max value is `34`)
//! - `number`: 7 highest bits (max value is `99999`)
//!
//! [arxivid]: https://info.arxiv.org/help/arxiv_identifier.html
//! [arxivscheme]: https://info.arxiv.org/help/arxiv_identifier_for_services.html
use std::{fmt::Display, mem::transmute, num::NonZero, str::FromStr};

mod archive;
mod parse;
#[cfg(test)]
mod tests;

use self::parse::tri;
pub use archive::Archive;

/// The [identifier style](crate::id#detailed-format-description).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    /// Old-style
    Old,
    /// New-style, short
    NewShort,
    /// New-style, long
    NewLong,
}

/// Returns if the given string corresponds to a valid arXiv identifier.
///
/// # Example
/// Check if an identifier is valid.
/// ```
/// use rsxiv::id::is_valid;
///
/// assert!(is_valid("math/0309136v2"));
/// assert!(!is_valid("bad-archive/0309136v2"));
/// ```
pub const fn is_valid(s: &str) -> bool {
    ArticleId::parse(s).is_ok()
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum IdError {
    DateOutOfRange,
    NumberOutOfRange,
    InvalidDate,
    InvalidNumber,
    InvalidVersion,
    InvalidArchive,
    IncorrectSeparator,
}

/// A validated arXiv identifier.
///
/// This is a compact and performant representation of an [arXiv identifier][arxivid]. For more
/// details on the arXiv identifier format and other details, see the [module-level docs](crate::id).
///
/// To construct a new identifier, use:
///
/// - [`ArticleId::parse`] to read from an identifier string, or
/// - [`ArticleId::new`] to construct directly from parameters.
///
///
/// [arxivid]: https://info.arxiv.org/help/arxiv_identifier.html
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArticleId {
    // Layout safety:
    // - The number(u24) bytes *must* be non-zero.
    // - The archive(u8) bytes *must* land in the range `0..=34`
    // If these rules are not upheld, the calls to `archive` and `number` will result in undefined
    // behaviour.
    raw: u64,
}

/// The earliest possible year of an arXiv identifier.
pub const ARXIV_EPOCH: u16 = 1991;

/// The maximum possible formatted length of an arXiv identifier.
///
/// This length is attained, for example, by `acc-phys/0001001v10000`.
/// ```
/// use rsxiv::id::{ArticleId, MAX_ID_FORMATTED_LEN};
/// assert_eq!(
///     ArticleId::parse("acc-phys/0001001v10000").unwrap().to_string().len(),
///     MAX_ID_FORMATTED_LEN
/// );
/// ```
pub const MAX_ID_FORMATTED_LEN: usize = 22;

/// A bitmask indicating which bits are currently used in the [binary
/// format](crate::id#in-memory-representation).
///
/// The bitmask is set to `1` if the bit is used, and `0` if the bit is always 0.
///
/// ### Examples
/// Masking with the bitmask never changes the serialized value.
/// ```
/// use rsxiv::id::{ArticleId, SERIALIZED_BITMASK};
/// let id = ArticleId::parse("math/0309136v2").unwrap();
/// let serialized = id.serialize();
/// assert_eq!(serialized, serialized & SERIALIZED_BITMASK);
/// ```
/// Store extra data inside the unused bits.
/// ```
/// # use rsxiv::id::{ArticleId, SERIALIZED_BITMASK};
/// let extra_data = 1039382085632;
///
/// // confirm that our data is stored inside the unused bits
/// assert_eq!(extra_data & !SERIALIZED_BITMASK, extra_data);
///
/// // obtain the serialized representation
/// let id = ArticleId::parse("math/0309136v2").unwrap();
/// let serialized = id.serialize();
///
/// // store our extra data inside the union
/// let union = serialized | extra_data;
///
/// // .. do some work, or send the data somewhere
///
/// // recover the original data
/// assert_eq!(ArticleId::deserialize(union & SERIALIZED_BITMASK), Some(id));
/// assert_eq!(union & !SERIALIZED_BITMASK, extra_data);
///
/// // failing to reset the bits will result in failed deserialization
/// assert!(ArticleId::deserialize(union).is_none());
/// ```
pub const SERIALIZED_BITMASK: u64 =
    0b01111111_00001111_00111111_00000001_11111111_11111111_11111111_11111111;

impl ArticleId {
    /// Obtain a new [`ArticleId`] by reading from its string representation.
    ///
    /// # Examples
    /// ```
    /// use rsxiv::id::ArticleId;
    ///
    /// let id_str = "math/0309136v2";
    /// let id = ArticleId::parse(id_str).unwrap();
    /// assert_eq!(id_str, id.to_string());
    /// ```
    #[inline]
    pub const fn parse(id: &str) -> Result<Self, IdError> {
        Self::parse_bytes(id.as_bytes())
    }

    /// Obtain a new [`ArticleId`] by reading from its representation in raw bytes.
    ///
    /// # Examples
    /// ```
    /// use rsxiv::id::ArticleId;
    ///
    /// let id_bytes = b"hep-th/0102001";
    /// let id = ArticleId::parse_bytes(id_bytes).unwrap();
    /// assert_eq!(id_bytes, id.to_string().as_bytes());
    /// ```
    pub const fn parse_bytes(id: &[u8]) -> Result<Self, IdError> {
        // it is not sufficient to check if the 5th byte is a `.`, since this will result in a
        // false-positive match on identifiers like `math.CA/`
        match id {
            [y1 @ b'0'..=b'9', y2, m1, m2, b'.', tail @ ..] => {
                let date = [*y1, *y2, *m1, *m2];
                let number: &[u8] = tail;
                let (years_since_epoch, month) = tri!(parse::date_new(date));
                let (number, version) = if years_since_epoch <= 23 {
                    // 23 <=> 2014
                    tri!(parse::number_and_version_len_4(number))
                } else {
                    tri!(parse::number_and_version_len_5(number))
                };
                Ok(Self::from_raw(
                    years_since_epoch,
                    month,
                    None,
                    number,
                    version,
                ))
            }
            _ => match archive::strip_prefix(id) {
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
                    Ok(Self::from_raw(
                        years_since_epoch,
                        month,
                        Some(archive),
                        number,
                        version,
                    ))
                }
                None => Err(IdError::InvalidArchive),
            },
        }
    }

    /// Construct a new identifier from components.
    ///
    /// This constructs an new-style identifier if `archive` is `None`, and otherwise constructs an
    /// old-style identifier with the given archive.
    ///
    /// # Examples
    /// ```
    /// use rsxiv::id::{Archive, ArticleId, IdError};
    /// use std::num::NonZero;
    ///
    /// let id = ArticleId::new(
    ///     2001,
    ///     12,
    ///     Some(Archive::Nlin),
    ///     NonZero::new(621).unwrap(),
    ///     NonZero::new(32)
    /// ).unwrap();
    ///
    /// assert_eq!(id.year(), 2001);
    /// assert_eq!(id.month(), 12);
    /// assert_eq!(id.archive(), Some(Archive::Nlin));
    /// assert_eq!(id.number().get(), 621);
    /// assert_eq!(id.version().unwrap().get(), 32);
    ///
    /// // an old-style `math/` identifier but with an invalid year
    /// let id_err = ArticleId::new(
    ///     2021,
    ///     03,
    ///     Some(Archive::Math),
    ///     NonZero::new(621).unwrap(),
    ///     None,
    /// );
    ///
    /// assert_eq!(id_err, Err(IdError::DateOutOfRange));
    /// ```
    pub const fn new(
        year: u16,
        month: u8,
        archive: Option<Archive>,
        number: NonZero<u32>,
        version: Option<NonZero<u16>>,
    ) -> Result<Self, IdError> {
        if month == 0 || month > 12 {
            return Err(IdError::DateOutOfRange);
        }

        if archive.is_some() {
            if !(1991 <= year && year <= 2007)
                || (year == 1991 && month <= 7)
                || (year == 2007 && month >= 4)
            {
                return Err(IdError::DateOutOfRange);
            }

            if number.get() >= 1000 {
                return Err(IdError::NumberOutOfRange);
            }
        } else {
            if !(2007 <= year && year <= 2107)
                || (year == 2007 && month < 4)
                || (year == 2107 && month >= 4)
            {
                return Err(IdError::DateOutOfRange);
            }

            let threshold = if year <= 2014 { 10_000 } else { 100_000 };
            if number.get() >= threshold {
                return Err(IdError::NumberOutOfRange);
            }
        }

        Ok(Self::from_raw(
            (year - ARXIV_EPOCH) as u8,
            month,
            archive,
            number,
            // SAFETY: Option<NonZero<u16>> has the same layout as u16
            unsafe { transmute::<Option<NonZero<u16>>, u16>(version) },
        ))
    }

    /// Construct the identifier from raw parts.
    const fn from_raw(
        years_since_epoch: u8,
        month: u8,
        archive: Option<Archive>,
        number: NonZero<u32>,
        version: u16,
    ) -> Self {
        let archive = match archive {
            Some(archive) => archive as u8,
            None => 0,
        };

        let number = number.get();

        // Optimized version equivalent to:
        // let [_, n1, n2, n3] = number.get().to_be_bytes();
        // let [v1, v2] = version.to_be_bytes();
        // let raw = u64::from_be_bytes([years_since_epoch, month, a, n1, n2, n3, v1, v2]);
        let raw = ((years_since_epoch as u64) << 56)
            | ((month as u64) << 48)
            | ((archive as u64) << 40)
            | ((number as u64) << 16)
            | (version as u64);
        Self { raw }
    }

    /// The identifier year, minus [`ARXIV_EPOCH`].
    #[inline]
    pub const fn years_since_epoch(&self) -> u8 {
        raw::years_since_epoch(self.raw)
    }

    /// The identifier year.
    #[inline]
    pub const fn year(&self) -> u16 {
        ARXIV_EPOCH + (self.years_since_epoch() as u16)
    }

    /// The identifier month, in the range `1..=12`.
    #[inline]
    pub const fn month(&self) -> u8 {
        raw::month(self.raw)
    }

    /// Returns the archive if this is an old-style identifier, and otherwise `None`.
    #[inline]
    pub const fn archive(&self) -> Option<Archive> {
        let a = raw::archive(self.raw);

        // This implementation should generate assembly equivalent to `transmute(a) but without being
        // dependent on the layout of `Option<Archive>`.
        if a == 0 {
            None
        } else {
            // SAFETY: the `archive` bytes, if non-zero, correspond to a `u8` variant in the
            // Archive enum
            unsafe { Some(transmute::<u8, Archive>(a)) }
        }
    }

    /// Returns the identifier style.
    ///
    /// # Example
    /// ```
    /// use rsxiv::id::{ArticleId, Style};
    ///
    /// assert_eq!(ArticleId::parse("math/0309136v2").unwrap().style(), Style::Old);
    /// assert_eq!(ArticleId::parse("0903.1252").unwrap().style(), Style::NewShort);
    /// assert_eq!(ArticleId::parse("1912.00002").unwrap().style(), Style::NewLong);
    /// ```
    #[inline]
    pub const fn style(&self) -> Style {
        if raw::is_new_style(self.raw) {
            if self.years_since_epoch() <= 23 {
                Style::NewShort
            } else {
                Style::NewLong
            }
        } else {
            Style::Old
        }
    }

    /// The article number.
    #[inline]
    pub const fn number(&self) -> NonZero<u32> {
        let n = raw::number(self.raw);

        // SAFETY: the number is guaranteed to be non-zero
        unsafe { NonZero::new_unchecked(n) }
    }

    /// Returns the version, if present.
    #[inline]
    pub const fn version(&self) -> Option<NonZero<u16>> {
        NonZero::new(raw::version(self.raw))
    }

    /// Serialize this value to a `u64`.
    ///
    /// # Examples
    /// ```
    /// use rsxiv::id::ArticleId;
    ///
    /// let id = ArticleId::parse("hep-th/0101001").unwrap();
    /// let n = 720879405588611072;
    ///
    /// assert_eq!(id.serialize(), n);
    /// ```
    pub const fn serialize(&self) -> u64 {
        self.raw
    }

    /// Deserialize the value from a `u64` previously constructed by the [`ArticleId::serialize`]
    /// method.
    ///
    /// # Examples
    /// ```
    /// use rsxiv::id::ArticleId;
    ///
    /// let id = ArticleId::parse("hep-th/0101001").unwrap();
    /// let n = 720879405588611072;
    ///
    /// assert_eq!(Some(id), ArticleId::deserialize(n));
    /// ```
    /// Returns `None` if the `u64` is invalid.
    /// ```
    /// # use rsxiv::id::ArticleId;
    /// assert!(ArticleId::deserialize(12345).is_none());
    /// ```
    pub const fn deserialize(raw: u64) -> Option<Self> {
        // we need to check that the raw format is valid; mainly the `number` and `archive` fields
        // (since these are required to uphold safety guarantees) and then the date, depending on
        // the presence of the archive. The version is always valid.
        let years_since_epoch = raw::years_since_epoch(raw);
        let month = raw::month(raw);
        let archive = raw::archive(raw);
        let number = raw::number(raw);

        // validate month
        if month == 0 || month > 12 {
            return None;
        }

        if number == 0 {
            return None;
        };

        // invalid archive number
        if archive > 34 {
            return None;
        }

        if archive == 0 {
            // old style
            if !(16 <= years_since_epoch && years_since_epoch <= 116)
                || (years_since_epoch == 16 && month < 4)
                || (years_since_epoch == 116 && month >= 4)
            {
                return None;
            }

            let threshold = if years_since_epoch <= 23 {
                10_000
            } else {
                100_000
            };

            if number >= threshold {
                return None;
            }
        } else if archive <= 34 {
            if (years_since_epoch > 16)
                || (years_since_epoch == 0 && month <= 7)
                || (years_since_epoch == 16 && month >= 4)
            {
                return None;
            }

            if number >= 1000 {
                return None;
            }
        } else {
            return None;
        }

        Some(Self { raw })
    }
}

impl FromStr for ArticleId {
    type Err = IdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl Display for ArticleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.archive() {
            Some(archive) => {
                // old-style
                f.write_str(archive.to_id())?;
                f.write_str("/")?;
                write!(
                    f,
                    "{:02}{:02}{:03}",
                    self.years_since_epoch().wrapping_add(91).rem_euclid(100),
                    self.month(),
                    self.number()
                )?;
            }
            None => {
                // new-style
                write!(
                    f,
                    "{:02}{:02}.",
                    self.years_since_epoch().wrapping_add(91).rem_euclid(100),
                    self.month(),
                )?;

                if self.years_since_epoch() <= 23 {
                    write!(f, "{:04}", self.number())?;
                } else {
                    write!(f, "{:05}", self.number())?;
                }
            }
        }

        if let Some(version) = self.version() {
            write!(f, "v{version}")?;
        }

        Ok(())
    }
}

mod raw {
    /// The years since `ARXIV_EPOCH`.
    #[inline]
    pub const fn years_since_epoch(raw: u64) -> u8 {
        // let [years_since_epoch, _, _, _, _, _, _, _] = val.to_be_bytes();
        // years_since_epoch
        (raw >> 56) as u8
    }

    /// The identifier month, in the range `1..=12`.
    #[inline]
    pub const fn month(raw: u64) -> u8 {
        // let [_, month, _, _, _, _, _, _] = self.raw.to_be_bytes();
        // month
        (raw >> 48) as u8
    }

    /// Returns the archive if this is an old-style identifier.
    #[inline]
    pub const fn archive(raw: u64) -> u8 {
        // let [_, _, a, _, _, _, _, _] = self.raw.to_be_bytes();
        (raw >> 40) as u8
    }

    /// The article number.
    #[inline]
    pub const fn number(raw: u64) -> u32 {
        // Optimized version equivalent to:
        // let [_, _, _, n1, n2, n3, _, _] = self.raw.to_be_bytes();
        // let n = u32::from_be_bytes([0, n1, n2, n3]);
        ((raw >> 16) as u32) & 0xFFFFFF
    }

    /// Returns the version, if present.
    #[inline]
    pub const fn version(raw: u64) -> u16 {
        // let [_, _, _, _, _, _, v1, v2] = self.raw.to_be_bytes();
        // let v = u16::from_be_bytes([v1, v2]);
        raw as u16
    }

    #[inline]
    pub const fn is_new_style(raw: u64) -> bool {
        // Just need to check if the archive is not 0
        const MASK: u64 = u64::from_be_bytes([0, 0, 0xFF, 0, 0, 0, 0, 0]);
        raw & MASK == 0
    }
}
