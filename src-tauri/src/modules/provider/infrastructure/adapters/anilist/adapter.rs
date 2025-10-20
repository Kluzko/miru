//! AniList provider adapter
//!
//! GraphQL-based adapter for the AniList API that implements the same interface
//! as the Jikan adapter, providing comprehensive anime data retrieval capabilities.

use chrono::Datelike;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};

use crate::{
    modules::provider::{
        domain::entities::anime_data::AnimeData, infrastructure::http_client::RateLimitClient,
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

        log::debug!("AniList: Sending GraphQL request body: {:?}", body);

        // Use the modern HTTP client with built-in rate limiting and retry logic
        let graphql_response: Value = self.http_client.post_json(&self.base_url, &body).await?;

        // Check for GraphQL errors
        if let Some(errors) = graphql_response.get("errors") {
            log::error!("AniList: GraphQL errors in response: {:?}", errors);
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

impl AniListAdapter {
    pub async fn search_anime(&self, query: &str, limit: usize) -> AppResult<Vec<AnimeData>> {
        let variables = json!({
            "search": query,
            "page": 1,
            "perPage": limit
        });

        log::info!("AniList: Searching for '{}' (limit: {})", query, limit);
        log::debug!("AniList: GraphQL variables: {:?}", variables);

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

    pub async fn get_anime_by_id(&self, id: &str) -> AppResult<Option<AnimeData>> {
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
}

// Additional AniList-specific functions following the same pattern as Jikan
impl AniListAdapter {
    /// Map AniList relation type to standardized string
    fn map_anilist_relation_type(&self, relation_type: &Option<String>) -> String {
        match relation_type.as_ref().map(|s| s.as_str()) {
            Some("SEQUEL") => "Sequel".to_string(),
            Some("PREQUEL") => "Prequel".to_string(),
            Some("SIDE_STORY") => "Side Story".to_string(),
            Some("SPIN_OFF") => "Spin-off".to_string(),
            Some("ALTERNATIVE") => "Alternative".to_string(),
            Some("SOURCE") => "Source".to_string(),
            Some("ADAPTATION") => "Adaptation".to_string(),
            Some("SUMMARY") => "Summary".to_string(),
            Some("COMPILATION") => "Summary".to_string(),
            Some("CONTAINS") => "Parent Story".to_string(),
            Some("CHARACTER") => "Shared Character".to_string(),
            Some("FULL_STORY") => "Full Story".to_string(),
            Some("PARENT") => "Parent Story".to_string(),
            Some("OTHER") => "Other".to_string(),
            _ => {
                log::warn!(
                    "Unknown AniList relation type: {:?}, defaulting to Other",
                    relation_type
                );
                "Other".to_string()
            }
        }
    }
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
            .map(|m| m.characters.edges)
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

        let staff = response.media.map(|m| m.staff.edges).unwrap_or_default();
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
            .map(|m| m.recommendations.edges)
            .unwrap_or_default();
        log::info!(
            "AniList: Found {} recommendations for anime ID '{}'",
            recommendations.len(),
            id
        );
        Ok(recommendations)
    }

    /// Get related anime (raw AniList data)
    pub async fn fetch_raw_relations(
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
            .map(|m| m.relations.edges)
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

    /// Optimized anime relations using complete franchise discovery
    pub async fn get_anime_relations_optimized(&self, id: u32) -> AppResult<Vec<(u32, String)>> {
        // Use the new franchise discovery method for better performance
        self.discover_complete_franchise(id).await
    }

    /// Complete franchise discovery using deep nested GraphQL query - gets entire franchise in 1 API call
    pub async fn discover_complete_franchise(&self, id: u32) -> AppResult<Vec<(u32, String)>> {
        let variables = json!({
            "id": id
        });

        log::info!(
            "AniList: Getting complete franchise for anime ID '{}' using deep nested query",
            id
        );

        let response: AniListFranchiseDiscoveryResponse = self
            .make_graphql_request(ANIME_FRANCHISE_DISCOVERY_QUERY, Some(variables))
            .await?;

        let mut all_relations = HashMap::new();
        let mut visited = HashSet::new();

        if let Some(media) = response.media {
            // Process the complete franchise tree
            self.process_franchise_relations(&media, &mut all_relations, &mut visited);
        }

        // Convert to simple tuple format, excluding the starting anime
        let franchise_relations: Vec<(u32, String)> = all_relations
            .into_iter()
            .filter(|(related_id, _)| *related_id != id) // Exclude self-reference
            .collect();

        log::info!(
            "AniList: Discovered {} franchise relations using single GraphQL query for ID '{}'",
            franchise_relations.len(),
            id
        );

        Ok(franchise_relations)
    }

    /// Complete franchise discovery with detailed information including titles
    pub async fn discover_complete_franchise_with_details(
        &self,
        id: u32,
    ) -> AppResult<Vec<FranchiseRelation>> {
        let variables = json!({
            "id": id
        });

        log::info!(
            "AniList: Getting complete franchise with details for anime ID '{}' using deep nested query",
            id
        );

        let response: AniListFranchiseDiscoveryResponse = self
            .make_graphql_request(ANIME_FRANCHISE_DISCOVERY_QUERY, Some(variables))
            .await?;

        let mut all_relations = HashMap::new();
        let mut visited = HashSet::new();

        if let Some(media) = response.media {
            // Process the complete franchise tree with details
            self.process_franchise_relations_with_details(&media, &mut all_relations, &mut visited);
        }

        // Convert to detailed format, excluding the starting anime
        let franchise_relations: Vec<FranchiseRelation> = all_relations
            .into_iter()
            .filter(|(related_id, _)| *related_id != id) // Exclude self-reference
            .map(|(_id, details)| details)
            .collect();

        log::info!(
            "AniList: Discovered {} franchise relations with details using single GraphQL query for ID '{}'",
            franchise_relations.len(),
            id
        );

        Ok(franchise_relations)
    }

    /// Complete franchise discovery with categorization and sorting
    pub async fn discover_categorized_franchise(&self, id: u32) -> AppResult<CategorizedFranchise> {
        // Get detailed franchise relations first
        let relations = self.discover_complete_franchise_with_details(id).await?;

        log::info!(
            "AniList: Categorizing {} franchise relations for anime ID '{}'",
            relations.len(),
            id
        );

        // Categorize the relations
        let mut categorized = CategorizedFranchise::new();
        for relation in relations {
            categorized.categorize_relation(relation);
        }

        // Sort each category chronologically
        categorized.sort_all_categories();

        log::info!(
            "AniList: Categorized franchise for ID '{}' - Main: {}, Side: {}, Movies: {}, OVA/Special: {}, Other: {}",
            id,
            categorized.main_story.len(),
            categorized.side_stories.len(),
            categorized.movies.len(),
            categorized.ovas_specials.len(),
            categorized.other.len()
        );

        Ok(categorized)
    }

    /// Recursively process complete franchise relations with detailed information
    fn process_franchise_relations_with_details(
        &self,
        media: &MediaWithFranchiseData,
        all_relations: &mut HashMap<u32, FranchiseRelation>,
        visited: &mut HashSet<u32>,
    ) {
        if let Some(current_id) = media.id {
            let current_id = current_id as u32;

            // Avoid infinite loops
            if visited.contains(&current_id) {
                return;
            }
            visited.insert(current_id);

            // Process direct relations
            if let Some(relations) = &media.relations {
                for relation in &relations.edges {
                    if let Some(node) = &relation.node {
                        if let Some(related_id) = node.id {
                            let related_id = related_id as u32;
                            let relation_type =
                                self.map_anilist_relation_type(&relation.relation_type);

                            // Only add anime relations (filter out manga/other types)
                            if let Some(media_type) = &node.media_type {
                                if media_type.to_uppercase() == "ANIME" {
                                    let title = node
                                        .title
                                        .as_ref()
                                        .and_then(|t| {
                                            t.romaji.clone().or_else(|| t.english.clone())
                                        })
                                        .unwrap_or_else(|| {
                                            format!("Unknown Title (ID: {})", related_id)
                                        });

                                    all_relations.insert(
                                        related_id,
                                        FranchiseRelation {
                                            id: related_id,
                                            title,
                                            relation_type: relation_type.clone(),
                                            format: node.format.clone(),
                                            status: node.status.clone(),
                                            episodes: node.episodes,
                                            start_year: node
                                                .start_date
                                                .as_ref()
                                                .and_then(|d| d.year),
                                        },
                                    );
                                }
                            } else {
                                // If no type specified, assume anime
                                let title = node
                                    .title
                                    .as_ref()
                                    .and_then(|t| t.romaji.clone().or_else(|| t.english.clone()))
                                    .unwrap_or_else(|| {
                                        format!("Unknown Title (ID: {})", related_id)
                                    });

                                all_relations.insert(
                                    related_id,
                                    FranchiseRelation {
                                        id: related_id,
                                        title,
                                        relation_type: relation_type.clone(),
                                        format: node.format.clone(),
                                        status: node.status.clone(),
                                        episodes: node.episodes,
                                        start_year: node.start_date.as_ref().and_then(|d| d.year),
                                    },
                                );
                            }

                            // Recursively process nested relations
                            self.process_franchise_relations_with_details(
                                node,
                                all_relations,
                                visited,
                            );
                        }
                    }
                }
            }
        }
    }

    /// Recursively process complete franchise relations from deep nested GraphQL response
    fn process_franchise_relations(
        &self,
        media: &MediaWithFranchiseData,
        all_relations: &mut HashMap<u32, String>,
        visited: &mut HashSet<u32>,
    ) {
        if let Some(current_id) = media.id {
            let current_id = current_id as u32;

            // Avoid infinite loops
            if visited.contains(&current_id) {
                return;
            }
            visited.insert(current_id);

            // Process direct relations
            if let Some(relations) = &media.relations {
                for relation in &relations.edges {
                    if let Some(node) = &relation.node {
                        if let Some(related_id) = node.id {
                            let related_id = related_id as u32;
                            let relation_type =
                                self.map_anilist_relation_type(&relation.relation_type);

                            // Only add anime relations (filter out manga/other types)
                            if let Some(media_type) = &node.media_type {
                                if media_type.to_uppercase() == "ANIME" {
                                    all_relations.insert(related_id, relation_type);
                                }
                            } else {
                                // If no type specified, assume anime
                                all_relations.insert(related_id, relation_type);
                            }

                            // Recursively process nested relations
                            self.process_franchise_relations(node, all_relations, visited);
                        }
                    }
                }
            }
        }
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
