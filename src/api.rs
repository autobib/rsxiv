#[cfg(test)]
mod tests;

pub mod search;

use std::fmt::{Display, Write as _};

use url::Url;

use crate::id::Identifier;

/// The ordering by which to sort the query results.
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
#[derive(Debug, Clone, Copy, Default)]
pub enum SortOrder {
    /// Sort in ascending order
    Ascending,
    /// Sort in descending order
    #[default]
    Descending,
}

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

    /// The default base url for the arXiv API query.
    const fn base_url() -> &'static str {
        "https://export.arxiv.org/api/query"
    }

    /// Returns if the query is empty; that is, if there is no `search_query` and no `id_list`
    /// present.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.search_query.is_empty() && self.id_list.is_empty()
    }

    /// Convert the [`Query`] into a [`Url`] to which the arXiv API request can be made.
    #[must_use]
    pub fn url(&self) -> Url {
        let mut url = Url::parse(Self::base_url()).unwrap();

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

    pub fn clear_search_query(&mut self) {
        self.search_query.clear();
    }

    /// Set a search query, replacing the existing query (if any).
    pub fn set_search_query<D: Display>(&mut self, query: D) -> &mut Self {
        self.search_query.clear();
        let _ = write!(self.search_query, "{query}");
        self
    }

    /// Set an identifier list, replacing the existing list (if any).
    pub fn set_id_list<I: Identifier, T: IntoIterator<Item = I>>(&mut self, ids: T) -> &mut Self {
        self.id_list.clear();
        let mut id_iter = ids.into_iter();
        match id_iter.next() {
            Some(first) => {
                let _ = write!(self.id_list, "{first}");
            }
            None => return self,
        }

        for id in id_iter {
            let _ = write!(self.id_list, ",{id}");
        }

        self
    }

    /// Limit the number of results, with pagination starting from `start` and containing
    /// `max_results` results.
    ///
    /// This method returns `None` if `start > 30000` or `max_results > 2000`, in which case
    /// the pagination will not be updated.
    pub fn paginate(&mut self, start: u16, max_results: u16) -> Option<&mut Self> {
        if start <= 30000 && max_results <= 2000 {
            self.pagination = Some((start, max_results));
            Some(self)
        } else {
            None
        }
    }

    /// Sort the API response using a given ordering function and in ascending or descending order.
    pub fn sorted(&mut self, by: SortBy, order: SortOrder) -> &mut Self {
        self.sort = Some((by, order));
        self
    }
}
