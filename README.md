[![Current crates.io release](https://img.shields.io/crates/v/rsxiv)](https://crates.io/crates/rsxiv)
[![Documentation](https://img.shields.io/badge/docs.rs-rsxiv-66c2a5?labelColor=555555&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K)](https://docs.rs/rsxiv/)

# RsXiv

A [Rust](https://www.rust-lang.org/) library to provide an interface for [arXiv identifiers](https://info.arxiv.org/help/arxiv_identifier.html) and the [arXiv API](https://info.arxiv.org/help/api/user-manual.html).

Key features:

- [Typed and validated representations](https://docs.rs/rsxiv/latest/rsxiv/id/index.html) of arXiv identifiers.
- A [query builder](https://docs.rs/rsxiv/latest/rsxiv/query/index.html) to programmatically construct query URLs for the arXiv API.
- A [response parser](https://docs.rs/rsxiv/latest/rsxiv/response/index.html) to parse the API response.
- A low-overhead [serde](https://serde.rs/) interface to [convert the API response](https://docs.rs/rsxiv/latest/rsxiv/de/index.html) to your own types.

This crate will not make the network request itself.
For that, you might use [ureq](https://crates.io/crates/ureq) or [reqwest](https://crates.io/crates/reqwest).

## Example
Example using [ureq](https://crates.io/crates/ureq):
```rust
use std::{borrow::Cow, collections::BTreeMap};

use rsxiv::{
    ArticleId, Query, Response,
    query::{Combine, Field, FieldGroup, SortBy, SortOrder},
    response::AuthorName,
};
use serde::Deserialize;
use ureq;

#[derive(Deserialize)]
struct Entry<'r> {
    // Built-in author name parsing
    authors: Vec<AuthorName>,
    title: Cow<'r, str>,
}

fn main() -> anyhow::Result<()> {
    let mut query = Query::new();
    query
        // sort the results
        .sort(SortBy::SubmittedDate, SortOrder::Ascending)
        // access handle to the search query
        .search_query()
        // require title matching 'proton'
        .init(Field::ti("Proton").unwrap())
        // and require author `Bob`, or author `John`
        .and(FieldGroup::init(Field::au("Bob").unwrap()).or(Field::au("John").unwrap()));

    // make the request using `ureq`
    let response_body = ureq::get(query.url().as_ref())
        .call()?
        .into_body()
        .read_to_vec()?;

    // deserialize API response
    let response = Response::<BTreeMap<ArticleId, Entry>>::from_xml(&response_body)?;

    // sort by year, month, archive, number, version
    for (id, entry) in response.entries.iter() {
        println!(
            "'{}' by {}{} [{id}]",
            entry.title,
            entry.authors[0],
            if entry.authors.len() > 1 {
                " et al."
            } else {
                ""
            }
        );
    }

    Ok(())
}
```
