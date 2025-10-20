use crate::{
    modules::provider::domain::entities::anime_data::AnimeData,
    modules::provider::infrastructure::http_client::RateLimitClient,
    shared::errors::{AppError, AppResult},
};

use super::mapper::JikanMapper;
use super::models::*;

/// Jikan (MyAnimeList) provider adapter with REST API
pub struct JikanAdapter {
    http_client: RateLimitClient,
    base_url: String,
    mapper: JikanMapper,
}

impl JikanAdapter {
    pub fn new() -> Self {
        Self {
            http_client: RateLimitClient::for_jikan(),
            base_url: "https://api.jikan.moe/v4".to_string(),
            mapper: JikanMapper::new(),
        }
    }

    /// Create adapter with custom HTTP client (for testing)
    pub fn with_client(http_client: RateLimitClient) -> Self {
        Self {
            http_client,
            base_url: "https://api.jikan.moe/v4".to_string(),
            mapper: JikanMapper::new(),
        }
    }

    /// Check if a request can be made immediately (for testing and monitoring)
    pub fn can_make_request_now(&self) -> bool {
        self.http_client.can_make_request_now()
    }
}

impl JikanAdapter {
    pub async fn search_anime(&self, query: &str, limit: usize) -> AppResult<Vec<AnimeData>> {
        let url = format!(
            "{}/anime?q={}&limit={}",
            self.base_url,
            urlencoding::encode(query),
            limit
        );

        log::info!("Jikan: Searching for '{}' (limit: {})", query, limit);

        let jikan_response: JikanList<Anime> = self.http_client.get(&url).await?;

        let anime_data: Result<Vec<_>, _> = jikan_response
            .data
            .into_iter()
            .map(|anime| self.mapper.map_to_anime_data(anime))
            .collect();

        let anime_data = anime_data
            .map_err(|e| AppError::MappingError(format!("Failed to map Jikan data: {}", e)))?;

        log::info!("Jikan: Found {} results for '{}'", anime_data.len(), query);
        Ok(anime_data)
    }

    pub async fn get_anime_by_id(&self, id: &str) -> AppResult<Option<AnimeData>> {
        let anime_id: u32 = id
            .parse()
            .map_err(|_| AppError::ValidationError(format!("Invalid MAL ID: {}", id)))?;

        let url = format!("{}/anime/{}", self.base_url, anime_id);

        log::info!("Jikan: Getting anime by ID '{}'", id);

        // Use new intelligent HTTP client with retry logic
        let jikan_response: JikanItem<Anime> = match self.http_client.get(&url).await {
            Ok(response) => response,
            Err(AppError::ApiError(msg)) if msg.contains("404") => {
                log::info!("Jikan: No anime found for ID '{}'", id);
                return Ok(None);
            }
            Err(e) => return Err(e),
        };

        let anime_data = self
            .mapper
            .map_to_anime_data(jikan_response.data)
            .map_err(|e| AppError::MappingError(format!("Failed to map Jikan data: {}", e)))?;
        log::info!("Jikan: Found anime by ID '{}'", id);
        Ok(Some(anime_data))
    }

    async fn get_anime_relations(&self, id: u32) -> AppResult<Vec<(u32, String)>> {
        let relation_groups = self.fetch_raw_relations(id).await?;

        let mut simple_relations = Vec::new();
        for group in relation_groups {
            for entry in group.entry {
                // Only include anime entries, not manga
                if entry.r#type.to_lowercase() == "anime" && entry.mal_id > 0 {
                    simple_relations.push((entry.mal_id as u32, group.relation.clone()));
                }
            }
        }

        Ok(simple_relations)
    }
}

impl JikanAdapter {
    // =============================================================================
    // CORE ANIME FUNCTIONS
    // =============================================================================

    /// Get anime by ID (basic information)
    pub async fn get_anime(&self, id: u32) -> AppResult<Option<AnimeData>> {
        self.get_anime_by_id(&id.to_string()).await
    }

