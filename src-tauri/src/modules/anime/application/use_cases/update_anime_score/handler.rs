use async_trait::async_trait;
use std::sync::Arc;

use crate::modules::anime::application::ports::{AnimeRepository, EventPublisher};
use crate::modules::anime::domain::aggregates::anime_aggregate::AnimeAggregate;
use crate::shared::{
    application::use_case::UseCase,
    errors::{AppError, AppResult},
};

use super::{command::UpdateAnimeScoreCommand, result::UpdateAnimeScoreResult};

/// Use case handler for updating an anime's score
pub struct UpdateAnimeScoreHandler {
    anime_repository: Arc<dyn AnimeRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl UpdateAnimeScoreHandler {
    pub fn new(
        anime_repository: Arc<dyn AnimeRepository>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            anime_repository,
            event_publisher,
        }
    }
}

#[async_trait]
impl UseCase<UpdateAnimeScoreCommand, UpdateAnimeScoreResult> for UpdateAnimeScoreHandler {
    async fn execute(&self, command: UpdateAnimeScoreCommand) -> AppResult<UpdateAnimeScoreResult> {
        // Find anime
        let Some(anime) = self.anime_repository.find_by_id(command.anime_id).await? else {
            return Err(AppError::NotFound(format!(
                "Anime with id {} not found",
                command.anime_id
            )));
        };

        let old_score = anime.score;

        // Load into aggregate
        let mut aggregate = AnimeAggregate::from_entity(anime);

        // Update score (validates business rules)
        aggregate
            .update_score(command.new_score)
            .map_err(|e| AppError::ValidationError(e))?;

        // Persist changes
        self.anime_repository.update(&aggregate).await?;

        // Publish domain events (consume and take ownership)
        let (_, events) = aggregate.into_parts();
        self.event_publisher.publish_all(events).await?;

        Ok(UpdateAnimeScoreResult::new(
            command.anime_id,
            old_score,
            command.new_score,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock implementations would go here for unit testing
    // For now, integration tests will cover this
}
