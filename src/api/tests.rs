use super::*;

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

    assert_eq!(
        Query::new()
            .set_search_query("all:electron AND all:proton")
            .url(),
        Query::new()
            .set_search_query(
                search::GroupList::new(search::Field::All("electron"))
                    .and(search::Field::All("proton"))
            )
            .url()
    );
}
