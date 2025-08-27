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
