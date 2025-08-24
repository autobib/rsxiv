use super::*;

#[test]
fn test_arxiv_name_parse() {
    fn assert_name_eq(raw: &str, firstnames: &str, keyname: &str, suffix: &str) {
        assert_eq!(
            AuthorName::from_arxiv(raw),
            AuthorName {
                keyname: keyname.to_owned(),
                firstnames: firstnames.to_owned(),
                suffix: suffix.to_owned(),
            }
        );
    }

    assert_name_eq("A. B. Doe", "A. B.", "Doe", "");
    assert_name_eq("John von Neumann", "John", "von Neumann", "");
    assert_name_eq("mac Arthur III", "", "mac Arthur", "III");
    assert_name_eq("Ursula von der Leyen", "Ursula", "von der Leyen", "");
    assert_name_eq("Robert Jr.", "", "Robert", "Jr.");
    assert_name_eq("Jean D'Arcy", "Jean", "D'Arcy", "");
    assert_name_eq("only lowercase names", "only lowercase", "names", "");
    assert_name_eq("Jr", "", "Jr", "");
    assert_name_eq("", "", "", "");
}

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
    assert_eq!(
        response.entry[9].author[0].name,
        AuthorName {
            firstnames: "Toshio".to_owned(),
            keyname: "Suzuki".to_owned(),
            suffix: String::new()
        }
    );
    assert_eq!(response.entry[9].author[0].affiliation, None);

    let contents = include_str!("tests/query_empty.xml");
    let response = Response::<Vec<Entry>>::from_xml(&contents).unwrap();
    assert_eq!(
        Ok(response.updated),
        chrono::DateTime::parse_from_rfc3339("2025-08-20T00:00:00-04:00")
    );
    assert!(response.entry.is_empty());
}
