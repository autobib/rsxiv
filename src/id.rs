//! # Typed representation of arXiv identifiers
//!
//! This module implements a typed representation of [arXiv identifiers][arxivid] such as `1501.00001`, `0706.0001`, or `math/0309136`.
//!
//! There are three primary entrypoints in this module.
//!
//! 1. [`ArticleId`]: A portable validated identifier format with efficient data access.
//!    Use this format if you want:
//!    - the data stored within the identifier.
//!    - a memory-efficient representation (fits inside a `u64`).
//!    - serialization and deserialization, or otherwise plan to store and load
//!      identifiers.
//!    - `const fn` methods.
//!    - identifier comparison with `PartialEq`, `Eq`, `Ord` implementations.
//! 2. [`Validated`]: A wrapper around an [`AsRef<str>`] type which has been validated by
//!    the identifier rules. Use this format if you:
//!    - only care that the identifier is valid but not about its contents.
//!    - mostly need to work with the string representation.
//! 3. [`validate`]: A function which checks if a given string satisfies the identifier rules.
//!
//! This module *only validates the format*: an identifier may or may not correspond to an actual
//! record in the arXiv database.
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
//! ## Maximum version
//! In principle, the version could be any positive integer. In practice, the version is required
//! to fit in a `u16`; that is, it can be at most `65535`. Since an arXiv version can only be
//! incremented at most once per day, this gives about 179 years worth of version labels. Currently
//! (August 15, 2025), the largest valid version of any article on arXiv is `0901.2093v152`.
//!
//! [arxivid]: https://info.arxiv.org/help/arxiv_identifier.html
//! [arxivscheme]: https://info.arxiv.org/help/arxiv_identifier_for_services.html
use std::{
    borrow::Cow,
    error::Error,
    fmt::{Debug, Display},
    mem::transmute,
    num::NonZero,
    str::FromStr,
};

mod archive;
mod parse;
#[cfg(test)]
mod tests;

use self::parse::tri;
pub use archive::{Archive, strip_archive_prefix};

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
/// use rsxiv::id::validate;
///
/// assert!(validate("math/0309136v2").is_ok());
/// assert!(validate("bad-archive/0309136v2").is_err());
/// ```
#[inline]
pub const fn validate(s: &str) -> Result<(), IdError> {
    match ArticleId::parse(s) {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}

/// An error which may result when parsing or validating an arXiv identifier.
///
/// # Examples
/// ```
/// use rsxiv::id::{Archive, ArticleId, IdError};
/// use std::num::NonZero;
///
/// // new-style identifiers before 2014 only have 4 digits
/// let id_err = ArticleId::new(
///     2009,
///     03,
///     None,
///     NonZero::new(12345).unwrap(),
///     None,
/// );
///
/// assert_eq!(id_err, Err(IdError::NumberOutOfRange));
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum IdError {
    /// The date is invalid for the given format.
    DateOutOfRange,
    /// The number is invalid for the given format.
    NumberOutOfRange,
    /// Failed to parse the date.
    InvalidDate,
    /// Failed to parse the number.
    InvalidNumber,
    /// Failed to parse the version.
    InvalidVersion,
    /// Failed to parse the archive.
    InvalidArchive,
}

impl Display for IdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            IdError::DateOutOfRange => "Date invalid for the given format",
            IdError::NumberOutOfRange => "Number invalid for the given format",
            IdError::InvalidDate => "Failed to parse the date",
            IdError::InvalidNumber => "Failed to parse the number",
            IdError::InvalidVersion => "Failed to parse the version",
            IdError::InvalidArchive => "Failed to parse the archive",
        };
        f.write_str(s)
    }
}

impl Error for IdError {}

