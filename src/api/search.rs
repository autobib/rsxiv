use std::{fmt::Display, ops::Range};

use chrono::naive::NaiveDateTime;

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

/// An internal enum used to represent the operator as described in the [API reference][ref]; and
/// is the boolean operators `AND`, `OR`, and `ANDNOT`.
///
/// [ref]: https://info.arxiv.org/help/api/user-manual.html#51-details-of-query-construction
#[derive(Debug, Clone)]
enum FieldOperator {
    And,
    Or,
    AndNot,
}

impl Display for FieldOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            FieldOperator::And => "AND",
            FieldOperator::Or => "OR",
            FieldOperator::AndNot => "ANDNOT",
        };
        write!(f, " {s} ")
    }
}

/// A group of search fields joined with brackets and combined with boolean operators.
#[derive(Debug, Clone)]
pub struct Group<D> {
    first: Field<D>,
    rest: Vec<(FieldOperator, Field<D>)>,
}

impl<D: Display> Group<D> {
    /// Construct a non-empty search field group with the given initial search field.
    #[must_use]
    pub fn new(first: Field<D>) -> Self {
        Self {
            first,
            rest: Vec::default(),
        }
    }

    /// Add a new search field to the group using the specified operator.
    #[must_use]
    fn push(self, op: FieldOperator, new: Field<D>) -> Self {
        let Self { first, mut rest } = self;
        rest.push((op, new));
        Self { first, rest }
    }

    /// Add a new search field to the group, combined with existing fields using the `AND` operator.
    #[must_use]
    pub fn and(self, new: Field<D>) -> Self {
        self.push(FieldOperator::And, new)
    }

    /// Add a new search field to the group, combined with existing fields using the `ANDNOT` operator.
    #[must_use]
    pub fn and_not(self, new: Field<D>) -> Self {
        self.push(FieldOperator::AndNot, new)
    }

    /// Add a new search field to the group, combined with existing fields using the `OR` operator.
    #[must_use]
    pub fn or(self, new: Field<D>) -> Self {
        self.push(FieldOperator::Or, new)
    }
}

impl<D: Display> Display for Group<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.rest.is_empty() {
            self.first.fmt(f)
        } else {
            f.write_str("(")?;
            self.first.fmt(f)?;
            for (op, field) in &self.rest {
                write!(f, "{op}{field}")?;
            }
            f.write_str(")")
        }
    }
}

impl<D: Display> From<Field<D>> for Group<D> {
    fn from(first: Field<D>) -> Self {
        Self::new(first)
    }
}

/// Construct
#[derive(Debug, Clone)]
pub struct GroupList<D> {
    first: Group<D>,
    rest: Vec<(FieldOperator, Group<D>)>,
}

impl<D: Display> GroupList<D> {
    /// Construct the search query string.
    #[must_use]
    pub fn new<T: Into<Group<D>>>(initial_query: T) -> Self {
        Self {
            first: initial_query.into(),
            rest: Vec::default(),
        }
    }

    /// Add a new search field to the group using the specified operator.
    #[must_use]
    fn push<T: Into<Group<D>>>(self, op: FieldOperator, new: T) -> Self {
        let Self { first, mut rest } = self;
        rest.push((op, new.into()));
        Self { first, rest }
    }

    /// Add a new [`Field`] or [`Group`] to the query, combined with existing parameters using the `AND` operator.
    ///
    /// Accepts [`Field`]s and [`Group`]s.
    #[must_use]
    pub fn and<T: Into<Group<D>>>(self, new: T) -> Self {
        self.push(FieldOperator::And, new)
    }

    /// Add a new [`Field`] or [`Group`] to the query, combined with existing parameters using the `ANDNOT` operator.
    ///
    /// Accepts [`Field`]s and [`Group`]s.
    #[must_use]
    pub fn and_not<T: Into<Group<D>>>(self, new: T) -> Self {
        self.push(FieldOperator::AndNot, new)
    }

    /// Add a new [`Field`] or [`Group`] to the query, combined with the `OR` operator.
    #[must_use]
    pub fn or<T: Into<Group<D>>>(self, new: T) -> Self {
        self.push(FieldOperator::Or, new)
    }
}

impl<D: Display> Display for GroupList<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.first.fmt(f)?;
        for (op, group) in &self.rest {
            op.fmt(f)?;
            group.fmt(f)?;
        }
        Ok(())
    }
}
