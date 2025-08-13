use std::{fmt::Display, num::NonZero, str::FromStr};

mod new;
mod old;
mod parse;

pub use self::{
    new::NewID,
    old::{Archive, OldID},
};

pub trait Identifier: Display + FromStr + Sized {
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

pub enum ArticleID {
    Old(OldID),
    New(NewID),
}

// #[cfg(test)]
// mod tests {
//     use super::*;
// }
