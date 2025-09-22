use crate::modules::anime::AnimeDetailed;
use crate::modules::provider::AnimeProvider;
use chrono::Datelike;
use std::collections::HashMap;
use tracing::{debug, info};

/// Service for aggregating anime data from multiple providers
/// Fills gaps in data by combining information from different sources
pub struct AnimeDataAggregator;

impl AnimeDataAggregator {
    /// Merge anime data from multiple providers to create the most complete record
    /// Uses a priority-based approach where primary provider data takes precedence
    pub fn merge_anime_data(
        results: HashMap<AnimeProvider, Vec<AnimeDetailed>>,
        primary_provider: AnimeProvider,
    ) -> Vec<AnimeDetailed> {
        if results.is_empty() {
            return Vec::new();
        }

        // Start with primary provider results if available
        let mut merged_results = if let Some(primary_results) = results.get(&primary_provider) {
            debug!(
                "Using {:?} as primary provider with {} results",
                primary_provider,
                primary_results.len()
            );
            primary_results.clone()
        } else {
            // Fallback to first available provider's results
            let (fallback_provider, fallback_results) = results.iter().next().unwrap();
            debug!(
                "Primary provider not available, using {:?} as fallback with {} results",
                fallback_provider,
                fallback_results.len()
            );
            fallback_results.clone()
        };

        // Enhance each anime with data from other providers
        for anime in &mut merged_results {
            Self::enhance_anime_with_additional_data(anime, &results, &primary_provider);
        }

        // Add unique anime from other providers that weren't in primary
        Self::add_unique_anime_from_other_providers(
            &mut merged_results,
            &results,
            &primary_provider,
        );

        info!(
            "Merged anime data from {} providers, final count: {}",
            results.len(),
            merged_results.len()
        );

        merged_results
    }

    /// Enhance a single anime record with data from additional providers
    fn enhance_anime_with_additional_data(
        anime: &mut AnimeDetailed,
        all_results: &HashMap<AnimeProvider, Vec<AnimeDetailed>>,
        primary_provider: &AnimeProvider,
    ) {
        for (provider, provider_results) in all_results {
            if provider == primary_provider {
                continue; // Skip primary provider as it's already the base
            }

            // Find matching anime in this provider's results
            if let Some(matching_anime) = Self::find_matching_anime(anime, provider_results) {
                Self::merge_anime_fields(anime, matching_anime, provider);
            }
        }
    }

