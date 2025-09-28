//! AniList provider adapter
//!
//! GraphQL-based adapter for the AniList API that implements the same interface
//! as the Jikan adapter, providing comprehensive anime data retrieval capabilities.

use async_trait::async_trait;
use chrono::Datelike;
use serde_json::{json, Value};

use crate::{
    modules::provider::{
        domain::entities::anime_data::AnimeData,
        infrastructure::{
            adapters::{mapper::AnimeMapper, provider_repository_adapter::ProviderAdapter},
            http_client::RateLimitClient,
        },
        AnimeProvider,
    },
    shared::errors::{AppError, AppResult},
};

use super::{mapper::AniListMapper, models::*, queries::*};

/// AniList provider adapter with GraphQL API
pub struct AniListAdapter {
    http_client: RateLimitClient,
    base_url: String,
    mapper: AniListMapper,
}

impl AniListAdapter {
    /// Create a new AniList adapter with default settings
    pub fn new() -> Self {
        Self {
            http_client: RateLimitClient::for_anilist(),
            base_url: "https://graphql.anilist.co".to_string(),
            mapper: AniListMapper::new(),
        }
    }

    /// Check if a request can be made now (for testing)
    pub fn can_make_request_now(&self) -> bool {
        self.http_client.can_make_request_now()
    }

    /// Make a GraphQL request to AniList API
    async fn make_graphql_request<T>(&self, query: &str, variables: Option<Value>) -> AppResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let mut body = json!({
            "query": query
        });

        if let Some(vars) = variables {
            body["variables"] = vars;
        }

        // Use the modern HTTP client with built-in rate limiting and retry logic
        let graphql_response: Value = self.http_client.post_json(&self.base_url, &body).await?;

        // Check for GraphQL errors
        if let Some(errors) = graphql_response.get("errors") {
            return Err(AppError::ApiError(format!(
                "AniList GraphQL errors: {}",
                errors
            )));
        }

        // Extract the data field
        let data = graphql_response
            .get("data")
            .ok_or_else(|| AppError::ApiError("No data field in AniList response".to_string()))?;

        serde_json::from_value(data.clone()).map_err(|e| {
            AppError::SerializationError(format!("Failed to deserialize AniList data: {}", e))
        })
    }
}

#[async_trait]
impl ProviderAdapter for AniListAdapter {
    async fn search_anime(&self, query: &str, limit: usize) -> AppResult<Vec<AnimeData>> {
        let variables = json!({
            "search": query,
            "perPage": limit,
            "sort": "POPULARITY_DESC"
        });

        log::info!("AniList: Searching for '{}' (limit: {})", query, limit);

        let response: AniListSearchResponse = self
            .make_graphql_request(ANIME_SEARCH_QUERY, Some(variables))
            .await?;

        let anime_data: Result<Vec<_>, _> = response
            .page
            .media
            .into_iter()
            .map(|anime| self.mapper.map_to_anime_data(anime))
            .collect();

        let anime_data = anime_data
            .map_err(|e| AppError::MappingError(format!("Failed to map AniList data: {}", e)))?;

        log::info!(
            "AniList: Found {} results for '{}'",
            anime_data.len(),
            query
        );
        Ok(anime_data)
    }

    async fn get_anime_by_id(&self, id: &str) -> AppResult<Option<AnimeData>> {
        let anime_id: u32 = id
            .parse()
            .map_err(|_| AppError::ValidationError(format!("Invalid AniList ID: {}", id)))?;

        let variables = json!({
            "id": anime_id
        });

        log::info!("AniList: Getting anime by ID '{}'", id);

        let response: AniListMediaResponse = self
            .make_graphql_request(MEDIA_DETAIL_QUERY, Some(variables))
            .await?;

        if response.media.is_none() {
            log::info!("AniList: No anime found for ID '{}'", id);
            return Ok(None);
        }

        let anime_data = self
            .mapper
            .map_to_anime_data(response.media.unwrap())
            .map_err(|e| AppError::MappingError(format!("Failed to map AniList data: {}", e)))?;

        log::info!("AniList: Found anime by ID '{}'", id);
        Ok(Some(anime_data))
    }

    async fn get_anime(&self, id: u32) -> AppResult<Option<AnimeData>> {
        self.get_anime_by_id(&id.to_string()).await
    }

