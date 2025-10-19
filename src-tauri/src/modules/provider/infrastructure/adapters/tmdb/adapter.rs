use crate::{
    modules::provider::infrastructure::{
        adapters::{provider_repository_adapter::ProviderAdapter, tmdb::mapper::TmdbMapper},
        http_client::RateLimitClient,
    },
    modules::provider::{domain::entities::anime_data::AnimeData, AnimeProvider},
    shared::errors::{AppError, AppResult},
};
use async_trait::async_trait;

use super::models::*;

/// TMDB (The Movie Database) provider adapter with REST API
/// Provides categorized images (posters, backdrops, logos) and videos (trailers, teasers, clips)
pub struct TmdbAdapter {
    http_client: RateLimitClient,
    base_url: String,
    api_key: String,
    mapper: TmdbMapper,
}

impl TmdbAdapter {
    pub fn new(api_key: String) -> Self {
        Self {
            http_client: RateLimitClient::for_tmdb(),
            base_url: "https://api.themoviedb.org/3".to_string(),
            api_key,
            mapper: TmdbMapper::new(),
        }
    }

    /// Create adapter with custom HTTP client (for testing)
    pub fn with_client(http_client: RateLimitClient, api_key: String) -> Self {
        Self {
            http_client,
            base_url: "https://api.themoviedb.org/3".to_string(),
            api_key,
            mapper: TmdbMapper::new(),
        }
    }

    /// Check if a request can be made immediately (for testing and monitoring)
    pub fn can_make_request_now(&self) -> bool {
        self.http_client.can_make_request_now()
    }

    /// Build URL with API key parameter
    fn build_url(&self, endpoint: &str) -> String {
        format!("{}{}?api_key={}", self.base_url, endpoint, self.api_key)
    }

    /// Build URL with API key and additional query parameters
    fn build_url_with_params(&self, endpoint: &str, params: &[(String, String)]) -> String {
        let mut url = format!("{}{}?api_key={}", self.base_url, endpoint, self.api_key);
        for (key, value) in params {
            if key != "api_key" {
                url.push_str(&format!("&{}={}", key, urlencoding::encode(value)));
            }
        }
        url
    }
}

#[async_trait]
impl ProviderAdapter for TmdbAdapter {
    async fn search_anime(&self, query: &str, limit: usize) -> AppResult<Vec<AnimeData>> {
        // TMDB doesn't have anime-specific search, so we search TV shows with Japanese origin
        let params = vec![
            ("query".to_string(), query.to_string()),
            ("page".to_string(), "1".to_string()),
            ("language".to_string(), "en-US".to_string()),
        ];

        let url = self.build_url_with_params("/search/tv", &params);

        log::info!("TMDB: Searching for '{}' (limit: {})", query, limit);

        let tmdb_response: TmdbSearchResponse = self.http_client.get(&url).await?;

        // Filter for Japanese anime and limit results
        let anime_shows: Vec<TvShow> = tmdb_response
            .results
            .into_iter()
            .filter(|show| {
                show.origin_country
                    .as_ref()
                    .map(|countries| countries.contains(&"JP".to_string()))
                    .unwrap_or(false)
            })
            .take(limit)
            .collect();

        let anime_data: Result<Vec<_>, _> = anime_shows
            .into_iter()
            .map(|show| self.mapper.map_to_anime_data(show))
            .collect();

        let anime_data = anime_data
            .map_err(|e| AppError::MappingError(format!("Failed to map TMDB data: {}", e)))?;

        log::info!("TMDB: Found {} results for '{}'", anime_data.len(), query);
        Ok(anime_data)
    }

    async fn get_anime_by_id(&self, id: &str) -> AppResult<Option<AnimeData>> {
        let tv_id: u32 = id
            .parse()
            .map_err(|_| AppError::ValidationError(format!("Invalid TMDB ID: {}", id)))?;

        let url = self.build_url(&format!("/tv/{}", tv_id));

        log::info!("TMDB: Getting TV show by ID '{}'", id);

        let tmdb_response: TvShowDetails = match self.http_client.get(&url).await {
            Ok(response) => response,
            Err(AppError::ApiError(msg)) if msg.contains("404") => {
                log::info!("TMDB: No TV show found for ID '{}'", id);
                return Ok(None);
            }
            Err(e) => return Err(e),
        };

        let anime_data = self
            .mapper
            .map_details_to_anime_data(tmdb_response)
            .map_err(|e| AppError::MappingError(format!("Failed to map TMDB data: {}", e)))?;

        log::info!("TMDB: Found TV show by ID '{}'", id);
        Ok(Some(anime_data))
    }

