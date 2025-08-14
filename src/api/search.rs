use std::{
    fmt::{Display, Write as _},
    num::NonZero,
    ops::Range,
};

use chrono::naive::NaiveDateTime;

/// A boolean operator used to combine elements in the search query.
#[derive(Debug, Clone)]
pub enum BooleanOp {
    /// The `AND` operator.
    And,
    /// The `OR` operator.
    Or,
    /// The `ANDNOT` operator.
    AndNot,
}

impl Display for BooleanOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            BooleanOp::And => "AND",
            BooleanOp::Or => "OR",
            BooleanOp::AndNot => "ANDNOT",
        };
        write!(f, " {s} ")
    }
}

/// A [`Search`] implementation is a non-empty search query which can be extended with
/// new components.
pub trait Search<E>: Display {
    /// Extend the query using the given boolean operation.
    fn push(&mut self, op: BooleanOp, element: E) -> &mut Self;

    /// Extend the query using an iterator of boolean operations. Equivalent to calling `push` for
    /// each `(op, element)` pair in the iterator.
    fn extend<T: IntoIterator<Item = (BooleanOp, E)>>(&mut self, elements: T) -> &mut Self {
        for (op, element) in elements {
            self.push(op, element);
        }
        self
    }

    /// Extend the query using [`BooleanOp::And`].
    fn and(&mut self, element: E) -> &mut Self {
        self.push(BooleanOp::And, element);
        self
    }

    /// Extend the query using [`BooleanOp::Or`].
    fn or(&mut self, element: E) -> &mut Self {
        self.push(BooleanOp::Or, element);
        self
    }

    /// Extend the query using [`BooleanOp::AndNot`].
    fn and_not(&mut self, element: E) -> &mut Self {
        self.push(BooleanOp::AndNot, element);
        self
    }
}

/// The possible search fields as enumerated in the [API reference][ref].
///
/// [ref]: https://info.arxiv.org/help/api/user-manual.html#51-details-of-query-construction
#[derive(Debug, Clone)]
pub enum Field<D> {
    /// Title
    Title(D),
    /// Author
    Author(D),
    /// Abstract
    Abstract(D),
    /// Comment
    Comment(D),
    /// Journal Reference
    JournalReference(D),
    /// Subject Category
    SubjectCategory(D),
    /// Report Number
    ReportNumber(D),
    /// All of the above
    All(D),
    /// Submitted Date Range
    SubmittedDate(Range<NaiveDateTime>),
}

impl<D: Display> Field<D> {
    fn prefix(&self) -> &'static str {
        match self {
            Field::Title(_) => "ti",
            Field::Author(_) => "au",
            Field::Abstract(_) => "abs",
            Field::Comment(_) => "co",
            Field::JournalReference(_) => "jr",
            Field::SubjectCategory(_) => "cat",
            Field::ReportNumber(_) => "rn",
            Field::All(_) => "all",
            Field::SubmittedDate(_) => "submittedDate",
        }
    }
}

impl<D: Display> Display for Field<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", self.prefix())?;
        match self {
            Field::Title(s)
            | Field::Author(s)
            | Field::Abstract(s)
            | Field::Comment(s)
            | Field::JournalReference(s)
            | Field::SubjectCategory(s)
            | Field::ReportNumber(s)
            | Field::All(s) => {
                write!(f, "{s}")
            }
            Field::SubmittedDate(date_time_range) => {
                write!(
                    f,
                    "[{} TO {}]",
                    date_time_range.start.format("%Y%m%d%H%M"),
                    date_time_range.end.format("%Y%m%d%H%M")
                )
            }
        }
    }
}

pub struct FieldGroup {
    inner: String,
    num_fields: NonZero<usize>,
}

impl<D: Display> Search<Field<D>> for FieldGroup {
    fn push(&mut self, op: BooleanOp, element: Field<D>) -> &mut Self {
        let _ = write!(self.inner, "{op}{element}");
        self.num_fields = self.num_fields.saturating_add(1);
        self
    }
}

impl<D: Display> From<Field<D>> for FieldGroup {
    fn from(field: Field<D>) -> Self {
        let mut inner = String::new();
        let _ = write!(&mut inner, "{field}");
        Self {
            inner,
            num_fields: NonZero::new(1).unwrap(),
        }
    }
}

/// A handle to edit an existing search query.
///
/// This struct is construted by the [`Query::search_query`](super::Query::search_query) method; see its documentation for more
/// detail.
pub struct SearchQuery<'q> {
    pub(super) buffer: &'q mut String,
}

impl<'q> SearchQuery<'q> {
    /// Initialize the query with a [`Field`], [`FieldGroup`], or any other type which can be
    /// converted into a [`FieldGroup`].
    ///
    /// This method deletes the existing query string.
    pub fn init<E: Into<FieldGroup>>(self, initial: E) -> NonEmptySearchQuery<'q> {
        self.buffer.clear();
        let _ = write!(self.buffer, "{}", initial.into());
        NonEmptySearchQuery {
            buffer: self.buffer,
        }
    }

    /// Obtain a handle to extend the existing search query with new elements. Returns `None` if
    /// the existing search query is empty.
    pub fn extend(self) -> Option<NonEmptySearchQuery<'q>> {
        if self.buffer.is_empty() {
            None
        } else {
            Some(NonEmptySearchQuery {
                buffer: self.buffer,
            })
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

impl<'q> Display for NonEmptySearchQuery<'q> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.buffer.fmt(f)
    }
}

impl<'q, D: Display> Search<Field<D>> for NonEmptySearchQuery<'q> {
    fn push(&mut self, op: BooleanOp, element: Field<D>) -> &mut Self {
        let _ = write!(self.buffer, "{op}{element}");
        self
    }
}

impl<'q> Search<FieldGroup> for NonEmptySearchQuery<'q> {
    fn push(&mut self, op: BooleanOp, element: FieldGroup) -> &mut Self {
        let _ = write!(self.buffer, "{op}{element}");
        self
    }
}

impl Display for FieldGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.num_fields == NonZero::<usize>::MIN {
            f.write_str(&self.inner)
        } else {
            f.write_str("(")?;
            f.write_str(&self.inner)?;
            f.write_str(")")
        }
    }
}
