use uuid::Uuid;

/// Result of updating an anime's score
#[derive(Debug, Clone)]
pub struct UpdateAnimeScoreResult {
    pub anime_id: Uuid,
    pub old_score: Option<f32>,
    pub new_score: f32,
}

impl UpdateAnimeScoreResult {
    pub fn new(anime_id: Uuid, old_score: Option<f32>, new_score: f32) -> Self {
        Self {
            anime_id,
            old_score,
            new_score,
        }
    }
}
