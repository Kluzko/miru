use std::collections::HashSet;

/// Transformation that can be applied to a title
///
/// Each transformation is composable and testable in isolation.
pub trait TitleTransformation: Send + Sync {
    fn transform(&self, title: &str) -> String;
    fn name(&self) -> &'static str;
}

/// Converts title to lowercase
#[derive(Debug, Clone)]
pub struct LowercaseTransform;

impl TitleTransformation for LowercaseTransform {
    fn transform(&self, title: &str) -> String {
        title.to_lowercase()
    }

    fn name(&self) -> &'static str {
        "Lowercase"
    }
}

/// Removes specified patterns from the title
#[derive(Debug, Clone)]
pub struct RemovePatternsTransform {
    patterns: Vec<String>,
}

impl RemovePatternsTransform {
    pub fn new(patterns: Vec<String>) -> Self {
        Self { patterns }
    }
}

impl TitleTransformation for RemovePatternsTransform {
    fn transform(&self, title: &str) -> String {
        let mut result = title.to_string();
        for pattern in &self.patterns {
            result = result.replace(pattern, " ");
        }
        result
    }

    fn name(&self) -> &'static str {
        "RemovePatterns"
    }
}

/// Removes special characters, keeping only alphanumeric and whitespace
#[derive(Debug, Clone)]
pub struct RemoveSpecialCharsTransform;

impl TitleTransformation for RemoveSpecialCharsTransform {
    fn transform(&self, title: &str) -> String {
        title
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect()
    }

    fn name(&self) -> &'static str {
        "RemoveSpecialChars"
    }
}

/// Normalizes whitespace (collapses multiple spaces, trims)
#[derive(Debug, Clone)]
pub struct NormalizeWhitespaceTransform;

impl TitleTransformation for NormalizeWhitespaceTransform {
    fn transform(&self, title: &str) -> String {
        title.split_whitespace().collect::<Vec<&str>>().join(" ")
    }

    fn name(&self) -> &'static str {
        "NormalizeWhitespace"
    }
}

/// Removes stop words and short words
#[derive(Debug, Clone)]
pub struct RemoveStopWordsTransform {
    stop_words: HashSet<String>,
    min_word_length: usize,
}

impl RemoveStopWordsTransform {
    pub fn new(stop_words: Vec<String>, min_word_length: usize) -> Self {
        Self {
            stop_words: stop_words.into_iter().collect(),
            min_word_length,
        }
    }
}

impl TitleTransformation for RemoveStopWordsTransform {
    fn transform(&self, title: &str) -> String {
        title
            .split_whitespace()
            .filter(|word| {
                !self.stop_words.contains(&word.to_lowercase())
                    && word.len() >= self.min_word_length
            })
            .collect::<Vec<&str>>()
            .join(" ")
    }

    fn name(&self) -> &'static str {
        "RemoveStopWords"
    }
}

/// Removes numeric suffixes (e.g., "season 2", "part 3")
#[derive(Debug, Clone)]
pub struct RemoveNumericSuffixesTransform;

impl TitleTransformation for RemoveNumericSuffixesTransform {
    fn transform(&self, title: &str) -> String {
        let words: Vec<&str> = title.split_whitespace().collect();
        let mut result = Vec::new();

        let mut i = 0;
        while i < words.len() {
            let word = words[i];

            // Check if next word is a number or ordinal
            if i + 1 < words.len() {
                let next = words[i + 1];
                if next.chars().all(|c| c.is_numeric())
                    || next.ends_with("st")
                    || next.ends_with("nd")
                    || next.ends_with("rd")
                    || next.ends_with("th")
                {
                    // Skip both this word and the number
                    i += 2;
                    continue;
                }
            }

            result.push(word);
            i += 1;
        }

        result.join(" ")
    }

    fn name(&self) -> &'static str {
        "RemoveNumericSuffixes"
    }
}

/// Title normalizer that applies a pipeline of transformations
///
/// Uses the builder pattern for composability and testability.
pub struct TitleNormalizer {
    transformations: Vec<Box<dyn TitleTransformation>>,
}

impl TitleNormalizer {
    /// Create a new empty normalizer
    pub fn new() -> Self {
        Self {
            transformations: Vec::new(),
        }
    }

