/// Configuration for the search results processor
///
/// Externalizes all magic numbers, thresholds, and rules to make the processor
/// configurable and testable.
#[derive(Debug, Clone)]
pub struct SearchProcessorConfig {
    // Fuzzy matching configuration
    /// Weight for Jaro-Winkler similarity (0.0 to 1.0)
    pub jaro_winkler_weight: f64,

    /// Weight for Levenshtein similarity (0.0 to 1.0)
    pub levenshtein_weight: f64,

    // Quality thresholds
    /// Default quality threshold if not specified in search criteria
    pub default_quality_threshold: f32,

    /// Minimum allowed quality threshold
    pub min_quality_threshold: f32,

    /// Maximum allowed quality threshold
    pub max_quality_threshold: f32,

    // Title normalization rules
    /// Patterns to remove from titles during normalization
    pub remove_patterns: Vec<String>,

    /// Stop words to remove (e.g., "the", "a", "an")
    pub stop_words: Vec<String>,

    /// Minimum word length to keep during normalization
    pub min_word_length: usize,

    // Pipeline limits
    /// Maximum results to accept from a single provider
    pub max_results_per_provider: usize,

    /// Maximum number of providers to merge for a single anime
    pub max_merge_group_size: usize,

    /// Enable/disable deduplication stage
    pub enable_deduplication: bool,

    /// Enable/disable merging stage
    pub enable_merging: bool,

    /// Enable/disable quality filtering stage
    pub enable_quality_filtering: bool,
}

impl SearchProcessorConfig {
    /// Creates a new configuration with sensible production defaults
    pub fn new() -> Self {
        Self {
            // Fuzzy matching: Jaro-Winkler is better for names, so weight it higher
            jaro_winkler_weight: 0.7,
            levenshtein_weight: 0.3,

            // Quality thresholds
            default_quality_threshold: 0.0,
            min_quality_threshold: 0.0,
            max_quality_threshold: 100.0,

            // Title normalization: Remove common anime metadata
            remove_patterns: vec![
                "(tv)".to_string(),
                "(movie)".to_string(),
                "(ova)".to_string(),
                "(ona)".to_string(),
                "(special)".to_string(),
                "season".to_string(),
                "part".to_string(),
                "cour".to_string(),
                "2nd".to_string(),
                "3rd".to_string(),
                "1st".to_string(),
            ],

            // Common stop words
            stop_words: vec!["the".to_string(), "a".to_string(), "an".to_string()],

            min_word_length: 2,

            // Pipeline limits
            max_results_per_provider: 100,
            max_merge_group_size: 5,

            // Enable all stages by default
            enable_deduplication: true,
            enable_merging: true,
            enable_quality_filtering: true,
        }
    }

    /// Creates a minimal configuration for testing
    #[cfg(test)]
    pub fn minimal() -> Self {
        Self {
            jaro_winkler_weight: 0.5,
            levenshtein_weight: 0.5,
            default_quality_threshold: 0.0,
            min_quality_threshold: 0.0,
            max_quality_threshold: 100.0,
            remove_patterns: vec![],
            stop_words: vec![],
            min_word_length: 1,
            max_results_per_provider: 10,
            max_merge_group_size: 3,
            enable_deduplication: true,
            enable_merging: true,
            enable_quality_filtering: false,
        }
    }

    /// Validates the configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate weights sum to ~1.0
        let weight_sum = self.jaro_winkler_weight + self.levenshtein_weight;
        if (weight_sum - 1.0).abs() > 0.01 {
            return Err(format!(
                "Fuzzy matching weights must sum to 1.0, got {}",
                weight_sum
            ));
        }

        // Validate weights are non-negative
        if self.jaro_winkler_weight < 0.0 || self.levenshtein_weight < 0.0 {
            return Err("Fuzzy matching weights must be non-negative".to_string());
        }

        // Validate quality thresholds
        if self.min_quality_threshold > self.max_quality_threshold {
            return Err(format!(
                "Min quality threshold ({}) cannot exceed max ({})",
                self.min_quality_threshold, self.max_quality_threshold
            ));
        }

        if self.default_quality_threshold < self.min_quality_threshold
            || self.default_quality_threshold > self.max_quality_threshold
        {
            return Err(format!(
                "Default quality threshold ({}) must be between {} and {}",
                self.default_quality_threshold,
                self.min_quality_threshold,
                self.max_quality_threshold
            ));
        }

        // Validate limits
        if self.max_results_per_provider == 0 {
            return Err("max_results_per_provider must be > 0".to_string());
        }

        if self.max_merge_group_size == 0 {
            return Err("max_merge_group_size must be > 0".to_string());
        }

        Ok(())
    }
}

impl Default for SearchProcessorConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for SearchProcessorConfig to make test setup easier
#[derive(Default)]
pub struct SearchProcessorConfigBuilder {
    config: SearchProcessorConfig,
}