    /// Get full anime details (comprehensive information)
    pub async fn get_anime_full(&self, id: u32) -> AppResult<Option<AnimeData>> {
        let url = format!("{}/anime/{}/full", self.base_url, id);

        log::info!("Jikan: Getting full anime details for ID '{}'", id);

        let jikan_response: JikanItem<AnimeFull> = match self.http_client.get(&url).await {
            Ok(response) => response,
            Err(AppError::ApiError(msg)) if msg.contains("404") => {
                log::info!("Jikan: No anime found for ID '{}'", id);
                return Ok(None);
            }
            Err(e) => return Err(e),
        };

        let anime_data = self
            .mapper
            .map_to_anime_data(jikan_response.data.core)
            .map_err(|e| AppError::MappingError(format!("Failed to map Jikan full data: {}", e)))?;

        log::info!("Jikan: Retrieved full details for anime ID '{}'", id);
        Ok(Some(anime_data))
    }

    /// Get anime characters
    pub async fn get_anime_characters(&self, id: u32) -> AppResult<Vec<AnimeCharacterEdge>> {
        let url = format!("{}/anime/{}/characters", self.base_url, id);

        log::info!("Jikan: Getting characters for anime ID '{}'", id);

        let jikan_response: JikanList<AnimeCharacterEdge> = self.http_client.get(&url).await?;

        log::info!(
            "Jikan: Found {} characters for anime ID '{}'",
            jikan_response.data.len(),
            id
        );
        Ok(jikan_response.data)
    }

    /// Get anime staff
    pub async fn get_anime_staff(&self, id: u32) -> AppResult<Vec<AnimeStaffEdge>> {
        let url = format!("{}/anime/{}/staff", self.base_url, id);

        log::info!("Jikan: Getting staff for anime ID '{}'", id);

        let jikan_response: JikanList<AnimeStaffEdge> = self.http_client.get(&url).await?;

        log::info!(
            "Jikan: Found {} staff members for anime ID '{}'",
            jikan_response.data.len(),
            id
        );
        Ok(jikan_response.data)
    }

    /// Get anime episodes (paginated)
    pub async fn get_anime_episodes(
        &self,
        id: u32,
        page: Option<u32>,
    ) -> AppResult<JikanList<AnimeEpisode>> {
        let mut url = format!("{}/anime/{}/episodes", self.base_url, id);
        if let Some(page) = page {
            url.push_str(&format!("?page={}", page));
        }

        log::info!(
            "Jikan: Getting episodes for anime ID '{}' (page: {:?})",
            id,
            page
        );

        let jikan_response: JikanList<AnimeEpisode> = self.http_client.get(&url).await?;

        log::info!(
            "Jikan: Found {} episodes for anime ID '{}'",
            jikan_response.data.len(),
            id
        );
        Ok(jikan_response)
    }

    /// Get specific episode
    pub async fn get_anime_episode(
        &self,
        id: u32,
        episode: u32,
    ) -> AppResult<Option<AnimeEpisode>> {
        let url = format!("{}/anime/{}/episodes/{}", self.base_url, id, episode);

        log::info!("Jikan: Getting episode {} for anime ID '{}'", episode, id);

        let jikan_response: JikanItem<AnimeEpisode> = match self.http_client.get(&url).await {
            Ok(response) => response,
            Err(AppError::ApiError(msg)) if msg.contains("404") => {
                log::info!("Jikan: Episode {} not found for anime ID '{}'", episode, id);
                return Ok(None);
            }
            Err(e) => return Err(e),
        };

        log::info!("Jikan: Retrieved episode {} for anime ID '{}'", episode, id);
        Ok(Some(jikan_response.data))
    }

    /// Get anime news
    pub async fn get_anime_news(
        &self,
        id: u32,
        page: Option<u32>,
    ) -> AppResult<JikanList<AnimeNewsItem>> {
        let mut url = format!("{}/anime/{}/news", self.base_url, id);
        if let Some(page) = page {
            url.push_str(&format!("?page={}", page));
        }

        log::info!(
            "Jikan: Getting news for anime ID '{}' (page: {:?})",
            id,
            page
        );

        let jikan_response: JikanList<AnimeNewsItem> = self.http_client.get(&url).await?;

        log::info!(
            "Jikan: Found {} news items for anime ID '{}'",
            jikan_response.data.len(),
            id
        );
        Ok(jikan_response)
    }

