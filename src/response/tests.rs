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
