use strsim::{jaro_winkler, normalized_levenshtein};

/// Strategy for calculating similarity between two strings
///
/// This trait enables different similarity algorithms to be used interchangeably,
/// making the system testable and extensible.
pub trait SimilarityStrategy: Send + Sync {
    /// Calculate similarity between query and target
    ///
    /// Returns a value between 0.0 (completely different) and 1.0 (identical)
    fn calculate(&self, query: &str, target: &str) -> f64;

    /// Get the name of this strategy for logging/debugging
    fn name(&self) -> &'static str;
}

/// Jaro-Winkler similarity strategy
///
/// Particularly good for short strings and names (like anime titles).
/// Gives more weight to matching prefixes.
#[derive(Debug, Clone)]
pub struct JaroWinklerStrategy;

impl SimilarityStrategy for JaroWinklerStrategy {
    fn calculate(&self, query: &str, target: &str) -> f64 {
        jaro_winkler(query, target)
    }

    fn name(&self) -> &'static str {
        "JaroWinkler"
    }
}

/// Normalized Levenshtein similarity strategy
///
/// Good for detecting typos and character-level differences.
/// Normalized to 0.0-1.0 range.
#[derive(Debug, Clone)]
pub struct LevenshteinStrategy;

impl SimilarityStrategy for LevenshteinStrategy {
    fn calculate(&self, query: &str, target: &str) -> f64 {
        normalized_levenshtein(query, target)
    }

    fn name(&self) -> &'static str {
        "Levenshtein"
    }
}

/// Hybrid strategy that combines multiple strategies with weighted average
///
/// Allows combining the strengths of different algorithms.
pub struct HybridStrategy {
    strategies: Vec<(Box<dyn SimilarityStrategy>, f64)>,
}

impl HybridStrategy {
    /// Create a new hybrid strategy
    ///
    /// # Arguments
    /// * `strategies` - Vec of (strategy, weight) tuples. Weights must sum to 1.0
    ///
    /// # Panics
    /// Panics if weights don't sum to approximately 1.0
    pub fn new(strategies: Vec<(Box<dyn SimilarityStrategy>, f64)>) -> Self {
        let weight_sum: f64 = strategies.iter().map(|(_, w)| w).sum();
        assert!(
            (weight_sum - 1.0).abs() < 0.01,
            "Strategy weights must sum to 1.0, got {}",
            weight_sum
        );
        Self { strategies }
    }

    /// Create a default hybrid with Jaro-Winkler (70%) + Levenshtein (30%)
    pub fn default_hybrid() -> Self {
        Self::new(vec![
            (Box::new(JaroWinklerStrategy), 0.7),
            (Box::new(LevenshteinStrategy), 0.3),
        ])
    }

    /// Create a balanced hybrid with equal weights
    pub fn balanced() -> Self {
        Self::new(vec![
            (Box::new(JaroWinklerStrategy), 0.5),
            (Box::new(LevenshteinStrategy), 0.5),
        ])
    }
}

impl SimilarityStrategy for HybridStrategy {
    fn calculate(&self, query: &str, target: &str) -> f64 {
        self.strategies
            .iter()
            .map(|(strategy, weight)| strategy.calculate(query, target) * weight)
            .sum()
    }

    fn name(&self) -> &'static str {
        "Hybrid"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Jaro-Winkler tests

    #[test]
    fn test_jaro_winkler_identical_strings() {
        let strategy = JaroWinklerStrategy;
        let similarity = strategy.calculate("naruto", "naruto");
        assert_eq!(similarity, 1.0);
    }

    #[test]
    fn test_jaro_winkler_completely_different() {
        let strategy = JaroWinklerStrategy;
        let similarity = strategy.calculate("abc", "xyz");
        assert!(similarity < 0.5);
    }

    #[test]
    fn test_jaro_winkler_similar_anime_titles() {
        let strategy = JaroWinklerStrategy;
        let similarity = strategy.calculate("naruto", "naruto shippuden");
        assert!(similarity > 0.7); // High similarity due to common prefix
    }

    #[test]
    fn test_jaro_winkler_case_sensitive() {
        let strategy = JaroWinklerStrategy;
        let sim1 = strategy.calculate("naruto", "NARUTO");
        assert!(sim1 < 1.0); // Case matters
    }

    // Levenshtein tests

    #[test]
    fn test_levenshtein_identical_strings() {
        let strategy = LevenshteinStrategy;
        let similarity = strategy.calculate("bleach", "bleach");
        assert_eq!(similarity, 1.0);
    }