    async fn get_anime(&self, id: u32) -> AppResult<Option<AnimeData>> {
        self.get_anime_by_id(&id.to_string()).await
    }

    async fn get_anime_full(&self, id: u32) -> AppResult<Option<AnimeData>> {
        TmdbAdapter::get_tv_show_full(self, id).await
    }

    async fn search_anime_basic(&self, query: &str, limit: usize) -> AppResult<Vec<AnimeData>> {
        self.search_anime(query, limit).await
    }

    async fn get_season_now(&self, limit: usize) -> AppResult<Vec<AnimeData>> {
        // Get currently airing TV shows from Japan
        let url = self.build_url_with_params(
            "/discover/tv",
            &[
                ("with_origin_country".to_string(), "JP".to_string()),
                ("sort_by".to_string(), "popularity.desc".to_string()),
                ("page".to_string(), "1".to_string()),
                ("language".to_string(), "en-US".to_string()),
            ],
        );

        log::info!("TMDB: Getting current season anime");

        let tmdb_response: TmdbSearchResponse = self.http_client.get(&url).await?;

        let anime_shows: Vec<TvShow> = tmdb_response.results.into_iter().take(limit).collect();

        let anime_data: Result<Vec<_>, _> = anime_shows
            .into_iter()
            .map(|show| self.mapper.map_to_anime_data(show))
            .collect();

        anime_data.map_err(|e| AppError::MappingError(format!("Failed to map TMDB data: {}", e)))
    }

    async fn get_season_upcoming(&self, _limit: usize) -> AppResult<Vec<AnimeData>> {
        // TMDB doesn't have a direct "upcoming" endpoint, return empty for now
        log::info!("TMDB: Upcoming anime not supported, returning empty list");
        Ok(vec![])
    }

    fn get_provider_type(&self) -> AnimeProvider {
        AnimeProvider::TMDB
    }

    fn can_make_request_now(&self) -> bool {
        self.can_make_request_now()
    }

    async fn get_anime_relations(&self, _id: u32) -> AppResult<Vec<(u32, String)>> {
        // TMDB doesn't have anime relations, return empty
        Ok(vec![])
    }
}

impl TmdbAdapter {
    // =============================================================================
    // CORE TV SHOW FUNCTIONS
    // =============================================================================

    /// Get TV show by ID (basic information)
    pub async fn get_tv_show(&self, id: u32) -> AppResult<Option<TvShowDetails>> {
        let url = self.build_url(&format!("/tv/{}", id));

        log::info!("TMDB: Getting TV show for ID '{}'", id);

        let tmdb_response: TvShowDetails = match self.http_client.get(&url).await {
            Ok(response) => response,
            Err(AppError::ApiError(msg)) if msg.contains("404") => {
                log::info!("TMDB: No TV show found for ID '{}'", id);
                return Ok(None);
            }
            Err(e) => return Err(e),
        };

        log::info!("TMDB: Retrieved TV show for ID '{}'", id);
        Ok(Some(tmdb_response))
    }

    /// Get full TV show details (comprehensive information)
    pub async fn get_tv_show_full(&self, id: u32) -> AppResult<Option<AnimeData>> {
        let url = self.build_url(&format!("/tv/{}", id));

        log::info!("TMDB: Getting full TV show details for ID '{}'", id);

        let tmdb_response: TvShowDetails = match self.http_client.get(&url).await {
            Ok(response) => response,
            Err(AppError::ApiError(msg)) if msg.contains("404") => {
                log::info!("TMDB: No TV show found for ID '{}'", id);
                return Ok(None);
            }
            Err(e) => return Err(e),
        };

        let anime_data = self
            .mapper
            .map_details_to_anime_data(tmdb_response)
            .map_err(|e| AppError::MappingError(format!("Failed to map TMDB full data: {}", e)))?;

        log::info!("TMDB: Retrieved full details for TV show ID '{}'", id);
        Ok(Some(anime_data))
    }

