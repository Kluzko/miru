use std::collections::HashMap;

use crate::modules::provider::{
    domain::{entities::ProviderConfig, value_objects::ProviderHealth},
    AnimeProvider,
};

/// Service for intelligent provider selection
pub struct ProviderSelectionService {
    configs: HashMap<AnimeProvider, ProviderConfig>,
    health_tracker: HashMap<AnimeProvider, ProviderHealth>,
}

impl ProviderSelectionService {
    pub fn new() -> Self {
        let mut configs = HashMap::new();
        let mut health_tracker = HashMap::new();

        // Initialize with default configurations
        let providers = [AnimeProvider::AniList, AnimeProvider::Jikan];
        for (i, provider) in providers.iter().enumerate() {
            configs.insert(*provider, ProviderConfig::new(*provider, true, i as u32));
            health_tracker.insert(*provider, ProviderHealth::new(*provider));
        }

        Self {
            configs,
            health_tracker,
        }
    }

    /// Get providers ordered by priority and health
    pub fn get_ordered_providers(&self) -> Vec<AnimeProvider> {
        let mut provider_scores: Vec<(AnimeProvider, f32)> = self
            .configs
            .iter()
            .filter(|(_, config)| config.is_available())
            .map(|(provider, config)| {
                let health_score = self
                    .health_tracker
                    .get(provider)
                    .map(|h| h.priority_score())
                    .unwrap_or(0.5);

                // Combine priority (lower is better) with health score (higher is better)
                let priority_score = (100.0 - config.priority as f32) / 100.0;
                let combined_score = (priority_score * 0.3) + (health_score * 0.7);

                (*provider, combined_score)
            })
            .collect();

        // Sort by combined score (highest first)
        provider_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        provider_scores
            .into_iter()
            .map(|(provider, _)| provider)
            .collect()
    }

    /// Get available (healthy) providers
    pub fn get_available_providers(&self) -> Vec<AnimeProvider> {
        self.configs
            .iter()
            .filter(|(provider, config)| {
                config.is_available()
                    && !self
                        .health_tracker
                        .get(provider)
                        .map(|h| h.should_avoid())
                        .unwrap_or(false)
            })
            .map(|(provider, _)| *provider)
            .collect()
    }

    /// Select best provider for a specific operation
    pub fn select_best_provider(&self, operation_type: OperationType) -> Option<AnimeProvider> {
        let available = self.get_available_providers();
        if available.is_empty() {
            return None;
        }

        // Apply operation-specific logic
        match operation_type {
            OperationType::Search => {
                // For search, prefer providers with faster response times
                available.into_iter().min_by_key(|provider| {
                    self.health_tracker
                        .get(provider)
                        .map(|h| h.average_response_time_ms)
                        .unwrap_or(u32::MAX)
                })
            }
            OperationType::GetDetails => {
                // For details, prefer providers with higher success rates
                available.into_iter().max_by(|a, b| {
                    let score_a = self
                        .health_tracker
                        .get(a)
                        .map(|h| h.success_rate())
                        .unwrap_or(0.0);
                    let score_b = self
                        .health_tracker
                        .get(b)
                        .map(|h| h.success_rate())
                        .unwrap_or(0.0);
                    score_a
                        .partial_cmp(&score_b)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
            }
        }
    }

    /// Record successful operation
    pub fn record_success(&mut self, provider: AnimeProvider, response_time: std::time::Duration) {
        if let Some(health) = self.health_tracker.get_mut(&provider) {
            health.record_success(response_time);
        }
    }

    /// Record failed operation
    pub fn record_failure(&mut self, provider: AnimeProvider) {
        if let Some(health) = self.health_tracker.get_mut(&provider) {
            health.record_failure();
        }
    }

    /// Get provider configuration
    pub fn get_config(&self, provider: &AnimeProvider) -> Option<&ProviderConfig> {
        self.configs.get(provider)
    }

    /// Get provider health status
    pub fn get_health(&self, provider: &AnimeProvider) -> Option<&ProviderHealth> {
        self.health_tracker.get(provider)
    }

    /// Get health summary for all providers
    pub fn get_health_summary(&self) -> HashMap<AnimeProvider, ProviderHealthSummary> {
        self.health_tracker
            .iter()
            .map(|(provider, health)| {
                let summary = ProviderHealthSummary {
                    is_healthy: !health.should_avoid(),
                    success_rate: health.success_rate(),
                    average_response_time_ms: health.average_response_time_ms,
                    total_requests: health.success_count + health.failure_count,
                    priority_score: health.priority_score(),
                };
                (*provider, summary)
            })
            .collect()
    }

    /// Update provider configuration
    pub fn update_config(&mut self, provider: AnimeProvider, config: ProviderConfig) {
        self.configs.insert(provider, config);
    }

    /// Enable/disable provider
    pub fn set_provider_enabled(&mut self, provider: AnimeProvider, enabled: bool) {
        if let Some(config) = self.configs.get_mut(&provider) {
            config.enabled = enabled;
        }
    }

    /// Reset health statistics for a provider
    pub fn reset_health(&mut self, provider: AnimeProvider) {
        if let Some(health) = self.health_tracker.get_mut(&provider) {
            *health = ProviderHealth::new(provider);
        }
    }
}

/// Types of operations for provider selection optimization
#[derive(Debug, Clone, Copy)]
pub enum OperationType {
    Search,
    GetDetails,
}

/// Simplified health summary for external consumption
#[derive(Debug, Clone)]
pub struct ProviderHealthSummary {
    pub is_healthy: bool,
    pub success_rate: f32,
    pub average_response_time_ms: u32,
    pub total_requests: u32,
    pub priority_score: f32,
}

impl Default for ProviderSelectionService {
    fn default() -> Self {
        Self::new()
    }
}
