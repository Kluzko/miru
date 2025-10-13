use chrono::Datelike;
use std::{sync::Arc, time::Instant};

use crate::{
    modules::{
        anime::domain::value_objects::AnimeTitle,
        provider::{
            domain::{
                entities::AnimeData,
                repositories::{AnimeProviderRepository, CacheRepository},
                value_objects::{PreferredLanguage, SearchCriteria},
            },
            AnimeProvider,
        },
    },
    shared::errors::AppResult,
};

use strsim::{jaro_winkler, normalized_levenshtein};

use crate::modules::anime::domain::services::data_quality_service::DataQualityService;

/// Clean anime search service using repository interfaces
pub struct AnimeSearchService {
    provider_repo: Arc<dyn AnimeProviderRepository>,
    cache_repo: Arc<dyn CacheRepository>,
    quality_service: DataQualityService,
}

impl AnimeSearchService {
    pub fn new(
        provider_repo: Arc<dyn AnimeProviderRepository>,
        cache_repo: Arc<dyn CacheRepository>,
        quality_service: DataQualityService,
    ) -> Self {
        Self {
            provider_repo,
            cache_repo,
            quality_service,
        }
    }

    /// Search for anime using intelligent provider selection with multi-provider data merging
    pub async fn search(
        &self,
        criteria: &SearchCriteria,
        available_providers: &[AnimeProvider],
    ) -> AppResult<Vec<AnimeData>> {
        criteria.validate()?;

        let start_time = Instant::now();
        let mut provider_results: Vec<Vec<AnimeData>> = Vec::new();
        let mut providers_tried = Vec::new();

        // Query ALL available providers to gather comprehensive data
        for &provider in &criteria.preferred_providers {
            if !available_providers.contains(&provider) {
                continue;
            }

            // Check if provider is available
            if !self.provider_repo.is_provider_available(&provider).await {
                continue;
            }

            providers_tried.push(provider);

            // Check cache first
            if let Some(cached_results) = self
                .cache_repo
                .get_search_results(&criteria.query, provider)
                .await
            {
                if !cached_results.is_empty() {
                    log::debug!(
                        "Cache hit for search '{}' with provider {:?}",
                        criteria.query,
                        provider
                    );
                    provider_results.push(cached_results);
                    continue;
                }
            }

            // Fetch from provider
            match self
                .provider_repo
                .search_anime(&criteria.query, criteria.limit, provider)
                .await
            {
                Ok(mut results) => {
                    log::debug!(
                        "Provider {:?} returned {} results for '{}'",
                        provider,
                        results.len(),
                        criteria.query
                    );

                    // Enhance with timing info
                    let fetch_time_ms = start_time.elapsed().as_millis() as u64;
                    for result in &mut results {
                        result.source.fetch_time_ms = fetch_time_ms;
                        result.source.providers_used = providers_tried.clone();
                    }

                    // Cache the results
                    self.cache_repo
                        .cache_search_results(&criteria.query, provider, results.clone())
                        .await;

                    // Rank results by relevance using fuzzy matching
                    let ranked_results = self.rank_results_by_fuzzy_relevance(
                        &criteria.query,
                        results,
                        &criteria.preferred_language,
                    );
                    provider_results.push(ranked_results);
                }
                Err(e) => {
                    log::warn!(
                        "Provider {:?} failed for search '{}': {}",
                        provider,
                        criteria.query,
                        e
                    );
                    continue;
                }
            }
        }

        // If no providers returned results, return empty
        if provider_results.is_empty() {
            return Ok(Vec::new());
        }

        // Merge results from multiple providers for better data quality
        log::info!(
            "SEARCH: Got {} provider result groups to merge",
            provider_results.len()
        );
        for (i, group) in provider_results.iter().enumerate() {
            log::info!("SEARCH: Provider group {} has {} results", i, group.len());
            for (j, anime) in group.iter().enumerate() {
                log::info!(
                    "SEARCH: Group {} Anime {}: '{}' age_restriction={:?}",
                    i,
                    j,
                    anime.anime.title.main,
                    anime.anime.age_restriction
                );
            }
        }
        let merged_results = self
            .merge_multi_provider_results(provider_results, criteria)
            .await?;
        log::info!(
            "SEARCH: After merging, got {} final results",
            merged_results.len()
        );

        // Filter by quality and return top results
        let quality_results = self.filter_by_quality(merged_results, criteria.quality_threshold)?;

        Ok(quality_results)
    }