impl SearchProcessorConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: SearchProcessorConfig::new(),
        }
    }

    pub fn jaro_winkler_weight(mut self, weight: f64) -> Self {
        self.config.jaro_winkler_weight = weight;
        self
    }

    pub fn levenshtein_weight(mut self, weight: f64) -> Self {
        self.config.levenshtein_weight = weight;
        self
    }

    pub fn default_quality_threshold(mut self, threshold: f32) -> Self {
        self.config.default_quality_threshold = threshold;
        self
    }

    pub fn remove_patterns(mut self, patterns: Vec<String>) -> Self {
        self.config.remove_patterns = patterns;
        self
    }

    pub fn stop_words(mut self, words: Vec<String>) -> Self {
        self.config.stop_words = words;
        self
    }

    pub fn min_word_length(mut self, length: usize) -> Self {
        self.config.min_word_length = length;
        self
    }

    pub fn max_results_per_provider(mut self, max: usize) -> Self {
        self.config.max_results_per_provider = max;
        self
    }

    pub fn enable_deduplication(mut self, enable: bool) -> Self {
        self.config.enable_deduplication = enable;
        self
    }

    pub fn enable_merging(mut self, enable: bool) -> Self {
        self.config.enable_merging = enable;
        self
    }

    pub fn enable_quality_filtering(mut self, enable: bool) -> Self {
        self.config.enable_quality_filtering = enable;
        self
    }

    pub fn build(self) -> Result<SearchProcessorConfig, String> {
        self.config.validate()?;
        Ok(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_is_valid() {
        let config = SearchProcessorConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_minimal_config_is_valid() {
        let config = SearchProcessorConfig::minimal();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_weights_must_sum_to_one() {
        let config = SearchProcessorConfigBuilder::new()
            .jaro_winkler_weight(0.5)
            .levenshtein_weight(0.3)
            .build();

        assert!(config.is_err());
        assert!(config.unwrap_err().contains("must sum to 1.0"));
    }

    #[test]
    fn test_weights_must_be_non_negative() {
        let config = SearchProcessorConfigBuilder::new()
            .jaro_winkler_weight(-0.5)
            .levenshtein_weight(1.5)
            .build();

        assert!(config.is_err());
        assert!(config.unwrap_err().contains("non-negative"));
    }

    #[test]
    fn test_quality_thresholds_must_be_ordered() {
        let mut config = SearchProcessorConfig::default();
        config.min_quality_threshold = 50.0;
        config.max_quality_threshold = 30.0;

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_default_threshold_must_be_in_range() {
        let mut config = SearchProcessorConfig::default();
        config.min_quality_threshold = 10.0;
        config.max_quality_threshold = 90.0;
        config.default_quality_threshold = 95.0;

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_builder_creates_valid_config() {
        let config = SearchProcessorConfigBuilder::new()
            .jaro_winkler_weight(0.6)
            .levenshtein_weight(0.4)
            .default_quality_threshold(30.0)
            .build();

        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.jaro_winkler_weight, 0.6);
        assert_eq!(config.levenshtein_weight, 0.4);
        assert_eq!(config.default_quality_threshold, 30.0);
    }

    #[test]
    fn test_zero_max_results_per_provider_is_invalid() {
        let mut config = SearchProcessorConfig::default();
        config.max_results_per_provider = 0;

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("max_results_per_provider"));
    }

    #[test]
    fn test_zero_max_merge_group_size_is_invalid() {
        let mut config = SearchProcessorConfig::default();
        config.max_merge_group_size = 0;

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("max_merge_group_size"));
    }

    #[test]
    fn test_extreme_weight_values() {
        // Test with weight 1.0 and 0.0 (valid edge case)
        let config = SearchProcessorConfigBuilder::new()
            .jaro_winkler_weight(1.0)
            .levenshtein_weight(0.0)
            .build();

        assert!(config.is_ok());
    }

    #[test]
    fn test_weight_sum_slightly_off_is_invalid() {
        // Test that we catch weights that sum to 1.02 (outside tolerance)
        let config = SearchProcessorConfigBuilder::new()
            .jaro_winkler_weight(0.7)
            .levenshtein_weight(0.32)
            .build();

        assert!(config.is_err());
    }

    #[test]
    fn test_threshold_boundaries() {
        // Test that min == max is valid
        let mut config = SearchProcessorConfig::default();
        config.min_quality_threshold = 50.0;
        config.max_quality_threshold = 50.0;
        config.default_quality_threshold = 50.0;

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_empty_patterns_and_stop_words() {
        // Empty lists should be valid
        let config = SearchProcessorConfigBuilder::new()
            .jaro_winkler_weight(0.5)
            .levenshtein_weight(0.5)
            .remove_patterns(vec![])
            .stop_words(vec![])
            .build();

        assert!(config.is_ok());
    }

    #[test]
    fn test_builder_chaining() {
        // Test that builder methods can be chained in any order
        let config = SearchProcessorConfigBuilder::new()
            .enable_deduplication(false)
            .enable_merging(true)
            .enable_quality_filtering(false)
            .max_results_per_provider(50)
            .min_word_length(1)
            .jaro_winkler_weight(0.5)
            .levenshtein_weight(0.5)
            .build();

        assert!(config.is_ok());
        let config = config.unwrap();
        assert!(!config.enable_deduplication);
        assert!(config.enable_merging);
        assert!(!config.enable_quality_filtering);
        assert_eq!(config.max_results_per_provider, 50);
    }
}