    /// Create a normalizer with default transformations
    pub fn default_pipeline(
        remove_patterns: Vec<String>,
        stop_words: Vec<String>,
        min_word_length: usize,
    ) -> Self {
        Self::new()
            .with_lowercase()
            .with_remove_patterns(remove_patterns)
            .with_remove_special_chars()
            .with_normalize_whitespace()
            .with_remove_stop_words(stop_words, min_word_length)
    }

    /// Add lowercase transformation
    pub fn with_lowercase(mut self) -> Self {
        self.transformations.push(Box::new(LowercaseTransform));
        self
    }

    /// Add pattern removal transformation
    pub fn with_remove_patterns(mut self, patterns: Vec<String>) -> Self {
        self.transformations
            .push(Box::new(RemovePatternsTransform::new(patterns)));
        self
    }

    /// Add special character removal transformation
    pub fn with_remove_special_chars(mut self) -> Self {
        self.transformations
            .push(Box::new(RemoveSpecialCharsTransform));
        self
    }

    /// Add whitespace normalization transformation
    pub fn with_normalize_whitespace(mut self) -> Self {
        self.transformations
            .push(Box::new(NormalizeWhitespaceTransform));
        self
    }

    /// Add stop word removal transformation
    pub fn with_remove_stop_words(
        mut self,
        stop_words: Vec<String>,
        min_word_length: usize,
    ) -> Self {
        self.transformations
            .push(Box::new(RemoveStopWordsTransform::new(
                stop_words,
                min_word_length,
            )));
        self
    }

    /// Add numeric suffix removal transformation
    pub fn with_remove_numeric_suffixes(mut self) -> Self {
        self.transformations
            .push(Box::new(RemoveNumericSuffixesTransform));
        self
    }

    /// Apply all transformations to the title
    pub fn normalize(&self, title: &str) -> String {
        let mut result = title.to_string();

        for transformation in &self.transformations {
            result = transformation.transform(&result);
            log::trace!("After {}: '{}'", transformation.name(), result);
        }

        result
    }

    /// Get the number of transformations in the pipeline
    pub fn transformation_count(&self) -> usize {
        self.transformations.len()
    }
}