    #[test]
    fn test_levenshtein_completely_different() {
        let strategy = LevenshteinStrategy;
        let similarity = strategy.calculate("abc", "xyz");
        assert_eq!(similarity, 0.0);
    }

    #[test]
    fn test_levenshtein_one_char_difference() {
        let strategy = LevenshteinStrategy;
        let similarity = strategy.calculate("naruto", "naruta");
        assert!(similarity > 0.8); // One char difference in 6-char string
    }

    #[test]
    fn test_levenshtein_typo_detection() {
        let strategy = LevenshteinStrategy;
        let similarity = strategy.calculate("attack on titan", "atack on titan");
        assert!(similarity > 0.9); // Single typo
    }

    // Hybrid strategy tests

    #[test]
    fn test_hybrid_default() {
        let strategy = HybridStrategy::default_hybrid();
        let similarity = strategy.calculate("naruto", "naruto");
        assert_eq!(similarity, 1.0);
    }

    #[test]
    fn test_hybrid_balanced() {
        let strategy = HybridStrategy::balanced();
        let similarity = strategy.calculate("naruto", "naruto");
        assert_eq!(similarity, 1.0);
    }

    #[test]
    fn test_hybrid_combines_strengths() {
        let strategy = HybridStrategy::default_hybrid();

        // Test on a string where both algorithms contribute
        let similarity = strategy.calculate("naruto", "naruta");

        // Should be between pure Jaro-Winkler and pure Levenshtein
        let jw = JaroWinklerStrategy.calculate("naruto", "naruta");
        let lev = LevenshteinStrategy.calculate("naruto", "naruta");

        // Weighted: 0.7 * jw + 0.3 * lev
        let expected = 0.7 * jw + 0.3 * lev;
        assert!((similarity - expected).abs() < 0.001);
    }

    #[test]
    #[should_panic(expected = "must sum to 1.0")]
    fn test_hybrid_invalid_weights() {
        HybridStrategy::new(vec![
            (Box::new(JaroWinklerStrategy), 0.5),
            (Box::new(LevenshteinStrategy), 0.3), // Sum is 0.8, should panic
        ]);
    }

    // Property tests

    #[test]
    fn test_similarity_is_commutative_jaro_winkler() {
        let strategy = JaroWinklerStrategy;
        let sim_ab = strategy.calculate("attack on titan", "titan attack");
        let sim_ba = strategy.calculate("titan attack", "attack on titan");
        assert_eq!(sim_ab, sim_ba);
    }

    #[test]
    fn test_similarity_is_commutative_levenshtein() {
        let strategy = LevenshteinStrategy;
        let sim_ab = strategy.calculate("bleach", "breach");
        let sim_ba = strategy.calculate("breach", "bleach");
        assert_eq!(sim_ab, sim_ba);
    }

    #[test]
    fn test_similarity_is_bounded_jaro_winkler() {
        let strategy = JaroWinklerStrategy;
        let test_cases = vec![
            ("naruto", "bleach"),
            ("one piece", "dragon ball"),
            ("attack on titan", "shingeki no kyojin"),
        ];

        for (a, b) in test_cases {
            let sim = strategy.calculate(a, b);
            assert!(
                sim >= 0.0 && sim <= 1.0,
                "Similarity {} out of bounds for '{}'/'{}'",
                sim,
                a,
                b
            );
        }
    }

    #[test]
    fn test_similarity_is_bounded_levenshtein() {
        let strategy = LevenshteinStrategy;
        let test_cases = vec![
            ("naruto", "bleach"),
            ("one piece", "dragon ball"),
            ("", "something"),
            ("a", ""),
        ];

        for (a, b) in test_cases {
            let sim = strategy.calculate(a, b);
            assert!(
                sim >= 0.0 && sim <= 1.0,
                "Similarity {} out of bounds for '{}'/'{}'",
                sim,
                a,
                b
            );
        }
    }

    #[test]
    fn test_empty_string_handling() {
        let jw = JaroWinklerStrategy;
        let lev = LevenshteinStrategy;

        // Empty vs empty should be 1.0 (identical)
        assert_eq!(jw.calculate("", ""), 1.0);
        assert_eq!(lev.calculate("", ""), 1.0);

        // Empty vs non-empty should be 0.0 (completely different)
        assert_eq!(jw.calculate("", "naruto"), 0.0);
        assert_eq!(lev.calculate("", "naruto"), 0.0);
    }

