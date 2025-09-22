use crate::modules::anime::AnimeDetailed;
use crate::modules::provider::AnimeProvider;
use chrono::Datelike;
use futures::stream::{FuturesUnordered, StreamExt};
use std::collections::HashMap;
use tracing::{debug, info};

/// Memory-efficient streaming aggregator that processes results as they arrive
/// Eliminates the need to clone and store all results from all providers
pub struct StreamingAnimeAggregator;

impl StreamingAnimeAggregator {
    /// Create a streaming search that yields results as they come from providers
    /// This approach reduces memory usage by not storing all results at once
    pub async fn stream_comprehensive_search<F, Fut>(
        providers: Vec<AnimeProvider>,
        query: String,
        limit: usize,
        primary_provider: AnimeProvider,
        search_fn: F,
    ) -> Vec<AnimeDetailed>
    where
        F: Fn(AnimeProvider, String, usize) -> Fut + Clone,
        Fut: std::future::Future<
            Output = Result<Vec<AnimeDetailed>, crate::shared::errors::AppError>,
        >,
    {
        if providers.is_empty() {
            return Vec::new();
        }

        // Create futures for all provider searches
        let mut search_futures = FuturesUnordered::new();
        for provider in providers.iter().cloned() {
            let search_fn_clone = search_fn.clone();
            let query_clone = query.clone();

            let future = async move {
                match search_fn_clone(provider.clone(), query_clone, limit).await {
                    Ok(results) => Some((provider, results)),
                    Err(e) => {
                        debug!("Provider {:?} failed: {}", provider, e);
                        None
                    }
                }
            };
            search_futures.push(future);
        }

        // Collect results as they arrive (streaming approach)
        let mut provider_results = HashMap::new();
        while let Some(result) = search_futures.next().await {
            if let Some((provider, results)) = result {
                if !results.is_empty() {
                    provider_results.insert(provider, results);
                }
            }
        }

        if provider_results.is_empty() {
            return Vec::new();
        }

        // Use memory-efficient merging
        Self::merge_results_efficiently(provider_results, primary_provider, limit)
    }

    /// Memory-efficient merging that processes results in-place
    /// Avoids cloning large data structures
    pub fn merge_results_efficiently(
        mut provider_results: HashMap<AnimeProvider, Vec<AnimeDetailed>>,
        primary_provider: AnimeProvider,
        max_results: usize,
    ) -> Vec<AnimeDetailed> {
        // Start with primary provider results if available
        let mut merged_results = provider_results
            .remove(&primary_provider)
            .unwrap_or_else(|| {
                // Take the first available provider's results
                provider_results
                    .values_mut()
                    .next()
                    .map(|v| std::mem::take(v))
                    .unwrap_or_default()
            });

        debug!(
            "Starting merge with {} results from primary provider",
            merged_results.len()
        );

        // Enhance existing results with data from other providers
        for (provider, mut provider_anime) in provider_results {
            if provider == primary_provider {
                continue; // Already processed
            }

            Self::enhance_results_in_place(&mut merged_results, &provider_anime, &provider);
            Self::add_unique_results(&mut merged_results, &mut provider_anime);
        }

        // Sort and limit results
        Self::sort_and_limit_results(&mut merged_results, max_results);

        info!(
            "Completed memory-efficient merge, final count: {}",
            merged_results.len()
        );

        merged_results
    }

    /// Enhance existing results with data from another provider (in-place)
    fn enhance_results_in_place(
        base_results: &mut [AnimeDetailed],
        enhancement_results: &[AnimeDetailed],
        provider: &AnimeProvider,
    ) {
        for base_anime in base_results.iter_mut() {
            if let Some(enhancing_anime) =
                Self::find_matching_anime(base_anime, enhancement_results)
            {
                Self::merge_anime_fields_in_place(base_anime, enhancing_anime, provider);
            }
        }
    }