/// A portable validated identifier format with efficient data access.
///
/// This is compact `u64` representation of an [arXiv identifier][arxivid]. For more
/// details on the arXiv identifier format and other details, see the [module-level docs](crate::id).
///
/// To construct a new identifier, use:
///
/// - [`ArticleId::parse`] to read from an identifier string, or
/// - [`ArticleId::new`] to construct directly from parameters.
///
/// ## Using the [`ArticleId`]
/// ### Parsing and displaying
/// An [`ArticleId`] can be obtained from a raw string using the [`ArticleId::parse`] method. The
/// string can be obtained again using its [`Display`] implementation, with the caveat that the [subject class will be removed](#no-subject-class). The [`ArticleId::parse`] method is
/// equivalent to the [`FromStr`] implementation, with the added feature that [`ArticleId::parse`] is a
/// `const fn`.
///
/// For example,
/// ```
/// use rsxiv::id::ArticleId;
///
/// let id_str = "math/0309136v2";
/// let id = ArticleId::parse(id_str).unwrap();
/// assert_eq!(id_str, id.to_string());
/// ```
///
/// ### Accessing fields
/// A variety of fields can be accessed using [`ArticleId`] methods.
/// ```
/// use rsxiv::id::{Archive, ArticleId};
///
/// let id = ArticleId::parse("hep-th/0309013v1").unwrap();
/// assert_eq!(id.year(), 2003);
/// assert_eq!(id.month(), 9);
/// assert_eq!(id.number().get(), 13);
/// assert_eq!(id.version().unwrap().get(), 1);
/// assert_eq!(id.archive(), Some(Archive::HepTh));
///
/// // a new-style identifier does not contain an archive
/// let id = ArticleId::parse("1204.0012").unwrap();
/// assert!(id.archive().is_none());
/// ```
///
/// ### Updating fields
/// Generally speaking, fields cannot be updated in-place since the new values may not be valid for
/// the given data. The suggested approach is to construct a new identifier using the fields of the
/// old identifier.
/// ```
/// use std::num::NonZero;
/// use rsxiv::id::{ArticleId, IdError};
///
/// /// Update the article number of an identifier.
/// fn update_number(id: ArticleId, new_number: NonZero<u32>) -> Result<ArticleId, IdError> {
///     ArticleId::new(
///         id.year(),
///         id.month(),
///         id.archive(),
///         new_number,
///         id.version(),
///     )
/// }
///
/// let id = ArticleId::parse("7209.01532v5").unwrap();
/// let new = update_number(id, NonZero::new(12).unwrap()).unwrap();
///
/// assert_eq!(
///     new.to_string(),
///     "7209.00012v5"
/// );
///
/// let id = ArticleId::parse("0801.0001").unwrap();
/// assert!(update_number(id, NonZero::new(12942).unwrap()).is_err());
/// ```
/// The exception is the article version, since the version is always valid if it is of the correct
/// type, and updating the version is a common operation for a given identifier.
/// ```
/// use std::num::NonZero;
/// use rsxiv::id::{ArticleId, IdError};
///
/// let id = ArticleId::parse("7209.01532v5").unwrap();
///
/// // setting the version always succeeds
/// assert_eq!(
///     id.set_version(NonZero::new(12)).to_string(),
///     "7209.01532v12"
/// );
///
/// // clear the version using `clear_version`
/// assert_eq!(
///     id.clear_version().to_string(),
///     "7209.01532"
/// );
///
/// // or equivalently by setting the version to `None`
/// assert_eq!(id.clear_version(), id.set_version(None));
/// ```
///
/// ### No subject class
/// The subject class in old-style identifiers is not stored. ArXiv does not check
/// validity of the subject class in their API, and the [official recommendation][arxivscheme] is to drop the subject class
/// from old-style identifiers when present.
/// ```
/// # use rsxiv::id::ArticleId;
///
/// // the subject class is dropped
/// let id = ArticleId::parse("math.PR/0002012").unwrap();
/// assert_eq!(id.to_string(), "math/0002012");
///
/// // the subject class need not be valid as long as it is in the format `.[A-Z][A-Z]`:
/// assert_eq!(ArticleId::parse("math.ZZ/0002012"), Ok(id));
/// ```
///
/// ### Ordering
/// [`ArticleId`] implements [`Ord`] and is sorted according in order of the following
/// parameters:
/// 1. Year
/// 2. Month
/// 3. Archive (if old-style ID)
/// 4. Number
/// 5. Version (no version, followed by `v1`, `v2`, etc.)
///
/// This is different than the lexicographic order of the identifier (as a string), which sorts by Archive first (if present) before the other parameters and
/// only takes into account the last two digits of the year.
/// ```
/// use rsxiv::id::ArticleId;
/// // sorts by year before archive
/// assert!(
///     ArticleId::parse("hep-th/0502001").unwrap() <= ArticleId::parse("astro-ph/0703999").unwrap()
/// );
///
/// // a new-style identifier date before 07/04 corresponds to a date in 2100:
/// // `0903...` is 2009/03
/// // `0407...` is 2104/07
/// assert!(
///     ArticleId::parse("0903.0001").unwrap() <= ArticleId::parse("0407.00001").unwrap()
/// );
/// ```
///
/// ### (De)serialization
/// Serialization and deserialization can be done with the [`ArticleId::deserialize`] and [`ArticleId::serialize`] methods.
/// ```
/// use rsxiv::id::ArticleId;
///
/// let id = ArticleId::parse("0903.0001").unwrap();
/// let n = 1297881117612900352;
///
/// assert_eq!(id.serialize(), n);
/// assert_eq!(ArticleId::deserialize(n), Some(id));
/// ```
/// Internally, an [`ArticleId`] is actually a `u64`, so serialization is free and deserialization
/// amounts to verifying that the `u64` corresponds to an actual identifier.
///
/// The deserialization format is guaranteed to remain unchanged for major versions of this crate. See the [in-memory representation](#in-memory-representation) section for
/// more detail.
///
/// ## In-memory representation
/// Internally, an [`ArticleId`] is just a [`u64`]. The big-endian memory layout is as follows:
/// ```txt
/// years_since_epoch(u8) month(u8) archive(u8) number(u24) version(u16)
/// ```
/// The various parameters are defined as follows:
///
/// - `years_since_epoch`: the value is the number of years since the arXiv epoch (`1991`, which is the constant [`ARXIV_EPOCH`]). For example,
///   `2` is equivalent to `1993`.
/// - `month`: the month in the range `1..=12` starting with `Jan = 1`, etc.
/// - `archive`: the `#[repr(u8)]` value of [`Archive`], with the special value `0` used
///   to indicate that the archive is not present (as is the case for new-style identifiers).
/// - `number`: the article number, which fits in the range since `2^24 - 1 = 16_777_215` gives
///   sufficient space to store up to 7 digits.
/// - `version`: the version, as a `u16`, with the value `0` indicating that the version is not
///   present.
///
/// In particular, the ordering and equality checks for an [`ArticleId`] are equivalent to the ordering and
/// equality checks of the underlying `u64`.
///
/// ### Unused bits
/// There are 14 bits which are always set to `0`. Note that the implementation *assumes* for
/// correctness that these bits are set to `0`, and therefore cannot be used to pack additional
/// information.
///
/// - `years_since_epoch`: 1 highest bit (max value is `116`)
/// - `month`: 4 highest bits (max value is `12`)
/// - `archive`: 2 highest bits (max value is `34`)
/// - `number`: 7 highest bits (max value is `99999`)
///
/// See [`ArticleId::SERIALIZED_BITMASK`] for a bitmask indicating precisely which bits are used in the
/// serialized format.
///
/// [arxivid]: https://info.arxiv.org/help/arxiv_identifier.html
/// [arxivscheme]: https://info.arxiv.org/help/arxiv_identifier_for_services.html
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
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
    #[inline]
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
            _ => match archive::strip_archive_prefix_bytes(id) {
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
    #[must_use]
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
    #[must_use]
    pub const fn years_since_epoch(self) -> u8 {
        raw::years_since_epoch(self.raw)
    }

    /// The identifier year.
    #[inline]
    #[must_use]
    pub const fn year(self) -> u16 {
        ARXIV_EPOCH + (self.years_since_epoch() as u16)
    }

    /// The identifier month, in the range `1..=12`.
    #[inline]
    #[must_use]
    pub const fn month(self) -> u8 {
        raw::month(self.raw)
    }

    /// Returns the archive if this is an old-style identifier, and otherwise `None`.
    #[inline]
    #[must_use]
    pub const fn archive(self) -> Option<Archive> {
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
    #[must_use]
    pub const fn style(self) -> Style {
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
    #[must_use]
    pub const fn number(self) -> NonZero<u32> {
        let n = raw::number(self.raw);

        // SAFETY: the number is guaranteed to be non-zero
        unsafe { NonZero::new_unchecked(n) }
    }

    /// Returns the version, if present.
    #[inline]
    #[must_use]
    pub const fn version(self) -> Option<NonZero<u16>> {
        NonZero::new(raw::version(self.raw))
    }

    /// Change the version to the specified value. Passing `None` clears the version identifier.
    pub const fn set_version(self, v: Option<NonZero<u16>>) -> Self {
        // SAFETY: Option<NonZero<u16>> has the same layout as u16
        let v = unsafe { transmute::<Option<NonZero<u16>>, u16>(v) };
        Self {
            raw: raw::set_version(self.raw, v),
        }
    }

    /// Clear the version, leaving the remaining fields unchanged.
    #[inline]
    #[must_use]
    pub const fn clear_version(self) -> Self {
        self.set_version(None)
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
    #[must_use]
    pub const fn serialize(self) -> u64 {
        self.raw
    }

    /// Deserialize the value from a `u64` previously obtained by [`ArticleId::serialize`].
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
    #[must_use]
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
        }

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

    /// A bitmask indicating which bits are currently used in the [binary
    /// format](crate::id::ArticleId#in-memory-representation).
    ///
    /// The bitmask is set to `1` if the bit is used, and `0` if the bit is always 0.
    ///
    /// ### Examples
    /// Masking with the bitmask never changes the serialized value.
    /// ```
    /// use rsxiv::id::ArticleId;
    /// let id = ArticleId::parse("math/0309136v2").unwrap();
    /// let serialized = id.serialize();
    /// assert_eq!(serialized, serialized & ArticleId::SERIALIZED_BITMASK);
    /// ```
    pub const SERIALIZED_BITMASK: u64 =
        0b01111111_00001111_00111111_00000001_11111111_11111111_11111111_11111111;
}

impl FromStr for ArticleId {
    type Err = IdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl Display for ArticleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(archive) = self.archive() {
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
        } else {
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

        if let Some(version) = self.version() {
            write!(f, "v{version}")?;
        }

        Ok(())
    }
}

impl Debug for ArticleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArticleId")
            .field("id", &format_args!("{}", self))
            .field("raw", &self.raw)
            .finish()
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

    /// Clear the version, leaving the remaining fields unchanged.
    #[inline]
    pub const fn set_version(raw: u64, v: u16) -> u64 {
        // let [_, _, _, _, _, _, v1, v2] = self.raw.to_be_bytes();
        // let v = u16::from_be_bytes([v1, v2]);
        (raw & 0xFFFF_FFFF_FFFF_0000) | (v as u64)
    }

    #[inline]
    pub const fn is_new_style(raw: u64) -> bool {
        // Just need to check if the archive is not 0
        const MASK: u64 = u64::from_be_bytes([0, 0, 0xFF, 0, 0, 0, 0, 0]);
        raw & MASK == 0
    }
}

/// A wrapper satisfying the arXiv identifier rules.
///
/// ### Ignored subject class
/// The subject class is [automatically dropped](crate::id::ArticleId#no-subject-class) in the [`Display`] and [`PartialEq`]
/// implementations.
/// ```
/// use rsxiv::id::Validated;
/// let valid = Validated::parse("math.CA/9203001").unwrap();
/// let valid_no_sc = Validated::parse("math/9203001").unwrap();
///
/// assert_eq!(valid, valid_no_sc);
/// assert_eq!(valid.to_string(), "math/9203001");
///
/// // the inner string is not modified
/// assert_eq!(valid.into_inner(), "math.CA/9203001");
/// ```
/// This is also the case for the [`Identifier::identifier`] method.
/// ```
/// use rsxiv::id::Identifier;
/// use std::borrow::Cow;
/// # use rsxiv::id::Validated;
/// # let valid = Validated::parse("math.CA/9203001").unwrap();
/// # let valid_no_sc = Validated::parse("math/9203001").unwrap();
///
/// // the subject class `.CA` must be dropped, which requires allocating
/// assert!(matches!(valid.identifier(), Cow::Owned(_)));
/// assert_eq!(valid.identifier(), "math/9203001");
/// // without a subject class, we can borrow from the internal buffer
/// assert!(matches!(valid_no_sc.identifier(), Cow::Borrowed(_)));
///
/// ```
///
/// ### Field access
/// In order to access the various fields, first convert to an [`ArticleId`].
///
/// The conversion to an [`ArticleId`] is cheaper than using [`ArticleId::parse`] since the format
/// is guaranteed to be valid.
/// ```
/// use rsxiv::id::{ArticleId, Validated};
/// let valid = Validated::parse("7304.01823v4234").unwrap();
/// let id = ArticleId::from(&valid);
/// assert_eq!(id.year(), 2073);
/// ```
#[derive(Debug, Clone)]
pub struct Validated<S> {
    inner: S,
}

impl<S: AsRef<str>, I: Identifier> PartialEq<I> for Validated<S> {
    fn eq(&self, other: &I) -> bool {
        self.identifier().eq(&other.identifier())
    }
}

impl<S: AsRef<str>> Eq for Validated<S> {}

impl<S: AsRef<str>> Display for Validated<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.inner.as_ref();
        match unsafe { split_subject_class_unchecked(s) } {
            Some((l, r)) => {
                f.write_str(l)?;
                f.write_str(r)
            }
            None => f.write_str(s),
        }
    }
}

