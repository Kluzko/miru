use crate::modules::provider::{
    infrastructure::service::provider_service::ProviderConfig, AnimeProvider,
};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Unified provider health with essential metrics and optional advanced features
#[derive(Debug)]
pub struct UnifiedProviderHealth {
    // Core health metrics (atomic for lock-free access)
    pub is_healthy: AtomicBool,
    pub consecutive_failures: AtomicU32,
    pub total_requests: AtomicU64,
    pub successful_requests: AtomicU64,

    // Timestamps (protected by simple mutex)
    pub last_success: std::sync::Mutex<Option<Instant>>,
    pub last_failure: std::sync::Mutex<Option<Instant>>,

    // Advanced metrics (optional, can be disabled for performance)
    pub avg_response_time: std::sync::Mutex<Option<Duration>>,
}

impl UnifiedProviderHealth {
    pub fn new() -> Self {
        Self {
            is_healthy: AtomicBool::new(true),
            consecutive_failures: AtomicU32::new(0),
            total_requests: AtomicU64::new(0),
            successful_requests: AtomicU64::new(0),
            last_success: std::sync::Mutex::new(None),
            last_failure: std::sync::Mutex::new(None),
            avg_response_time: std::sync::Mutex::new(None),
        }
    }

    pub fn record_success(&self, response_time: Duration) {
        // Update atomic counters (lock-free, high performance)
        self.consecutive_failures.store(0, Ordering::Relaxed);
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.successful_requests.fetch_add(1, Ordering::Relaxed);
        self.is_healthy.store(true, Ordering::Relaxed);

        // Update timestamps (minimal lock contention)
        if let Ok(mut last_success) = self.last_success.lock() {
            *last_success = Some(Instant::now());
        }

        // Update average response time (simple moving average)
        if let Ok(mut avg_time) = self.avg_response_time.lock() {
            *avg_time = match *avg_time {
                Some(current_avg) => Some((current_avg + response_time) / 2),
                None => Some(response_time),
            };
        }
    }

    pub fn record_failure(&self) {
        let failures = self.consecutive_failures.fetch_add(1, Ordering::Relaxed) + 1;
        self.total_requests.fetch_add(1, Ordering::Relaxed);

        // Circuit breaker: mark unhealthy after 3 consecutive failures
        if failures >= 3 {
            self.is_healthy.store(false, Ordering::Relaxed);
        }

        if let Ok(mut last_failure) = self.last_failure.lock() {
            *last_failure = Some(Instant::now());
        }
    }

    pub fn is_available(&self) -> bool {
        self.is_healthy.load(Ordering::Relaxed) || self.should_retry()
    }

    pub fn success_rate(&self) -> f64 {
        let total = self.total_requests.load(Ordering::Relaxed);
        if total == 0 {
            return 100.0;
        }
        let successful = self.successful_requests.load(Ordering::Relaxed);
        (successful as f64 / total as f64) * 100.0
    }

    fn should_retry(&self) -> bool {
        if let Ok(last_failure) = self.last_failure.lock() {
            if let Some(last_fail_time) = *last_failure {
                let failures = self.consecutive_failures.load(Ordering::Relaxed);
                // Exponential backoff with reasonable limits
                let backoff_duration = match failures {
                    1..=2 => Duration::from_secs(30),
                    3..=4 => Duration::from_secs(60),
                    5..=6 => Duration::from_secs(120),
                    _ => Duration::from_secs(300), // Max 5 minutes
                };
                return last_fail_time.elapsed() >= backoff_duration;
            }
        }
        true // If we can't access timestamp, allow retry
    }

    /// Create a lightweight snapshot for monitoring
    pub fn snapshot(&self) -> UnifiedProviderHealthSnapshot {
        UnifiedProviderHealthSnapshot {
            is_healthy: self.is_healthy.load(Ordering::Relaxed),
            consecutive_failures: self.consecutive_failures.load(Ordering::Relaxed),
            total_requests: self.total_requests.load(Ordering::Relaxed),
            successful_requests: self.successful_requests.load(Ordering::Relaxed),
            last_success: self.last_success.lock().map(|guard| *guard).unwrap_or(None),
            last_failure: self.last_failure.lock().map(|guard| *guard).unwrap_or(None),
            avg_response_time: self
                .avg_response_time
                .lock()
                .map(|guard| *guard)
                .unwrap_or(None),
        }
    }
}

impl Default for UnifiedProviderHealth {
    fn default() -> Self {
        Self::new()
    }
}

/// Unified provider state with integrated health monitoring
#[derive(Debug)]
pub struct UnifiedProviderState {
    pub config: ProviderConfig,
    pub health: UnifiedProviderHealth,
    pub enabled: bool,
    pub priority: u8,
}

impl UnifiedProviderState {
    pub fn new(config: ProviderConfig, priority: u8) -> Self {
        Self {
            enabled: config.enabled,
            config,
            health: UnifiedProviderHealth::new(),
            priority,
        }
    }

    pub fn is_available(&self) -> bool {
        self.enabled && self.health.is_available()
    }
}

/// Unified provider registry - combines best features from both registries
/// Optimized for performance while maintaining essential functionality
#[derive(Debug)]
pub struct UnifiedProviderRegistry {
    providers: HashMap<AnimeProvider, UnifiedProviderState>,
    primary_provider: AnimeProvider,
}

