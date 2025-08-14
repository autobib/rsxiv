use std::{fmt::Display, num::NonZero, str::FromStr};

mod new;
mod old;
mod parse;

#[cfg(test)]
mod tests;

pub use self::{
    new::NewID,
    old::{Archive, OldID},
};

pub trait Identifier: Display + FromStr<Err = IdentifierError> + Sized {
    /// The archive type associated with the identifier.
    type Archive;

    /// The archive, only present in old-style identifiers.
    fn archive(&self) -> Self::Archive;

    /// The year.
    fn year(&self) -> u16;

    /// The month.
    fn month(&self) -> u8;

    /// The number.
    fn number(&self) -> NonZero<u32>;

    /// The version.
    fn version(&self) -> Option<NonZero<u8>>;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum IdentifierError {
    DateOutOfRange,
    NumberOutOfRange,
    InvalidDate,
    InvalidNumber,
    InvalidVersion,
    InvalidArchive,
    IncorrectSeparator,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum ArticleID {
    Old(OldID),
    New(NewID),
}

impl ArticleID {
    #[must_use]
    pub const fn is_old_style(&self) -> bool {
        matches!(self, ArticleID::Old(_))
    }

    #[must_use]
    pub const fn is_new_style(&self) -> bool {
        matches!(self, ArticleID::New(_))
    }

    pub const fn parse(id: &str) -> Result<Self, IdentifierError> {
        Self::parse_bytes(id.as_bytes())
    }

    pub const fn parse_bytes(id: &[u8]) -> Result<Self, IdentifierError> {
        match id.first() {
            Some(b'1'..=b'9') => match NewID::parse_bytes(id) {
                Ok(n) => Ok(ArticleID::New(n)),
                Err(e) => Err(e),
            },
            _ => match OldID::parse_bytes(id) {
                Ok(n) => Ok(ArticleID::Old(n)),
                Err(e) => Err(e),
            },
        }
    }
}

impl Identifier for ArticleID {
    type Archive = Option<Archive>;

    fn archive(&self) -> Option<Archive> {
        match self {
            ArticleID::Old(old_id) => Some(old_id.archive()),
            ArticleID::New(_) => None,
        }
    }

    fn year(&self) -> u16 {
        match self {
            ArticleID::Old(old_id) => old_id.year(),
            ArticleID::New(new_id) => new_id.year(),
        }
    }

    fn month(&self) -> u8 {
        match self {
            ArticleID::Old(old_id) => old_id.month(),
            ArticleID::New(new_id) => new_id.month(),
        }
    }

    fn number(&self) -> NonZero<u32> {
        match self {
            ArticleID::Old(old_id) => old_id.number(),
            ArticleID::New(new_id) => new_id.number(),
        }
    }

    fn version(&self) -> Option<NonZero<u8>> {
        match self {
            ArticleID::Old(old_id) => old_id.version(),
            ArticleID::New(new_id) => new_id.version(),
        }
    }
}

impl FromStr for ArticleID {
    type Err = IdentifierError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl Display for ArticleID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArticleID::Old(old_id) => old_id.fmt(f),
            ArticleID::New(new_id) => new_id.fmt(f),
        }
    }
}