    #[test]
    fn test_single_character_strings() {
        let jw = JaroWinklerStrategy;
        let lev = LevenshteinStrategy;

        // Same character
        assert_eq!(jw.calculate("a", "a"), 1.0);
        assert_eq!(lev.calculate("a", "a"), 1.0);

        // Different characters
        assert!(jw.calculate("a", "b") < 1.0);
        assert_eq!(lev.calculate("a", "b"), 0.0);
    }

    #[test]
    fn test_very_long_strings() {
        let jw = JaroWinklerStrategy;
        let lev = LevenshteinStrategy;

        let long_str = "a".repeat(1000);
        let long_str_similar = format!("{}b", "a".repeat(999));

        // Very similar long strings should have high similarity
        assert!(jw.calculate(&long_str, &long_str_similar) > 0.95);
        assert!(lev.calculate(&long_str, &long_str_similar) > 0.95);
    }

    #[test]
    fn test_unicode_strings() {
        let jw = JaroWinklerStrategy;
        let lev = LevenshteinStrategy;

        // Japanese characters
        assert_eq!(jw.calculate("進撃の巨人", "進撃の巨人"), 1.0);
        assert_eq!(lev.calculate("進撃の巨人", "進撃の巨人"), 1.0);

        // Different unicode strings
        let sim_jw = jw.calculate("進撃の巨人", "鋼の錬金術師");
        let sim_lev = lev.calculate("進撃の巨人", "鋼の錬金術師");
        assert!(sim_jw < 1.0);
        assert!(sim_lev < 1.0);
    }

    #[test]
    fn test_whitespace_sensitivity() {
        let jw = JaroWinklerStrategy;
        let lev = LevenshteinStrategy;

        // Whitespace matters
        assert!(jw.calculate("attack on titan", "attackontitan") < 1.0);
        assert!(lev.calculate("attack on titan", "attackontitan") < 1.0);

        // Extra spaces matter
        assert!(jw.calculate("attack on titan", "attack  on  titan") < 1.0);
        assert!(lev.calculate("attack on titan", "attack  on  titan") < 1.0);
    }

    #[test]
    fn test_substring_similarity() {
        let jw = JaroWinklerStrategy;
        let lev = LevenshteinStrategy;

        // Substring should have reasonably high similarity
        let sim_jw = jw.calculate("naruto", "naruto shippuden");
        let sim_lev = lev.calculate("naruto", "naruto shippuden");

        assert!(
            sim_jw > 0.6,
            "Jaro-Winkler should detect substring similarity"
        );
        assert!(
            sim_lev > 0.3,
            "Levenshtein should detect substring similarity"
        );
    }

    #[test]
    fn test_hybrid_strategy_with_zero_weight() {
        // Test edge case where one strategy has zero weight
        let strategy = HybridStrategy::new(vec![
            (Box::new(JaroWinklerStrategy), 1.0),
            (Box::new(LevenshteinStrategy), 0.0),
        ]);

        // Should behave exactly like pure Jaro-Winkler
        let jw = JaroWinklerStrategy;
        let hybrid_sim = strategy.calculate("naruto", "naruta");
        let jw_sim = jw.calculate("naruto", "naruta");

        assert_eq!(hybrid_sim, jw_sim);
    }

    #[test]
    fn test_strategy_names() {
        assert_eq!(JaroWinklerStrategy.name(), "JaroWinkler");
        assert_eq!(LevenshteinStrategy.name(), "Levenshtein");
        assert_eq!(HybridStrategy::default_hybrid().name(), "Hybrid");
    }

    #[test]
    fn test_reflexivity() {
        // Property: similarity(x, x) should always be 1.0
        let jw = JaroWinklerStrategy;
        let lev = LevenshteinStrategy;

        let test_strings = vec![
            "naruto",
            "attack on titan",
            "one piece",
            "進撃の巨人",
            "a",
            "",
        ];

        for s in test_strings {
            assert_eq!(jw.calculate(s, s), 1.0, "JW reflexivity failed for '{}'", s);
            assert_eq!(
                lev.calculate(s, s),
                1.0,
                "Lev reflexivity failed for '{}'",
                s
            );
        }
    }

    #[test]
    fn test_special_characters_similarity() {
        let jw = JaroWinklerStrategy;
        let lev = LevenshteinStrategy;

        // Special characters should be treated as regular characters
        assert_eq!(jw.calculate("re:zero", "re:zero"), 1.0);
        assert_eq!(lev.calculate("fate/stay night", "fate/stay night"), 1.0);

        // Different special chars
        assert!(jw.calculate("k-on!", "k_on?") < 1.0);
        assert!(lev.calculate("k-on!", "k_on?") < 1.0);
    }
}