    /// Get external IDs for a TV show (IMDb, TVDB, etc.)
    pub async fn get_external_ids(&self, id: u32) -> AppResult<ExternalIds> {
        let url = self.build_url(&format!("/tv/{}/external_ids", id));

        log::info!("TMDB: Getting external IDs for TV show ID '{}'", id);

        let external_ids: ExternalIds = self.http_client.get(&url).await?;

        log::info!("TMDB: Retrieved external IDs for TV show ID '{}'", id);
        Ok(external_ids)
    }

    /// Get content ratings for a TV show
    pub async fn get_content_ratings(&self, id: u32) -> AppResult<Vec<ContentRating>> {
        let url = self.build_url(&format!("/tv/{}/content_ratings", id));

        log::info!("TMDB: Getting content ratings for TV show ID '{}'", id);

        let response: ContentRatingsResponse = self.http_client.get(&url).await?;

        let ratings = response.results.unwrap_or_default();

        log::info!(
            "TMDB: Found {} content ratings for TV show ID '{}'",
            ratings.len(),
            id
        );
        Ok(ratings)
    }

    // =============================================================================
    // IMAGE FUNCTIONS (KEY FEATURE)
    // =============================================================================

    /// Get all images for a TV show (posters, backdrops, logos)
    pub async fn get_images(&self, id: u32) -> AppResult<ImagesResponse> {
        let url = self.build_url(&format!("/tv/{}/images", id));

        log::info!("TMDB: Getting images for TV show ID '{}'", id);

        let images: ImagesResponse = self.http_client.get(&url).await?;

        log::info!(
            "TMDB: Retrieved {} posters, {} backdrops, {} logos for TV show ID '{}'",
            images.posters.as_ref().map(|p| p.len()).unwrap_or(0),
            images.backdrops.as_ref().map(|b| b.len()).unwrap_or(0),
            images.logos.as_ref().map(|l| l.len()).unwrap_or(0),
            id
        );
        Ok(images)
    }

    /// Get posters only
    pub async fn get_posters(&self, id: u32) -> AppResult<Vec<Image>> {
        let images = self.get_images(id).await?;
        Ok(images.posters.unwrap_or_default())
    }

    /// Get backdrops/backgrounds only
    pub async fn get_backdrops(&self, id: u32) -> AppResult<Vec<Image>> {
        let images = self.get_images(id).await?;
        Ok(images.backdrops.unwrap_or_default())
    }

    /// Get logos only
    pub async fn get_logos(&self, id: u32) -> AppResult<Vec<Image>> {
        let images = self.get_images(id).await?;
        Ok(images.logos.unwrap_or_default())
    }

    /// Build full image URL from file path
    pub fn build_image_url(&self, file_path: &str, size: &str) -> String {
        format!("https://image.tmdb.org/t/p/{}{}", size, file_path)
    }

    // =============================================================================
    // VIDEO FUNCTIONS (KEY FEATURE)
    // =============================================================================

    /// Get all videos for a TV show (trailers, teasers, clips, etc.)
    pub async fn get_videos(&self, id: u32) -> AppResult<Vec<Video>> {
        let url = self.build_url(&format!("/tv/{}/videos", id));

        log::info!("TMDB: Getting videos for TV show ID '{}'", id);

        let response: VideosResponse = self.http_client.get(&url).await?;

        let videos = response.results.unwrap_or_default();

        log::info!(
            "TMDB: Found {} videos for TV show ID '{}'",
            videos.len(),
            id
        );
        Ok(videos)
    }

