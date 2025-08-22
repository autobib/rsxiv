use super::*;

#[test]
fn test_query_parse() {
    let contents = include_str!("tests/query.xml");
    let response = Response::<Vec<Entry>>::from_xml(&contents).unwrap();
    assert_eq!(
        Ok(response.updated),
        chrono::DateTime::parse_from_rfc3339("2025-08-20T00:00:00-04:00")
    );
    assert_eq!(response.pagination.total_results, 7370,);
    assert_eq!(response.pagination.start_index, 0);
    assert_eq!(response.pagination.items_per_page, 10);
    assert_eq!(
        Ok(response.entry[0].id),
        crate::id::ArticleId::parse("astro-ph/9904306v1")
    );
    assert_eq!(response.entry.len(), 10);
    assert_eq!(response.entry[9].author[0].name, "Toshio Suzuki");
    assert_eq!(response.entry[9].author[0].affiliation, None);

    let contents = include_str!("tests/query_empty.xml");
    let response = Response::<Vec<Entry>>::from_xml(&contents).unwrap();
    assert_eq!(
        Ok(response.updated),
        chrono::DateTime::parse_from_rfc3339("2025-08-20T00:00:00-04:00")
    );
    assert!(response.entry.is_empty());
}