    async fn get_anime_full(&self, id: u32) -> AppResult<Option<AnimeData>> {
        let variables = json!({
            "id": id
        });

        log::info!(
            "AniList: Getting full anime details for ID '{}' (trait)",
            id
        );

        let response: AniListMediaResponse = self
            .make_graphql_request(MEDIA_DETAIL_QUERY, Some(variables))
            .await?;

        if let Some(anilist_media) = response.media {
            let anime_data = self.mapper.map_to_anime_data(anilist_media).map_err(|e| {
                AppError::MappingError(format!("Failed to map AniList data: {}", e))
            })?;
            log::info!(
                "AniList: Retrieved full details for anime ID '{}' (trait)",
                id
            );
            Ok(Some(anime_data))
        } else {
            log::info!("AniList: No anime found for ID '{}' (trait)", id);
            Ok(None)
        }
    }

    async fn search_anime_basic(&self, query: &str, limit: usize) -> AppResult<Vec<AnimeData>> {
        self.search_anime(query, limit).await
    }

    async fn get_season_now(&self, limit: usize) -> AppResult<Vec<AnimeData>> {
        // Calculate current season and year
        let now = chrono::Utc::now();
        let year = now.year();
        let season = match now.month() {
            1..=3 => "WINTER",
            4..=6 => "SPRING",
            7..=9 => "SUMMER",
            10..=12 => "FALL",
            _ => "FALL",
        };

        let variables = json!({
            "perPage": limit,
            "season": season,
            "seasonYear": year
        });

        log::info!(
            "AniList: Getting current season anime ({} {}) (trait)",
            season,
            year
        );

        let response: AniListSearchResponse = self
            .make_graphql_request(SEASONAL_ANIME_QUERY, Some(variables))
            .await?;

        let anime_list = response.page.media;
        let anime_data: Result<Vec<_>, _> = anime_list
            .into_iter()
            .map(|anime| self.mapper.map_to_anime_data(anime))
            .collect();
        anime_data.map_err(|e| AppError::MappingError(format!("Failed to map AniList data: {}", e)))
    }

    async fn get_season_upcoming(&self, limit: usize) -> AppResult<Vec<AnimeData>> {
        let variables = json!({
            "perPage": limit,
            "status": "NOT_YET_RELEASED"
        });

        log::info!("AniList: Getting upcoming anime (trait)");

        let response: AniListSearchResponse = self
            .make_graphql_request(ANIME_SEARCH_QUERY, Some(variables))
            .await?;

        let anime_list = response.page.media;
        let anime_data: Result<Vec<_>, _> = anime_list
            .into_iter()
            .map(|anime| self.mapper.map_to_anime_data(anime))
            .collect();
        anime_data.map_err(|e| AppError::MappingError(format!("Failed to map AniList data: {}", e)))
    }

    fn get_provider_type(&self) -> AnimeProvider {
        AnimeProvider::AniList
    }

    fn can_make_request_now(&self) -> bool {
        self.can_make_request_now()
    }
}

// Additional AniList-specific functions following the same pattern as Jikan
impl AniListAdapter {
    // =============================================================================
    // CORE ANIME FUNCTIONS
    // =============================================================================

    /// Get anime by ID (basic information)
    pub async fn get_anime(&self, id: u32) -> AppResult<Option<AniListMedia>> {
        let variables = json!({
            "id": id
        });

        log::info!("AniList: Getting anime by ID '{}'", id);

        let response: AniListMediaResponse = self
            .make_graphql_request(MEDIA_DETAIL_QUERY, Some(variables))
            .await?;

        if response.media.is_some() {
            log::info!("AniList: Found anime by ID '{}'", id);
        } else {
            log::info!("AniList: No anime found for ID '{}'", id);
        }
        Ok(response.media)
    }

    /// Get anime with full details
    pub async fn get_anime_full(&self, id: u32) -> AppResult<Option<AniListMedia>> {
        let variables = json!({
            "id": id
        });

        log::info!("AniList: Getting full anime details for ID '{}'", id);

        let response: AniListMediaResponse = self
            .make_graphql_request(MEDIA_DETAIL_QUERY, Some(variables))
            .await?;

        log::info!("AniList: Retrieved full details for anime ID '{}'", id);
        Ok(response.media)
    }

