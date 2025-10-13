use crate::modules::anime::domain::{
    entities::anime_detailed::AnimeDetailed, repositories::anime_repository::AnimeRepository,
};
use crate::modules::provider::{
    application::service::ProviderService, domain::entities::anime_data::AnimeData, AnimeProvider,
};
use crate::shared::errors::{AppError, AppResult};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
// json import removed - no longer needed with simplified relations approach
use specta::Type;
use std::sync::Arc;
use uuid::Uuid;

/// Basic relation information for instant loading (Stage 1)
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BasicRelations {
    pub anime_id: String,
    pub relations: Vec<RelationLink>,
    pub has_more: bool,
    pub cache_timestamp: DateTime<Utc>,
    pub source: RelationSource,
}

/// Simple relation link with minimal data for fast loading
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RelationLink {
    pub target_id: String,
    pub relation_type: String,
    pub title: Option<String>,
    pub provider: AnimeProvider,
    pub category: String, // Backend-determined category (mainStory, sideStory, movie, etc.)
}

/// Detailed relations with enriched metadata (Stage 2)
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DetailedRelations {
    pub relations: Vec<RelationWithMetadata>,
    pub franchise_info: Option<FranchiseInfo>,
    pub completeness_score: f32,
    pub enrichment_timestamp: DateTime<Utc>,
}

/// Relation with full metadata for enhanced display
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RelationWithMetadata {
    pub relation: RelationLink,
    pub metadata: RelationMetadata,
}

/// Complete anime snapshot data for relation display
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RelationMetadata {
    // Comprehensive title information - all variants for user preference
    pub title_romaji: Option<String>,
    pub title_english: Option<String>,
    pub title_native: Option<String>,
    pub title_main: String, // Fallback primary title

    // Core anime information
    pub synopsis: Option<String>,
    pub thumbnail_url: Option<String>,
    pub air_date_from: Option<String>, // ISO format date
    pub air_date_to: Option<String>,   // ISO format date
    pub status: Option<String>,        // "Finished Airing", "Currently Airing", etc.
    pub anime_type: Option<String>,    // "TV", "Movie", "OVA", "Special", etc.
    pub episodes: Option<i32>,
    pub score: Option<f32>,

    // Provider information
    pub provider_id: String, // External provider ID (AniList, MAL, etc.)
    pub provider: AnimeProvider,

    // Relation-specific data
    pub relation_type: String, // "Sequel", "Prequel", "Side Story", etc.
    pub category: String,      // "mainStory", "movie", "sideStory", etc.
}

/// Basic franchise information
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct FranchiseInfo {
    pub name: String,
    #[specta(type = u32)]
    pub total_entries: usize,
    pub main_series_id: Option<String>,
}

/// Complete franchise discovery result (Stage 3)
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct FranchiseDiscovery {
    pub franchise_tree: Vec<FranchiseNode>,
    pub discovery_metadata: DiscoveryMetadata,
    pub timestamp: DateTime<Utc>,
}

/// Node in the franchise tree
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct FranchiseNode {
    pub anime_id: String,
    pub relation_type: String,
    pub children: Vec<FranchiseNode>,
    pub metadata: RelationMetadata,
}

/// Metadata about the discovery process
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DiscoveryMetadata {
    pub provider_used: AnimeProvider,
    pub discovery_depth: u32,
    #[specta(type = u32)]
    pub total_discovered: usize,
    pub confidence: f32,
}

/// Source of relation data
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum RelationSource {
    Cache,
    Database,
    Api,
    Provider(AnimeProvider),
}

impl RelationMetadata {
    pub fn placeholder(
        relation_type: String,
        category: String,
        provider_id: String,
        provider: AnimeProvider,
    ) -> Self {
        Self {
            title_romaji: None,
            title_english: None,
            title_native: None,
            title_main: "Loading...".to_string(),
            synopsis: None,
            thumbnail_url: None,
            air_date_from: None,
            air_date_to: None,
            status: None,
            anime_type: None,
            episodes: None,
            score: None,
            provider_id,
            provider,
            relation_type,
            category,
        }
    }

