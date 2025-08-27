//! # XML response parsing
//!
//! This module implements an XML parsing layer on top of [`quick_xml::Reader`]. The main benefit
//! of this implementation is the iteration method [`Reader::find_before`].
use std::borrow::Cow;

use quick_xml::{
    Reader as QReader,
    errors::Error,
    events::{BytesEnd, BytesStart, Event as QEvent},
};

/// Similar to [`quick_xml::events::Event`], but only containing the events which are important for
/// this module.
pub enum Event<'r> {
    Start(BytesStart<'r>),
    End(BytesEnd<'r>),
    Empty(BytesStart<'r>),
}

/// A wrapper for [`quick_xml::Reader`] with peeking and other convenience iteration methods.
pub struct Reader<'r> {
    reader: QReader<&'r [u8]>,
    peeked: Option<Event<'r>>,
    xml: &'r [u8],
}

impl<'r> Reader<'r> {
    /// Initialize from raw xml content.
    pub fn new(xml: &'r [u8]) -> Self {
        Self {
            reader: QReader::from_reader(xml),
            peeked: None,
            xml,
        }
    }

    /// Read the next event without checking if there is a peeked event.
    #[inline]
    fn read_inner(&mut self) -> Result<Option<Event<'r>>, Error> {
        Ok(loop {
            break match self.reader.read_event()? {
                QEvent::Start(bytes_start) => Some(Event::Start(bytes_start)),
                QEvent::End(bytes_end) => Some(Event::End(bytes_end)),
                QEvent::Empty(bytes_start) => Some(Event::Empty(bytes_start)),
                QEvent::Eof => None,
                _ => continue,
            };
        })
    }

    /// Read the next event, or `None` if the end of the file was reached.
    pub fn read(&mut self) -> Result<Option<Event<'r>>, Error> {
        if let Some(event) = self.peeked.take() {
            return Ok(Some(event));
        }

        self.read_inner()
    }

    /// Read the text between the provided start tag and the matching end tag.
    pub fn read_text(&mut self, start: &BytesStart<'_>) -> Result<Cow<'r, str>, Error> {
        self.reader.read_text(start.to_end().name())
    }

    /// Find an event, but halting if `halt` applied to the upcoming element returns `true`,
    /// in which case the subsequent element is not consumed and will be returned on the next call
    /// to [`read`](Self::read).
    pub fn find_before<B, F, H>(&mut self, mut f: F, mut halt: H) -> Result<Option<B>, Error>
    where
        F: FnMut(Event<'r>) -> Option<B>,
        H: FnMut(&Event<'r>) -> bool,
    {
        let Some(mut current) = self.read()? else {
            return Ok(None);
        };

        // self.peeked is `None`
        loop {
            if halt(&current) {
                // if we halt, put the element back and return `None`
                self.peeked = Some(current);
                return Ok(None);
            } else {
                // otherwise, consume it with `f` (returning if relevant) and queue the next
                // element
                if let Some(mapped) = f(current) {
                    return Ok(Some(mapped));
                }

                current = match self.read_inner()? {
                    Some(e) => e,
                    None => return Ok(None),
                }
            }
        }
    }

    /// Find an opening tag with the provided tag name.
    fn find_matching_tag(&mut self, tag: &[u8]) -> Result<Option<BytesStart<'r>>, Error> {
        self.find_before(
            |event| match event {
                Event::Start(bytes_start) if bytes_start.name().0 == tag => Some(bytes_start),
                _ => None,
            },
            |_| false,
        )
    }

    /// Returns the decoded text contents in the first tag matching the provided tag name.
    pub fn find_text_matching_tag(&mut self, tag: &[u8]) -> Result<Option<Cow<'r, str>>, Error> {
        if let Some(bytes_start) = self.find_matching_tag(tag)? {
            // read the text
            Ok(Some(self.read_text(&bytes_start)?))
        } else {
            Ok(None)
        }
    }

    /// Returns the raw tag contents (as bytes) of the first tag matching the provided tag name.
    pub fn find_raw_matching_tag(&mut self, tag: &[u8]) -> Result<Option<&'r [u8]>, Error> {
        if let Some(bytes_start) = self.find_matching_tag(tag)? {
            // read the bytes
            let span = self.reader.read_to_end(bytes_start.to_end().name())?;
            Ok(self.xml.get(span.start as usize..span.end as usize))
        } else {
            Ok(None)
        }
    }
}