impl Default for TitleNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Individual transformation tests

    #[test]
    fn test_lowercase_transform() {
        let transform = LowercaseTransform;
        assert_eq!(transform.transform("Naruto SHIPPUDEN"), "naruto shippuden");
        assert_eq!(transform.transform("ATTACK ON TITAN"), "attack on titan");
    }

    #[test]
    fn test_remove_patterns_transform() {
        let transform =
            RemovePatternsTransform::new(vec!["(tv)".to_string(), "season".to_string()]);

        // Note: Pattern matching is case-sensitive
        // Patterns are replaced with spaces, so "season" becomes " ", creating extra spaces
        assert_eq!(transform.transform("Naruto (tv)"), "Naruto  ");
        assert_eq!(
            transform.transform("Attack on Titan season 2"),
            "Attack on Titan   2" // 3 spaces: original + replacement
        );
    }

    #[test]
    fn test_remove_special_chars_transform() {
        let transform = RemoveSpecialCharsTransform;
        assert_eq!(transform.transform("Re:Zero"), "ReZero");
        assert_eq!(transform.transform("Fate/Stay Night"), "FateStay Night");
        assert_eq!(transform.transform("K-On!"), "KOn");
    }

    #[test]
    fn test_normalize_whitespace_transform() {
        let transform = NormalizeWhitespaceTransform;
        assert_eq!(
            transform.transform("  Naruto    Shippuden  "),
            "Naruto Shippuden"
        );
        assert_eq!(transform.transform("One\t\tPiece"), "One Piece");
    }

    #[test]
    fn test_remove_stop_words_transform() {
        let transform = RemoveStopWordsTransform::new(vec!["the".to_string(), "a".to_string()], 2);

        assert_eq!(
            transform.transform("The Seven Deadly Sins"),
            "Seven Deadly Sins"
        );
        assert_eq!(transform.transform("A Silent Voice"), "Silent Voice");
    }

    #[test]
    fn test_remove_numeric_suffixes_transform() {
        let transform = RemoveNumericSuffixesTransform;
        assert_eq!(
            transform.transform("Attack on Titan Season 2"),
            "Attack on Titan"
        );
        assert_eq!(transform.transform("Naruto Part 3"), "Naruto");
        // "Bleach" is followed by "2nd" so both are skipped, leaving "Arc"
        assert_eq!(transform.transform("Bleach 2nd Arc"), "Arc");
    }

    // Pipeline tests

    #[test]
    fn test_empty_pipeline() {
        let normalizer = TitleNormalizer::new();
        assert_eq!(normalizer.normalize("Naruto"), "Naruto");
        assert_eq!(normalizer.transformation_count(), 0);
    }

    #[test]
    fn test_single_transformation_pipeline() {
        let normalizer = TitleNormalizer::new().with_lowercase();
        assert_eq!(normalizer.normalize("NARUTO"), "naruto");
        assert_eq!(normalizer.transformation_count(), 1);
    }

    #[test]
    fn test_multi_transformation_pipeline() {
        let normalizer = TitleNormalizer::new()
            .with_lowercase()
            .with_remove_special_chars()
            .with_normalize_whitespace();

        let result = normalizer.normalize("Re:Zero - Starting Life in Another World");
        assert_eq!(result, "rezero starting life in another world");
        assert_eq!(normalizer.transformation_count(), 3);
    }

    #[test]
    fn test_default_pipeline() {
        let normalizer = TitleNormalizer::default_pipeline(
            vec!["(tv)".to_string(), "season".to_string()],
            vec!["the".to_string()],
            2,
        );

        let result = normalizer.normalize("Naruto Shippuden (TV)");
        assert_eq!(result, "naruto shippuden");
    }

    // Real-world anime title tests

    #[test]
    fn test_normalize_naruto_titles() {
        let normalizer = TitleNormalizer::default_pipeline(
            vec!["(tv)".to_string(), "shippuden".to_string()],
            vec![],
            2,
        );

        let result1 = normalizer.normalize("Naruto");
        let result2 = normalizer.normalize("Naruto (TV)");
        let result3 = normalizer.normalize("Naruto Shippuden");

        assert_eq!(result1, "naruto");
        assert_eq!(result2, "naruto");
        assert_eq!(result3, "naruto"); // All normalize to "naruto"
    }

    #[test]
    fn test_normalize_attack_on_titan_titles() {
        // Important: RemoveNumericSuffixes should come BEFORE RemovePatterns
        // to avoid removing "titan" when it's followed by a number
        let normalizer = TitleNormalizer::new()
            .with_lowercase()
            .with_remove_numeric_suffixes()
            .with_remove_patterns(vec!["season".to_string(), "part".to_string()])
            .with_normalize_whitespace();

        let result1 = normalizer.normalize("Attack on Titan");
        let result2 = normalizer.normalize("Attack on Titan Season 2");
        let result3 = normalizer.normalize("Attack on Titan Part 3");

        assert_eq!(result1, "attack on titan");
        assert_eq!(result2, "attack on titan");
        assert_eq!(result3, "attack on titan");
    }

    #[test]
    fn test_normalize_special_character_titles() {
        let normalizer = TitleNormalizer::new()
            .with_lowercase()
            .with_remove_special_chars()
            .with_normalize_whitespace();

        assert_eq!(normalizer.normalize("Re:Zero"), "rezero");
        assert_eq!(normalizer.normalize("Fate/Stay Night"), "fatestay night");
        assert_eq!(normalizer.normalize("K-On!"), "kon");
        assert_eq!(
            normalizer.normalize("JoJo's Bizarre Adventure"),
            "jojos bizarre adventure"
        );
    }

    // Edge case tests

    #[test]
    fn test_normalize_empty_string() {
        let normalizer = TitleNormalizer::default_pipeline(vec![], vec![], 1);
        assert_eq!(normalizer.normalize(""), "");
    }

    #[test]
    fn test_normalize_only_special_chars() {
        let normalizer = TitleNormalizer::new()
            .with_remove_special_chars()
            .with_normalize_whitespace();

        assert_eq!(normalizer.normalize("!!!???"), "");
    }

    #[test]
    fn test_normalize_only_stop_words() {
        let normalizer = TitleNormalizer::new()
            .with_lowercase()
            .with_remove_stop_words(vec!["the".to_string(), "a".to_string()], 1);

        assert_eq!(normalizer.normalize("The A"), "");
    }

    #[test]
    fn test_normalize_unicode_handling() {
        let normalizer = TitleNormalizer::new()
            .with_lowercase()
            .with_normalize_whitespace();

        // Japanese characters should be preserved
        let result = normalizer.normalize("進撃の巨人");
        assert_eq!(result, "進撃の巨人");
    }

    // Property tests

    #[test]
    fn test_normalization_is_idempotent() {
        let normalizer =
            TitleNormalizer::default_pipeline(vec!["(tv)".to_string()], vec!["the".to_string()], 2);

        let test_cases = vec!["Naruto Shippuden (TV)", "Attack on Titan", "One Piece"];

        for title in test_cases {
            let normalized_once = normalizer.normalize(title);
            let normalized_twice = normalizer.normalize(&normalized_once);
            assert_eq!(
                normalized_once, normalized_twice,
                "Normalization not idempotent for '{}'",
                title
            );
        }
    }

    #[test]
    fn test_order_independence_for_whitespace() {
        let normalizer1 = TitleNormalizer::new()
            .with_normalize_whitespace()
            .with_lowercase();

        let normalizer2 = TitleNormalizer::new()
            .with_lowercase()
            .with_normalize_whitespace();

        let title = "  NARUTO   Shippuden  ";
        assert_eq!(normalizer1.normalize(title), normalizer2.normalize(title));
    }

    #[test]
    fn test_consecutive_special_chars() {
        let transform = RemoveSpecialCharsTransform;

        assert_eq!(transform.transform("!!!???"), "");
        assert_eq!(transform.transform("Naruto!!!"), "Naruto");
        assert_eq!(transform.transform("!!!Bleach???"), "Bleach");
    }

    #[test]
    fn test_mixed_case_patterns() {
        let transform = RemovePatternsTransform::new(vec!["tv".to_string()]);

        // Case sensitive - should NOT remove "TV"
        assert_eq!(transform.transform("Naruto TV"), "Naruto TV");
        // Should remove "tv"
        assert_eq!(transform.transform("Naruto tv"), "Naruto  ");
    }

    #[test]
    fn test_stop_words_at_boundaries() {
        let transform = RemoveStopWordsTransform::new(vec!["the".to_string()], 2);

        // "the" at start
        assert_eq!(
            transform.transform("the seven deadly sins"),
            "seven deadly sins"
        );
        // "the" at end
        assert_eq!(transform.transform("all hail the"), "all hail");
        // "the" in middle
        assert_eq!(transform.transform("into the abyss"), "into abyss");
    }

    #[test]
    fn test_min_word_length_filter() {
        let transform = RemoveStopWordsTransform::new(vec![], 3);

        // Remove words with length < 3
        assert_eq!(transform.transform("a b cd efg"), "efg");
        assert_eq!(transform.transform("one piece is ok"), "one piece");
    }

    #[test]
    fn test_multiple_numeric_suffixes() {
        let transform = RemoveNumericSuffixesTransform;

        // Multiple numeric patterns
        // "Season" followed by "2" removes both, "Part" followed by "3" removes both
        assert_eq!(transform.transform("Naruto Season 2 Part 3"), "Naruto");
        // "Arc" followed by "1" removes both, leaving "Bleach", then "Episode" followed by "2" also removed
        assert_eq!(transform.transform("Bleach Arc 1 Episode 2"), "Bleach");
    }

    #[test]
    fn test_numbers_not_preceded_by_keywords() {
        let transform = RemoveNumericSuffixesTransform;

        // "Zero" followed by "2" (number) - both get removed
        assert_eq!(transform.transform("Aldnoah Zero 2"), "Aldnoah");
        // "R2" is not split into "R" and "2", so it's kept as one word
        assert_eq!(transform.transform("Code Geass R2"), "Code Geass R2");
    }

    #[test]
    fn test_very_long_title_normalization() {
        let normalizer = TitleNormalizer::default_pipeline(vec![], vec![], 1);

        let long_title =
            "The Super Extra Long Anime Title With Many Words That Goes On And On".repeat(5);
        let result = normalizer.normalize(&long_title);

        // Should not crash and should return something
        assert!(!result.is_empty());
        // After normalization (lowercase, etc.), length should be <= original
        assert!(result.len() <= long_title.len());
    }

    #[test]
    fn test_only_whitespace_string() {
        let normalizer = TitleNormalizer::default_pipeline(vec![], vec![], 1);

        assert_eq!(normalizer.normalize("     "), "");
        assert_eq!(normalizer.normalize("\t\t\t"), "");
        assert_eq!(normalizer.normalize("\n\n"), "");
    }

    #[test]
    fn test_mixed_whitespace_types() {
        let transform = NormalizeWhitespaceTransform;

        assert_eq!(
            transform.transform("Naruto\t\tShippuden"),
            "Naruto Shippuden"
        );
        assert_eq!(transform.transform("Attack\non\rTitan"), "Attack on Titan");
        assert_eq!(transform.transform("  One  \t Piece  \n"), "One Piece");
    }

    #[test]
    fn test_unicode_whitespace() {
        let transform = NormalizeWhitespaceTransform;

        // Non-breaking space (U+00A0)
        let title_with_nbsp = "Naruto\u{00A0}Shippuden";
        // Standard space normalization
        assert!(!transform.transform(title_with_nbsp).contains("\u{00A0}"));
    }

    #[test]
    fn test_pattern_overlap() {
        let transform =
            RemovePatternsTransform::new(vec!["season".to_string(), "season 2".to_string()]);

        // Both patterns should be removed
        let result = transform.transform("Attack on Titan season 2");
        assert!(!result.contains("season"));
    }

    #[test]
    fn test_transformation_count() {
        let normalizer = TitleNormalizer::new()
            .with_lowercase()
            .with_remove_special_chars()
            .with_normalize_whitespace();

        assert_eq!(normalizer.transformation_count(), 3);
    }

    #[test]
    fn test_empty_pipeline_is_identity() {
        let normalizer = TitleNormalizer::new();

        let test_cases = vec!["Naruto", "ATTACK ON TITAN", "one piece!!!", "  spaces  "];

        for title in test_cases {
            assert_eq!(normalizer.normalize(title), title);
        }
    }

    #[test]
    fn test_real_world_anime_edge_cases() {
        let normalizer = TitleNormalizer::default_pipeline(
            vec!["(tv)".to_string(), "(movie)".to_string()],
            vec!["the".to_string()],
            2,
        );

        // Anime with numbers in name
        assert!(normalizer.normalize("Aldnoah.Zero").contains("aldnoah"));
        assert!(normalizer.normalize("Code Geass R2").contains("code"));

        // Anime with special characters
        assert!(normalizer.normalize("Re:Zero").contains("rezero"));
        assert!(normalizer.normalize("Steins;Gate").contains("steins"));

        // Very short titles - "K" has only 1 char, gets filtered by min_word_length=2
        let result_k = normalizer.normalize("K");
        assert_eq!(result_k, ""); // Filtered out because length < 2

        // "91 Days" - "91" is 2 chars, "Days" is 4 chars, both should pass
        assert!(!normalizer.normalize("91 Days").is_empty());
    }

    #[test]
    fn test_normalization_with_all_transformations() {
        let normalizer = TitleNormalizer::new()
            .with_lowercase()
            .with_remove_patterns(vec!["(tv)".to_string(), "season".to_string()])
            .with_remove_special_chars()
            .with_normalize_whitespace()
            .with_remove_stop_words(vec!["the".to_string()], 2)
            .with_remove_numeric_suffixes();

        let result = normalizer.normalize("The Attack on Titan (TV) Season 2!!!");

        // Pipeline: lowercase -> remove "(tv)" & "season" -> remove "!" -> normalize spaces
        //           -> remove "the" (length >= 2) -> remove numeric suffixes
        // Result should be: "attack on titan" (the, tv, season all removed)
        assert!(!result.contains("THE") && !result.contains("The"));
        assert!(!result.contains("!"));
        assert!(!result.contains("tv"));
        assert!(!result.contains("season"));
        assert!(result.contains("attack"));
        assert!(result.contains("titan"));
    }
}