    pub fn from_anime_data(
        data: &AnimeData,
        relation_type: String,
        category: String,
        provider_id: String,
        provider: AnimeProvider,
    ) -> Self {
        Self {
            title_romaji: data.anime.title.romaji.clone(),
            title_english: data.anime.title.english.clone(),
            title_native: data.anime.title.japanese.clone(),
            title_main: data.anime.title.main.clone(),
            synopsis: data.anime.synopsis.clone(),
            thumbnail_url: data.anime.image_url.clone(),
            air_date_from: data
                .anime
                .aired
                .from
                .map(|d| d.format("%Y-%m-%d").to_string()),
            air_date_to: data
                .anime
                .aired
                .to
                .map(|d| d.format("%Y-%m-%d").to_string()),
            status: Some(data.anime.status.to_string()),
            anime_type: Some(data.anime.anime_type.to_string()),
            episodes: data.anime.episodes.map(|e| e as i32),
            score: data.anime.score,
            provider_id,
            provider,
            relation_type,
            category,
        }
    }

    pub fn from_anime_detailed(
        anime: &AnimeDetailed,
        relation_type: String,
        category: String,
        provider_id: String,
        provider: AnimeProvider,
    ) -> Self {
        Self {
            title_romaji: anime.title.romaji.clone(),
            title_english: anime.title.english.clone(),
            title_native: anime.title.japanese.clone(),
            title_main: anime.title.main.clone(),
            synopsis: anime.synopsis.clone(),
            thumbnail_url: anime.image_url.clone(),
            air_date_from: anime.aired.from.map(|d| d.format("%Y-%m-%d").to_string()),
            air_date_to: anime.aired.to.map(|d| d.format("%Y-%m-%d").to_string()),
            status: Some(anime.status.to_string()),
            anime_type: Some(anime.anime_type.to_string()),
            episodes: anime.episodes.map(|e| e as i32),
            score: anime.score,
            provider_id,
            provider,
            relation_type,
            category,
        }
    }
}

/// Progressive anime relations service
pub struct AnimeRelationsService {
    cache: Arc<RelationsCache>,
    anime_repo: Option<Arc<dyn AnimeRepository>>,
    provider_service: Arc<ProviderService>,
    ingestion_service:
        Option<Arc<crate::modules::anime::application::ingestion_service::AnimeIngestionService>>,
}

impl AnimeRelationsService {
    pub fn new(
        cache: Arc<RelationsCache>,
        anime_repo: Option<Arc<dyn AnimeRepository>>,
        provider_service: Arc<ProviderService>,
    ) -> Self {
        Self {
            cache,
            anime_repo,
            provider_service,
            ingestion_service: None,
        }
    }

    pub fn with_ingestion_service(
        mut self,
        ingestion_service: Arc<
            crate::modules::anime::application::ingestion_service::AnimeIngestionService,
        >,
    ) -> Self {
        self.ingestion_service = Some(ingestion_service);
        self
    }

    /// Check if the service is available
    pub fn is_available(&self) -> bool {
        // Service is available if we have either cache or provider access
        true // Provider service is always available, cache is always available
    }

    /// Stage 1: Get basic relations instantly from cache/DB
    pub async fn get_basic_relations(&self, anime_id: &str) -> AppResult<Option<BasicRelations>> {
        log::debug!("Getting basic relations for anime: {}", anime_id);

        // Check cache first (fastest)
        if let Some(cached) = self.cache.get_basic(anime_id).await {
            if cached.is_fresh(Duration::hours(24)) {
                log::debug!("Returning cached basic relations for {}", anime_id);
                return Ok(Some(cached));
            }
        }

        // Check database if available
        if let Some(repo) = &self.anime_repo {
            match self.get_relations_from_anime_data(anime_id, repo).await {
                Ok(Some(relations)) if !relations.is_empty() => {
                    let basic = BasicRelations {
                        anime_id: anime_id.to_string(),
                        relations,
                        has_more: true,
                        cache_timestamp: Utc::now(),
                        source: RelationSource::Database,
                    };

                    // Cache the result asynchronously
                    let cache_clone = Arc::clone(&self.cache);
                    let basic_clone = basic.clone();
                    tokio::spawn(async move {
                        let _ = cache_clone.store_basic(&basic_clone).await;
                    });

                    log::debug!("Returning database basic relations for {}", anime_id);
                    return Ok(Some(basic));
                }
                Ok(_) => {
                    log::debug!(
                        "No relations found in database for {}, attempting discovery",
                        anime_id
                    );

                    // Try to discover relations from provider
                    if let Some(discovered) =
                        self.discover_and_store_relations(anime_id, repo).await?
                    {
                        return Ok(Some(discovered));
                    }
                }
                Err(e) => {
                    log::warn!(
                        "Database error when fetching relations for {}: {}",
                        anime_id,
                        e
                    );
                }
            }
        }

        log::debug!("No basic relations found for {}", anime_id);
        Ok(None)
    }