    /// Get anime videos
    pub async fn get_anime_videos(&self, id: u32) -> AppResult<AnimeVideos> {
        let url = format!("{}/anime/{}/videos", self.base_url, id);

        log::info!("Jikan: Getting videos for anime ID '{}'", id);

        let jikan_response: JikanItem<AnimeVideos> = self.http_client.get(&url).await?;

        log::info!("Jikan: Retrieved videos for anime ID '{}'", id);
        Ok(jikan_response.data)
    }

    /// Get anime pictures
    pub async fn get_anime_pictures(&self, id: u32) -> AppResult<Vec<Images>> {
        let url = format!("{}/anime/{}/pictures", self.base_url, id);

        log::info!("Jikan: Getting pictures for anime ID '{}'", id);

        let jikan_response: PicturesVariants = self.http_client.get(&url).await?;

        log::info!(
            "Jikan: Found {} pictures for anime ID '{}'",
            jikan_response.data.len(),
            id
        );
        Ok(jikan_response.data)
    }

    /// Get anime statistics
    pub async fn get_anime_statistics(&self, id: u32) -> AppResult<AnimeStatistics> {
        let url = format!("{}/anime/{}/statistics", self.base_url, id);

        log::info!("Jikan: Getting statistics for anime ID '{}'", id);

        let jikan_response: JikanItem<AnimeStatistics> = self.http_client.get(&url).await?;

        log::info!("Jikan: Retrieved statistics for anime ID '{}'", id);
        Ok(jikan_response.data)
    }

    /// Get additional information
    pub async fn get_anime_more_info(&self, id: u32) -> AppResult<Option<String>> {
        let url = format!("{}/anime/{}/moreinfo", self.base_url, id);

        log::info!("Jikan: Getting more info for anime ID '{}'", id);

        let jikan_response: JikanItem<MoreInfo> = match self.http_client.get(&url).await {
            Ok(response) => response,
            Err(AppError::ApiError(msg)) if msg.contains("404") => {
                return Ok(None);
            }
            Err(e) => return Err(e),
        };

        log::info!("Jikan: Retrieved additional info for anime ID '{}'", id);
        Ok(Some(jikan_response.data.moreinfo))
    }

    /// Get anime recommendations
    pub async fn get_anime_recommendations(&self, id: u32) -> AppResult<Vec<EntryRecommendation>> {
        let url = format!("{}/anime/{}/recommendations", self.base_url, id);

        log::info!("Jikan: Getting recommendations for anime ID '{}'", id);

        let jikan_response: JikanList<EntryRecommendation> = self.http_client.get(&url).await?;

        log::info!(
            "Jikan: Found {} recommendations for anime ID '{}'",
            jikan_response.data.len(),
            id
        );
        Ok(jikan_response.data)
    }

    /// Get related anime
    pub async fn fetch_raw_relations(&self, id: u32) -> AppResult<Vec<RelationGroup>> {
        let url = format!("{}/anime/{}/relations", self.base_url, id);

        log::info!("Jikan: Getting relations for anime ID '{}'", id);

        let jikan_response: JikanList<RelationGroup> = self.http_client.get(&url).await?;

        log::info!(
            "Jikan: Found {} relation groups for anime ID '{}'",
            jikan_response.data.len(),
            id
        );
        Ok(jikan_response.data)
    }

    // =============================================================================
    // SEARCH FUNCTIONS
    // =============================================================================

    /// Search anime (existing method, kept for compatibility)
    pub async fn search_anime_basic(&self, query: &str, limit: usize) -> AppResult<Vec<AnimeData>> {
        self.search_anime(query, limit).await
    }

