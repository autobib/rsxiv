//! # ArXiv API query builder
//!
//! This module provides a interface interface to build query URLs for the [arXiv API][api]. The
//! main entry point is the [`Query`] struct.
//!
//! [api]: https://info.arxiv.org/help/api/user-manual.html
mod field;
mod search;
#[cfg(test)]
mod tests;

use std::fmt::Write as _;

use url::Url;

pub use self::{
    field::{BooleanOp, Combine, Field, FieldGroup, FieldType},
    search::{NonEmptySearchQuery, SearchQuery},
};
use crate::id::Identifier;

/// The ordering by which to sort the query results.
///
/// See the [`Query::paginate`] method for more detail.
#[derive(Debug, Clone, Copy, Default)]
pub enum SortBy {
    /// Sort by relevance
    #[default]
    Relevance,
    /// Sory by last updated date
    LastUpdatedDate,
    /// Sory by last submitted date
    SubmittedDate,
}

/// Whether to sort in ascending or descending order.
///
/// See the [`Query::paginate`] method for more detail.
#[derive(Debug, Clone, Copy, Default)]
pub enum SortOrder {
    /// Sort in ascending order
    Ascending,
    /// Sort in descending order
    #[default]
    Descending,
}

/// A handle used to update the identifier list in a query.
///
/// See the [`Query::id_list`] method for more detail.
pub struct IdList<'q> {
    buffer: &'q mut String,
}

impl IdList<'_> {
    /// Add a single identifier to the list.
    pub fn push<I: Identifier>(&mut self, id: &I) -> &mut Self {
        if !self.buffer.is_empty() {
            self.buffer.push(',');
        }
        id.write_identifier(self.buffer);
        self
    }

    /// Add identifiers to the list from an iterator.
    pub fn extend<I: Identifier, T: IntoIterator<Item = I>>(&mut self, ids: T) -> &mut Self {
        let mut id_iter = ids.into_iter();

        // if the id list is empty, write the first identifier without a comma
        if self.buffer.is_empty() {
            match id_iter.next() {
                Some(first) => {
                    first.write_identifier(self.buffer);
                }
                None => return self,
            }
        }

        for id in id_iter {
            id.write_identifier(self.buffer);
        }

        self
    }

    /// Clear the identifier list.
    pub fn clear(&mut self) -> &mut Self {
        self.buffer.clear();
        self
    }
}

/// A validated arXiv API query.
///
/// A [`Query`] is a typed representation of an [arXiv API query][api].
///
/// ### Example
/// Build a [`Query`] from components.
/// ```
/// use rsxiv::query::{Combine, Field, FieldGroup, Query, SortBy, SortOrder};
///
/// let mut query = Query::new();
///
/// query
///     // use http
///     .http()
///     // sort the results
///     .sort(SortBy::SubmittedDate, SortOrder::Ascending)
///     // access handle to the search query
///     .search_query()
///     // require title matching 'proton'
///     .init(Field::ti("Proton").unwrap())
///     // and require author `Bob`, or author `John`
///     .and(FieldGroup::init(Field::au("Bob").unwrap()).or(Field::au("John").unwrap()));
///
/// assert_eq!(
///     String::from(query.url()),
///     "http://export.arxiv.org/api/query?search_query=ti%3AProton+AND+%28au%3ABob+OR+au%3AJohn%29&sortBy=submittedDate&sortOrder=ascending"
/// );
/// ```
/// ### Component escaping
/// The query components are automatically escaped.
/// ```
/// use rsxiv::{query::{Field, Query}, id::Validated};
///
/// let mut query = Query::new();
/// query
///     .search_query()
///     // attempt to 'manually escape'
///     .init(Field::all(r#"""&id_list=2301.00001"#).unwrap());
///
/// assert_eq!(
///     String::from(query.url()),
///     // the '&' and '=' characters are escaped, so the query becomes a literal
///     // `all:""&id_list=2301.00001`
///     "https://export.arxiv.org/api/query?search_query=all%3A%22%22%26id_list%3D2301.00001"
/// );
///
/// let mut query = Query::new();
/// query
///     .search_query()
///     .init(Field::all(r#""""#).unwrap());
///
/// query
///     .id_list()
///     .push(&Validated::parse("2301.00001").unwrap());
///
/// assert_eq!(
///     String::from(query.url()),
///     // setting the `id_list` normally yields a different URL
///     "https://export.arxiv.org/api/query?search_query=all%3A%22%22&id_list=2301.00001"
/// );
/// ```
///
/// [api]: https://info.arxiv.org/help/api/user-manual.html#51-details-of-query-construction
#[derive(Debug, Default, Clone)]
pub struct Query {
    search_query: String,
    id_list: String,
    pagination: Option<(u16, u16)>,
    sort: Option<(SortBy, SortOrder)>,
    http: bool,
}

