//! # Base response parsing methods
//!
//! Basic parsing methods for working with the arXiv API response.
use std::borrow::Cow;

use chrono::{DateTime, FixedOffset};

use super::{Pagination, ResponseError};
use crate::xml::{Event, Reader};

/// A convenience trait to unwrap a `Result<Option<_>, ResponseError>` using the
/// [`ResponseError::MissingTag`] variant.
trait AndNotMissing<T> {
    fn and_not_missing(self, tag: &'static str) -> Result<T, ResponseError>;
}

impl<T> AndNotMissing<T> for Result<Option<T>, ResponseError> {
    fn and_not_missing(self, tag: &'static str) -> Result<T, ResponseError> {
        self.and_then(|val| val.ok_or(ResponseError::MissingTag(tag)))
    }
}

/// An empty tag which is expected to have an attribute named `term`.
pub struct Term<'r> {
    inner: quick_xml::events::BytesStart<'r>,
}

impl<'r> Term<'r> {
    /// Get the value of the attribute named `term`.
    ///
    /// Because of lifetime issues with the default `quick_xml` implementation, the lifetime of the
    /// resulting `Cow` os tied to the lifetime of the buffer itself, rather than the underlying
    /// record.
    pub fn get(&self) -> Result<Cow<'_, str>, ResponseError> {
        match self.inner.try_get_attribute(b"term")? {
            Some(attribute) => Ok(attribute.unescape_value()?),
            None => Err(ResponseError::MissingTerm),
        }
    }
}

/// A reader with methods specialized for the arXiv API response.
///
/// The call order of the methods are very important, since we expect the tags to be in a specific
/// order. However, the methods are implemented so that repeated calls to the same search met will
/// not read beyond the current entry, with the exception of [`Self::next_id`].
pub struct ResponseReader<'r> {
    xml_reader: Reader<'r>,
}

impl<'r> ResponseReader<'r> {
    /// Initialize the driver, parsing some header information and setting the cursor
    /// immediately preceding the first `<entry>` (if any).
    pub fn init(xml: &'r [u8]) -> Result<(DateTime<FixedOffset>, Pagination, Self), ResponseError> {
        let mut resp = Self::new(xml)?;
        let updated = resp.read_updated()?;
        let pagination = resp.read_pagination()?;

        Ok((updated, pagination, resp))
    }

    fn new(xml: &'r [u8]) -> Result<Self, ResponseError> {
        let driver = Reader::new(xml);
        Ok(Self { xml_reader: driver })
    }

    fn read_updated(&mut self) -> Result<DateTime<FixedOffset>, ResponseError> {
        let Some(datetime) = self
            .xml_reader
            .find_text_matching_tag(|t| t == b"updated")?
        else {
            return Err(ResponseError::MissingTag("updated"));
        };

        Ok(DateTime::parse_from_rfc3339(&datetime)?)
    }

    /// Interpret the contents of a tag with the provided name as a `u64`.
    fn read_tag_u64(&mut self, name: &'static str) -> Result<u64, ResponseError> {
        let Some(total_results) = self
            .xml_reader
            .find_text_matching_tag(|t| t == name.as_bytes())?
        else {
            return Err(ResponseError::MissingTag(name));
        };

        total_results.parse().map_err(|_| {
            ResponseError::InvalidHeader(
                "expected pagination to be non-negative integer".to_owned(),
            )
        })
    }

    /// Parse pagination data from the response header information.
    fn read_pagination(&mut self) -> Result<Pagination, ResponseError> {
        let items_per_page = self.read_tag_u64("opensearch:itemsPerPage")?;
        let total_results = self.read_tag_u64("opensearch:totalResults")?;
        let start_index = self.read_tag_u64("opensearch:startIndex")?;
        Ok(Pagination {
            total_results,
            start_index,
            items_per_page,
        })
    }

