use super::relations::AnimeRelation;
/// Anime Aggregate Root
///
/// This is the new DDD-compliant aggregate root that will eventually replace AnimeDetailed.
/// For now, we're creating the structure and will migrate gradually.
use crate::modules::anime::domain::{
    entities::anime_detailed::AnimeDetailed,
    events::{AnimeCreatedEvent, AnimeScoreUpdatedEvent, DomainEvent, RelationsDiscoveredEvent},
    value_objects::{AnimeTier, AnimeTitle},
};
use crate::shared::domain::value_objects::{AnimeProvider, ProviderMetadata};
use uuid::Uuid;

/// Anime Aggregate Root
///
/// Encapsulates all business logic and invariants for an anime entity.
/// Publishes domain events for state changes.
pub struct AnimeAggregate {
    /// The underlying entity (for gradual migration)
    entity: AnimeDetailed,

    /// Domain events that occurred during this session
    /// These should be published after persistence
    pending_events: Vec<Box<dyn DomainEvent>>,
}

impl AnimeAggregate {
    /// Create a new anime aggregate
    pub fn create(provider: AnimeProvider, external_id: String, title: String) -> Self {
        let entity = AnimeDetailed::new(provider, external_id.clone(), title.clone());

        let event =
            AnimeCreatedEvent::new(entity.id, title, format!("{:?}", provider), external_id);

        Self {
            entity,
            pending_events: vec![Box::new(event)],
        }
    }

    /// Load existing anime from entity (when reading from database)
    pub fn from_entity(entity: AnimeDetailed) -> Self {
        Self {
            entity,
            pending_events: Vec::new(),
        }
    }

    /// Get the underlying entity (for persistence)
    ///
    /// Note: This is temporary for backward compatibility.
    /// Eventually persistence will work directly with the aggregate.
    pub fn entity(&self) -> &AnimeDetailed {
        &self.entity
    }

    /// Get mutable reference to entity (for migration)
    pub fn entity_mut(&mut self) -> &mut AnimeDetailed {
        &mut self.entity
    }

    /// Consume aggregate and return entity + events
    pub fn into_parts(self) -> (AnimeDetailed, Vec<Box<dyn DomainEvent>>) {
        (self.entity, self.pending_events)
    }

    // ============================================================================================
    // BUSINESS OPERATIONS (Commands)
    // ============================================================================================

    /// Update score with business validation
    pub fn update_score(&mut self, new_score: f32) -> Result<(), String> {
        let old_score = self.entity.score;

        // Use the entity's validation
        self.entity.update_score(new_score)?;

        // Publish domain event
        let event = AnimeScoreUpdatedEvent::new(self.entity.id, old_score, new_score);
        self.pending_events.push(Box::new(event));

        Ok(())
    }

    /// Update title with validation
    pub fn update_title(&mut self, title: AnimeTitle) -> Result<(), String> {
        self.entity.update_title(title)?;
        // Could publish AnimeUpdatedEvent here
        Ok(())
    }

    /// Discover and add relations for this anime
    pub fn discover_relations(&mut self, relations: Vec<AnimeRelation>, source: String) {
        // Publish domain event
        let event = RelationsDiscoveredEvent::new(self.entity.id, relations.len(), source);
        self.pending_events.push(Box::new(event));

        // Note: The relations themselves are persisted separately via AnimeRelationsRepository
        // This aggregate just publishes the event that relations were discovered
    }

    // ============================================================================================
    // QUERIES (Read-only)
    // ============================================================================================

    /// Get anime ID
    pub fn id(&self) -> Uuid {
        self.entity.id
    }

    /// Get anime title
    pub fn title(&self) -> &AnimeTitle {
        &self.entity.title
    }

    /// Get score
    pub fn score(&self) -> Option<f32> {
        self.entity.score
    }

    /// Get tier
    pub fn tier(&self) -> AnimeTier {
        self.entity.tier
    }

    /// Get provider metadata
    pub fn provider_metadata(&self) -> &ProviderMetadata {
        &self.entity.provider_metadata
    }

    // ============================================================================================
    // EVENT HANDLING
    // ============================================================================================

    /// Get pending domain events (to be published)
    pub fn pending_events(&self) -> &[Box<dyn DomainEvent>] {
        &self.pending_events
    }

    /// Clear pending events (after they've been published)
    pub fn clear_events(&mut self) {
        self.pending_events.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_anime_aggregate() {
        let aggregate = AnimeAggregate::create(
            AnimeProvider::AniList,
            "12345".to_string(),
            "Death Note".to_string(),
        );

        assert_eq!(aggregate.title().main, "Death Note");
        assert_eq!(aggregate.pending_events().len(), 1);
        assert_eq!(aggregate.pending_events()[0].event_type(), "AnimeCreated");
    }

    #[test]
    fn test_update_score_publishes_event() {
        let mut aggregate = AnimeAggregate::create(
            AnimeProvider::AniList,
            "12345".to_string(),
            "Death Note".to_string(),
        );

        aggregate.clear_events(); // Clear creation event

        aggregate.update_score(8.5).unwrap();

        assert_eq!(aggregate.score(), Some(8.5));
        assert_eq!(aggregate.pending_events().len(), 1);
        assert_eq!(
            aggregate.pending_events()[0].event_type(),
            "AnimeScoreUpdated"
        );
    }

    #[test]
    fn test_invalid_score_rejected() {
        let mut aggregate = AnimeAggregate::create(
            AnimeProvider::AniList,
            "12345".to_string(),
            "Death Note".to_string(),
        );

        let result = aggregate.update_score(11.0);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("0-10"));
    }
}
