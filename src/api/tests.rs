use super::*;

use crate::api::search::Search;

#[test]
fn test_url() {
    assert_eq!(
        Query::new().url().to_string(),
        "https://export.arxiv.org/api/query?"
    );

    assert_eq!(
        Query::new().http().url().to_string(),
        "http://export.arxiv.org/api/query?"
    );

    let mut query = Query::new();
    query
        .search_query()
        .init(search::Field::All("electron"))
        .and(search::Field::All("proton"));

    assert_eq!(
        query.url().to_string(),
        "https://export.arxiv.org/api/query?search_query=all%3Aelectron+AND+all%3Aproton"
    );
}