/// Split a string slice at a 'subject class'
///
/// The string must have originally resulted from a valid arxiv identifier.
unsafe fn split_subject_class_unchecked(s: &str) -> Option<(&str, &str)> {
    // TODO: come up with a better implementation
    unsafe {
        if let Some((l, r)) = s.split_once('.') {
            // SAFETY: every identifier contains at least one character following the `.`
            if r.as_bytes().get_unchecked(0).is_ascii_uppercase() {
                // SAFETY: we matched a subject class
                let rest = r.get_unchecked(2..);
                return Some((l, rest));
            }
        }
    }
    None
}

/// A special error type used by [`Validated::parse`] to return the original argument in the
/// presence of an error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError<S> {
    pub invalid: S,
    pub id_err: IdError,
}

impl<S> From<ValidationError<S>> for IdError {
    fn from(value: ValidationError<S>) -> Self {
        value.id_err
    }
}

impl<S: Display> Display for ValidationError<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error parsing {}: {}", self.invalid, self.id_err)
    }
}

impl<S: Display + Debug> Error for ValidationError<S> {}

impl<S: AsRef<str>> Validated<S> {
    /// Construct a new validated identifier.
    ///
    /// # Examples
    /// ```
    /// use rsxiv::id::Validated;
    ///
    /// ```
    pub fn parse(s: S) -> Result<Self, ValidationError<S>> {
        match validate(s.as_ref()) {
            Ok(()) => Ok(Self { inner: s }),
            Err(id_err) => Err(ValidationError { invalid: s, id_err }),
        }
    }

