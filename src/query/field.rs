use std::{
    fmt::{Display, Write as _},
    num::NonZero,
    ops::Range,
};

use chrono::NaiveDateTime;

/// A non-empty search query which can be extended with new components.
pub trait Combine<E>: Display + Sized {
    /// Extend the query using the given boolean operation.
    fn push(self, op: BooleanOp, element: E) -> Self;

    /// Extend the query using an iterator of boolean operations. Equivalent to calling `push` for
    /// each `(op, element)` pair in the iterator.
    fn extend<T: IntoIterator<Item = (BooleanOp, E)>>(self, elements: T) -> Self {
        elements
            .into_iter()
            .fold(self, |acc, (op, element)| acc.push(op, element))
    }

    /// Extend the query using [`BooleanOp::And`].
    fn and(self, element: E) -> Self {
        self.push(BooleanOp::And, element)
    }

    /// Extend the query using [`BooleanOp::Or`].
    fn or(self, element: E) -> Self {
        self.push(BooleanOp::Or, element)
    }

    /// Extend the query using [`BooleanOp::AndNot`].
    fn and_not(self, element: E) -> Self {
        self.push(BooleanOp::AndNot, element)
    }
}

/// A boolean operator used to combine elements in the search query.
///
/// Used in conjuction with the [`Combine`] trait to build [`FieldGroup`]s or extend
/// [`SearchQuery`](super::SearchQuery)s.
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

#[derive(Debug, Clone, Copy)]
pub enum FieldType {
    /// Title
    Title,
    /// Author
    Author,
    /// Abstract
    Abstract,
    /// Comment
    Comment,
    /// Journal Reference
    JournalReference,
    /// Subject Category
    SubjectCategory,
    /// Report Number
    ReportNumber,
    /// All of the above
    All,
}

/// The possible search field types as enumerated in the [API reference][ref].
///
/// [ref]: https://info.arxiv.org/help/api/user-manual.html#51-details-of-query-construction
impl FieldType {
    pub fn as_prefix(&self) -> &'static str {
        match self {
            Self::Title => "ti",
            Self::Author => "au",
            Self::Abstract => "abs",
            Self::Comment => "co",
            Self::JournalReference => "jr",
            Self::SubjectCategory => "cat",
            Self::ReportNumber => "rn",
            Self::All => "all",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Field<S> {
    field_type: FieldType,
    value: S,
}

impl<S: AsRef<str>> Display for Field<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.field_type.as_prefix())?;
        f.write_str(":")?;
        f.write_str(self.value.as_ref())
    }
}

macro_rules! field_impl {
    ($fname:ident, $target:ident) => {
        /// A convenience function to call [`Field::init`] with
        #[doc = concat!("[`FieldType::", stringify!($target), "`]")]
        pub fn $fname(value: S) -> Option<Self> {
            Self::init(FieldType::$target, value)
        }
    };
}

impl<S: AsRef<str>> Field<S> {
    fn check_value(value: &str) -> Option<()> {
        if value.contains(" AND ")
            || value.contains(" OR ")
            || value.contains(" ANDNOT ")
            || value.contains(')')
            || value.contains('(')
            || value.contains(':')
        {
            None
        } else {
            Some(())
        }
    }

    /// Initialize a new field of the given type.
    ///
    /// Returns `None` if the field contents are invalid, which is the case if it contains any of
    /// the following substrings:
    /// ```txt
    /// [" AND ", " OR ", " ANDNOT ", "(", ")"]
    /// ```
    pub fn init(field_type: FieldType, value: S) -> Option<Self> {
        Self::check_value(value.as_ref())?;
        Some(Self { field_type, value })
    }

    field_impl!(ti, Title);
    field_impl!(au, Author);
    field_impl!(abs, Abstract);
    field_impl!(co, Comment);
    field_impl!(jr, JournalReference);
    field_impl!(cat, SubjectCategory);
    field_impl!(rn, ReportNumber);
    field_impl!(all, All);
}

/// An ordered collection of [`Field`]s, grouped together using brackets if necessary.
///
/// ### Example
/// ```
/// use rsxiv::query::{Combine, Field, FieldGroup};
///
/// let group = FieldGroup::init(Field::all("a").unwrap())
///     .and(Field::au("John").unwrap())
///     .or(Field::au("Doe").unwrap());
/// assert_eq!(group.to_string(), "(all:a AND au:John OR au:Doe)");
///
/// let group = FieldGroup::init(Field::ti("title").unwrap());
/// assert_eq!(group.to_string(), "ti:title");
/// ```
pub struct FieldGroup {
    inner: String,
    num_fields: NonZero<usize>,
}

impl FieldGroup {
    pub fn init<S: AsRef<str>>(initial: Field<S>) -> Self {
        let mut inner = String::new();
        let _ = write!(&mut inner, "{initial}");
        Self {
            inner,
            num_fields: NonZero::new(1).unwrap(),
        }
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

impl<S: AsRef<str>> Combine<Field<S>> for FieldGroup {
    fn push(mut self, op: BooleanOp, element: Field<S>) -> Self {
        let _ = write!(&mut self.inner, "{op}{element}");
        self.num_fields = self.num_fields.saturating_add(1);
        self
    }
}

impl Combine<Range<NaiveDateTime>> for FieldGroup {
    fn push(mut self, op: BooleanOp, element: Range<NaiveDateTime>) -> Self {
        let _ = write!(
            &mut self.inner,
            "{}submittedDate:[{} TO {}]",
            op,
            element.start.format("%Y%m%d%H%M"),
            element.end.format("%Y%m%d%H%M")
        );
        self
    }
}

impl<S: AsRef<str>> From<Field<S>> for FieldGroup {
    fn from(field: Field<S>) -> Self {
        let mut inner = String::new();
        let _ = write!(&mut inner, "{field}");
        Self {
            inner,
            num_fields: NonZero::new(1).unwrap(),
        }
    }
}
