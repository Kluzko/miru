use crate::shared::application::pagination::PaginationParams;

/// Query for searching anime
#[derive(Debug, Clone)]
pub struct SearchAnimeQuery {
    pub search_term: String,
    pub pagination: PaginationParams,
}

impl SearchAnimeQuery {
    pub fn new(search_term: String, pagination: PaginationParams) -> Self {
        Self {
            search_term,
            pagination,
        }
    }
}
