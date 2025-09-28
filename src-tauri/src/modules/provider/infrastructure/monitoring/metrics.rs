use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::modules::provider::AnimeProvider;

/// Performance metrics collector
pub struct MetricsCollector {
    metrics: Arc<RwLock<HashMap<AnimeProvider, ProviderMetrics>>>,
}

/// Provider performance metrics
#[derive(Debug, Clone)]
pub struct ProviderMetrics {
    pub provider: AnimeProvider,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub total_response_time: Duration,
    pub fastest_response: Duration,
    pub slowest_response: Duration,
    pub last_request_time: Option<Instant>,
    pub requests_per_minute: f32,
}

/// Metrics summary for external consumption
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct MetricsSummary {
    pub provider: AnimeProvider,
    pub total_requests: u64,
    pub success_rate: f32,
    pub average_response_time_ms: u64,
    pub fastest_response_ms: u64,
    pub slowest_response_ms: u64,
    pub requests_per_minute: f32,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record a successful request
    pub async fn record_success(&self, provider: AnimeProvider, response_time: Duration) {
        let mut metrics = self.metrics.write().await;
        let entry = metrics
            .entry(provider)
            .or_insert_with(|| ProviderMetrics::new(provider));

        entry.total_requests += 1;
        entry.successful_requests += 1;
        entry.total_response_time += response_time;
        entry.last_request_time = Some(Instant::now());

        // Update fastest/slowest response times
        if entry.fastest_response > response_time || entry.total_requests == 1 {
            entry.fastest_response = response_time;
        }
        if entry.slowest_response < response_time || entry.total_requests == 1 {
            entry.slowest_response = response_time;
        }

        entry.update_requests_per_minute();
    }

    /// Record a failed request
    pub async fn record_failure(&self, provider: AnimeProvider, response_time: Option<Duration>) {
        let mut metrics = self.metrics.write().await;
        let entry = metrics
            .entry(provider)
            .or_insert_with(|| ProviderMetrics::new(provider));

        entry.total_requests += 1;
        entry.failed_requests += 1;
        entry.last_request_time = Some(Instant::now());

        if let Some(time) = response_time {
            entry.total_response_time += time;
            if entry.slowest_response < time {
                entry.slowest_response = time;
            }
        }

        entry.update_requests_per_minute();
    }

    /// Get metrics for a specific provider
    pub async fn get_provider_metrics(&self, provider: &AnimeProvider) -> Option<MetricsSummary> {
        let metrics = self.metrics.read().await;
        metrics.get(provider).map(|m| m.to_summary())
    }

    /// Get metrics for all providers
    pub async fn get_all_metrics(&self) -> HashMap<AnimeProvider, MetricsSummary> {
        let metrics = self.metrics.read().await;
        metrics
            .iter()
            .map(|(provider, metrics)| (*provider, metrics.to_summary()))
            .collect()
    }

    /// Reset metrics for a provider
    pub async fn reset_provider_metrics(&self, provider: AnimeProvider) {
        let mut metrics = self.metrics.write().await;
        metrics.insert(provider, ProviderMetrics::new(provider));
    }

    /// Clear all metrics
    pub async fn clear_all_metrics(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.clear();
    }
}

impl ProviderMetrics {
    pub fn new(provider: AnimeProvider) -> Self {
        Self {
            provider,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            total_response_time: Duration::from_secs(0),
            fastest_response: Duration::from_secs(u64::MAX),
            slowest_response: Duration::from_secs(0),
            last_request_time: None,
            requests_per_minute: 0.0,
        }
    }

    /// Update requests per minute calculation
    fn update_requests_per_minute(&mut self) {
        // Simple calculation: total requests in last minute
        // In a real implementation, you'd maintain a sliding window
        self.requests_per_minute = self.total_requests as f32 / 60.0;
    }

    /// Convert to summary for external consumption
    pub fn to_summary(&self) -> MetricsSummary {
        let average_response_time = if self.successful_requests > 0 {
            self.total_response_time.as_millis() as u64 / self.successful_requests
        } else {
            0
        };

        let success_rate = if self.total_requests > 0 {
            self.successful_requests as f32 / self.total_requests as f32
        } else {
            0.0
        };

        MetricsSummary {
            provider: self.provider,
            total_requests: self.total_requests,
            success_rate,
            average_response_time_ms: average_response_time,
            fastest_response_ms: self.fastest_response.as_millis() as u64,
            slowest_response_ms: self.slowest_response.as_millis() as u64,
            requests_per_minute: self.requests_per_minute,
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}