    /// Get anime characters
    pub async fn get_anime_characters(
        &self,
        id: u32,
        limit: usize,
    ) -> AppResult<Vec<AniListCharacter>> {
        let variables = json!({
            "id": id,
            "perPage": limit
        });

        log::info!("AniList: Getting characters for anime ID '{}'", id);

        let response: AniListCharactersResponse = self
            .make_graphql_request(ANIME_CHARACTERS_QUERY, Some(variables))
            .await?;

        let characters = response
            .media
            .map(|m| m.characters.nodes)
            .unwrap_or_default();
        log::info!(
            "AniList: Found {} characters for anime ID '{}'",
            characters.len(),
            id
        );
        Ok(characters)
    }

    /// Get anime staff
    pub async fn get_anime_staff(&self, id: u32, limit: usize) -> AppResult<Vec<AniListStaff>> {
        let variables = json!({
            "id": id,
            "perPage": limit
        });

        log::info!("AniList: Getting staff for anime ID '{}'", id);

        let response: AniListStaffResponse = self
            .make_graphql_request(ANIME_STAFF_QUERY, Some(variables))
            .await?;

        let staff = response.media.map(|m| m.staff.nodes).unwrap_or_default();
        log::info!(
            "AniList: Found {} staff members for anime ID '{}'",
            staff.len(),
            id
        );
        Ok(staff)
    }

    /// Get anime statistics
    pub async fn get_anime_statistics(&self, id: u32) -> AppResult<AniListStatistics> {
        let variables = json!({
            "id": id
        });

        log::info!("AniList: Getting statistics for anime ID '{}'", id);

        let response: AniListStatisticsResponse = self
            .make_graphql_request(ANIME_STATISTICS_QUERY, Some(variables))
            .await?;

        log::info!("AniList: Retrieved statistics for anime ID '{}'", id);
        Ok(response.media.map(|m| m.stats).unwrap_or_default())
    }

    /// Get anime recommendations
    pub async fn get_anime_recommendations(
        &self,
        id: u32,
        limit: usize,
    ) -> AppResult<Vec<AniListRecommendation>> {
        let variables = json!({
            "id": id,
            "perPage": limit
        });

        log::info!("AniList: Getting recommendations for anime ID '{}'", id);

        let response: AniListRecommendationsResponse = self
            .make_graphql_request(ANIME_RECOMMENDATIONS_QUERY, Some(variables))
            .await?;

        let recommendations = response
            .media
            .map(|m| m.recommendations.nodes)
            .unwrap_or_default();
        log::info!(
            "AniList: Found {} recommendations for anime ID '{}'",
            recommendations.len(),
            id
        );
        Ok(recommendations)
    }

    /// Get related anime
    pub async fn get_anime_relations(
        &self,
        id: u32,
        limit: usize,
    ) -> AppResult<Vec<AniListRelation>> {
        let variables = json!({
            "id": id,
            "perPage": limit
        });

        log::info!("AniList: Getting relations for anime ID '{}'", id);

        let response: AniListRelationsResponse = self
            .make_graphql_request(ANIME_RELATIONS_QUERY, Some(variables))
            .await?;

        let relations = response
            .media
            .map(|m| m.relations.nodes)
            .unwrap_or_default();
        log::info!(
            "AniList: Found {} relation groups for anime ID '{}'",
            relations.len(),
            id
        );
        Ok(relations)
    }

    // =============================================================================
    // SEARCH FUNCTIONS
    // =============================================================================

    /// Search anime with basic parameters (existing method, kept for compatibility)
    pub async fn search_anime_basic(
        &self,
        query: &str,
        limit: usize,
    ) -> AppResult<Vec<AniListMedia>> {
        let variables = json!({
            "search": query,
            "perPage": limit,
            "sort": "POPULARITY_DESC"
        });

        log::info!("AniList: Basic search for '{}' (limit: {})", query, limit);

        let response: AniListSearchResponse = self
            .make_graphql_request(ANIME_SEARCH_QUERY, Some(variables))
            .await?;

        log::info!(
            "AniList: Basic search found {} results",
            response.page.media.len()
        );
        Ok(response.page.media)
    }

    /// Advanced anime search with filters
    pub async fn search_anime_advanced(
        &self,
        params: AniListSearchParams,
    ) -> AppResult<AniListSearchResponse> {
        let variables = params.to_json();

        log::info!("AniList: Advanced search with parameters");

        let response: AniListSearchResponse = self
            .make_graphql_request(ANIME_SEARCH_ADVANCED_QUERY, Some(variables))
            .await?;

        log::info!(
            "AniList: Advanced search found {} results",
            response.page.media.len()
        );
        Ok(response)
    }

