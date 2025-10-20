use std::collections::HashMap;
use std::sync::Arc;

use strsim::{jaro_winkler, normalized_levenshtein};

use crate::modules::{
    anime::domain::{
        services::data_quality_service::DataQualityService, value_objects::AnimeTitle,
    },
    provider::domain::{
        entities::AnimeData,
        value_objects::{PreferredLanguage, SearchCriteria},
    },
};
use crate::shared::errors::AppResult;

/// Processes search results through a pipeline of operations
///
/// This service implements the Chain of Responsibility pattern for search result processing.
/// It handles deduplication, merging, ranking, and filtering in a clear, testable pipeline.
///
/// # Responsibilities:
/// - Deduplicate results from multiple providers by normalized title
/// - Merge data from multiple providers for the same anime
/// - Rank results by relevance using fuzzy matching
/// - Filter results by quality threshold
///
/// # Design Pattern: Chain of Responsibility
/// Each processing stage transforms the data and passes it to the next stage.
pub struct SearchResultsProcessor {
    quality_service: Arc<DataQualityService>,
}

impl SearchResultsProcessor {
    pub fn new(quality_service: Arc<DataQualityService>) -> Self {
        Self { quality_service }
    }

    /// Main processing pipeline
    ///
    /// Executes all processing stages in order:
    /// 1. Deduplicate by normalized title
    /// 2. Merge multi-provider data
    /// 3. Rank by relevance
    /// 4. Filter by quality
    /// 5. Apply limit
    pub async fn process(
        &self,
        provider_results: Vec<Vec<AnimeData>>,
        criteria: &SearchCriteria,
    ) -> AppResult<Vec<AnimeData>> {
        log::info!(
            "PROCESSOR: Starting pipeline with {} provider groups",
            provider_results.len()
        );

        // Stage 1: Deduplicate results
        let deduplicated = self.deduplicate_results(provider_results)?;
        log::info!("PROCESSOR: Deduplicated to {} groups", deduplicated.len());

        // Stage 2: Merge multi-provider data
        let merged = self.merge_results(deduplicated).await?;
        log::info!("PROCESSOR: Merged to {} results", merged.len());

        // Stage 3: Rank by relevance
        let ranked = self.rank_by_relevance(merged, &criteria.query, &criteria.preferred_language);
        log::info!("PROCESSOR: Ranked {} results", ranked.len());

        // Stage 4: Filter by quality threshold
        let filtered = self.filter_by_quality(ranked, criteria.quality_threshold)?;
        log::info!(
            "PROCESSOR: Filtered to {} high-quality results",
            filtered.len()
        );

        // Stage 5: Apply limit
        let mut final_results = filtered;
        final_results.truncate(criteria.limit);
        log::info!(
            "PROCESSOR: Final {} results after limit",
            final_results.len()
        );

        Ok(final_results)
    }

    /// Stage 1: Deduplicate results by normalized title
    ///
    /// Groups anime from different providers by normalized title similarity
    fn deduplicate_results(
        &self,
        provider_results: Vec<Vec<AnimeData>>,
    ) -> AppResult<HashMap<String, Vec<AnimeData>>> {
        let mut grouped: HashMap<String, Vec<AnimeData>> = HashMap::new();

        for results in provider_results {
            for anime_data in results {
                let key = self.normalize_title(&anime_data.anime.title.main);
                grouped.entry(key).or_insert_with(Vec::new).push(anime_data);
            }
        }

        Ok(grouped)
    }

    /// Stage 2: Merge results from multiple providers
    ///
    /// For each group of anime, either:
    /// - Return single result if only one provider has it
    /// - Merge data from multiple providers using quality service
    async fn merge_results(
        &self,
        grouped: HashMap<String, Vec<AnimeData>>,
    ) -> AppResult<Vec<AnimeData>> {
        let mut merged = Vec::new();

        for (title, group) in grouped {
            if group.len() == 1 {
                // Single provider - use as-is
                merged.push(group.into_iter().next().unwrap());
            } else {
                // Multiple providers - merge for better quality
                log::debug!(
                    "Merging {} provider results for anime: {}",
                    group.len(),
                    title
                );

                match self.quality_service.merge_anime_data(group.clone()) {
                    Ok(merged_anime) => {
                        log::info!(
                            "Successfully merged {} providers for '{}' - age_restriction: {:?}",
                            merged_anime.source.providers_used.len(),
                            merged_anime.anime.title.main,
                            merged_anime.anime.age_restriction
                        );
                        merged.push(merged_anime);
                    }
                    Err(e) => {
                        log::warn!("Failed to merge anime data for '{}': {}", title, e);
                        // Fallback: use highest quality data
                        let mut sorted_group = group;
                        sorted_group.sort_by(|a, b| {
                            b.quality
                                .score
                                .partial_cmp(&a.quality.score)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        });
                        merged.push(sorted_group.into_iter().next().unwrap());
                    }
                }
            }
        }

        Ok(merged)
    }

