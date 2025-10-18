/// Domain events for the Anime aggregate
///
/// These events represent business-meaningful state changes that have occurred.
/// They can be used for:
/// - Event sourcing
/// - Publishing to message queues
/// - Triggering side effects (e.g., cache invalidation)
/// - Auditing
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Base trait for all domain events
pub trait DomainEvent: Send + Sync {
    /// When the event occurred
    fn occurred_at(&self) -> DateTime<Utc>;

    /// Unique identifier for this event
    fn event_id(&self) -> Uuid;

    /// Type of event (for serialization/routing)
    fn event_type(&self) -> &'static str;
}

/// Anime was created in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimeCreatedEvent {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub anime_id: Uuid,
    pub title: String,
    pub provider: String,
    pub external_id: String,
}

impl AnimeCreatedEvent {
    pub fn new(anime_id: Uuid, title: String, provider: String, external_id: String) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            anime_id,
            title,
            provider,
            external_id,
        }
    }
}

impl DomainEvent for AnimeCreatedEvent {
    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }

    fn event_id(&self) -> Uuid {
        self.event_id
    }

    fn event_type(&self) -> &'static str {
        "AnimeCreated"
    }
}

/// Anime score was updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimeScoreUpdatedEvent {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub anime_id: Uuid,
    pub old_score: Option<f32>,
    pub new_score: f32,
}

impl AnimeScoreUpdatedEvent {
    pub fn new(anime_id: Uuid, old_score: Option<f32>, new_score: f32) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            anime_id,
            old_score,
            new_score,
        }
    }
}

impl DomainEvent for AnimeScoreUpdatedEvent {
    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }

    fn event_id(&self) -> Uuid {
        self.event_id
    }

    fn event_type(&self) -> &'static str {
        "AnimeScoreUpdated"
    }
}

/// Relations were discovered for an anime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationsDiscoveredEvent {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub anime_id: Uuid,
    pub relations_count: usize,
    pub source: String, // "AniList", "Jikan", etc.
}

impl RelationsDiscoveredEvent {
    pub fn new(anime_id: Uuid, relations_count: usize, source: String) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            anime_id,
            relations_count,
            source,
        }
    }
}

impl DomainEvent for RelationsDiscoveredEvent {
    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }

    fn event_id(&self) -> Uuid {
        self.event_id
    }

    fn event_type(&self) -> &'static str {
        "RelationsDiscovered"
    }
}

/// Anime was enriched with additional data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimeEnrichedEvent {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub anime_id: Uuid,
    pub enrichment_source: String,
    pub fields_updated: Vec<String>,
}

impl AnimeEnrichedEvent {
    pub fn new(anime_id: Uuid, enrichment_source: String, fields_updated: Vec<String>) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            anime_id,
            enrichment_source,
            fields_updated,
        }
    }
}

impl DomainEvent for AnimeEnrichedEvent {
    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }

    fn event_id(&self) -> Uuid {
        self.event_id
    }

    fn event_type(&self) -> &'static str {
        "AnimeEnriched"
    }
}