    /// Get anime details with fallback across providers
    pub async fn get_details(
        &self,
        id: &str,
        preferred_provider: Option<AnimeProvider>,
        available_providers: &[AnimeProvider],
    ) -> AppResult<Option<AnimeData>> {
        let providers_to_try = if let Some(provider) = preferred_provider {
            vec![provider]
        } else {
            available_providers.to_vec()
        };

        for provider in providers_to_try {
            if !available_providers.contains(&provider) {
                continue;
            }

            // Check if provider is available
            if !self.provider_repo.is_provider_available(&provider).await {
                continue;
            }

            // Check cache first
            if let Some(cached_result) = self.cache_repo.get_anime_details(id, provider).await {
                log::debug!(
                    "Cache hit for anime details '{}' with provider {:?}",
                    id,
                    provider
                );
                return Ok(Some(cached_result));
            }

            // Fetch from provider
            match self.provider_repo.get_anime_by_id(id, provider).await {
                Ok(Some(anime_data)) => {
                    log::debug!("Provider {:?} returned details for '{}'", provider, id);

                    // Cache the result
                    self.cache_repo
                        .cache_anime_details(id, provider, anime_data.clone())
                        .await;

                    return Ok(Some(anime_data));
                }
                Ok(None) => {
                    log::debug!("Provider {:?} found no details for '{}'", provider, id);
                    continue;
                }
                Err(e) => {
                    log::warn!("Provider {:?} failed for details '{}': {}", provider, id, e);
                    continue;
                }
            }
        }

        Ok(None)
    }

    /// Enhance anime data by combining multiple provider sources
    pub async fn enhance_with_multiple_providers(
        &self,
        base_anime: AnimeData,
        additional_providers: &[AnimeProvider],
    ) -> AppResult<AnimeData> {
        let mut all_data = vec![base_anime.clone()];

        // Try to get the same anime from other providers for data enhancement
        let search_query = &base_anime.anime.title.main;

        for &provider in additional_providers {
            if provider == base_anime.source.primary_provider {
                continue; // Skip the provider we already have
            }

            if !self.provider_repo.is_provider_available(&provider).await {
                continue;
            }

            match self
                .provider_repo
                .search_anime(search_query, 1, provider)
                .await
            {
                Ok(results) if !results.is_empty() => {
                    // Take the first result as it should be most relevant
                    let mut additional_data = results.into_iter().next().unwrap();
                    additional_data.source.providers_used =
                        all_data[0].source.providers_used.clone();
                    additional_data.source.providers_used.push(provider);
                    all_data.push(additional_data);
                }
                _ => continue,
            }
        }

        if all_data.len() == 1 {
            // No additional data found
            return Ok(base_anime);
        }

        // Merge data from multiple providers using quality service
        self.quality_service.merge_anime_data(all_data)
    }