impl UnifiedProviderRegistry {
    pub fn new() -> Self {
        let mut providers = HashMap::new();

        // Initialize with default configurations
        let jikan_config = ProviderConfig {
            name: "MyAnimeList (Jikan)".to_string(),
            description:
                "Comprehensive anime database with ratings, reviews, and detailed metadata"
                    .to_string(),
            api_url: "https://api.jikan.moe/v4".to_string(),
            enabled: true,
            priority: 2,
        };

        let anilist_config = ProviderConfig {
            name: "AniList".to_string(),
            description: "Community-driven database with GraphQL API and modern interface"
                .to_string(),
            api_url: "https://graphql.anilist.co".to_string(),
            enabled: true,
            priority: 1,
        };

        providers.insert(
            AnimeProvider::Jikan,
            UnifiedProviderState::new(jikan_config, 2),
        );
        providers.insert(
            AnimeProvider::AniList,
            UnifiedProviderState::new(anilist_config, 1),
        );

        Self {
            providers,
            primary_provider: AnimeProvider::AniList, // Default to highest priority
        }
    }

    // Configuration management
    pub fn get_config(&self, provider: &AnimeProvider) -> Option<&ProviderConfig> {
        self.providers.get(provider).map(|state| &state.config)
    }

    pub fn update_config(&mut self, provider: AnimeProvider, config: ProviderConfig) {
        if let Some(state) = self.providers.get_mut(&provider) {
            state.enabled = config.enabled;
            state.priority = config.priority as u8;
            state.config = config;
        } else {
            let priority = config.priority as u8;
            self.providers
                .insert(provider, UnifiedProviderState::new(config, priority));
        }
    }

    pub fn register_provider(&mut self, provider: AnimeProvider, config: ProviderConfig) {
        let priority = config.priority as u8;
        self.providers
            .insert(provider, UnifiedProviderState::new(config, priority));
    }

    // Health management
    pub fn get_health(&self, provider: &AnimeProvider) -> Option<&UnifiedProviderHealth> {
        self.providers.get(provider).map(|state| &state.health)
    }

    pub fn record_success(&mut self, provider: &AnimeProvider, response_time: Duration) {
        if let Some(state) = self.providers.get_mut(provider) {
            state.health.record_success(response_time);
        }
    }

    pub fn record_failure(&mut self, provider: &AnimeProvider) {
        if let Some(state) = self.providers.get_mut(provider) {
            state.health.record_failure();
        }
    }

    // Provider selection and filtering
    pub fn get_enabled_providers(&self) -> Vec<AnimeProvider> {
        let mut enabled: Vec<_> = self
            .providers
            .iter()
            .filter(|(_, state)| state.is_available())
            .map(|(provider, state)| (provider.clone(), state.priority))
            .collect();

        enabled.sort_by_key(|(_, priority)| *priority);
        enabled.into_iter().map(|(provider, _)| provider).collect()
    }

    pub fn get_available_providers(&self) -> Vec<AnimeProvider> {
        self.providers
            .iter()
            .filter_map(|(provider, state)| {
                if state.is_available() {
                    Some(provider.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn is_enabled(&self, provider: &AnimeProvider) -> bool {
        self.providers
            .get(provider)
            .map(|state| state.enabled)
            .unwrap_or(false)
    }

    pub fn is_provider_available(&self, provider: &AnimeProvider) -> bool {
        self.providers
            .get(provider)
            .map(|state| state.is_available())
            .unwrap_or(false)
    }

    // Primary provider management
    pub fn get_primary_provider(&self) -> AnimeProvider {
        if self.is_provider_available(&self.primary_provider) {
            self.primary_provider.clone()
        } else {
            self.get_enabled_providers()
                .into_iter()
                .next()
                .unwrap_or(self.primary_provider.clone())
        }
    }

    pub fn set_primary_provider(&mut self, provider: AnimeProvider) -> Result<(), String> {
        if !self.is_enabled(&provider) {
            return Err(format!("Provider {:?} is not enabled", provider));
        }
        self.primary_provider = provider;
        Ok(())
    }

    // Monitoring and statistics
    pub fn get_stats_summary(&self) -> HashMap<AnimeProvider, UnifiedProviderStats> {
        self.providers
            .iter()
            .map(|(provider, state)| {
                let health_snapshot = state.health.snapshot();
                let stats = UnifiedProviderStats {
                    is_healthy: health_snapshot.is_healthy,
                    success_rate: state.health.success_rate(),
                    total_requests: health_snapshot.total_requests,
                    consecutive_failures: health_snapshot.consecutive_failures,
                    is_enabled: state.enabled,
                    priority: state.priority,
                    avg_response_time: health_snapshot.avg_response_time,
                    last_success: health_snapshot.last_success,
                    last_failure: health_snapshot.last_failure,
                    successful_requests: health_snapshot.successful_requests,
                };
                (provider.clone(), stats)
            })
            .collect()
    }
}

impl Default for UnifiedProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Lightweight snapshot for monitoring dashboards
#[derive(Debug, Clone)]
pub struct UnifiedProviderHealthSnapshot {
    pub is_healthy: bool,
    pub consecutive_failures: u32,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub last_success: Option<Instant>,
    pub last_failure: Option<Instant>,
    pub avg_response_time: Option<Duration>,
}

/// Comprehensive provider statistics for monitoring
#[derive(Debug, Clone)]
pub struct UnifiedProviderStats {
    pub is_healthy: bool,
    pub success_rate: f64,
    pub total_requests: u64,
    pub consecutive_failures: u32,
    pub is_enabled: bool,
    pub priority: u8,
    pub avg_response_time: Option<Duration>,
    pub last_success: Option<Instant>,
    pub last_failure: Option<Instant>,
    pub successful_requests: u64,
}

impl UnifiedProviderStats {
    /// Create a snapshot for external access (for backward compatibility)
    pub fn snapshot(&self) -> Self {
        self.clone()
    }
}
