use serde::{Deserialize, Serialize};
use specta::Type;

use crate::modules::provider::AnimeProvider;

/// Request DTO for provider health check
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct HealthCheckRequest {
    /// Optional specific provider to check (if None, checks all providers)
    pub provider: Option<AnimeProvider>,
    /// Whether to include detailed statistics
    pub include_details: Option<bool>,
}

impl HealthCheckRequest {
    pub fn new() -> Self {
        Self {
            provider: None,
            include_details: Some(true),
        }
    }

    pub fn for_provider(provider: AnimeProvider) -> Self {
        Self {
            provider: Some(provider),
            include_details: Some(true),
        }
    }

    pub fn summary_only() -> Self {
        Self {
            provider: None,
            include_details: Some(false),
        }
    }
}

impl Default for HealthCheckRequest {
    fn default() -> Self {
        Self::new()
    }
}
