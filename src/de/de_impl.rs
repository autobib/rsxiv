use std::borrow::Cow;

use serde::{
    de::{
        Deserializer, Error, IntoDeserializer, MapAccess, SeqAccess, Visitor,
        value::BorrowedStrDeserializer,
    },
    forward_to_deserialize_any,
};

use crate::response::{ResponseError, ResponseReader, Term};

/// A deserializer for the list of `<entry>` in the response.
pub struct ResponseDeserializer<'a, 'de> {
    reader: &'a mut ResponseReader<'de>,
}

impl<'a, 'de> ResponseDeserializer<'a, 'de> {
    pub fn from_reader(reader: &'a mut ResponseReader<'de>) -> Self {
        Self { reader }
    }
}

impl<'a, 'de> Deserializer<'de> for ResponseDeserializer<'a, 'de> {
    type Error = ResponseError;

    /// By default, deserialize a sequence of `<entry>`s.
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(self)
    }

    /// Deserialize an option as though it were a sequence of length exactly 0 or 1.
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.reader.next_id()? {
            Some(id) => {
                let val = visitor.visit_some(EntryDeserializer {
                    reader: &mut *self.reader,
                    id: Some(id),
                });
                if !self.reader.next_id()?.is_none() {
                    Err(ResponseError::TrailingEntries)
                } else {
                    val
                }
            }

            None => visitor.visit_none(),
        }
    }

    /// Deserialize a map using the `id` as the key.
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    /// Skip everything, checking for errors.
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        while self.reader.next_id()?.is_some() {}
        visitor.visit_unit()
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct struct enum identifier
    }
}

impl<'a, 'de> SeqAccess<'de> for ResponseDeserializer<'a, 'de> {
    type Error = ResponseError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        match self.reader.next_id()? {
            Some(id) => seed
                .deserialize(EntryDeserializer {
                    reader: &mut *self.reader,
                    id: Some(id),
                })
                .map(Some),
            None => Ok(None),
        }
    }
}

impl<'a, 'de> MapAccess<'de> for ResponseDeserializer<'a, 'de> {
    type Error = ResponseError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        match self.reader.next_id()? {
            Some(id) => seed.deserialize(IdDeserializer { id }).map(Some),
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        // we already passed the `id` key to `next_key_seed`, so we deserialize as
        // though it does not exist
        seed.deserialize(EntryDeserializer {
            reader: &mut *self.reader,
            id: None,
        })
    }
}

/// A deserializer holding an identifier.
///
/// The identifier can be deserialized as:
///
/// - `bytes`, `byte_buf`, `any`: as borrowed bytes
/// - `str`, `string`, `identifier`: as borrowed str
/// - `u64`: as the `ArticleId::serialize` format
pub struct IdDeserializer<'de> {
    id: &'de [u8],
}

impl<'de> Deserializer<'de> for IdDeserializer<'de> {
    type Error = ResponseError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(std::str::from_utf8(self.id)?)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_bytes(self.id)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_bytes(self.id)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let parsed = crate::id::ArticleId::parse_bytes(self.id)?;
        visitor.visit_u64(parsed.serialize())
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u128 f32 f64 char
        option unit unit_struct seq tuple
        tuple_struct map struct enum str string identifier
    }
}

pub struct EntryDeserializer<'a, 'de> {
    reader: &'a mut ResponseReader<'de>,
    id: Option<&'de [u8]>,
}

static ALLOWED_FIELDS: [&str; 11] = [
    "id",
    "title",
    "updated",
    "summary",
    "categories",
    "published",
    "comment",
    "primary_category",
    "journal_ref",
    "authors",
    "doi",
];

impl<'a, 'de> Deserializer<'de> for EntryDeserializer<'a, 'de> {
    type Error = ResponseError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_struct("", &ALLOWED_FIELDS, visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let Self { reader, id } = self;
        visitor.visit_map(EntryMapAccess {
            reader,
            id,
            fields,
            idx: 0,
        })
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct seq tuple
        tuple_struct map enum identifier
    }
}

pub struct EntryMapAccess<'a, 'de> {
    reader: &'a mut ResponseReader<'de>,
    id: Option<&'de [u8]>,
    fields: &'static [&'static str],
    idx: usize,
}

