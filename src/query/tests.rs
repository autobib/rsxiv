use super::*;
use crate::query::field::{Combine, Field, FieldGroup};

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
        .init(Field::all("electron").unwrap())
        .and(Field::all("proton").unwrap());

    assert_eq!(
        query.url().to_string(),
        "https://export.arxiv.org/api/query?search_query=all%3Aelectron+AND+all%3Aproton"
    );

    let mut query = Query::new();

    let group1 = FieldGroup::init(Field::ti("a").unwrap())
        .or(Field::rn("b").unwrap())
        .and_not(Field::all("c").unwrap());
    let group2 = FieldGroup::init(Field::au("b").unwrap());

    query.search_query().init(group1).and(group2);
    query
        .paginate(20, 10)
        .unwrap()
        .sort(SortBy::SubmittedDate, SortOrder::Ascending);

    assert_eq!(
        query.url().to_string(),
        "https://export.arxiv.org/api/query?search_query=%28ti%3Aa+OR+rn%3Ab+ANDNOT+all%3Ac%29+AND+au%3Ab&start=20&max_results=10&sortBy=submittedDate&sortOrder=ascending"
    );
}
