use uuid::Uuid;

/// Command for discovering relations for an anime
#[derive(Debug, Clone)]
pub struct DiscoverRelationsCommand {
    pub anime_id: Uuid,
    pub source: String, // e.g., "jikan", "anilist"
}

impl DiscoverRelationsCommand {
    pub fn new(anime_id: Uuid, source: String) -> Self {
        Self { anime_id, source }
    }
}
