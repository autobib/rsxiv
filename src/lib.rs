//! # RsXiv
//! This crate provides bindings related to [arXiv identifiers][arxid] and the [arXiv api][api]:
//!
//! 1. The [`id`] module contains typed representations of arXiv identifiers, such as `2301.00001`.
//! 2. The [`query`] module provides a builder interface to generate URLs to make requests to the
//!    arXiv API service.
//! 3. The [`response`] module a function to parse the XML response obtained
//!    from the arXiv API service.
//! 4. The [`de`] module provides methods to deserialize the API response into your own types using
//!    a flexible [`serde`] interface.
//!
//! Notably, this crate will not make the network request itself. For that, you might use a crate
//! such as [reqwest](https://crates.io/crates/reqwest) or [ureq](https://crates.io/crates/ureq).
//!
//! ## Examples
//! See the [examples](https://github.com/autobib/rsxiv/blob/master/examples/README.md) directory
//! on GitHub.
//!
//! [arxid]: https://info.arxiv.org/help/arxiv_identifier.html
//! [api]: https://info.arxiv.org/help/api/user-manual.html

#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
pub mod de;
pub mod id;
pub mod query;
pub mod response;
mod xml;

pub use self::{
    id::{ArticleId, Validated},
    query::Query,
    response::Response,
};
