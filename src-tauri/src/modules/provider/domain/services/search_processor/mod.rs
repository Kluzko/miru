/// Search processor module providing modern, configurable, testable components
/// for processing anime search results.
///
/// # Architecture
///
/// This module uses several design patterns:
/// - **Strategy Pattern**: `SimilarityStrategy` for pluggable matching algorithms
/// - **Builder Pattern**: `TitleNormalizer` for composable transformations
/// - **Configuration Pattern**: `SearchProcessorConfig` for externalized settings
/// - **Metrics Pattern**: `PipelineMetrics` for observability
///
/// # Usage
///
/// ```rust
/// use search_processor::*;
///
/// // Create configuration
/// let config = SearchProcessorConfig::default();
///
/// // Create normalizer
/// let normalizer = TitleNormalizer::default_pipeline(
///     config.remove_patterns.clone(),
///     config.stop_words.clone(),
///     config.min_word_length,
/// );
///
/// // Create similarity strategy
/// let strategy = HybridStrategy::default_hybrid();
///
/// // Use in processor...
/// ```
pub mod config;
pub mod metrics;
pub mod similarity_strategy;
pub mod title_normalizer;

// Re-export main types
pub use config::{SearchProcessorConfig, SearchProcessorConfigBuilder};
pub use metrics::{MetricsBuilder, PipelineMetrics, StageTimer};
pub use similarity_strategy::{
    HybridStrategy, JaroWinklerStrategy, LevenshteinStrategy, SimilarityStrategy,
};
pub use title_normalizer::{
    LowercaseTransform, NormalizeWhitespaceTransform, RemoveNumericSuffixesTransform,
    RemovePatternsTransform, RemoveSpecialCharsTransform, RemoveStopWordsTransform,
    TitleNormalizer, TitleTransformation,
};