    /// Stage 2: Get detailed relations with metadata enrichment
    pub async fn get_detailed_relations(&self, anime_id: &str) -> AppResult<DetailedRelations> {
        log::debug!("Getting detailed relations for anime: {}", anime_id);

        // Get basic relations first (this handles discovery if needed)
        let basic = self
            .get_basic_relations(anime_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("No relations found for {}", anime_id)))?;

        // Check cache for detailed version
        if let Some(cached_detailed) = self.cache.get_detailed(anime_id).await {
            if cached_detailed.is_fresh(Duration::days(7)) {
                log::debug!("Returning cached detailed relations for {}", anime_id);
                return Ok(cached_detailed);
            }
        }

        // Enrich metadata for each relation in parallel
        let metadata_futures: Vec<_> = basic
            .relations
            .iter()
            .map(|rel| self.enrich_relation_metadata(rel))
            .collect();

        let enriched_results = futures::future::join_all(metadata_futures).await;

        let mut detailed_relations = Vec::new();
        for (relation, metadata_result) in basic.relations.iter().zip(enriched_results) {
            match metadata_result {
                Ok(metadata) => detailed_relations.push(RelationWithMetadata {
                    relation: relation.clone(),
                    metadata,
                }),
                Err(e) => {
                    log::warn!("Failed to enrich relation {}: {}", relation.target_id, e);
                    detailed_relations.push(RelationWithMetadata {
                        relation: relation.clone(),
                        metadata: RelationMetadata::placeholder(
                            relation.relation_type.clone(),
                            relation.category.clone(),
                            relation.target_id.clone(),
                            relation.provider.clone(),
                        ),
                    });
                }
            }
        }

        let completeness_score = self.calculate_completeness_score(&detailed_relations);

        let detailed = DetailedRelations {
            relations: detailed_relations,
            franchise_info: None, // Will be calculated in stage 3
            completeness_score,
            enrichment_timestamp: Utc::now(),
        };

        // Cache the detailed result asynchronously
        let cache_clone = Arc::clone(&self.cache);
        let detailed_clone = detailed.clone();
        let anime_id_clone = anime_id.to_string();
        tokio::spawn(async move {
            let _ = cache_clone
                .store_detailed(&anime_id_clone, &detailed_clone)
                .await;
        });