    /// Read the contents of the next `<id>` tag, stripping the URL prefix and raising an error if
    /// it is an `error-style` identifier. If this method returns `Ok(Some(_))`, the cursor is
    /// placed immediately after the closing `id` tag. Otherwise, the cursor position is at the end
    /// of the file.
    ///
    /// This method is implemented in this way since some arxiv responses are malformed and to not
    /// even contain an `<id>` identifier. Instead of trying to parse these entries or worry about
    /// errors, we just skip the entries automatically.
    pub fn next_id(&mut self) -> Result<Option<&'r [u8]>, ResponseError> {
        match self.xml_reader.find_raw_matching_tag(|t| t == b"id")? {
            Some(url) => {
                if url.starts_with(b"http://arxiv.org/api/errors#") {
                    match self
                        .xml_reader
                        .find_text_matching_tag(|t| t == b"summary")?
                    {
                        Some(contents) => Err(ResponseError::Arxiv(contents.into())),
                        None => Err(ResponseError::InvalidError(
                            "missing `summary` tag".to_owned(),
                        )),
                    }
                } else {
                    match url.strip_prefix(b"http://arxiv.org/abs/") {
                        Some(id_bytes) => Ok(Some(id_bytes)),
                        None => Err(ResponseError::InvalidHeader(format!(
                            "`id` tag in unexpected format: {}",
                            String::from_utf8_lossy(url)
                        ))),
                    }
                }
            }
            None => Ok(None),
        }
    }

    /// Find the next tag with the provided `name`, but do not read an end tag named
    /// `limit`.
    fn next_tag_with_name_limit(
        &mut self,
        name: &str,
        limit: &str,
    ) -> Result<Option<Cow<'r, str>>, ResponseError> {
        match self.xml_reader.find_before(
            |event| match event {
                Event::Start(bytes_start) if bytes_start.name().0 == name.as_bytes() => {
                    Some(bytes_start)
                }
                _ => None,
            },
            |event| {
                matches!(event,
                Event::End(bytes_end) if bytes_end.name().0 == limit.as_bytes())
            },
        )? {
            Some(bytes_start) => Ok(Some(self.xml_reader.read_text(&bytes_start)?)),
            None => Ok(None),
        }
    }

    /// Returns the contents of the next `<title>' tag in the entry, but not reading beyond the
    /// current entry.
    #[inline]
    pub fn next_title(&mut self) -> Result<Cow<'r, str>, ResponseError> {
        self.next_tag_with_name_limit("title", "entry")
            .and_not_missing("title")
    }

    /// Returns the contents of the next `<updated>' tag in the entry, but not reading beyond the
    /// current entry.
    #[inline]
    pub fn next_updated(&mut self) -> Result<Cow<'r, str>, ResponseError> {
        self.next_tag_with_name_limit("updated", "entry")
            .and_not_missing("updated")
    }

    /// Returns the contents of the next `<summary>' tag in the entry, but not reading beyond the
    /// current entry.
    #[inline]
    pub fn next_summary(&mut self) -> Result<Cow<'r, str>, ResponseError> {
        self.next_tag_with_name_limit("summary", "entry")
            .and_not_missing("summary")
    }

    /// Find an empty tag with the given name, not exceeding any event on which to halt.
    fn next_term<H>(&mut self, name: &str, halt: H) -> Result<Option<Term<'r>>, ResponseError>
    where
        H: FnMut(&Event<'r>) -> bool,
    {
        match self.xml_reader.find_before(
            |event| match event {
                Event::Empty(bytes_start)
                    if bytes_start.local_name().as_ref() == name.as_bytes() =>
                {
                    Some(bytes_start)
                }
                _ => None,
            },
            halt,
        )? {
            Some(bytes_start) => Ok(Some(Term { inner: bytes_start })),
            None => Ok(None),
        }
    }

    /// Read the next `category` tag.
    pub fn next_category(&mut self) -> Result<Option<Term<'r>>, ResponseError> {
        self.next_term("category", |event| match event {
            Event::Start(bytes_start) => bytes_start.name().0 == b"published",
            Event::End(bytes_end) => bytes_end.name().0 == b"entry",
            _ => false,
        })
    }

    /// Returns the contents of the next `<published>' tag in the entry, but not reading beyond the
    /// current entry.
    #[inline]
    pub fn next_published(&mut self) -> Result<Cow<'r, str>, ResponseError> {
        self.next_tag_with_name_limit("published", "entry")
            .and_not_missing("published")
    }

    /// Read the next `comment` tag.
    ///
    /// This will not read past any of the following tags:
    /// - `Empty(primary_category)`
    pub fn next_comment(&mut self) -> Result<Option<Cow<'r, str>>, ResponseError> {
        match self.xml_reader.find_before(
            |entry| match entry {
                Event::Start(bytes_start) if bytes_start.local_name().as_ref() == b"comment" => {
                    Some(bytes_start)
                }
                _ => None,
            },
            |entry| matches!(entry, Event::Empty(bytes_start) if bytes_start.local_name().as_ref() == b"primary_category"),
        )? {
            Some(bytes_start) => Ok(Some(self.xml_reader.read_text(&bytes_start)?)),
            None => Ok(None),
        }
    }

    /// Read the next `primary_category` tag.
    pub fn next_primary_category(&mut self) -> Result<Term<'r>, ResponseError> {
        self.next_term("primary_category", |entry| match entry {
            Event::End(bytes_end) => bytes_end.name().0 == b"entry",
            _ => false,
        })
        .and_not_missing("primary_category")
    }

    /// Read the next `journal_ref` tag.
    pub fn next_journal_ref(&mut self) -> Result<Option<Cow<'r, str>>, ResponseError> {
        match self.xml_reader.find_before(
            |entry| match entry {
                Event::Start(bytes_start)
                    if bytes_start.local_name().as_ref() == b"journal_ref" =>
                {
                    Some(bytes_start)
                }
                _ => None,
            },
            |entry| match entry {
                Event::Start(bytes_start) => {
                    matches!(bytes_start.local_name().as_ref(), b"author" | b"doi")
                }
                Event::End(bytes_start) => {
                    matches!(bytes_start.name().0, b"entry")
                }
                _ => false,
            },
        )? {
            Some(bytes_start) => Ok(Some(self.xml_reader.read_text(&bytes_start)?)),
            None => Ok(None),
        }
    }

    /// Enter the next `<author>` tag if present, not reading beyond the current entry.
    ///
    /// If this function returns `Ok(true)`, an `<author>` tag was found and the cursor is
    /// immmediately following the tag.
    ///
    /// If this function returns `Ok(false)`, the next tag is not an `<author>` tag.
    ///
    /// This will not read past any of the following tags:
    /// - `Start(doi)`
    pub fn next_author(&mut self) -> Result<bool, ResponseError> {
        match self.xml_reader.find_before(
            |entry| match entry {
                Event::Start(bytes_start) if bytes_start.name().0 == b"author" => Some(()),
                _ => None,
            },
            |entry| match entry {
                Event::Start(bytes_start) => bytes_start.local_name().as_ref() == b"doi",
                Event::End(bytes_start) => {
                    matches!(bytes_start.name().0, b"entry")
                }
                Event::Empty(_) => false,
            },
        )? {
            Some(()) => Ok(true),
            None => Ok(false),
        }
    }

    /// After entering an `<author>` tag, find the next `<name>` tag in the author.
    ///
    /// This will not read past the closing `</author>` tag.
    pub fn next_author_name(&mut self) -> Result<Cow<'r, str>, ResponseError> {
        self.next_tag_with_name_limit("name", "author")
            .and_not_missing("name")
    }

    /// After entering an `<author>` tag, find the next `<affiliation>` tag in the author.
    ///
    /// This will not read past the closing `</author>` tag.
    pub fn next_author_affiliation(&mut self) -> Result<Option<Cow<'r, str>>, ResponseError> {
        self.next_tag_with_name_limit("affiliation", "author")
    }

    /// Read the next `doi` tag.
    ///
    /// This will not read past any of the following tags:
    /// - `Start(comment)`
    /// - `Start(journal_ref)`
    /// - `Empty(primary_category)`
    pub fn next_doi(&mut self) -> Result<Option<Cow<'r, str>>, ResponseError> {
        match self.xml_reader.find_before(
            |entry| match entry {
                Event::Start(bytes_start) if bytes_start.local_name().as_ref() == b"doi" => {
                    Some(bytes_start)
                }
                _ => None,
            },
            |entry| match entry {
                Event::End(bytes_start) => {
                    matches!(bytes_start.name().0, b"entry")
                }
                _ => false,
            },
        )? {
            Some(bytes_start) => Ok(Some(self.xml_reader.read_text(&bytes_start)?)),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_parsing() -> Result<(), ResponseError> {
        let xml = include_str!("tests/query.xml").as_bytes();
        let (updated, pagination, mut reader) = ResponseReader::init(xml)?;
        assert_eq!(
            updated,
            DateTime::parse_from_rfc3339("2025-11-11T18:29:40+00:00").unwrap()
        );
        assert_eq!(
            pagination,
            Pagination {
                total_results: 7432,
                start_index: 0,
                items_per_page: 10,
            }
        );

        // first entry
        assert_eq!(reader.next_id()?, Some("nucl-ex/0408020v1".as_bytes()),);
        assert_eq!(
            reader
                .next_category()?
                .map(|term| String::from(term.get().unwrap())),
            Some("nucl-ex".to_string())
        );
        assert_eq!(
            reader
                .next_category()?
                .map(|term| String::from(term.get().unwrap())),
            Some("hep-ph".to_string())
        );
        assert!(reader.next_category()?.is_none());

        assert_eq!(reader.next_published()?, "2004-08-18T23:13:32Z");

        assert_eq!(
            reader.next_comment()?,
            Some(Cow::Borrowed(
                "13 pages, 10 figures. Proposal for a comparison of electron-proton and positron-proton scattering at VEPP-3"
            ))
        );

        // second entry; skip
        assert!(reader.next_id()?.is_some());

        // third entry
        assert_eq!(reader.next_id()?, Some("1606.02159v1".as_bytes()));

        assert_eq!(reader.next_journal_ref()?, None);

        assert!(reader.next_author()?);
        assert!(reader.next_author()?);
        assert!(!reader.next_author()?);

        // fourth entry
        assert_eq!(reader.next_id()?.unwrap(), "1610.08734v3".as_bytes());
        assert!(reader.next_author()?);
        assert_eq!(reader.next_author_name()?, "Yangmei Li");
        assert_eq!(reader.next_author_affiliation()?, None);
        assert_eq!(reader.next_author_affiliation()?, None);
        assert_eq!(
            reader.next_doi()?.unwrap(),
            "10.1103/PhysRevAccelBeams.20.101301"
        );
        assert!(reader.next_doi()?.is_none());
        assert!(reader.next_doi()?.is_none());

        // fifth entry
        assert_eq!(reader.next_id()?.unwrap(), "2102.00018v2".as_bytes());
        assert_eq!(
            reader.next_journal_ref()?.unwrap(),
            "Phys. Rev. D 103, 096005 (2021)"
        );

        Ok(())
    }
}