    /// Return the unmodified inner component.
    pub fn into_inner(self) -> S {
        self.inner
    }
}

macro_rules! validated_from_impl {
    ($target:ty) => {
        impl<S: AsRef<str>> From<$target> for ArticleId {
            fn from(value: $target) -> Self {
                // SAFETY: There are only two ways to construct a `Validated<S>`.
                //
                // 1. Via the `::parse` method, which is internally a call to ArticleId::parse and which
                //    discards the resulting identifier. Since ArticleId::parse is a const fn, it is
                //    guaranteed that
                //    the subsequent calls will result in the same output.
                // 2. Via the `::from` implementation, which internally uses the ArticleId Display
                //    implementation and therefore results in an identifier which is valid.
                unsafe { ArticleId::parse(value.inner.as_ref()).unwrap_unchecked() }
            }
        }
    };
}

validated_from_impl!(Validated<S>);
validated_from_impl!(&Validated<S>);
validated_from_impl!(&mut Validated<S>);

impl FromStr for Validated<String> {
    type Err = IdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s.to_owned()).map_err(|err| err.id_err)
    }
}

impl From<ArticleId> for Validated<String> {
    fn from(value: ArticleId) -> Self {
        Self {
            inner: value.to_string(),
        }
    }
}

/// Types which are arXiv identifiers.
///
/// This trait is sealed and cannot be implemented outside this crate.
/// It is implemented by [`ArticleId`] and [`Validated<S>`].
pub trait Identifier: private::Sealed {
    /// Append the identifier to the provided string buffer.
    ///
    /// This is the equivalent to using [`Identifier::identifier`], but without
    /// intermediate allocations.
    /// ```
    /// use rsxiv::id::{Validated, Identifier};
    /// let validated_id = Validated::parse("math.CA/0001004v3").unwrap();
    ///
    /// let mut buffer = "arXiv:".to_owned();
    /// validated_id.write_identifier(&mut buffer);
    ///
    /// assert_eq!(buffer, "arXiv:math/0001004v3");
    /// ```
    fn write_identifier(&self, buffer: &mut String);

