//! # RsXiv
//! This crate provides bindings related to the [arXiv identifiers][arxid] and the [arXiv api][api]:
//!
//! 1. The [`id`] module contains typed representations of arXiv identifiers, such as `2301.00001`.
//! 2. The [`query`] module provides a builder interface to generate URLs to make requests to the
//!    arXiv API service.
//! 3. The [`response`] module provides a deserialization interface for the XML response obtained
//!    from the arXiv API service.
//!
//! Notably, this crate will not make the network request itself. For that, you might use a crate
//! such as [reqwest](https://crates.io/crates/reqwest) or [ureq](https://crates.io/crates/ureq).
//!
//! [arxid]: https://info.arxiv.org/help/arxiv_identifier.html
//! [api]: https://info.arxiv.org/help/api/user-manual.html


pub mod id;
pub mod query;
pub mod response;

pub use self::{
    id::{ArticleId, Validated},
    query::Query,
    response::Response,
};
