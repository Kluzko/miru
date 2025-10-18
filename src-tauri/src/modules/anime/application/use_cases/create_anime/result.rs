use uuid::Uuid;

/// Result of creating a new anime
#[derive(Debug, Clone)]
pub struct CreateAnimeResult {
    pub anime_id: Uuid,
    pub title: String,
    pub was_created: bool, // false if anime already existed
}

impl CreateAnimeResult {
    pub fn new(anime_id: Uuid, title: String, was_created: bool) -> Self {
        Self {
            anime_id,
            title,
            was_created,
        }
    }
}
