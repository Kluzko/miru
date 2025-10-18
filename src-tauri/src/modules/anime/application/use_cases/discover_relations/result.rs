use uuid::Uuid;

/// Result of discovering relations for an anime
#[derive(Debug, Clone)]
pub struct DiscoverRelationsResult {
    pub anime_id: Uuid,
    pub relations_count: usize,
    pub source: String,
}

impl DiscoverRelationsResult {
    pub fn new(anime_id: Uuid, relations_count: usize, source: String) -> Self {
        Self {
            anime_id,
            relations_count,
            source,
        }
    }
}