impl<'a, 'de> MapAccess<'de> for EntryMapAccess<'a, 'de> {
    type Error = ResponseError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        while self.idx < 11 {
            let name = ALLOWED_FIELDS[self.idx];
            if self.fields.contains(&name) {
                return seed
                    .deserialize(BorrowedStrDeserializer::new(name))
                    .map(Some);
            } else {
                self.idx += 1;
            }
        }
        Ok(None)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        // after `next_key_seed` is called, ALLOWED_FIELDS[self.idx] is the field that the
        // Deserialize impl is requesting
        let val = match self.idx {
            // id
            0 => {
                if let Some(id) = self.id {
                    seed.deserialize(IdDeserializer { id })
                } else {
                    Err(Self::Error::custom(
                        "`id` tag was already deserialized as the map key",
                    ))
                }
            }
            // title
            1 => seed.deserialize(StrTagDeserializer {
                reader: &mut *self.reader,
                getter: ResponseReader::next_title,
            }),
            // updated
            2 => seed.deserialize(StrTagDeserializer {
                reader: &mut *self.reader,
                getter: ResponseReader::next_updated,
            }),
            // summary
            3 => seed.deserialize(StrTagDeserializer {
                reader: &mut *self.reader,
                getter: ResponseReader::next_summary,
            }),
            // category..
            4 => seed.deserialize(CategorySeqAccess {
                reader: &mut *self.reader,
            }),
            // published
            5 => seed.deserialize(StrTagDeserializer {
                reader: &mut *self.reader,
                getter: ResponseReader::next_published,
            }),
            // comment?
            6 => seed.deserialize(StrTagOptDeserializer {
                reader: &mut *self.reader,
                getter: ResponseReader::next_comment,
            }),
            // primary_category
            7 => {
                let term = self.reader.next_primary_category()?;
                seed.deserialize(TermDeserializer { term })
            }
            // journal_ref?
            8 => seed.deserialize(StrTagOptDeserializer {
                reader: &mut *self.reader,
                getter: ResponseReader::next_journal_ref,
            }),
            // author..
            9 => seed.deserialize(AuthorSeqAccess {
                reader: &mut *self.reader,
            }),
            // doi?
            10 => seed.deserialize(StrTagOptDeserializer {
                reader: &mut *self.reader,
                getter: ResponseReader::next_doi,
            }),
            _ => unreachable!(),
        };
        self.idx += 1;
        val
    }
}

pub struct AuthorSeqAccess<'a, 'de> {
    reader: &'a mut ResponseReader<'de>,
}

impl<'a, 'de> Deserializer<'de> for AuthorSeqAccess<'a, 'de> {
    type Error = ResponseError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(self)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct seq tuple
        tuple_struct map struct enum identifier
    }
}

impl<'a, 'de> SeqAccess<'de> for AuthorSeqAccess<'a, 'de> {
    type Error = ResponseError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        if self.reader.next_author()? {
            seed.deserialize(AuthorDeserializer {
                reader: &mut *self.reader,
                idx: 0,
            })
            .map(Some)
        } else {
            Ok(None)
        }
    }
}

pub struct CategorySeqAccess<'a, 'de> {
    reader: &'a mut ResponseReader<'de>,
}

impl<'a, 'de> Deserializer<'de> for CategorySeqAccess<'a, 'de> {
    type Error = ResponseError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(self)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        bytes byte_buf unit unit_struct seq tuple string option
        tuple_struct map struct enum identifier
    }
}

impl<'a, 'de> SeqAccess<'de> for CategorySeqAccess<'a, 'de> {
    type Error = ResponseError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        match self.reader.next_category()? {
            Some(term) => seed.deserialize(TermDeserializer { term }).map(Some),
            None => Ok(None),
        }
    }
}

pub struct TermDeserializer<'de> {
    term: Term<'de>,
}

impl<'de> Deserializer<'de> for TermDeserializer<'de> {
    type Error = ResponseError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(&self.term.get()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(String::from(self.term.get()?))
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(self.term.get()?.into_deserializer())
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        bytes byte_buf unit unit_struct seq tuple option
        tuple_struct map struct identifier ignored_any
    }
}

pub struct AuthorDeserializer<'a, 'de> {
    reader: &'a mut ResponseReader<'de>,
    idx: usize,
}

