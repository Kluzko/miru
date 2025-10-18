use crate::shared::domain::value_objects::AnimeProvider;

/// Command for creating a new anime
#[derive(Debug, Clone)]
pub struct CreateAnimeCommand {
    pub provider: AnimeProvider,
    pub external_id: String,
    pub title: String,
}

impl CreateAnimeCommand {
    pub fn new(provider: AnimeProvider, external_id: String, title: String) -> Self {
        Self {
            provider,
            external_id,
            title,
        }
    }
}
