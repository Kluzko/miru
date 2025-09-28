use super::super::domain::{
    entities::anime_detailed::AnimeDetailed, repositories::anime_repository::AnimeRepository,
    services::score_calculator::ScoreCalculator,
};
use crate::modules::provider::{AnimeProvider, ProviderService};
use crate::shared::errors::AppResult;
use crate::shared::utils::logger::LogContext;
use crate::{log_debug, log_info};
use std::sync::Arc;
use uuid::Uuid;

pub struct AnimeService {
    anime_repo: Arc<dyn AnimeRepository>,
    provider_service: Arc<ProviderService>,
    #[allow(dead_code)]
    score_calculator: Arc<ScoreCalculator>,
}

impl AnimeService {
    pub fn new(
        anime_repo: Arc<dyn AnimeRepository>,
        provider_service: Arc<ProviderService>,
    ) -> Self {
        Self {
            anime_repo,
            provider_service,
            score_calculator: Arc::new(ScoreCalculator::new()),
        }
    }

    pub async fn search_anime(&self, query: &str) -> AppResult<Vec<AnimeDetailed>> {
        // Use comprehensive search which aggregates data from multiple providers
        let comprehensive_results = self.provider_service.search_anime(query, 20).await?;

        if !comprehensive_results.is_empty() {
            // Save new anime to database (the repository will handle duplicates)
            for anime in &comprehensive_results {
                match self.anime_repo.save(anime).await {
                    Ok(_) => log_info!(
                        "Successfully saved anime from comprehensive search: {}",
                        anime.title.main
                    ),
                    Err(e) => LogContext::error_with_context(
                        &e,
                        &format!("Failed to save anime {}", anime.title.main),
                    ),
                }
            }

            log_info!(
                "Comprehensive search found {} results for '{}'",
                comprehensive_results.len(),
                query
            );
            return Ok(comprehensive_results);
        }

        // If comprehensive search fails, fall back to database search
        let db_results = self.anime_repo.search(query, 20).await?;
        log_debug!(
            "Comprehensive search failed, falling back to database: {} results",
            db_results.len()
        );
        Ok(db_results)
    }

    pub async fn get_anime_by_id(&self, id: &Uuid) -> AppResult<Option<AnimeDetailed>> {
        self.anime_repo.find_by_id(id).await
    }

    pub async fn get_top_anime(&self, limit: usize) -> AppResult<Vec<AnimeDetailed>> {
        // Always fetch fresh data via provider service for top anime
        let anime_list = self.provider_service.search_anime("popular", limit).await?;

        // Save to database (handling duplicates)
        let mut saved_anime = Vec::new();
        for anime in anime_list {
            match self.anime_repo.save(&anime).await {
                Ok(saved) => saved_anime.push(saved),
                Err(e) => {
                    // Log but don't fail the entire operation
                    LogContext::error_with_context(
                        &e,
                        &format!("Failed to save anime {}", anime.title),
                    );
                    // Still include the anime in results even if save failed
                    saved_anime.push(anime);
                }
            }
        }

        Ok(saved_anime)
    }

    pub async fn get_seasonal_anime(
        &self,
        year: i32,
        season: &str,
        limit: usize,
    ) -> AppResult<Vec<AnimeDetailed>> {
        // Fetch via provider service
        let query = format!("{} {} anime", season, year);
        let anime_list = self.provider_service.search_anime(&query, limit).await?;

        // Save to database (handling duplicates)
        let mut saved_anime = Vec::new();
        for anime in anime_list {
            match self.anime_repo.save(&anime).await {
                Ok(saved) => saved_anime.push(saved),
                Err(e) => {
                    // Log but don't fail the entire operation
                    LogContext::error_with_context(
                        &e,
                        &format!("Failed to save anime {}", anime.title),
                    );
                    // Still include the anime in results even if save failed
                    saved_anime.push(anime);
                }
            }
        }

        Ok(saved_anime)
    }

    /// Create new anime with proper score calculation
    pub async fn create_anime(&self, anime: &AnimeDetailed) -> AppResult<AnimeDetailed> {
        // Calculate scores before saving
        let mut new_anime = anime.clone();
        new_anime.update_scores(&self.score_calculator);

        self.anime_repo.save(&new_anime).await
    }

    #[allow(dead_code)]
    pub async fn update_anime(&self, anime: &AnimeDetailed) -> AppResult<AnimeDetailed> {
        // Recalculate scores before saving
        let mut updated_anime = anime.clone();
        updated_anime.update_scores(&self.score_calculator);

        self.anime_repo.update(&updated_anime).await
    }

    #[allow(dead_code)]
    pub async fn delete_anime(&self, id: &Uuid) -> AppResult<()> {
        self.anime_repo.delete(id).await
    }

    /// External-only search without database interaction
    /// Use when you want fresh external data without saving to DB
    pub async fn search_anime_external_only(
        &self,
        query: &str,
        limit: usize,
    ) -> AppResult<Vec<AnimeDetailed>> {
        log_debug!("External-only search for '{}' with limit {}", query, limit);

        let results = self.provider_service.search_anime(query, limit).await?;

        log_info!(
            "External-only search found {} results for '{}'",
            results.len(),
            query
        );

        Ok(results)
    }

    /// Get anime by external provider ID (e.g., AniList ID, MAL ID)
    /// Use when you have a specific provider ID and want comprehensive data
    pub async fn get_anime_by_external_id(
        &self,
        id: &str,
        preferred_provider: Option<AnimeProvider>,
    ) -> AppResult<Option<AnimeDetailed>> {
        log_debug!(
            "Getting anime by external ID '{}' with provider {:?}",
            id,
            preferred_provider
        );

        let result = self
            .provider_service
            .get_anime_by_id(id, preferred_provider.unwrap_or(AnimeProvider::Jikan))
            .await?;

        match &result {
            Some(anime) => log_info!("Found anime by external ID '{}': {}", id, anime.title.main),
            None => log_debug!("No anime found for external ID '{}'", id),
        }

        Ok(result)
    }
}