    // =============================================================================
    // SEASONAL & DISCOVERY FUNCTIONS
    // =============================================================================

    /// Get current season anime
    pub async fn get_season_now(&self, limit: usize) -> AppResult<Vec<AniListMedia>> {
        // Calculate current season and year
        let now = chrono::Utc::now();
        let year = now.year();
        let season = match now.month() {
            1..=3 => "WINTER",
            4..=6 => "SPRING",
            7..=9 => "SUMMER",
            10..=12 => "FALL",
            _ => "FALL",
        };

        let variables = json!({
            "perPage": limit,
            "season": season,
            "seasonYear": year
        });

        log::info!(
            "AniList: Getting current season anime ({} {})",
            season,
            year
        );

        let response: AniListSearchResponse = self
            .make_graphql_request(SEASONAL_ANIME_QUERY, Some(variables))
            .await?;

        log::info!(
            "AniList: Found {} current season anime",
            response.page.media.len()
        );
        Ok(response.page.media)
    }

    /// Get anime from specific season
    pub async fn get_season(
        &self,
        year: u32,
        season: &str,
        limit: usize,
        page: Option<u32>,
    ) -> AppResult<AniListSearchResponse> {
        let variables = json!({
            "perPage": limit,
            "page": page.unwrap_or(1),
            "season": season.to_uppercase(),
            "seasonYear": year
        });

        log::info!("AniList: Getting {} {} season anime", season, year);

        let response: AniListSearchResponse = self
            .make_graphql_request(SEASONAL_ANIME_QUERY, Some(variables))
            .await?;

        log::info!(
            "AniList: Found {} anime for {} {}",
            response.page.media.len(),
            season,
            year
        );
        Ok(response)
    }

    /// Get upcoming anime
    pub async fn get_season_upcoming(&self, limit: usize) -> AppResult<Vec<AniListMedia>> {
        let variables = json!({
            "perPage": limit,
            "status": "NOT_YET_RELEASED"
        });

        log::info!("AniList: Getting upcoming anime");

        let response: AniListSearchResponse = self
            .make_graphql_request(ANIME_SEARCH_QUERY, Some(variables))
            .await?;

        log::info!(
            "AniList: Found {} upcoming anime",
            response.page.media.len()
        );
        Ok(response.page.media)
    }

    /// Get anime broadcast schedule
    pub async fn get_schedules(&self, limit: usize) -> AppResult<Vec<AniListSchedule>> {
        let now = chrono::Utc::now().timestamp();
        let tomorrow = now + 86400; // 24 hours later

        let variables = json!({
            "perPage": limit,
            "airingAt_greater": now,
            "airingAt_lesser": tomorrow
        });

        log::info!("AniList: Getting broadcast schedule");

        let response: AniListScheduleResponse = self
            .make_graphql_request(AIRING_SCHEDULE_QUERY, Some(variables))
            .await?;

        let schedules = response.page.airing_schedules.unwrap_or_default();
        log::info!("AniList: Found {} scheduled anime", schedules.len());
        Ok(schedules)
    }

    /// Get trending anime
    pub async fn get_trending(&self, limit: usize) -> AppResult<Vec<AniListMedia>> {
        let variables = json!({
            "perPage": limit,
            "sort": "TRENDING_DESC"
        });

        log::info!("AniList: Getting trending anime");

        let response: AniListSearchResponse = self
            .make_graphql_request(ANIME_SEARCH_QUERY, Some(variables))
            .await?;

        log::info!(
            "AniList: Found {} trending anime",
            response.page.media.len()
        );
        Ok(response.page.media)
    }

    /// Get popular anime
    pub async fn get_popular(&self, limit: usize) -> AppResult<Vec<AniListMedia>> {
        let variables = json!({
            "perPage": limit,
            "sort": "POPULARITY_DESC"
        });

        log::info!("AniList: Getting popular anime");

        let response: AniListSearchResponse = self
            .make_graphql_request(ANIME_SEARCH_QUERY, Some(variables))
            .await?;

        log::info!("AniList: Found {} popular anime", response.page.media.len());
        Ok(response.page.media)
    }
}
