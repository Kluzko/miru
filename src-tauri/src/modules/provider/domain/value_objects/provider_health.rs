use serde::{Deserialize, Serialize};
use specta::Type;
use std::time::{Duration, Instant};

use crate::modules::provider::AnimeProvider;

/// Provider health status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
pub enum ProviderHealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Provider health metrics for monitoring and selection
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ProviderHealthMetrics {
    pub provider: AnimeProvider,
    pub status: ProviderHealthStatus,
    pub success_count: u32,
    pub failure_count: u32,
    pub consecutive_failures: u32,
    pub last_success: Option<String>, // ISO 8601 timestamp
    pub last_failure: Option<String>, // ISO 8601 timestamp
    pub average_response_time_ms: u32,
    pub last_updated: String, // ISO 8601 timestamp
}

/// Internal provider health tracking (runtime only)
#[derive(Debug, Clone)]
pub struct ProviderHealth {
    pub provider: AnimeProvider,
    pub status: ProviderHealthStatus,
    pub success_count: u32,
    pub failure_count: u32,
    pub consecutive_failures: u32,
    pub last_success: Option<Instant>,
    pub last_failure: Option<Instant>,
    pub average_response_time_ms: u32,
    pub last_updated: Instant,
}

impl ProviderHealth {
    pub fn new(provider: AnimeProvider) -> Self {
        Self {
            provider,
            status: ProviderHealthStatus::Unknown,
            success_count: 0,
            failure_count: 0,
            consecutive_failures: 0,
            last_success: None,
            last_failure: None,
            average_response_time_ms: 0,
            last_updated: Instant::now(),
        }
    }

    /// Record a successful operation
    pub fn record_success(&mut self, response_time: Duration) {
        self.success_count += 1;
        self.consecutive_failures = 0;
        self.last_success = Some(Instant::now());
        self.last_updated = Instant::now();

        // Update average response time (simple moving average)
        let response_ms = response_time.as_millis() as u32;
        if self.average_response_time_ms == 0 {
            self.average_response_time_ms = response_ms;
        } else {
            // Weighted average (70% old, 30% new)
            self.average_response_time_ms =
                (self.average_response_time_ms * 7 + response_ms * 3) / 10;
        }

        self.update_status();
    }

    /// Record a failed operation
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.consecutive_failures += 1;
        self.last_failure = Some(Instant::now());
        self.last_updated = Instant::now();

        self.update_status();
    }

    /// Calculate success rate (0.0 to 1.0)
    pub fn success_rate(&self) -> f32 {
        let total = self.success_count + self.failure_count;
        if total == 0 {
            1.0 // Assume healthy until proven otherwise
        } else {
            self.success_count as f32 / total as f32
        }
    }

    /// Check if provider should be avoided
    pub fn should_avoid(&self) -> bool {
        matches!(self.status, ProviderHealthStatus::Unhealthy)
            || self.consecutive_failures >= 5
            || self.success_rate() < 0.3
    }

    /// Get priority score for provider selection (0.0 to 100.0)
    pub fn priority_score(&self) -> f32 {
        if self.should_avoid() {
            return 0.0;
        }

        let mut score = match self.status {
            ProviderHealthStatus::Healthy => 100.0,
            ProviderHealthStatus::Degraded => 70.0,
            ProviderHealthStatus::Unhealthy => 10.0,
            ProviderHealthStatus::Unknown => 50.0,
        };

        // Success rate multiplier
        score *= self.success_rate();

        // Response time penalty
        if self.average_response_time_ms > 5000 {
            score *= 0.5;
        } else if self.average_response_time_ms > 2000 {
            score *= 0.8;
        }

        score.max(0.0)
    }

    /// Update health status based on current metrics
    fn update_status(&mut self) {
        if self.consecutive_failures >= 5 {
            self.status = ProviderHealthStatus::Unhealthy;
        } else if self.success_rate() < 0.5 || self.consecutive_failures >= 3 {
            self.status = ProviderHealthStatus::Degraded;
        } else if self.success_rate() >= 0.8 && self.consecutive_failures == 0 {
            self.status = ProviderHealthStatus::Healthy;
        }
        // Keep current status if no clear change is warranted
    }

    /// Convert to metrics for external consumption
    pub fn to_metrics(&self) -> ProviderHealthMetrics {
        ProviderHealthMetrics {
            provider: self.provider,
            status: self.status.clone(),
            success_count: self.success_count,
            failure_count: self.failure_count,
            consecutive_failures: self.consecutive_failures,
            last_success: self.last_success.map(|_| chrono::Utc::now().to_rfc3339()),
            last_failure: self.last_failure.map(|_| chrono::Utc::now().to_rfc3339()),
            average_response_time_ms: self.average_response_time_ms,
            last_updated: chrono::Utc::now().to_rfc3339(),
        }
    }
}

impl Default for ProviderHealth {
    fn default() -> Self {
        Self::new(AnimeProvider::AniList)
    }
}