        log::debug!("Returning enriched detailed relations for {}", anime_id);
        Ok(detailed)
    }

    /// Stage 3: Discover complete franchise tree (on-demand)
    pub async fn discover_complete_franchise(
        &self,
        anime_id: &str,
    ) -> AppResult<FranchiseDiscovery> {
        log::info!("Starting franchise discovery for anime: {}", anime_id);

        // Check cache first
        if let Some(cached) = self.cache.get_franchise(anime_id).await {
            if cached.is_fresh(Duration::days(7)) {
                log::debug!("Returning cached franchise discovery for {}", anime_id);
                return Ok(cached);
            }
        }

        // Try providers in order of preference
        let providers = [AnimeProvider::AniList, AnimeProvider::Jikan];

        for provider in &providers {
            match self
                .discover_franchise_from_provider(anime_id, provider)
                .await
            {
                Ok(discovery) => {
                    // Cache the discovery result
                    let cache_clone = Arc::clone(&self.cache);
                    let discovery_clone = discovery.clone();
                    let anime_id_clone = anime_id.to_string();
                    tokio::spawn(async move {
                        let _ = cache_clone
                            .store_franchise(&anime_id_clone, &discovery_clone)
                            .await;
                    });

                    log::info!(
                        "Franchise discovery completed for {} using {:?}",
                        anime_id,
                        provider
                    );
                    return Ok(discovery);
                }
                Err(e) => {
                    log::warn!("Franchise discovery failed with {:?}: {}", provider, e);
                    continue;
                }
            }
        }

        Err(AppError::ServiceUnavailable(
            "No providers available for franchise discovery".to_string(),
        ))
    }

    /// Get anime with their relation metadata using the batch repository method
    /// This is the optimized approach that fetches relation_type + synced_at from anime_relations
    /// and complete anime data in a batch, avoiding multiple enrichment calls
    pub async fn get_anime_with_relations(&self, anime_id: &str) -> AppResult<Vec<crate::modules::anime::domain::repositories::anime_repository::AnimeWithRelationMetadata>>{
        log::debug!(
            "Getting anime with relations using batch approach for: {}",
            anime_id
        );

        if let Some(repo) = &self.anime_repo {
            let anime_uuid = match Uuid::parse_str(anime_id) {
                Ok(uuid) => uuid,
                Err(_) => {
                    log::warn!("Invalid UUID format for anime_id: {}", anime_id);
                    return Ok(Vec::new());
                }
            };

            match repo.get_anime_with_relations(&anime_uuid).await {
                Ok(relations) if !relations.is_empty() => {
                    log::debug!(
                        "Successfully fetched {} anime with relations",
                        relations.len()
                    );
                    Ok(relations)
                }
                Ok(_) => {
                    // No relations found in database - trigger auto-discovery
                    log::info!(
                        "No relations found for {} in database, triggering auto-discovery",
                        anime_id
                    );

                    // Attempt to discover and store relations
                    match self.discover_and_store_relations(anime_id, repo).await {
                        Ok(Some(_)) => {
                            // Discovery successful, retry the query
                            log::info!(
                                "Auto-discovery completed for {}, refetching relations",
                                anime_id
                            );
                            match repo.get_anime_with_relations(&anime_uuid).await {
                                Ok(relations) => {
                                    log::info!(
                                        "Successfully loaded {} relations after auto-discovery",
                                        relations.len()
                                    );
                                    Ok(relations)
                                }
                                Err(e) => {
                                    log::error!(
                                        "Failed to fetch relations after discovery for {}: {}",
                                        anime_id,
                                        e
                                    );
                                    Ok(Vec::new()) // Return empty instead of error
                                }
                            }
                        }
                        Ok(None) => {
                            log::info!(
                                "No relations discovered for {} (anime may not have franchise relations or missing AniList ID)",
                                anime_id
                            );
                            Ok(Vec::new())
                        }
                        Err(e) => {
                            log::warn!(
                                "Auto-discovery failed for {}: {}. Returning empty relations.",
                                anime_id,
                                e
                            );
                            Ok(Vec::new()) // Return empty instead of propagating error
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to get anime with relations for {}: {}", anime_id, e);
                    Err(e)
                }
            }
        } else {
            log::warn!("No repository available for batch relations fetch");
            Ok(Vec::new())
        }
    }

    /// Get relations from anime data using the database
    async fn get_relations_from_anime_data(
        &self,
        anime_id: &str,
        repo: &Arc<dyn AnimeRepository>,
    ) -> AppResult<Option<Vec<RelationLink>>> {
        let anime_uuid = match Uuid::parse_str(anime_id) {
            Ok(uuid) => uuid,
            Err(_) => {
                log::warn!("Invalid UUID format for anime_id: {}", anime_id);
                return Ok(None);
            }
        };

        // Get relations from database using simplified repository method
        match repo.get_relations(&anime_uuid).await {
            Ok(relations) if !relations.is_empty() => {
                let mut relation_links = Vec::new();

                for (related_uuid, relation_type) in relations {
                    // Get the complete anime record for metadata (since we now store complete records)
                    match repo.find_by_id(&related_uuid).await {
                        Ok(Some(related_anime)) => {
                            // Determine category based on relation type and anime type
                            let category = match relation_type.to_lowercase().as_str() {
                                "sequel" | "prequel" => "mainStory",
                                "side_story" | "spin_off" | "alternative" => "sideStory",
                                "movie" => "movie",
                                "special" | "summary" => "ova",
                                _ => "other",
                            }
                            .to_string();

                            // Use the primary provider from external_ids, default to AniList
                            let provider = related_anime
                                .provider_metadata
                                .external_ids
                                .keys()
                                .find(|&p| matches!(p, AnimeProvider::AniList))
                                .or_else(|| {
                                    related_anime.provider_metadata.external_ids.keys().next()
                                })
                                .cloned()
                                .unwrap_or(AnimeProvider::AniList);

                            // Get the external ID for the provider (this is what target_id should be)
                            let target_id = related_anime
                                .provider_metadata
                                .external_ids
                                .get(&provider)
                                .cloned()
                                .unwrap_or_else(|| related_uuid.to_string()); // Fallback to UUID if no external ID

                            relation_links.push(RelationLink {
                                target_id, // Use AniList ID, not UUID
                                relation_type,
                                title: Some(related_anime.title.main.clone()),
                                provider,
                                category,
                            });
                        }
                        Ok(None) => {
                            log::warn!("Related anime {} not found for relation - this should not happen with our new approach", related_uuid);
                            // Skip this relation since we should always have complete anime records
                            continue;
                        }
                        Err(e) => {
                            log::error!("Error fetching related anime {}: {}", related_uuid, e);
                            // Skip this relation on error
                            continue;
                        }
                    }
                }

                log::debug!(
                    "Found {} relations in database for {}",
                    relation_links.len(),
                    anime_id
                );
                Ok(Some(relation_links))
            }
            Ok(_) => {
                log::debug!("No relations found in database for {}", anime_id);
                Ok(None)
            }
            Err(e) => {
                log::error!("Database error fetching relations for {}: {}", anime_id, e);
                Err(e)
            }
        }
    }

    /// Discover and store relations from provider when database is empty
    async fn discover_and_store_relations(
        &self,
        anime_id: &str,
        repo: &Arc<dyn AnimeRepository>,
    ) -> AppResult<Option<BasicRelations>> {
        log::debug!(
            "Attempting to discover relations for {} from provider",
            anime_id
        );

        // First, get the anime to find its AniList ID
        let anime_uuid = match Uuid::parse_str(anime_id) {
            Ok(uuid) => uuid,
            Err(_) => {
                log::warn!("Invalid UUID format for anime_id: {}", anime_id);
                return Ok(None);
            }
        };

        let anime = match repo.find_by_id(&anime_uuid).await {
            Ok(Some(anime)) => anime,
            Ok(None) => {
                log::warn!("Anime {} not found in database", anime_id);
                return Ok(None);
            }
            Err(e) => {
                log::error!("Error fetching anime {}: {}", anime_id, e);
                return Ok(None);
            }
        };

        // Check if anime has AniList ID
        let anilist_id = match anime
            .provider_metadata
            .external_ids
            .get(&AnimeProvider::AniList)
        {
            Some(id) => match id.parse::<u32>() {
                Ok(parsed_id) => parsed_id,
                Err(_) => {
                    log::warn!("Invalid AniList ID for {}: {}", anime_id, id);
                    return Ok(None);
                }
            },
            None => {
                log::debug!(
                    "No AniList ID found for {}, cannot discover relations",
                    anime_id
                );
                return Ok(None);
            }
        };

        log::info!(
            "Found AniList ID {} for anime {}, fetching relations",
            anilist_id,
            anime_id
        );

        // Fetch categorized franchise from AniList (much richer data)
        let categorized_franchise = match self
            .provider_service
            .discover_categorized_franchise(anilist_id)
            .await
        {
            Ok(franchise) if franchise.total_count() > 0 => {
                log::info!(
                    "Successfully fetched franchise data for AniList ID {}: {} total relations",
                    anilist_id,
                    franchise.total_count()
                );
                franchise
            }
            Ok(_) => {
                log::info!("No relations found for AniList ID {}", anilist_id);
                return Ok(None);
            }
            Err(e) => {
                log::error!(
                    "Failed to fetch franchise for AniList ID {}: {}",
                    anilist_id,
                    e
                );
                return Ok(None);
            }
        };

        log::info!(
            "Processing {} discovered relations for AniList ID {} (main_story: {}, side_stories: {}, movies: {}, ovas: {}, other: {})",
            categorized_franchise.total_count(),
            anilist_id,
            categorized_franchise.main_story.len(),
            categorized_franchise.side_stories.len(),
            categorized_franchise.movies.len(),
            categorized_franchise.ovas_specials.len(),
            categorized_franchise.other.len()
        );

        // Convert all categorized relations to our format with proper categories
        let mut relations_to_save: Vec<RelationLink> = Vec::new();

        // Helper function to convert a single relation with complete metadata
        fn convert_relation_with_metadata(
            franchise_rel: &crate::modules::provider::infrastructure::adapters::anilist::models::FranchiseRelation,
            category: &str,
        ) -> RelationLink {
            RelationLink {
                target_id: franchise_rel.id.to_string(), // Always use AniList ID - we'll resolve to local ID if needed
                relation_type: franchise_rel.relation_type.clone(),
                title: Some(franchise_rel.title.clone()),
                provider: AnimeProvider::AniList,
                category: category.to_string(),
            }
        }

        // Convert all categories with proper category labels
        for rel in &categorized_franchise.main_story {
            relations_to_save.push(convert_relation_with_metadata(rel, "mainStory"));
        }
        for rel in &categorized_franchise.side_stories {
            relations_to_save.push(convert_relation_with_metadata(rel, "sideStory"));
        }
        for rel in &categorized_franchise.movies {
            relations_to_save.push(convert_relation_with_metadata(rel, "movie"));
        }
        for rel in &categorized_franchise.ovas_specials {
            relations_to_save.push(convert_relation_with_metadata(rel, "ova"));
        }
        for rel in &categorized_franchise.other {
            relations_to_save.push(convert_relation_with_metadata(rel, "other"));
        }

        // Save ALL discovered relations to database with rich metadata from API
        log::info!(
            "Saving {} discovered relations for {} to database",
            relations_to_save.len(),
            anime_id
        );

        // Create placeholder anime records using AnimeIngestionService
        let mut enriched_relations = Vec::new();

        for rel in &relations_to_save {
            // Check if this anime already exists in our database by AniList ID
            let existing_anime = repo
                .find_by_external_id(&AnimeProvider::AniList, &rel.target_id)
                .await;

            let storage_uuid = match existing_anime {
                Ok(Some(existing)) => {
                    log::debug!(
                        "Using existing anime {} for relation {}",
                        existing.id,
                        rel.target_id
                    );
                    existing.id
                }
                _ => {
                    log::debug!("Creating anime for external relation {}", rel.target_id);

                    // Use AnimeIngestionService if available, otherwise fallback to direct save
                    if let Some(ref ingestion_service) = self.ingestion_service {
                        // Use the ingestion pipeline to create anime with proper scoring
                        let source = crate::modules::anime::application::ingestion_service::AnimeSource::RelationDiscovery {
                            anilist_id: rel.target_id.parse::<u32>().unwrap_or(0),
                            relation_type: rel.category.clone(),
                            source_anime_id: anime_id.to_string(),
                        };

                        let options = crate::modules::anime::application::ingestion_service::IngestionOptions {
                            skip_duplicates: false,
                            skip_provider_fetch: false,
                            enrich_async: true,  // Queue enrichment job if quality is low
                            fetch_relations: false,  // Don't recursively fetch relations
                            priority: crate::modules::anime::application::ingestion_service::JobPriority::Low,
                        };

                        match ingestion_service.ingest_anime(source, options).await {
                            Ok(result) => {
                                log::info!(
                                    "Created anime {} (tier: {:?}, score: {:.2}) for relation {}",
                                    result.anime.id,
                                    result.anime.tier,
                                    result.anime.composite_score,
                                    rel.target_id
                                );
                                result.anime.id
                            }
                            Err(e) => {
                                log::error!(
                                    "Failed to ingest anime for relation {}: {}",
                                    rel.target_id,
                                    e
                                );
                                continue; // Skip this relation if ingestion fails
                            }
                        }
                    } else {
                        // Fallback: direct repository save (legacy behavior)
                        log::warn!("AnimeIngestionService not available, using direct repository save for {}", rel.target_id);

                        let placeholder_uuid = Uuid::new_v4();
                        let placeholder_anime_detailed = match self
                            .provider_service
                            .get_anime_by_id(&rel.target_id, AnimeProvider::AniList)
                            .await
                        {
                            Ok(Some(complete_anime)) => {
                                let mut anime_with_metadata = complete_anime;
                                anime_with_metadata.id = placeholder_uuid;
                                self.provider_service
                                    .calculate_quality_metrics(&mut anime_with_metadata);
                                anime_with_metadata
                            }
                            _ => {
                                log::warn!(
                                    "Failed to fetch data for relation {}, skipping",
                                    rel.target_id
                                );
                                continue;
                            }
                        };

                        match repo.save(&placeholder_anime_detailed).await {
                            Ok(_) => {
                                log::info!(
                                    "Created anime {} for relation {} (legacy mode)",
                                    placeholder_uuid,
                                    rel.target_id
                                );
                                placeholder_uuid
                            }
                            Err(e) => {
                                log::error!(
                                    "Failed to save anime for relation {}: {}",
                                    rel.target_id,
                                    e
                                );
                                continue;
                            }
                        }
                    }
                }
            };

            enriched_relations.push((storage_uuid, rel.category.clone()));
        }

        // Now save relations - this should work since anime records exist
        if let Err(e) = repo.save_relations(&anime_uuid, &enriched_relations).await {
            log::error!("Failed to save relations for {}: {}", anime_id, e);
            // Continue anyway - we can still return the API data
        } else {
            log::info!(
                "Successfully saved {} relations for {} to database",
                enriched_relations.len(),
                anime_id
            );
        }

        // Create BasicRelations result - always return the discovered relations
        let basic = BasicRelations {
            anime_id: anime_id.to_string(),
            relations: relations_to_save.clone(),
            has_more: false, // Provider gives us complete data
            cache_timestamp: Utc::now(),
            source: RelationSource::Database, // Data is now saved in DB
        };

        // Store in cache as backup
        let cache_clone = Arc::clone(&self.cache);
        let basic_clone = basic.clone();
        tokio::spawn(async move {
            if let Err(e) = cache_clone.store_basic(&basic_clone).await {
                log::warn!("Failed to cache relations: {}", e);
            }
        });

        log::debug!(
            "Successfully discovered and returning {} relations for {}",
            relations_to_save.len(),
            anime_id
        );
        Ok(Some(basic))
    }

    /// Enrich a relation with metadata using multi-provider strategy
    /// AniList is used only for relations discovery, other providers for enrichment
    async fn enrich_relation_metadata(
        &self,
        relation: &RelationLink,
    ) -> AppResult<RelationMetadata> {
        // First, check if we have this anime in our database with complete metadata
        if let Some(repo) = &self.anime_repo {
            // Try to find by AniList ID (since relation.target_id is AniList ID)
            if let Ok(Some(existing_anime)) = repo
                .find_by_external_id(&AnimeProvider::AniList, &relation.target_id)
                .await
            {
                log::debug!(
                    "Using database anime data for relation enrichment: {}",
                    relation.target_id
                );
                return Ok(RelationMetadata::from_anime_detailed(
                    &existing_anime,
                    relation.relation_type.clone(),
                    relation.category.clone(),
                    relation.target_id.clone(),
                    AnimeProvider::AniList, // Keep AniList as the relation provider
                ));
            }
        }

        // If not in database, use multi-provider enrichment strategy
        // Try providers in order of preference: Jikan (MAL), then fallback to AniList
        log::debug!(
            "Enriching relation {} using multi-provider strategy",
            relation.target_id
        );

        // First, try to get MAL ID from AniList for cross-provider enrichment
        let mal_id = match self
            .provider_service
            .get_anime_by_id(&relation.target_id, AnimeProvider::AniList)
            .await
        {
            Ok(Some(anilist_anime)) => {
                // Extract MAL ID if available
                anilist_anime
                    .provider_metadata
                    .external_ids
                    .get(&crate::modules::provider::AnimeProvider::Jikan)
                    .cloned()
            }
            _ => None,
        };

        // Try Jikan (MAL) first if we have MAL ID
        if let Some(mal_id) = mal_id {
            log::debug!("Trying Jikan enrichment for MAL ID: {}", mal_id);
            match self
                .provider_service
                .get_anime_by_id(&mal_id, crate::modules::provider::AnimeProvider::Jikan)
                .await
            {
                Ok(Some(jikan_anime)) => {
                    log::debug!(
                        "Successfully enriched relation {} using Jikan",
                        relation.target_id
                    );
                    return Ok(RelationMetadata::from_anime_detailed(
                        &jikan_anime,
                        relation.relation_type.clone(),
                        relation.category.clone(),
                        relation.target_id.clone(), // Keep AniList ID as target_id
                        crate::modules::provider::AnimeProvider::Jikan, // Provider used for enrichment
                    ));
                }
                Err(e) => {
                    log::debug!("Jikan enrichment failed for {}: {}", mal_id, e);
                }
                _ => {
                    log::debug!("No data found from Jikan for MAL ID: {}", mal_id);
                }
            }
        }

        // Fallback to AniList (minimal usage)
        log::debug!(
            "Falling back to AniList for relation enrichment: {}",
            relation.target_id
        );
        match self
            .provider_service
            .get_anime_by_id(&relation.target_id, AnimeProvider::AniList)
            .await
        {
            Ok(Some(anime_detailed)) => {
                log::debug!(
                    "Successfully enriched relation {} using AniList fallback",
                    relation.target_id
                );
                Ok(RelationMetadata::from_anime_detailed(
                    &anime_detailed,
                    relation.relation_type.clone(),
                    relation.category.clone(),
                    relation.target_id.clone(),
                    AnimeProvider::AniList,
                ))
            }
            Ok(None) => {
                log::warn!(
                    "No anime found for relation {} in any provider",
                    relation.target_id
                );
                // Use minimal fallback metadata
                Ok(RelationMetadata {
                    title_romaji: None,
                    title_english: None,
                    title_native: None,
                    title_main: relation
                        .title
                        .clone()
                        .unwrap_or_else(|| format!("Anime {}", relation.target_id)),
                    synopsis: None,
                    thumbnail_url: None,
                    air_date_from: None,
                    air_date_to: None,
                    status: None,
                    anime_type: None,
                    episodes: None,
                    score: None,
                    provider_id: relation.target_id.clone(),
                    provider: AnimeProvider::AniList,
                    relation_type: relation.relation_type.clone(),
                    category: relation.category.clone(),
                })
            }
            Err(e) => {
                log::error!(
                    "All providers failed for relation enrichment {}: {}",
                    relation.target_id,
                    e
                );
                // Use minimal fallback metadata
                Ok(RelationMetadata {
                    title_romaji: None,
                    title_english: None,
                    title_native: None,
                    title_main: relation
                        .title
                        .clone()
                        .unwrap_or_else(|| format!("Anime {}", relation.target_id)),
                    synopsis: None,
                    thumbnail_url: None,
                    air_date_from: None,
                    air_date_to: None,
                    status: None,
                    anime_type: None,
                    episodes: None,
                    score: None,
                    provider_id: relation.target_id.clone(),
                    provider: AnimeProvider::AniList,
                    relation_type: relation.relation_type.clone(),
                    category: relation.category.clone(),
                })
            }
        }
    }

    /// Calculate completeness score for detailed relations
    fn calculate_completeness_score(&self, relations: &[RelationWithMetadata]) -> f32 {
        if relations.is_empty() {
            return 0.0;
        }

        let total_fields = relations.len() * 6; // 6 metadata fields per relation
        let filled_fields: usize = relations
            .iter()
            .map(|rel| {
                let meta = &rel.metadata;
                [
                    meta.synopsis.is_some(),
                    meta.thumbnail_url.is_some(),
                    meta.air_date_from.is_some(),
                    meta.episodes.is_some(),
                    meta.status.is_some(),
                    meta.score.is_some(),
                ]
                .iter()
                .filter(|&&x| x)
                .count()
            })
            .sum();

        filled_fields as f32 / total_fields as f32
    }

    /// Discover franchise from a specific provider
    async fn discover_franchise_from_provider(
        &self,
        anime_id: &str,
        provider: &AnimeProvider,
    ) -> AppResult<FranchiseDiscovery> {
        // TODO: Implement actual provider-specific franchise discovery
        // This is a placeholder that will use the existing provider service methods

        log::debug!(
            "Attempting franchise discovery for {} using {:?}",
            anime_id,
            provider
        );

        // For now, return a minimal discovery result
        Ok(FranchiseDiscovery {
            franchise_tree: Vec::new(),
            discovery_metadata: DiscoveryMetadata {
                provider_used: provider.clone(),
                discovery_depth: 0,
                total_discovered: 0,
                confidence: 0.0,
            },
            timestamp: Utc::now(),
        })
    }
}

/// Cache service for relations data
pub struct RelationsCache {
    // TODO: Implement actual caching mechanism
    // For now, this is a placeholder
}

impl RelationsCache {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn get_basic(&self, _anime_id: &str) -> Option<BasicRelations> {
        // TODO: Implement cache retrieval
        None
    }

    pub async fn store_basic(&self, _basic: &BasicRelations) -> AppResult<()> {
        // TODO: Implement cache storage
        Ok(())
    }

    pub async fn get_detailed(&self, _anime_id: &str) -> Option<DetailedRelations> {
        // TODO: Implement cache retrieval
        None
    }

    pub async fn store_detailed(
        &self,
        _anime_id: &str,
        _detailed: &DetailedRelations,
    ) -> AppResult<()> {
        // TODO: Implement cache storage
        Ok(())
    }

    pub async fn get_franchise(&self, _anime_id: &str) -> Option<FranchiseDiscovery> {
        // TODO: Implement cache retrieval
        None
    }

    pub async fn store_franchise(
        &self,
        _anime_id: &str,
        _discovery: &FranchiseDiscovery,
    ) -> AppResult<()> {
        // TODO: Implement cache storage
        Ok(())
    }
}

impl BasicRelations {
    pub fn is_fresh(&self, max_age: Duration) -> bool {
        Utc::now().signed_duration_since(self.cache_timestamp) < max_age
    }
}

impl DetailedRelations {
    pub fn is_fresh(&self, max_age: Duration) -> bool {
        Utc::now().signed_duration_since(self.enrichment_timestamp) < max_age
    }
}

impl FranchiseDiscovery {
    pub fn is_fresh(&self, max_age: Duration) -> bool {
        Utc::now().signed_duration_since(self.timestamp) < max_age
    }
}