    /// Merge results from multiple providers to create comprehensive anime data
    async fn merge_multi_provider_results(
        &self,
        provider_results: Vec<Vec<AnimeData>>,
        criteria: &SearchCriteria,
    ) -> AppResult<Vec<AnimeData>> {
        use std::collections::HashMap;

        log::info!(
            "MERGE_FUNCTION: Starting merge with {} provider result groups",
            provider_results.len()
        );

        // Group anime by title similarity for merging
        let mut anime_groups: HashMap<String, Vec<AnimeData>> = HashMap::new();

        for results in provider_results {
            for anime_data in results {
                // Use normalized title as grouping key
                let group_key = self.normalize_title(&anime_data.anime.title.main);
                anime_groups
                    .entry(group_key)
                    .or_insert_with(Vec::new)
                    .push(anime_data);
            }
        }

        let mut merged_results = Vec::new();

        // Merge each group of similar anime
        for (title, group_data) in anime_groups {
            if group_data.len() == 1 {
                // Only one provider had this anime
                merged_results.push(group_data.into_iter().next().unwrap());
            } else {
                // Multiple providers have this anime - merge for better quality
                log::debug!(
                    "Merging {} provider results for anime: {}",
                    group_data.len(),
                    title
                );

                log::info!(
                    "MERGE_FUNCTION: Merging {} anime data for title '{}'",
                    group_data.len(),
                    title
                );
                match self.quality_service.merge_anime_data(group_data.clone()) {
                    Ok(merged_anime) => {
                        log::info!(
                            "MERGE_FUNCTION: Successfully merged anime data for '{}' from {} providers - final age_restriction: {:?}",
                            merged_anime.anime.title.main,
                            merged_anime.source.providers_used.len(),
                            merged_anime.anime.age_restriction
                        );
                        merged_results.push(merged_anime);
                    }
                    Err(e) => {
                        log::warn!("Failed to merge anime data for '{}': {}", title, e);
                        // Fall back to best quality data
                        let mut sorted_group = group_data;
                        sorted_group.sort_by(|a, b| {
                            b.quality
                                .score
                                .partial_cmp(&a.quality.score)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        });
                        merged_results.push(sorted_group.into_iter().next().unwrap());
                    }
                }
            }
        }

        // Sort by relevance score (most relevant first), then by quality as tiebreaker
        merged_results.sort_by(|a, b| {
            // Primary sort: relevance score (higher is better)
            let relevance_cmp = b
                .quality
                .relevance_score
                .partial_cmp(&a.quality.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal);

            if relevance_cmp != std::cmp::Ordering::Equal {
                relevance_cmp
            } else {
                // Tiebreaker: quality score (higher is better)
                b.quality
                    .score
                    .partial_cmp(&a.quality.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }
        });

        log::info!("FINAL SORT: Results ordered by relevance score");
        for (idx, result) in merged_results.iter().enumerate() {
            log::info!(
                "FINAL RANK #{}: '{}' (relevance: {:.1}, quality: {:.2})",
                idx + 1,
                result.anime.title.main,
                result.quality.relevance_score,
                result.quality.score
            );
        }
        merged_results.truncate(criteria.limit);

        Ok(merged_results)
    }

    /// Normalize title for grouping similar anime across providers
    fn normalize_title(&self, title: &str) -> String {
        let mut normalized = title.to_lowercase();

        log::info!("NORMALIZE: Original title: '{}'", title);

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

        log::info!("NORMALIZE: After basic cleanup: '{}'", normalized);

        // Handle specific problematic cases
        let words: Vec<&str> = normalized.split_whitespace().collect();

        // Case 1: If it's a complex spin-off title, extract the main series name
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
                    log::info!(
                        "NORMALIZE: Found main anime keyword '{}' in complex title",
                        keyword
                    );
                    return keyword.to_string();
                }
            }