impl Query {
    /// Construct a new arXiv API query.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns if the query corresponds to no results; namely, that the `search_query` and `id_list` are not present.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.search_query.is_empty() && self.id_list.is_empty()
    }

    /// Returns a [`Url`] representing the arXiv API request.
    #[must_use]
    pub fn url(&self) -> Url {
        let mut url = Url::parse("https://export.arxiv.org/api/query").unwrap();

        // set scheme
        if self.http {
            let _ = url.set_scheme("http");
        }

        let mut query_pairs = url.query_pairs_mut();

        // set search queries
        if !self.search_query.is_empty() {
            query_pairs.append_pair("search_query", &self.search_query);
        }

        // set id_list
        if !self.id_list.is_empty() {
            query_pairs.append_pair("id_list", &self.id_list);
        }

        // set pagination
        if let Some((start, max_results)) = self.pagination {
            let mut scratch: String = String::with_capacity(5);
            let _ = write!(&mut scratch, "{start}");
            query_pairs.append_pair("start", &scratch);

            scratch.clear();
            let _ = write!(&mut scratch, "{max_results}");
            query_pairs.append_pair("max_results", &scratch);
        }

        // set sort params
        if let Some((sort_by, sort_order)) = self.sort {
            let s = match sort_by {
                SortBy::Relevance => "relevance",
                SortBy::LastUpdatedDate => "lastUpdatedDate",
                SortBy::SubmittedDate => "submittedDate",
            };
            query_pairs.append_pair("sortBy", s);

            let s = match sort_order {
                SortOrder::Ascending => "ascending",
                SortOrder::Descending => "descending",
            };
            query_pairs.append_pair("sortOrder", s);
        }

        drop(query_pairs);

        url
    }

    /// Use `http://` protocol.
    pub fn http(&mut self) -> &mut Self {
        self.http = true;
        self
    }

    /// Use `https://` protocol (default).
    pub fn https(&mut self) -> &mut Self {
        self.http = false;
        self
    }

    /// Returns a handle to modify the search parameters.
    ///
    /// See the [`SearchQuery`] documentation for examples and more detail.
    pub fn search_query(&mut self) -> SearchQuery<'_> {
        SearchQuery {
            buffer: &mut self.search_query,
        }
    }

    /// Returns a handle to modify the identifier list.
    pub fn id_list(&mut self) -> IdList<'_> {
        IdList {
            buffer: &mut self.id_list,
        }
    }

    /// Limit the number of results, with pagination starting from `start` and containing
    /// `max_results` results.
    ///
    /// This method returns `None` if `start > 30000` or `max_results > 2000`, in which case
    /// the pagination will not be updated.
    ///
    /// Corresponds to [§3.1.1.2 of the API manual][api].
    ///
    /// [api]: https://info.arxiv.org/help/api/user-manual.html#3112-start-and-max_results-paging
    pub fn paginate(&mut self, start: u16, max_results: u16) -> Option<&mut Self> {
        if start <= 30000 && max_results <= 2000 {
            self.pagination = Some((start, max_results));
            Some(self)
        } else {
            None
        }
    }

    /// Sort the API response using the ordering function, in ascending or descending order.
    ///
    /// Corresponds to [§3.1.1.3 of the API manual][api].
    ///
    /// [api]: https://info.arxiv.org/help/api/user-manual.html#3113-sort-order-for-return-results
    pub fn sort(&mut self, by: SortBy, order: SortOrder) -> &mut Self {
        self.sort = Some((by, order));
        self
    }
}
