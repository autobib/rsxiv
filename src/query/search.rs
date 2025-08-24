use std::{
    fmt::{Display, Write as _},
    ops::Range,
};

use chrono::NaiveDateTime;

use crate::query::{BooleanOp, Combine, Field, FieldGroup};

/// A handle to edit an existing search query.
///
/// This struct is construted by the [`Query::search_query`](super::Query::search_query) method.
///
/// ## Syntax
/// A search query is a non-empty list of [search fields](Field) or [`Range<NaiveDateTime>`] combined with [boolean operators](BooleanOp).
///
/// - A [`Field`] is a structured search component corresponding for example to the arXiv search
///   syntax `ti:Title`. The field component must not contained a boolean operator or one of the
///   characters `)<:`.
/// - A [`Range<NaiveDateTime>`] is a constraint on the allowed submission dates returned by the
///   query.
///
/// In order to override the default operator precedence, search fields can be combined into [field groups](FieldGroup).
///
/// ## Example
/// ```
/// use chrono::{NaiveDate, NaiveTime, NaiveDateTime};
/// use rsxiv::query::{Field, FieldGroup, Query, Combine};
///
///
/// // using the combinator methods requires the `Combine` trait
/// let group = FieldGroup::init(Field::ti("a").unwrap())
///     .or(Field::rn("b").unwrap());
///
/// let mut query = Query::new();
/// query.search_query().init(group).and(Field::all("c").unwrap());
///
/// assert_eq!(
///     query.url().to_string(),
///     "https://export.arxiv.org/api/query?search_query=%28ti%3Aa+OR+rn%3Ab%29+AND+all%3Ac"
///     // unencoded query: (ti:a OR rn:b) AND all:c
/// );
///
/// // extend the search query with a submission date constraint
/// let start = NaiveDateTime::new(NaiveDate::from_ymd_opt(2016, 7, 8).unwrap(), NaiveTime::MIN);
/// let end = NaiveDateTime::new(NaiveDate::from_ymd_opt(2023, 2, 18).unwrap(), NaiveTime::MIN);
///
/// query
///     .search_query()
///     .extend()
///     // `extend()` returns `None` if the search query is not set
///     .unwrap()
///     .and_not(start..end);
///
/// assert_eq!(
///     query.url().to_string(),
///     "https://export.arxiv.org/api/query?search_query=%28ti%3Aa+OR+rn%3Ab%29+AND+all%3Ac+ANDNOT+submittedDate%3A%5B201607080000+TO+202302180000%5D"
///     // unencoded query: (ti:a OR rn:b) AND all:c ANDNOT submittedDate:[201607080000 TO 202302180000]
/// );
/// ```
///
/// [api]: https://info.arxiv.org/help/api/user-manual.html#query_details
pub struct SearchQuery<'q> {
    pub(super) buffer: &'q mut String,
}

impl<'q> SearchQuery<'q> {
    /// Initialize the query with a [`Field`], [`FieldGroup`], or any other type which can be
    /// converted into a [`FieldGroup`].
    ///
    /// This method deletes the existing query string.
    #[inline]
    pub fn init<E: Into<FieldGroup>>(self, initial: E) -> NonEmptySearchQuery<'q> {
        self.buffer.clear();
        let _ = write!(self.buffer, "{}", initial.into());
        NonEmptySearchQuery {
            buffer: self.buffer,
        }
    }

    /// Obtain a handle to extend the existing search query with new elements. Returns `None` if
    /// the existing search query is empty.
    #[inline]
    pub fn extend(self) -> Option<NonEmptySearchQuery<'q>> {
        if self.buffer.is_empty() {
            None
        } else {
            Some(NonEmptySearchQuery {
                buffer: self.buffer,
            })
        }
    }

    /// Extend the existing search query with new elements, using the provided operator to extend
    /// the existing search query if it is non-empty, and otherwise adding the provided.
    /// the existing search query is empty.
    pub fn init_or_extend<E: Into<FieldGroup>>(
        self,
        op: BooleanOp,
        element: E,
    ) -> NonEmptySearchQuery<'q> {
        if self.buffer.is_empty() {
            let _ = write!(self.buffer, "{}", element.into());
            NonEmptySearchQuery {
                buffer: self.buffer,
            }
        } else {
            let new = NonEmptySearchQuery {
                buffer: self.buffer,
            };
            new.push(op, element.into())
        }
    }

    /// Clear the search query.
    pub fn clear(self) -> SearchQuery<'q> {
        self.buffer.clear();
        self
    }
}

/// A handle to extend an existing search query with new elements.
pub struct NonEmptySearchQuery<'q> {
    pub(super) buffer: &'q mut String,
}

impl Display for NonEmptySearchQuery<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.buffer.fmt(f)
    }
}

impl<S: AsRef<str>> Combine<Field<S>> for NonEmptySearchQuery<'_> {
    fn push(mut self, op: BooleanOp, element: Field<S>) -> Self {
        let _ = write!(&mut self.buffer, "{op}{element}");
        self
    }
}

impl Combine<FieldGroup> for NonEmptySearchQuery<'_> {
    fn push(mut self, op: BooleanOp, element: FieldGroup) -> Self {
        let _ = write!(&mut self.buffer, "{op}{element}");
        self
    }
}

impl Combine<Range<NaiveDateTime>> for NonEmptySearchQuery<'_> {
    fn push(mut self, op: BooleanOp, element: Range<NaiveDateTime>) -> Self {
        let _ = write!(
            &mut self.buffer,
            "{}submittedDate:[{} TO {}]",
            op,
            element.start.format("%Y%m%d%H%M"),
            element.end.format("%Y%m%d%H%M")
        );
        self
    }
}
