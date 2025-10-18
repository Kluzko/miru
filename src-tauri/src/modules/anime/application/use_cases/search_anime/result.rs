use crate::modules::anime::domain::entities::anime_detailed::AnimeDetailed;
use crate::shared::application::pagination::PaginatedResult;

/// Result of searching anime (uses PaginatedResult from shared)
pub type SearchAnimeResult = PaginatedResult<AnimeDetailed>;
