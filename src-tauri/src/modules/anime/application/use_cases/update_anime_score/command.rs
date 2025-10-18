use uuid::Uuid;

/// Command for updating an anime's score
#[derive(Debug, Clone)]
pub struct UpdateAnimeScoreCommand {
    pub anime_id: Uuid,
    pub new_score: f32,
}

impl UpdateAnimeScoreCommand {
    pub fn new(anime_id: Uuid, new_score: f32) -> Self {
        Self {
            anime_id,
            new_score,
        }
    }
}
