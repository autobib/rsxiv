use std::{fmt::Display, num::NonZero, str::FromStr};

mod new;
mod old;
mod parse;

#[cfg(test)]
mod tests;

pub use self::{
    new::NewId,
    old::{Archive, OldId},
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
pub enum ArticleId {
    Old(OldId),
    New(NewId),
}

impl ArticleId {
    #[must_use]
    pub const fn is_old_style(&self) -> bool {
        matches!(self, ArticleId::Old(_))
    }

    #[must_use]
    pub const fn is_new_style(&self) -> bool {
        matches!(self, ArticleId::New(_))
    }

    pub const fn parse(id: &str) -> Result<Self, IdentifierError> {
        Self::parse_bytes(id.as_bytes())
    }

    pub const fn parse_bytes(id: &[u8]) -> Result<Self, IdentifierError> {
        match id.first() {
            Some(b'0'..=b'9') => match NewId::parse_bytes(id) {
                Ok(n) => Ok(ArticleId::New(n)),
                Err(e) => Err(e),
            },
            _ => match OldId::parse_bytes(id) {
                Ok(n) => Ok(ArticleId::Old(n)),
                Err(e) => Err(e),
            },
        }
    }
}

impl Identifier for ArticleId {
    type Archive = Option<Archive>;

    fn archive(&self) -> Option<Archive> {
        match self {
            ArticleId::Old(old_id) => Some(old_id.archive()),
            ArticleId::New(_) => None,
        }
    }

    fn year(&self) -> u16 {
        match self {
            ArticleId::Old(old_id) => old_id.year(),
            ArticleId::New(new_id) => new_id.year(),
        }
    }

    fn month(&self) -> u8 {
        match self {
            ArticleId::Old(old_id) => old_id.month(),
            ArticleId::New(new_id) => new_id.month(),
        }
    }

    fn number(&self) -> NonZero<u32> {
        match self {
            ArticleId::Old(old_id) => old_id.number(),
            ArticleId::New(new_id) => new_id.number(),
        }
    }

    fn version(&self) -> Option<NonZero<u8>> {
        match self {
            ArticleId::Old(old_id) => old_id.version(),
            ArticleId::New(new_id) => new_id.version(),
        }
    }
}

impl FromStr for ArticleId {
    type Err = IdentifierError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl Display for ArticleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArticleId::Old(old_id) => old_id.fmt(f),
            ArticleId::New(new_id) => new_id.fmt(f),
        }
    }
}
