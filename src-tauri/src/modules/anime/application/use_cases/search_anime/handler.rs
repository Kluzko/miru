use async_trait::async_trait;
use std::sync::Arc;

use crate::modules::anime::application::ports::AnimeRepository;
use crate::shared::{application::use_case::Query, errors::AppResult};

use super::{query::SearchAnimeQuery, result::SearchAnimeResult};

/// Query handler for searching anime
pub struct SearchAnimeHandler {
    anime_repository: Arc<dyn AnimeRepository>,
}

impl SearchAnimeHandler {
    pub fn new(anime_repository: Arc<dyn AnimeRepository>) -> Self {
        Self { anime_repository }
    }
}

#[async_trait]
impl Query<SearchAnimeQuery, SearchAnimeResult> for SearchAnimeHandler {
    async fn execute(&self, query: SearchAnimeQuery) -> AppResult<SearchAnimeResult> {
        // Delegate to repository
        self.anime_repository
            .search(&query.search_term, query.pagination)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock implementations would go here for unit testing
    // For now, integration tests will cover this
}