    /// Advanced anime search with filters
    pub async fn search_anime_advanced(
        &self,
        params: JikanSearchParams,
    ) -> AppResult<JikanList<Anime>> {
        let mut url = format!("{}/anime", self.base_url);
        let query_params = params.to_query_params();

        if !query_params.is_empty() {
            url.push('?');
            let param_strings: Vec<String> = query_params
                .iter()
                .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
                .collect();
            url.push_str(&param_strings.join("&"));
        }

        log::info!("Jikan: Advanced search with URL: {}", url);

        let jikan_response: JikanList<Anime> = self.http_client.get(&url).await?;

        log::info!(
            "Jikan: Advanced search found {} results",
            jikan_response.data.len()
        );
        Ok(jikan_response)
    }

    // =============================================================================
    // SEASONAL & DISCOVERY FUNCTIONS
    // =============================================================================

    /// Get currently airing anime
    pub async fn get_season_now(
        &self,
        limit: Option<u32>,
        page: Option<u32>,
    ) -> AppResult<JikanList<Anime>> {
        let mut url = format!("{}/seasons/now", self.base_url);
        let mut query_params = Vec::new();

        if let Some(limit) = limit {
            query_params.push(format!("limit={}", limit));
        }
        if let Some(page) = page {
            query_params.push(format!("page={}", page));
        }

        if !query_params.is_empty() {
            url.push_str(&format!("?{}", query_params.join("&")));
        }

        log::info!("Jikan: Getting current season anime");

        let jikan_response: JikanList<Anime> = self.http_client.get(&url).await?;

        log::info!(
            "Jikan: Found {} currently airing anime",
            jikan_response.data.len()
        );
        Ok(jikan_response)
    }

    /// Get anime from specific season
    pub async fn get_season(
        &self,
        year: u32,
        season: &str,
        limit: Option<u32>,
        page: Option<u32>,
    ) -> AppResult<JikanList<Anime>> {
        let mut url = format!("{}/seasons/{}/{}", self.base_url, year, season);
        let mut query_params = Vec::new();

        if let Some(limit) = limit {
            query_params.push(format!("limit={}", limit));
        }
        if let Some(page) = page {
            query_params.push(format!("page={}", page));
        }

        if !query_params.is_empty() {
            url.push_str(&format!("?{}", query_params.join("&")));
        }

        log::info!("Jikan: Getting {} {} season anime", season, year);

        let jikan_response: JikanList<Anime> = self.http_client.get(&url).await?;

        log::info!(
            "Jikan: Found {} anime for {} {}",
            jikan_response.data.len(),
            season,
            year
        );
        Ok(jikan_response)
    }

    /// Get upcoming anime
    pub async fn get_season_upcoming(
        &self,
        limit: Option<u32>,
        page: Option<u32>,
    ) -> AppResult<JikanList<Anime>> {
        let mut url = format!("{}/seasons/upcoming", self.base_url);
        let mut query_params = Vec::new();

        if let Some(limit) = limit {
            query_params.push(format!("limit={}", limit));
        }
        if let Some(page) = page {
            query_params.push(format!("page={}", page));
        }

        if !query_params.is_empty() {
            url.push_str(&format!("?{}", query_params.join("&")));
        }

        log::info!("Jikan: Getting upcoming anime");

        let jikan_response: JikanList<Anime> = self.http_client.get(&url).await?;

        log::info!("Jikan: Found {} upcoming anime", jikan_response.data.len());
        Ok(jikan_response)
    }

    /// Get anime broadcast schedule
    pub async fn get_schedules(&self, day: Option<&str>) -> AppResult<Vec<Anime>> {
        let mut url = format!("{}/schedules", self.base_url);
        if let Some(day) = day {
            url.push_str(&format!("/{}", day));
        }

        log::info!("Jikan: Getting broadcast schedule for day: {:?}", day);

        let jikan_response: JikanList<Anime> = self.http_client.get(&url).await?;

        log::info!("Jikan: Found {} scheduled anime", jikan_response.data.len());
        Ok(jikan_response.data)
    }
}