    /// Get trailers only
    pub async fn get_trailers(&self, id: u32) -> AppResult<Vec<Video>> {
        let videos = self.get_videos(id).await?;
        Ok(videos
            .into_iter()
            .filter(|v| v.r#type == "Trailer")
            .collect())
    }

    /// Get teasers only
    pub async fn get_teasers(&self, id: u32) -> AppResult<Vec<Video>> {
        let videos = self.get_videos(id).await?;
        Ok(videos
            .into_iter()
            .filter(|v| v.r#type == "Teaser")
            .collect())
    }

    /// Get clips only
    pub async fn get_clips(&self, id: u32) -> AppResult<Vec<Video>> {
        let videos = self.get_videos(id).await?;
        Ok(videos.into_iter().filter(|v| v.r#type == "Clip").collect())
    }

    // =============================================================================
    // SEARCH FUNCTIONS
    // =============================================================================

    /// Search TV shows with basic query
    pub async fn search_tv_basic(&self, query: &str, limit: usize) -> AppResult<Vec<TvShow>> {
        let params = vec![
            ("query".to_string(), query.to_string()),
            ("page".to_string(), "1".to_string()),
        ];

        let url = self.build_url_with_params("/search/tv", &params);

        log::info!("TMDB: Searching TV shows for '{}'", query);

        let tmdb_response: TmdbSearchResponse = self.http_client.get(&url).await?;

        let shows: Vec<TvShow> = tmdb_response.results.into_iter().take(limit).collect();

        log::info!("TMDB: Found {} TV shows for '{}'", shows.len(), query);
        Ok(shows)
    }

    /// Advanced TV search with filters
    pub async fn search_tv_advanced(
        &self,
        params: TmdbSearchParams,
    ) -> AppResult<TmdbSearchResponse> {
        let query_params = params.to_query_params(&self.api_key);

        let url = self.build_url_with_params("/search/tv", &query_params);

        log::info!("TMDB: Advanced TV search");

        let tmdb_response: TmdbSearchResponse = self.http_client.get(&url).await?;

        log::info!(
            "TMDB: Advanced search found {} results",
            tmdb_response.results.len()
        );
        Ok(tmdb_response)
    }

    // =============================================================================
    // EXTERNAL ID LOOKUP (KEY FEATURE)
    // =============================================================================

    /// Find TV show by external ID (IMDb, TVDB, etc.)
    /// external_source can be: "imdb_id", "tvdb_id", "freebase_mid", "freebase_id", "tvrage_id"
    pub async fn find_by_external_id(
        &self,
        external_id: &str,
        external_source: &str,
    ) -> AppResult<FindResponse> {
        let url = self.build_url_with_params(
            &format!("/find/{}", external_id),
            &[("external_source".to_string(), external_source.to_string())],
        );

        log::info!(
            "TMDB: Finding content by external ID '{}' (source: {})",
            external_id,
            external_source
        );

        let find_response: FindResponse = self.http_client.get(&url).await?;

        log::info!(
            "TMDB: Found {} TV results, {} movie results for external ID '{}'",
            find_response
                .tv_results
                .as_ref()
                .map(|r| r.len())
                .unwrap_or(0),
            find_response
                .movie_results
                .as_ref()
                .map(|r| r.len())
                .unwrap_or(0),
            external_id
        );
        Ok(find_response)
    }

    /// Find by IMDb ID
    pub async fn find_by_imdb_id(&self, imdb_id: &str) -> AppResult<FindResponse> {
        self.find_by_external_id(imdb_id, "imdb_id").await
    }

    /// Find by TVDB ID
    pub async fn find_by_tvdb_id(&self, tvdb_id: u32) -> AppResult<FindResponse> {
        self.find_by_external_id(&tvdb_id.to_string(), "tvdb_id")
            .await
    }

    // =============================================================================
    // DISCOVERY FUNCTIONS
    // =============================================================================

    /// Discover TV shows with filters
    pub async fn discover_tv(&self, params: &[(String, String)]) -> AppResult<TmdbSearchResponse> {
        let url = self.build_url_with_params("/discover/tv", params);

        log::info!("TMDB: Discovering TV shows with filters");

        let tmdb_response: TmdbSearchResponse = self.http_client.get(&url).await?;

        log::info!("TMDB: Discovered {} TV shows", tmdb_response.results.len());
        Ok(tmdb_response)
    }

    /// Get popular TV shows from Japan (likely anime)
    pub async fn get_popular_japanese_shows(&self, limit: usize) -> AppResult<Vec<TvShow>> {
        let url = self.build_url_with_params(
            "/discover/tv",
            &[
                ("with_origin_country".to_string(), "JP".to_string()),
                ("sort_by".to_string(), "popularity.desc".to_string()),
                ("page".to_string(), "1".to_string()),
            ],
        );

        log::info!("TMDB: Getting popular Japanese TV shows");

        let tmdb_response: TmdbSearchResponse = self.http_client.get(&url).await?;

        let shows: Vec<TvShow> = tmdb_response.results.into_iter().take(limit).collect();

        log::info!("TMDB: Found {} popular Japanese shows", shows.len());
        Ok(shows)
    }
}