            // Generic fallback: try to extract meaningful words
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
                let result = main_words.join(" ");
                log::info!("NORMALIZE: Extracted main words: '{}'", result);
                return result;
            }
        }

        log::info!("NORMALIZE: Final normalized title: '{}'", normalized);
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

        // Get titles to compare based on language preference
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

            log::debug!(
                "SIMILARITY: Query='{}' vs Title='{}' -> JW={:.3}, Lev={:.3}, Combined={:.3}",
                query_normalized,
                title_normalized,
                jw_similarity,
                lev_similarity,
                combined_similarity
            );

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
                    titles.insert(0, english.clone()); // Prioritize english
                }
                if let Some(romaji) = &anime_title.romaji {
                    titles.push(romaji.clone());
                }
            }
            PreferredLanguage::Romaji => {
                if let Some(romaji) = &anime_title.romaji {
                    titles.insert(0, romaji.clone()); // Prioritize romaji
                }
                if let Some(english) = &anime_title.english {
                    titles.push(english.clone());
                }
            }
            PreferredLanguage::Japanese => {
                if let Some(native) = &anime_title.native {
                    titles.insert(0, native.clone()); // Prioritize native
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

    /// Generate deduplication key based on similarity and metadata
    fn generate_dedup_key(
        &self,
        anime_data: &AnimeData,
        search_query: &str,
        criteria: &SearchCriteria,
    ) -> AppResult<String> {
        // Strategy 1: Use MAL ID if available from both providers
        if let Some(mal_id) = anime_data
            .anime
            .provider_metadata
            .external_ids
            .get(&AnimeProvider::Jikan)
        {
            return Ok(format!("mal_{}", mal_id));
        }

        // Strategy 2: High title similarity + same year for grouping
        let title_similarity = self.calculate_title_similarity(
            search_query,
            &anime_data.anime.title,
            &criteria.preferred_language,
        );

        if title_similarity > criteria.similarity_threshold {
            let year = anime_data
                .anime
                .aired
                .from
                .map(|date| date.year().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            let normalized_title = self.normalize_title(&anime_data.anime.title.main);
            return Ok(format!(
                "similar_{}_{}",
                normalized_title.replace(" ", "_"),
                year
            ));
        }

        // Strategy 3: Fallback to unique key per anime
        Ok(format!(
            "unique_{}_{}",
            anime_data
                .source
                .primary_provider
                .to_string()
                .to_lowercase(),
            anime_data
                .anime
                .provider_metadata
                .external_ids
                .values()
                .next()
                .unwrap_or(&anime_data.anime.id.to_string())
        ))
    }

    /// Rank search results by relevance using fuzzy matching
    fn rank_results_by_fuzzy_relevance(
        &self,
        query: &str,
        mut results: Vec<AnimeData>,
        preferred_language: &PreferredLanguage,
    ) -> Vec<AnimeData> {
        // Calculate fuzzy similarity scores for each result
        for anime_data in &mut results {
            let similarity_score =
                self.calculate_title_similarity(query, &anime_data.anime.title, preferred_language);

            // Convert similarity to relevance score (0-100 scale)
            let relevance_score = (similarity_score * 100.0) as f32;
            anime_data.quality.relevance_score = relevance_score;

            log::info!(
                "FUZZY RELEVANCE: Query='{}', Title='{}', Similarity={:.3}, Relevance={:.1}",
                query,
                anime_data.anime.title.main,
                similarity_score,
                relevance_score
            );
        }

        // Sort by relevance score (highest first)
        results.sort_by(|a, b| {
            b.quality
                .relevance_score
                .partial_cmp(&a.quality.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Log the ranking results
        for (idx, anime_data) in results.iter().enumerate() {
            log::info!(
                "FUZZY RANK #{}: '{}' (score: {:.1})",
                idx + 1,
                anime_data.anime.title.main,
                anime_data.quality.relevance_score
            );
        }

        results
    }

    /// Old rank search results by relevance to the query (kept for reference)
    fn rank_results_by_relevance(
        &self,
        query: &str,
        mut results: Vec<AnimeData>,
    ) -> Vec<AnimeData> {
        let query_normalized = query.to_lowercase();

        // Calculate relevance scores for each result
        for anime_data in &mut results {
            let title = anime_data.anime.title.main.to_lowercase();
            let mut score = 0.0f32;

            // Exact match gets highest score
            if title == query_normalized {
                score = 100.0;
            }
            // Title starts with query gets high score
            else if title.starts_with(&query_normalized) {
                score = 90.0;
            }
            // Query is contained in title
            else if title.contains(&query_normalized) {
                score = 80.0;
            }
            // Check individual words
            else {
                let query_words: Vec<&str> = query_normalized.split_whitespace().collect();
                let title_words: Vec<&str> = title.split_whitespace().collect();

                let matching_words = query_words
                    .iter()
                    .filter(|word| title_words.contains(word))
                    .count();

                if matching_words > 0 {
                    score = (matching_words as f32 / query_words.len() as f32) * 70.0;
                }
            }

            // Bonus for exact title length match (avoid spin-offs)
            let query_words = query_normalized.split_whitespace().count();
            let title_words = title.split_whitespace().count();
            if query_words == title_words {
                score += 10.0;
            }

            // Store the relevance score
            anime_data.quality.relevance_score = score;

            log::info!(
                "RELEVANCE: Query='{}', Title='{}', Score={:.1}",
                query,
                anime_data.anime.title.main,
                score
            );
        }

        // Sort by relevance score (highest first)
        results.sort_by(|a, b| {
            b.quality
                .relevance_score
                .partial_cmp(&a.quality.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }

    /// Filter results by quality threshold
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
}