    /// Obtain the identifier text corresponding to the identifier.
    fn identifier(&self) -> Cow<'_, str> {
        let mut buffer = String::with_capacity(MAX_ID_FORMATTED_LEN);
        self.write_identifier(&mut buffer);
        Cow::Owned(buffer)
    }
}

impl Identifier for ArticleId {
    fn write_identifier(&self, buffer: &mut String) {
        use std::fmt::Write;
        let _ = write!(buffer, "{self}");
    }
}

impl<S: AsRef<str>> Identifier for Validated<S> {
    fn identifier(&self) -> Cow<'_, str> {
        let s = self.inner.as_ref();
        match unsafe { split_subject_class_unchecked(s) } {
            Some((l, r)) => {
                let mut owned = String::with_capacity(l.len() + r.len());
                owned.push_str(l);
                owned.push_str(r);
                Cow::Owned(owned)
            }
            None => Cow::Borrowed(s),
        }
    }

    fn write_identifier(&self, buffer: &mut String) {
        let s = self.inner.as_ref();
        match unsafe { split_subject_class_unchecked(s) } {
            Some((l, r)) => {
                buffer.push_str(l);
                buffer.push_str(r);
            }
            None => buffer.push_str(s),
        }
    }
}

mod private {
    use super::{ArticleId, Validated};

    pub trait Sealed {}
    impl Sealed for ArticleId {}
    impl<S: AsRef<str>> Sealed for Validated<S> {}
}