impl<'a, 'de> Deserializer<'de> for AuthorDeserializer<'a, 'de> {
    type Error = ResponseError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(self)
    }

    #[inline]
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    #[inline]
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.reader.next_author_name()? {
            Cow::Borrowed(name) => visitor.visit_borrowed_str(&name),
            Cow::Owned(name) => visitor.visit_string(name),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let name = self.reader.next_author_name()?;
        visitor.visit_string(String::from(name))
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.reader.next_author_name()? {
            Cow::Borrowed(s) => visitor.visit_enum(BorrowedStrDeserializer::new(s)),
            Cow::Owned(s) => visitor.visit_enum(s.into_deserializer()),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char
        bytes byte_buf option unit unit_struct newtype_struct
        map struct identifier ignored_any
    }
}

impl<'a, 'de> SeqAccess<'de> for AuthorDeserializer<'a, 'de> {
    type Error = ResponseError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        let val = match self.idx {
            0 => seed.deserialize(StrTagDeserializer {
                reader: &mut *self.reader,
                getter: ResponseReader::next_author_name,
            }),
            1 => seed.deserialize(StrTagOptDeserializer {
                reader: &mut *self.reader,
                getter: ResponseReader::next_author_affiliation,
            }),
            _ => return Ok(None),
        };
        self.idx += 1;
        val.map(Some)
    }
}

impl<'a, 'de> MapAccess<'de> for AuthorDeserializer<'a, 'de> {
    type Error = ResponseError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        match self.idx {
            0 => seed
                .deserialize(BorrowedStrDeserializer::new("name"))
                .map(Some),
            1 => seed
                .deserialize(BorrowedStrDeserializer::new("affiliation"))
                .map(Some),
            _ => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let val = match self.idx {
            0 => seed.deserialize(StrTagDeserializer {
                reader: &mut *self.reader,
                getter: ResponseReader::next_author_name,
            }),
            1 => seed.deserialize(StrTagOptDeserializer {
                reader: &mut *self.reader,
                getter: ResponseReader::next_author_affiliation,
            }),
            _ => unreachable!(),
        };
        self.idx += 1;
        val
    }
}

pub struct StrTagDeserializer<'a, 'de> {
    reader: &'a mut ResponseReader<'de>,
    getter: fn(&'a mut ResponseReader<'de>) -> Result<Cow<'de, str>, ResponseError>,
}

impl<'a, 'de> Deserializer<'de> for StrTagDeserializer<'a, 'de> {
    type Error = ResponseError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match (self.getter)(&mut *self.reader)? {
            Cow::Borrowed(name) => visitor.visit_borrowed_str(&name),
            Cow::Owned(name) => visitor.visit_string(name),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let v = (self.getter)(&mut *self.reader)?;
        visitor.visit_string(String::from(v))
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match (self.getter)(&mut *self.reader)? {
            Cow::Borrowed(s) => visitor.visit_enum(BorrowedStrDeserializer::new(s)),
            Cow::Owned(s) => visitor.visit_enum(s.into_deserializer()),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char
        bytes byte_buf unit unit_struct seq tuple str identifier
        tuple_struct map struct option
    }
}

pub struct StrTagOptDeserializer<'a, 'de> {
    reader: &'a mut ResponseReader<'de>,
    getter: fn(&'a mut ResponseReader<'de>) -> Result<Option<Cow<'de, str>>, ResponseError>,
}

impl<'a, 'de> Deserializer<'de> for StrTagOptDeserializer<'a, 'de> {
    type Error = ResponseError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match (self.getter)(&mut *self.reader)? {
            Some(Cow::Borrowed(s)) => visitor.visit_some(BorrowedStrDeserializer::new(s)),
            Some(Cow::Owned(s)) => visitor.visit_some(s.into_deserializer()),
            None => visitor.visit_none(),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match (self.getter)(&mut *self.reader)? {
            Some(v) => visitor.visit_string(String::from(v)),
            None => visitor.visit_none(),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match (self.getter)(&mut *self.reader)? {
            Some(Cow::Borrowed(v)) => visitor.visit_borrowed_str(&v),
            Some(Cow::Owned(v)) => visitor.visit_string(v),
            None => visitor.visit_none(),
        }
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match (self.getter)(&mut *self.reader)? {
            Some(Cow::Borrowed(s)) => visitor.visit_enum(BorrowedStrDeserializer::new(s)),
            Some(Cow::Owned(s)) => visitor.visit_enum(s.into_deserializer()),
            None => visitor.visit_none(),
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char
        bytes byte_buf unit unit_struct seq tuple identifier
        tuple_struct map struct option
    }
}
