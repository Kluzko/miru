use std::collections::HashMap;
use std::sync::Arc;

use crate::{
    modules::provider::{
        domain::services::{OperationType, ProviderHealthSummary, ProviderSelectionService},
        AnimeProvider,
    },
    shared::errors::AppResult,
};

use super::super::dto::{HealthCheckRequest, HealthCheckResponse, ProviderStatus};

/// Use case for checking provider health
pub struct HealthCheckUseCase {
    provider_service: Arc<ProviderSelectionService>,
}

impl HealthCheckUseCase {
    pub fn new(provider_service: Arc<ProviderSelectionService>) -> Self {
        Self { provider_service }
    }

    /// Execute health check for all providers
    pub async fn execute(&self, _request: HealthCheckRequest) -> AppResult<HealthCheckResponse> {
        let health_summary = self.provider_service.get_health_summary();
        let available_providers = self.provider_service.get_available_providers();

        let mut provider_statuses = HashMap::new();

        for (provider, health) in health_summary {
            let status = ProviderStatus {
                provider,
                is_healthy: health.is_healthy,
                is_available: available_providers.contains(&provider),
                success_rate: health.success_rate,
                average_response_time_ms: health.average_response_time_ms,
                total_requests: health.total_requests,
                priority_score: health.priority_score,
                status_message: self.get_status_message(&health),
            };

            provider_statuses.insert(provider, status);
        }

        // Overall system health
        let healthy_providers = provider_statuses
            .values()
            .filter(|status| status.is_healthy && status.is_available)
            .count();

        let system_healthy = healthy_providers > 0;
        let system_message = if system_healthy {
            if healthy_providers == provider_statuses.len() {
                "All providers healthy".to_string()
            } else {
                format!(
                    "{}/{} providers healthy",
                    healthy_providers,
                    provider_statuses.len()
                )
            }
        } else {
            "No healthy providers available".to_string()
        };

        Ok(HealthCheckResponse {
            system_healthy,
            system_message,
            provider_statuses,
            recommended_provider: self
                .provider_service
                .select_best_provider(OperationType::Search),
        })
    }

    /// Execute health check for a specific provider
    pub async fn execute_for_provider(
        &self,
        provider: AnimeProvider,
    ) -> AppResult<Option<ProviderStatus>> {
        if let Some(health) = self.provider_service.get_health(&provider) {
            let available_providers = self.provider_service.get_available_providers();

            let health_summary = ProviderHealthSummary {
                is_healthy: !health.should_avoid(),
                success_rate: health.success_rate(),
                average_response_time_ms: health.average_response_time_ms,
                total_requests: health.success_count + health.failure_count,
                priority_score: health.priority_score(),
            };

            let status = ProviderStatus {
                provider,
                is_healthy: health_summary.is_healthy,
                is_available: available_providers.contains(&provider),
                success_rate: health_summary.success_rate,
                average_response_time_ms: health_summary.average_response_time_ms,
                total_requests: health_summary.total_requests,
                priority_score: health_summary.priority_score,
                status_message: self.get_status_message(&health_summary),
            };

            Ok(Some(status))
        } else {
            Ok(None)
        }
    }

    /// Get human-readable status message
    fn get_status_message(&self, health: &ProviderHealthSummary) -> String {
        if !health.is_healthy {
            "Unhealthy - high failure rate".to_string()
        } else if health.total_requests == 0 {
            "No requests processed yet".to_string()
        } else if health.average_response_time_ms > 5000 {
            "Slow response times".to_string()
        } else if health.success_rate < 0.8 {
            "Moderate reliability".to_string()
        } else if health.total_requests < 10 {
            "Insufficient data".to_string()
        } else {
            "Healthy and performing well".to_string()
        }
    }
}