    /// Find a matching anime in a provider's results using multiple matching strategies
    fn find_matching_anime<'a>(
        target: &AnimeDetailed,
        provider_results: &'a [AnimeDetailed],
    ) -> Option<&'a AnimeDetailed> {
        // Strategy 1: Exact title match (most reliable)
        for anime in provider_results {
            if anime.title == target.title {
                return Some(anime);
            }
        }

        // Strategy 2: English title match
        for anime in provider_results {
            if let (Some(target_english), Some(anime_english)) =
                (&target.title.english, &anime.title.english)
            {
                if target_english == anime_english {
                    return Some(anime);
                }
            }
        }

        // Strategy 3: Japanese title match
        for anime in provider_results {
            if let (Some(target_japanese), Some(anime_japanese)) =
                (&target.title.japanese, &anime.title.japanese)
            {
                if target_japanese == anime_japanese {
                    return Some(anime);
                }
            }
        }

        // Strategy 4: Similar title (fuzzy matching)
        for anime in provider_results {
            if Self::is_similar_title(&target.title.main, &anime.title.main) {
                return Some(anime);
            }
        }

        None
    }

    /// Simple fuzzy title matching
    fn is_similar_title(title1: &str, title2: &str) -> bool {
        let normalized1 = Self::normalize_title(title1);
        let normalized2 = Self::normalize_title(title2);

        // Check if one title contains the other (for cases like "Attack on Titan" vs "Shingeki no Kyojin")
        normalized1.contains(&normalized2) || normalized2.contains(&normalized1)
    }

    /// Normalize title for comparison
    fn normalize_title(title: &str) -> String {
        title
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Merge fields from secondary anime into primary anime (non-destructive)
    fn merge_anime_fields(
        primary: &mut AnimeDetailed,
        secondary: &AnimeDetailed,
        provider: &AnimeProvider,
    ) {
        debug!(
            "Enhancing anime '{}' with data from {:?}",
            primary.title, provider
        );

        // Fill missing English title
        if primary.title.english.is_none() && secondary.title.english.is_some() {
            primary.title.english = secondary.title.english.clone();
            debug!("Added English title from {:?}", provider);
        }

        // Fill missing Japanese title
        if primary.title.japanese.is_none() && secondary.title.japanese.is_some() {
            primary.title.japanese = secondary.title.japanese.clone();
            debug!("Added Japanese title from {:?}", provider);
        }

        // Fill missing synopsis (prefer longer descriptions)
        if secondary.synopsis.is_some() {
            match (&primary.synopsis, &secondary.synopsis) {
                (None, Some(sec_synopsis)) => {
                    primary.synopsis = Some(sec_synopsis.clone());
                    debug!("Added synopsis from {:?}", provider);
                }
                (Some(prim_synopsis), Some(sec_synopsis))
                    if sec_synopsis.len() > prim_synopsis.len() =>
                {
                    primary.synopsis = Some(sec_synopsis.clone());
                    debug!("Replaced synopsis with longer version from {:?}", provider);
                }
                _ => {}
            }
        }

        // Fill missing score (prefer higher scores if reasonable)
        if primary.score.is_none() && secondary.score.is_some() {
            primary.score = secondary.score;
            debug!("Added score from {:?}", provider);
        }

        // Fill missing episode count
        if primary.episodes.is_none() && secondary.episodes.is_some() {
            primary.episodes = secondary.episodes;
            debug!("Added episode count from {:?}", provider);
        }

        // Fill missing aired dates
        if primary.aired.from.is_none() && secondary.aired.from.is_some() {
            primary.aired.from = secondary.aired.from;
            debug!("Added aired start date from {:?}", provider);
        }
        if primary.aired.to.is_none() && secondary.aired.to.is_some() {
            primary.aired.to = secondary.aired.to;
            debug!("Added aired end date from {:?}", provider);
        }

        // Status is now a required field, not an Option

        // Merge genres (combine unique genres)
        if !secondary.genres.is_empty() {
            let mut combined_genres = primary.genres.clone();
            for genre in &secondary.genres {
                if !combined_genres.contains(genre) {
                    combined_genres.push(genre.clone());
                }
            }
            if combined_genres.len() > primary.genres.len() {
                primary.genres = combined_genres;
                debug!(
                    "Added {} new genres from {:?}",
                    primary.genres.len() - secondary.genres.len(),
                    provider
                );
            }
        }

        // Fill missing image URL (prefer higher quality)
        if secondary.image_url.is_some() {
            match (&primary.image_url, &secondary.image_url) {
                (None, Some(sec_url)) => {
                    primary.image_url = Some(sec_url.clone());
                    debug!("Added image URL from {:?}", provider);
                }
                (Some(prim_url), Some(sec_url))
                    if Self::is_higher_quality_image(sec_url, prim_url) =>
                {
                    primary.image_url = Some(sec_url.clone());
                    debug!("Replaced image URL with higher quality from {:?}", provider);
                }
                _ => {}
            }
        }
    }

    /// Simple heuristic to determine if an image URL represents higher quality
    fn is_higher_quality_image(new_url: &str, current_url: &str) -> bool {
        // AniList typically has higher quality images than Jikan
        new_url.contains("anilist") && current_url.contains("jikan")
            || new_url.contains("large") && !current_url.contains("large")
            || new_url.contains("extraLarge")
    }

    /// Add unique anime from other providers that weren't found in primary provider
    fn add_unique_anime_from_other_providers(
        merged_results: &mut Vec<AnimeDetailed>,
        all_results: &HashMap<AnimeProvider, Vec<AnimeDetailed>>,
        primary_provider: &AnimeProvider,
    ) {
        for (provider, provider_results) in all_results {
            if provider == primary_provider {
                continue;
            }

            for anime in provider_results {
                // Check if this anime is already in our merged results
                let already_exists = merged_results
                    .iter()
                    .any(|existing| Self::is_same_anime(existing, anime));

                if !already_exists {
                    merged_results.push(anime.clone());
                    debug!("Added unique anime '{}' from {:?}", anime.title, provider);
                }
            }
        }
    }

    /// Check if two anime records represent the same anime
    fn is_same_anime(anime1: &AnimeDetailed, anime2: &AnimeDetailed) -> bool {
        // Exact title match
        if anime1.title == anime2.title {
            return true;
        }

        // English title match
        if let (Some(title1), Some(title2)) = (&anime1.title.english, &anime2.title.english) {
            if title1 == title2 {
                return true;
            }
        }

        // Japanese title match
        if let (Some(title1), Some(title2)) = (&anime1.title.japanese, &anime2.title.japanese) {
            if title1 == title2 {
                return true;
            }
        }

        // Similar title with same aired year (strong indicator)
        if let (Some(aired1), Some(aired2)) = (anime1.aired.from, anime2.aired.from) {
            if aired1.date_naive().year() == aired2.date_naive().year()
                && Self::is_similar_title(&anime1.title.main, &anime2.title.main)
            {
                return true;
            }
        }

        false
    }

    /// Create a comprehensive search result by searching multiple providers
    pub fn create_comprehensive_search_results(
        provider_results: HashMap<AnimeProvider, Vec<AnimeDetailed>>,
        primary_provider: AnimeProvider,
        max_results: usize,
    ) -> Vec<AnimeDetailed> {
        let merged = Self::merge_anime_data(provider_results, primary_provider);

        // Sort by relevance (score, popularity, etc.)
        let mut sorted_results = merged;
        sorted_results.sort_by(|a, b| {
            // Sort by score first (if available), then by title
            match (a.score, b.score) {
                (Some(score_a), Some(score_b)) => score_b
                    .partial_cmp(&score_a)
                    .unwrap_or(std::cmp::Ordering::Equal),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => a.title.cmp(&b.title),
            }
        });

        // Limit results
        sorted_results.truncate(max_results);
        sorted_results
    }
}
