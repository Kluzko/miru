use async_trait::async_trait;
use std::sync::Arc;

use crate::modules::anime::application::ports::{AnimeRepository, EventPublisher};
use crate::modules::anime::domain::aggregates::anime_aggregate::AnimeAggregate;
use crate::shared::{application::use_case::UseCase, errors::AppResult};

use super::{command::CreateAnimeCommand, result::CreateAnimeResult};

/// Use case handler for creating a new anime
pub struct CreateAnimeHandler {
    anime_repository: Arc<dyn AnimeRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl CreateAnimeHandler {
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
impl UseCase<CreateAnimeCommand, CreateAnimeResult> for CreateAnimeHandler {
    async fn execute(&self, command: CreateAnimeCommand) -> AppResult<CreateAnimeResult> {
        // Check if anime already exists
        if let Some(existing) = self
            .anime_repository
            .find_by_external_id(command.provider, &command.external_id)
            .await?
        {
            return Ok(CreateAnimeResult::new(
                existing.id,
                existing.title.main,
                false,
            ));
        }

        // Create new aggregate
        let aggregate =
            AnimeAggregate::create(command.provider, command.external_id, command.title.clone());

        // Persist aggregate
        self.anime_repository.save(&aggregate).await?;

        // Publish domain events (consume and take ownership)
        let (entity, events) = aggregate.into_parts();
        self.event_publisher.publish_all(events).await?;

        let anime_id = entity.id;
        let title = entity.title.main;

        Ok(CreateAnimeResult::new(anime_id, title, true))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::domain::value_objects::AnimeProvider;

    // Mock implementations would go here for unit testing
    // For now, integration tests will cover this
}
