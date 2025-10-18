use async_trait::async_trait;

use crate::modules::anime::domain::events::DomainEvent;
use crate::shared::errors::AppResult;

/// Port (interface) for publishing domain events
/// Infrastructure layer implements this (in-memory, message queue, event store, etc.)
#[async_trait]
pub trait EventPublisher: Send + Sync {
    /// Publish a single domain event
    async fn publish(&self, event: Box<dyn DomainEvent>) -> AppResult<()>;

    /// Publish multiple domain events
    async fn publish_all(&self, events: Vec<Box<dyn DomainEvent>>) -> AppResult<()>;
}
