//! # Response deserialization
//!
//! The [`Response<T>`] struct can the entry list in the API response into any container `T` of
//! type `map` or `seq` in the [serde data model](https://serde.rs/data-model.html)
//! which contains an [`Entry`](crate::response::Entry)-like `map` into which to deserialize each entry.
//!
//! Jump to:
//!
//! - [Data model](#data-model)
//! - [Examples](#examples)
//!
//! ## Data model
//! This is a detailed summary of the deserialization model supported by this crate, using the
//! terminology from the [serde data model](https://serde.rs/data-model.html).
//!
//! Each type is recursively documented, and the default value (that is, the variant provided by
//! `deserialize_any`) is the first value in the numeric list.
//!
//! ### `Response<T>`
//! There are three container options for `T`:
//! 1. `Seq<Entry>`: all of the fields (including the identifier) are passed to `Entry`.
//! 2. `Map<ArticleId, EntryNoId>`: the article identifier for the entry is used as the key and
//!   the remaining fields are passed to `EntryNoId`.
//! 3. `Option<Entry>`: same as `Seq<Entry>`, but expects either `0` or `1` entries.
//!
//! ### `Entry`
//! An `Entry` is a `Map` with the following explicit keys and corresponding values:
//! - `id`: `ArticleId`
//! - `updated`: `DateTime`
//! - `published`: `DateTime`
//! - `title`: `Str`
//! - `summary`: `Str`
//! - `author`: `Seq<Author>`,
//! - `doi`: `Option<Str>`,
//! - `comment`: `Option<Str>`,
//! - `journal_ref`: `Option<Str>`,
//! - `primary_category`: `Option<Str>`,
//! - `category`: `Seq<Str>`,
//!
//! ### `EntryNoId`
//! Identical to `Entry`, but without the `id` field.
//!
//! ### `ArticleId`
//! A representation of a arXiv identifier. Can be deserialized as:
//!
//! 1. `Str`: the identifier string, like `0212.1234v3`.
//! 2. `Bytes`: the identifier as bytes.
//! 3. `u64`: the portable `u64` format obtained from
//!   [`ArticleId::serialize`](crate::id::ArticleId::serialize)
//!
//! Can be deserialized as an [`ArticleId`](crate::id::ArticleId).
//!
//! ### DateTime
//! A datetime in RFC 3339 format.
//!
//! 1. `Str`: the raw value, like `1996-12-19T16:39:57-08:00`
//!
//! Can be deserialized using [`DateTime<FixedOffset>`](`chrono::DateTime::parse_from_rfc3339`).
//!
//! ### `Author`
//! A representation of an arXiv author. Can be deserialized as:
//! 1. `AuthorMap`: a map containing the `name` and optional affiliation
//! 2. `Str`: the raw value, unparsed. Parse using the [`AuthorName`].
//!
//! Can be deserialized as an [`AuthorName`]
//!
//! ### `AuthorMap`
//! A representation of an arXiv author, also capturing affiliation data. Contains fields:
//! - `name`: `Str` (can be deserialized as an [`AuthorName`])
//! - `affiliation`: `Option<Str>`
//!
//! ### `Str`
//! Any serde string type, like `str` or `string` or `borrowed_str`. Whenever possible, this
//! borrows from the input data, but this is not always possible because of escape sequences.
//!
//! ## Examples
//! ### Basic usage example
//! A basic example is as follows.
//! ```
//! use std::collections::BTreeSet;
//!
//! use rsxiv::{response::{Entry, Response}, id::ArticleId};
//! use serde::Deserialize;
//!
//! // abridged arXiv response obtained by querying
//! // `export.arxiv.org/api/query?search_query=cat:math.CA AND ti:diffuse`
//! let xml = // br#"<feed xmlns...
#![doc = include_str!("response/tests/query_doc_long.txt")]
//! # let xml = xml.as_bytes();
//!
//! // deserialize an entry, only keeping the title
//! #[derive(PartialEq, Eq, PartialOrd, Ord, Deserialize)]
//! struct EntryTitle {
//!     title: String,
//! }
//!
//! let response = Response::<BTreeSet<EntryTitle>>::from_xml(&xml).unwrap();
//! assert_eq!(response.entries.len(), 10);
//! assert_eq!(
//!     response.entries.first().unwrap().title,
//!     "A Note on the Axisymmetric Diffusion equation"
//! );
//! ```
//! It is also possible to deserialize into a `map`, in which case the identifier is used as the
//! key. Since [`ArticleId`](crate::ArticleId) implements [`Deserialize`], we can use it directly
//! as the key type.
//! ```
//! use std::collections::BTreeMap;
//!
//! use rsxiv::{response::{Entry, Response, AuthorName}, id::ArticleId};
//! use serde::Deserialize;
//!
//! // abridged arXiv response
//! let xml = // br#"<feed xmlns...
#![doc = include_str!("response/tests/query_doc_long.txt")]
//! # let xml = xml.as_bytes();
//!
//! // deserialize an entry, only keeping the author names
//! #[derive(PartialEq, Eq, PartialOrd, Ord, Deserialize)]
//! struct EntryTitle {
//!     // can also be deserialized as a Vec<Author { name, affiliation }>
//!     authors: Vec<AuthorName>,
//! }
//!
//! let response = Response::<BTreeMap<ArticleId, EntryTitle>>::from_xml(&xml).unwrap();
//! assert_eq!(response.entries.len(), 10);
//! assert_eq!(
//!     response.entries.get(&ArticleId::parse("1810.03952v2").unwrap()).unwrap().authors[2].to_string(),
//!     "John Harlim"
//! );
//! ```
//! ### Complete `Entry` struct
//! This is an entry struct capturing as much data as possible from the arXiv response. Designed to
//! be used as a `Vec<Entry>`.
//! ```
//! use std::borrow::Cow;
//!
//! use chrono::{DateTime, FixedOffset};
//! use serde::Deserialize;
//! use rsxiv::{id::ArticleId, response::AuthorName};
//!
//! #[derive(Deserialize)]
//! pub struct Entry {
//!     pub id: ArticleId,
//!     pub updated: DateTime<FixedOffset>,
//!     pub published: DateTime<FixedOffset>,
//!     pub title: String,
//!     pub summary: String,
//!     pub author: Vec<Author>,
//!     pub doi: Option<String>,
//!     pub comment: Option<String>,
//!     pub journal_ref: Option<String>,
//!     pub primary_category: String,
//!     pub category: Vec<String>,
//! }
//!
//! #[derive(Deserialize)]
//! pub struct Author {
//!     pub name: AuthorName,
//!     pub affiliation: Option<String>,
//! }
//! ```

mod de_impl;
#[cfg(test)]
mod tests;

use serde::{
    Deserialize,
    de::{Deserializer, Error, Visitor},
};

use self::de_impl::ResponseDeserializer;
use crate::response::{AuthorName, Response, ResponseError, ResponseReader};

#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl<'de, T: Deserialize<'de>> Response<T> {
    /// Read a [`Response<T>`] from the raw XML response returned by the arXiv API.
    pub fn from_xml(xml: &'de [u8]) -> Result<Self, ResponseError> {
        let (updated, pagination, mut reader) = ResponseReader::init(xml)?;
        let entries = T::deserialize(ResponseDeserializer::from_reader(&mut reader))?;
        Ok(Response {
            updated,
            pagination,
            entries,
        })
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl Error for ResponseError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Custom(msg.to_string())
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl<'de> Deserialize<'de> for AuthorName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct AuthorNameVisitor;

        impl<'de> Visitor<'de> for AuthorNameVisitor {
            type Value = AuthorName;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a name in arXiv author format")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(AuthorName::from_arxiv(v))
            }
        }
        deserializer.deserialize_str(AuthorNameVisitor)
    }
}