    /// Stage 3: Rank by relevance using fuzzy matching
    ///
    /// Calculates fuzzy similarity between search query and each anime title,
    /// then sorts by relevance score (highest first)
    fn rank_by_relevance(
        &self,
        mut results: Vec<AnimeData>,
        query: &str,
        preferred_language: &PreferredLanguage,
    ) -> Vec<AnimeData> {
        // Calculate fuzzy similarity for each result
        for anime_data in &mut results {
            let similarity =
                self.calculate_title_similarity(query, &anime_data.anime.title, preferred_language);
            anime_data.quality.relevance_score = (similarity * 100.0) as f32;

            log::debug!(
                "Relevance: '{}' -> {:.1}",
                anime_data.anime.title.main,
                anime_data.quality.relevance_score
            );
        }

        // Sort by relevance score (highest first), with quality as tiebreaker
        results.sort_by(|a, b| {
            let relevance_cmp = b
                .quality
                .relevance_score
                .partial_cmp(&a.quality.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal);

            if relevance_cmp != std::cmp::Ordering::Equal {
                relevance_cmp
            } else {
                // Tiebreaker: quality score
                b.quality
                    .score
                    .partial_cmp(&a.quality.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }
        });

        results
    }

    /// Stage 4: Filter by quality threshold
    ///
    /// Removes anime that don't meet the minimum quality score
    fn filter_by_quality(
        &self,
        results: Vec<AnimeData>,
        threshold: f32,
    ) -> AppResult<Vec<AnimeData>> {
        let filtered: Vec<AnimeData> = results
            .into_iter()
            .filter(|anime| anime.quality.score >= threshold)
            .collect();

        Ok(filtered)
    }

    // Helper methods

    /// Normalize title for grouping similar anime across providers
    fn normalize_title(&self, title: &str) -> String {
        let mut normalized = title.to_lowercase();

        // Remove common prefixes/suffixes that might differ between providers
        normalized = normalized
            .replace("(tv)", "")
            .replace("(movie)", "")
            .replace("(ova)", "")
            .replace("(ona)", "")
            .replace("(special)", "")
            .replace("season", "")
            .replace("part", "")
            .replace("cour", "");

        // Remove special characters and extra whitespace
        normalized = normalized
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ");

        // Handle complex titles
        let words: Vec<&str> = normalized.split_whitespace().collect();
        if words.len() > 4 {
            // Look for main anime names in complex titles
            let main_anime_keywords = [
                "naruto",
                "bleach",
                "one piece",
                "dragon ball",
                "attack on titan",
            ];

            for &keyword in &main_anime_keywords {
                if normalized.contains(keyword) {
                    return keyword.to_string();
                }
            }

            // Generic fallback: extract meaningful words
            let main_words: Vec<&str> = words
                .iter()
                .filter(|&&word| {
                    word.len() > 3
                        && ![
                            "next",
                            "generations",
                            "rope",
                            "boruto",
                            "shippuden",
                            "kai",
                            "brotherhood",
                        ]
                        .contains(&word)
                })
                .take(2)
                .copied()
                .collect();

            if !main_words.is_empty() {
                return main_words.join(" ");
            }
        }

        normalized
    }

    /// Calculate similarity between search query and anime title using fuzzy matching
    fn calculate_title_similarity(
        &self,
        search_query: &str,
        anime_title: &AnimeTitle,
        preferred_lang: &PreferredLanguage,
    ) -> f64 {
        let query_normalized = self.normalize_title(search_query);
        let comparison_titles = self.get_comparison_titles(anime_title, preferred_lang);

        let mut max_similarity: f64 = 0.0;

        for title in comparison_titles {
            let title_normalized = self.normalize_title(&title);

            // Use Jaro-Winkler for name matching (good for anime titles)
            let jw_similarity = jaro_winkler(&query_normalized, &title_normalized);

            // Use normalized Levenshtein as backup
            let lev_similarity = normalized_levenshtein(&query_normalized, &title_normalized);

            // Weighted combination (Jaro-Winkler is better for names)
            let combined_similarity = (jw_similarity * 0.7) + (lev_similarity * 0.3);

            max_similarity = max_similarity.max(combined_similarity);
        }

        max_similarity
    }

    /// Get comparison titles based on language preference
    fn get_comparison_titles(
        &self,
        anime_title: &AnimeTitle,
        preferred_lang: &PreferredLanguage,
    ) -> Vec<String> {
        let mut titles = vec![anime_title.main.clone()];

        // Prioritize based on language preference
        match preferred_lang {
            PreferredLanguage::English => {
                if let Some(english) = &anime_title.english {
                    titles.insert(0, english.clone());
                }
                if let Some(romaji) = &anime_title.romaji {
                    titles.push(romaji.clone());
                }
            }
            PreferredLanguage::Romaji => {
                if let Some(romaji) = &anime_title.romaji {
                    titles.insert(0, romaji.clone());
                }
                if let Some(english) = &anime_title.english {
                    titles.push(english.clone());
                }
            }
            PreferredLanguage::Japanese => {
                if let Some(native) = &anime_title.native {
                    titles.insert(0, native.clone());
                }
                if let Some(japanese) = &anime_title.japanese {
                    titles.push(japanese.clone());
                }
            }
            PreferredLanguage::Auto => {
                // Include all available titles
                if let Some(english) = &anime_title.english {
                    titles.push(english.clone());
                }
                if let Some(romaji) = &anime_title.romaji {
                    titles.push(romaji.clone());
                }
                if let Some(native) = &anime_title.native {
                    titles.push(native.clone());
                }
            }
        }

        // Add synonyms
        titles.extend(anime_title.synonyms.clone());

        // Remove empty titles and duplicates
        titles
            .into_iter()
            .filter(|t| !t.trim().is_empty())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_title() {
        let processor = SearchResultsProcessor::new(Arc::new(DataQualityService::new()));

        assert_eq!(
            processor.normalize_title("Naruto Shippuden (TV)"),
            "naruto shippuden"
        );
        assert_eq!(
            processor.normalize_title("Attack on Titan Season 2"),
            "attack on titan"
        );
        assert_eq!(
            processor.normalize_title("One Piece (Movie 14)"),
            "one piece"
        );
    }

    #[test]
    fn test_filter_by_quality() {
        let processor = SearchResultsProcessor::new(Arc::new(DataQualityService::new()));

        // Test filtering logic (would need actual AnimeData instances)
        // This is a placeholder for the test structure
    }
}
