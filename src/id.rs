use std::{fmt::Display, num::NonZero, str::FromStr};

mod new;
mod old;
mod parse;

pub use self::{
    new::NewID,
    old::{Archive, OldID},
};

pub trait Identifier: Display + FromStr<Err = IdentifierError> + Sized {
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
}

#[derive(Debug, Clone, Copy)]
pub enum ArticleID {
    Old(OldID),
    New(NewID),
}

impl ArticleID {
    #[must_use]
    pub fn is_old_style(&self) -> bool {
        matches!(self, ArticleID::Old(_))
    }

    #[must_use]
    pub fn is_new_style(&self) -> bool {
        matches!(self, ArticleID::New(_))
    }
}

impl Identifier for ArticleID {
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
        match s.as_bytes().first() {
            Some(b'1'..=b'9') => NewID::from_str(s).map(ArticleID::New),
            _ => OldID::from_str(s).map(ArticleID::Old),
        }
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

// #[cfg(test)]
// mod tests {
//     use super::*;
// }