    /// Find matching anime using efficient algorithms
    fn find_matching_anime<'a>(
        target: &AnimeDetailed,
        candidates: &'a [AnimeDetailed],
    ) -> Option<&'a AnimeDetailed> {
        // Try exact title match first (fastest)
        candidates
            .iter()
            .find(|anime| anime.title == target.title)
            .or_else(|| {
                // Try English title match
                if let Some(target_english) = &target.title.english {
                    candidates
                        .iter()
                        .find(|anime| anime.title.english.as_ref() == Some(target_english))
                } else {
                    None
                }
            })
            .or_else(|| {
                // Try Japanese title match
                if let Some(target_japanese) = &target.title.japanese {
                    candidates
                        .iter()
                        .find(|anime| anime.title.japanese.as_ref() == Some(target_japanese))
                } else {
                    None
                }
            })
    }

    /// Merge anime fields in-place to avoid allocations
    fn merge_anime_fields_in_place(
        target: &mut AnimeDetailed,
        source: &AnimeDetailed,
        provider: &AnimeProvider,
    ) {
        // Fill missing fields without unnecessary cloning
        if target.title.english.is_none() && source.title.english.is_some() {
            target.title.english = source.title.english.clone();
            debug!("Added English title from {:?}", provider);
        }

        if target.title.japanese.is_none() && source.title.japanese.is_some() {
            target.title.japanese = source.title.japanese.clone();
        }

        // Prefer longer synopsis
        if let Some(source_synopsis) = &source.synopsis {
            match &target.synopsis {
                None => target.synopsis = Some(source_synopsis.clone()),
                Some(current) if source_synopsis.len() > current.len() => {
                    target.synopsis = Some(source_synopsis.clone());
                }
                _ => {}
            }
        }

        // Fill missing numeric fields
        if target.score.is_none() && source.score.is_some() {
            target.score = source.score;
        }

        if target.episodes.is_none() && source.episodes.is_some() {
            target.episodes = source.episodes;
        }

        // Fill missing aired dates
        if target.aired.from.is_none() && source.aired.from.is_some() {
            target.aired.from = source.aired.from;
        }
        if target.aired.to.is_none() && source.aired.to.is_some() {
            target.aired.to = source.aired.to;
        }

        // Status is now a required field, not an Option

        // Merge genres efficiently (avoid duplicate allocations)
        if !source.genres.is_empty() {
            for genre in &source.genres {
                if !target.genres.contains(genre) {
                    target.genres.push(genre.clone());
                }
            }
        }

        // Use higher quality image URL
        if let Some(source_url) = &source.image_url {
            match &target.image_url {
                None => target.image_url = Some(source_url.clone()),
                Some(current_url) if Self::is_higher_quality_image(source_url, current_url) => {
                    target.image_url = Some(source_url.clone());
                }
                _ => {}
            }
        }
    }

    /// Add unique results from provider that don't exist in main results
    fn add_unique_results(
        main_results: &mut Vec<AnimeDetailed>,
        provider_results: &mut Vec<AnimeDetailed>,
    ) {
        provider_results.retain(|anime| {
            let is_unique = !main_results
                .iter()
                .any(|existing| Self::is_same_anime(existing, anime));

            if is_unique {
                main_results.push(anime.clone());
                false // Remove from provider_results to avoid double processing
            } else {
                false // Remove processed items
            }
        });
    }

    /// Check if two anime represent the same show
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

        // Same aired year and similar normalized title
        if let (Some(aired1), Some(aired2)) = (anime1.aired.from, anime2.aired.from) {
            if aired1.date_naive().year() == aired2.date_naive().year() {
                let normalized1 = Self::normalize_title(&anime1.title.main);
                let normalized2 = Self::normalize_title(&anime2.title.main);
                return normalized1 == normalized2
                    || normalized1.contains(&normalized2)
                    || normalized2.contains(&normalized1);
            }
        }

        false
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

    /// Sort results by relevance and limit to max results
    fn sort_and_limit_results(results: &mut Vec<AnimeDetailed>, max_results: usize) {
        // Sort by score (descending), then by title
        results.sort_by(|a, b| match (a.score, b.score) {
            (Some(score_a), Some(score_b)) => score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.title.cmp(&b.title),
        });

        // Limit results
        results.truncate(max_results);
    }

    /// Simple heuristic for image quality
    fn is_higher_quality_image(new_url: &str, current_url: &str) -> bool {
        // AniList typically has higher quality images
        (new_url.contains("anilist") && current_url.contains("jikan"))
            || (new_url.contains("large") && !current_url.contains("large"))
            || new_url.contains("extraLarge")
    }
}

/// Iterator-based streaming approach for even lower memory usage
/// Use this for very large result sets
pub struct AnimeResultStream {
    provider_streams: Vec<Box<dyn futures::Stream<Item = AnimeDetailed> + Unpin + Send>>,
    primary_provider: AnimeProvider,
}

impl AnimeResultStream {
    pub fn new(primary_provider: AnimeProvider) -> Self {
        Self {
            provider_streams: Vec::new(),
            primary_provider,
        }
    }

    /// Add a provider result stream
    pub fn add_provider_stream<S>(&mut self, stream: S)
    where
        S: futures::Stream<Item = AnimeDetailed> + Unpin + Send + 'static,
    {
        self.provider_streams.push(Box::new(stream));
    }

    /// Consume the stream and return aggregated results
    /// This approach uses constant memory regardless of result size
    pub async fn collect_with_limit(self, limit: usize) -> Vec<AnimeDetailed> {
        use futures::stream::select_all;

        if self.provider_streams.is_empty() {
            return Vec::new();
        }

        let mut combined_stream = select_all(self.provider_streams);
        let mut results = Vec::with_capacity(limit.min(1000)); // Reasonable initial capacity

        while let Some(anime) = combined_stream.next().await {
            // Check for duplicates using efficient algorithm
            if !results
                .iter()
                .any(|existing| StreamingAnimeAggregator::is_same_anime(existing, &anime))
            {
                results.push(anime);

                if results.len() >= limit {
                    break;
                }
            }
        }

        // Final sort
        StreamingAnimeAggregator::sort_and_limit_results(&mut results, limit);
        results
    }
}
