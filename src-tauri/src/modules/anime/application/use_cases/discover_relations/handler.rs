use async_trait::async_trait;
use std::sync::Arc;

use crate::modules::anime::application::ports::{
    AnimeRelationsRepository, AnimeRepository, EventPublisher, ProviderClient,
};
use crate::modules::anime::domain::aggregates::anime_aggregate::AnimeAggregate;
use crate::shared::{
    application::use_case::UseCase,
    errors::{AppError, AppResult},
};

use super::{command::DiscoverRelationsCommand, result::DiscoverRelationsResult};

/// Use case handler for discovering anime relations
pub struct DiscoverRelationsHandler {
    anime_repository: Arc<dyn AnimeRepository>,
    relations_repository: Arc<dyn AnimeRelationsRepository>,
    provider_clients: Vec<Arc<dyn ProviderClient>>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl DiscoverRelationsHandler {
    pub fn new(
        anime_repository: Arc<dyn AnimeRepository>,
        relations_repository: Arc<dyn AnimeRelationsRepository>,
        provider_clients: Vec<Arc<dyn ProviderClient>>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            anime_repository,
            relations_repository,
            provider_clients,
            event_publisher,
        }
    }
}

#[async_trait]
impl UseCase<DiscoverRelationsCommand, DiscoverRelationsResult> for DiscoverRelationsHandler {
    async fn execute(
        &self,
        command: DiscoverRelationsCommand,
    ) -> AppResult<DiscoverRelationsResult> {
        // Find anime
        let anime = self
            .anime_repository
            .find_by_id(command.anime_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("Anime with id {} not found", command.anime_id))
            })?;

        // Load into aggregate
        let mut aggregate = AnimeAggregate::from_entity(anime.clone());

        // Find the appropriate provider client
        // For now, use the first matching provider from external_ids
        let external_id = anime
            .provider_metadata
            .external_ids
            .iter()
            .next()
            .ok_or_else(|| AppError::ValidationError("No external IDs available".to_string()))?;

        // Find provider client
        let provider_client = self
            .provider_clients
            .iter()
            .find(|client| client.provider() == *external_id.0)
            .ok_or_else(|| {
                AppError::NotFound(format!("Provider client not found for {:?}", external_id.0))
            })?;

        // Fetch relations from provider
        let relations = provider_client.fetch_relations(external_id.1).await?;

        // Discover relations through aggregate (publishes event)
        aggregate.discover_relations(relations.clone(), command.source.clone());

        // Save relations
        self.relations_repository
            .save_relations(command.anime_id, relations.clone())
            .await?;

        // Publish domain events (consume and take ownership)
        let (_, events) = aggregate.into_parts();
        self.event_publisher.publish_all(events).await?;

        Ok(DiscoverRelationsResult::new(
            command.anime_id,
            relations.len(),
            command.source,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock implementations would go here for unit testing
    // For now, integration tests will cover this
}
