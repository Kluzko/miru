use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use crate::modules::provider::{
    domain::value_objects::{ProviderHealth, ProviderHealthMetrics},
    AnimeProvider,
};

/// Configuration for health monitoring
#[derive(Debug, Clone)]
pub struct HealthMonitorConfig {
    pub failure_threshold: u32,
    pub recovery_threshold: u32,
    pub health_check_interval: Duration,
    /// Minimum response time to consider provider as fast (ms)
    pub fast_response_threshold: Duration,
    /// Maximum acceptable response time before marking as slow (ms)
    pub slow_response_threshold: Duration,
    /// Weight for response time in health calculation (0.0-1.0)
    pub response_time_weight: f32,
}

impl Default for HealthMonitorConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_threshold: 3,
            health_check_interval: Duration::from_secs(60),
            fast_response_threshold: Duration::from_millis(500),
            slow_response_threshold: Duration::from_millis(3000),
            response_time_weight: 0.3,
        }
    }
}

/// Simple health monitor for tracking provider status
pub struct HealthMonitor {
    provider_health: Arc<RwLock<HashMap<AnimeProvider, ProviderHealth>>>,
    config: HealthMonitorConfig,
}

impl HealthMonitor {
    pub fn new(config: HealthMonitorConfig) -> Self {
        let mut provider_health = HashMap::new();

        // Initialize health tracking for all providers
        provider_health.insert(
            AnimeProvider::AniList,
            ProviderHealth::new(AnimeProvider::AniList),
        );
        provider_health.insert(
            AnimeProvider::Jikan,
            ProviderHealth::new(AnimeProvider::Jikan),
        );

        Self {
            provider_health: Arc::new(RwLock::new(provider_health)),
            config,
        }
    }

    /// Record successful operation
    pub async fn record_success(&self, provider: AnimeProvider, response_time: Duration) {
        let mut health_map = self.provider_health.write().await;
        if let Some(health) = health_map.get_mut(&provider) {
            health.record_success(response_time);
            log::debug!(
                "Recorded success for provider {:?}, response time: {:?}",
                provider,
                response_time
            );
        }
    }

    /// Record failed operation
    pub async fn record_failure(&self, provider: AnimeProvider) {
        let mut health_map = self.provider_health.write().await;
        if let Some(health) = health_map.get_mut(&provider) {
            health.record_failure();
            log::debug!("Recorded failure for provider {:?}", provider);
        }
    }

    /// Get health metrics for a specific provider
    pub async fn get_provider_health(
        &self,
        provider: &AnimeProvider,
    ) -> Option<ProviderHealthMetrics> {
        let health_map = self.provider_health.read().await;
        health_map.get(provider).map(|health| health.to_metrics())
    }

    /// Get health metrics for all providers
    pub async fn get_all_health(&self) -> HashMap<AnimeProvider, ProviderHealthMetrics> {
        let health_map = self.provider_health.read().await;
        health_map
            .iter()
            .map(|(provider, health)| (*provider, health.to_metrics()))
            .collect()
    }

    /// Get providers ordered by health and priority
    pub async fn get_healthy_providers(&self) -> Vec<AnimeProvider> {
        let health_map = self.provider_health.read().await;
        let mut providers: Vec<_> = health_map
            .iter()
            .filter(|(_, health)| !health.should_avoid())
            .map(|(provider, health)| (*provider, health.priority_score()))
            .collect();

        // Sort by priority score (highest first)
        providers.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        providers
            .into_iter()
            .map(|(provider, _)| provider)
            .collect()
    }

    /// Check if provider should be avoided
    pub async fn should_avoid_provider(&self, provider: &AnimeProvider) -> bool {
        let health_map = self.provider_health.read().await;
        health_map
            .get(provider)
            .map(|health| health.should_avoid())
            .unwrap_or(false)
    }

    /// Get best provider for new requests
    pub async fn get_best_provider(&self) -> Option<AnimeProvider> {
        self.get_healthy_providers().await.into_iter().next()
    }

    /// Reset health statistics for a provider
    pub async fn reset_provider_health(&self, provider: AnimeProvider) {
        let mut health_map = self.provider_health.write().await;
        health_map.insert(provider, ProviderHealth::new(provider));
        log::info!("Reset health statistics for provider {:?}", provider);
    }

    /// Get overall system health status
    pub async fn get_system_health(&self) -> SystemHealthStatus {
        let health_map = self.provider_health.read().await;
        let healthy_count = health_map
            .values()
            .filter(|health| !health.should_avoid())
            .count();

        let total_count = health_map.len();

        SystemHealthStatus {
            healthy_providers: healthy_count,
            total_providers: total_count,
            system_healthy: healthy_count > 0,
            health_percentage: if total_count > 0 {
                (healthy_count as f32 / total_count as f32) * 100.0
            } else {
                0.0
            },
        }
    }
}

/// Overall system health status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct SystemHealthStatus {
    pub healthy_providers: usize,
    pub total_providers: usize,
    pub system_healthy: bool,
    pub health_percentage: f32,
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new(HealthMonitorConfig::default())
    }
}
