use super::*;
use crate::ArticleId;
use chrono::{DateTime, FixedOffset};
use std::borrow::Cow;

#[test]
fn test_malfored_query() {
    use serde::Deserialize;

    use std::collections::BTreeMap;

    #[derive(Debug, Clone, PartialEq, Deserialize)]
    pub struct Entry<'r> {
        pub authors: Vec<AuthorName>,
        pub comment: Option<&'r str>,
    }

    let contents = include_str!("../response/tests/query_missing_id.xml").as_bytes();

    let response = Response::<Vec<Entry>>::from_xml(contents).unwrap();
    assert_eq!(response.entries.len(), 1);
    assert!(response.entries[0].comment.is_some());

    let response = Response::<BTreeMap<ArticleId, Entry>>::from_xml(contents).unwrap();
    assert_eq!(
        response
            .entries
            .get(&ArticleId::parse("2201.13452v1").unwrap())
            .unwrap()
            .authors[0]
            .to_string(),
        "Hong-Ming Yin"
    );
}

#[cfg(feature = "serde")]
#[test]
fn test_query_de() {
    use serde::Deserialize;

    /// Typed representation of a single entry in the arXiv API response.
    #[derive(Debug, Clone, PartialEq, Deserialize)]
    pub struct Entry<'r> {
        /// The arXiv identifier of the entry.
        pub id: ArticleId,
        /// The date that the retrieved version of the article was submitted.
        pub updated: DateTime<FixedOffset>,
        /// The date that version 1 was submitted.
        pub published: DateTime<FixedOffset>,
        /// The title of the article.
        #[serde(borrow)]
        pub title: Cow<'r, str>,
        /// The article abstract.
        #[serde(borrow)]
        pub summary: Cow<'r, str>,
        /// The article authors.
        pub authors: Vec<Author<'r>>,
        /// A url for the resolved DOI to an external resource.
        #[serde(borrow)]
        pub doi: Option<Cow<'r, str>>,
        /// The author comment.
        #[serde(borrow)]
        pub comment: Option<Cow<'r, str>>,
        /// A journal reference.
        #[serde(borrow)]
        pub journal_ref: Option<Cow<'r, str>>,
        /// The primary arXiv or ACM or MSC category for an article.
        #[serde(borrow)]
        pub primary_category: Cow<'r, str>,
        /// The arXiv or ACM or MSC category for an article.
        #[serde(borrow)]
        pub categories: Vec<Cow<'r, str>>,
        extra: Option<String>,
    }

    /// An article author.
    #[derive(Debug, Clone, PartialEq, Deserialize)]
    pub struct Author<'r> {
        /// The name of the author.
        pub name: AuthorName,
        /// The affiliation of the author.
        #[serde(borrow)]
        pub affiliation: Option<Cow<'r, str>>,
    }

    let contents = include_str!("../response/tests/query.xml").as_bytes();
    let response = Response::<Vec<Entry>>::from_xml(contents).unwrap();
    assert_eq!(
        Ok(response.updated),
        chrono::DateTime::parse_from_rfc3339("2025-08-20T00:00:00-04:00")
    );
    assert_eq!(response.pagination.total_results, 7370,);
    assert_eq!(response.pagination.start_index, 0);
    assert_eq!(response.pagination.items_per_page, 10);
    assert_eq!(
        Ok(response.entries[0].id),
        crate::id::ArticleId::parse("astro-ph/9904306v1")
    );
    assert_eq!(response.entries.len(), 10);
    assert_eq!(
        response.entries[9].authors[0].name,
        AuthorName {
            firstnames: "Toshio".to_owned(),
            keyname: "Suzuki".to_owned(),
            suffix: String::new()
        }
    );
    assert_eq!(response.entries[9].authors[0].affiliation, None);

    let contents = include_str!("../response/tests/query_empty.xml").as_bytes();
    let response = Response::<Vec<Entry>>::from_xml(contents).unwrap();
    assert_eq!(
        Ok(response.updated),
        chrono::DateTime::parse_from_rfc3339("2025-08-20T00:00:00-04:00")
    );
    assert!(response.entries.is_empty());
}
