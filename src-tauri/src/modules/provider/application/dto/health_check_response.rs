use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;

use crate::modules::provider::AnimeProvider;

/// Response DTO for provider health check
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct HealthCheckResponse {
    /// Overall system health status
    pub system_healthy: bool,
    /// Human-readable system status message
    pub system_message: String,
    /// Health status for each provider
    pub provider_statuses: HashMap<AnimeProvider, ProviderStatus>,
    /// Currently recommended provider for new requests
    pub recommended_provider: Option<AnimeProvider>,
}

/// Individual provider status
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ProviderStatus {
    pub provider: AnimeProvider,
    pub is_healthy: bool,
    pub is_available: bool,
    pub success_rate: f32,
    pub average_response_time_ms: u32,
    pub total_requests: u32,
    pub priority_score: f32,
    pub status_message: String,
}

impl HealthCheckResponse {
    /// Get only healthy providers
    pub fn healthy_providers(&self) -> Vec<AnimeProvider> {
        self.provider_statuses
            .iter()
            .filter(|(_, status)| status.is_healthy && status.is_available)
            .map(|(provider, _)| *provider)
            .collect()
    }

    /// Get the best performing provider
    pub fn best_performing_provider(&self) -> Option<AnimeProvider> {
        self.provider_statuses
            .iter()
            .filter(|(_, status)| status.is_healthy && status.is_available)
            .max_by(|(_, a), (_, b)| {
                a.priority_score
                    .partial_cmp(&b.priority_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(provider, _)| *provider)
    }

    /// Get overall system performance summary
    pub fn performance_summary(&self) -> String {
        let healthy_count = self.healthy_providers().len();
        let total_count = self.provider_statuses.len();

        if healthy_count == 0 {
            "All providers are down or unhealthy".to_string()
        } else if healthy_count == total_count {
            "All providers are healthy and performing well".to_string()
        } else {
            format!(
                "{}/{} providers are healthy and available",
                healthy_count, total_count
            )
        }
    }

    /// Check if system can handle requests
    pub fn can_serve_requests(&self) -> bool {
        self.system_healthy && !self.healthy_providers().is_empty()
    }
}

impl ProviderStatus {
    /// Get performance rating
    pub fn performance_rating(&self) -> &'static str {
        if !self.is_healthy || !self.is_available {
            return "Poor";
        }

        match (self.success_rate, self.average_response_time_ms) {
            (rate, time) if rate >= 0.95 && time <= 1000 => "Excellent",
            (rate, time) if rate >= 0.90 && time <= 2000 => "Good",
            (rate, time) if rate >= 0.80 && time <= 3000 => "Fair",
            _ => "Poor",
        }
    }

    /// Check if provider needs attention
    pub fn needs_attention(&self) -> bool {
        !self.is_healthy
            || self.success_rate < 0.8
            || self.average_response_time_ms > 5000
            || self.priority_score < 0.5
    }
}
