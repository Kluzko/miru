use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use diesel::prelude::*;
use tokio::task;
use uuid::Uuid;

use crate::domain::{
    entities::{
        anime_detailed::{AiredDates, AnimeDetailed},
        Genre,
    },
    repositories::AnimeRepository,
    value_objects::{AnimeProvider, AnimeTitle, ProviderMetadata, QualityMetrics},
    // AnimeTier removed - not used in this file
};
use crate::infrastructure::database::{
    connection::Database,
    models::*,
    schema::{anime, anime_genres, anime_studios, genres, quality_metrics, studios},
};
use crate::shared::{
    errors::{AppError, AppResult},
    utils::Validator,
};

pub struct AnimeRepositoryImpl {
    db: Arc<Database>,
}

impl AnimeRepositoryImpl {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    // Helper: Convert Anime to AnimeDetailed
    fn model_to_entity(
        model: Anime,
        genres: Vec<Genre>,
        studios: Vec<String>,
        quality_metrics: Option<QualityMetrics>,
    ) -> AnimeDetailed {
        // Create AnimeTitle from database fields
        let mut title = AnimeTitle::with_variants(
            model.title_main,
            model.title_english,
            model.title_japanese,
            model.title_romaji,
        );

        // Set native title and synonyms
        title.native = model.title_native;
        title.synonyms = model
            .title_synonyms
            .and_then(|v| serde_json::from_value::<Vec<String>>(v).ok())
            .unwrap_or_default();

        // Create ProviderMetadata - we'll populate from external_ids table later
        // For now, create minimal metadata with Jikan as default
        let provider_metadata = ProviderMetadata::new(
            AnimeProvider::Jikan, // Default provider
            "0".to_string(),      // Will be populated from external_ids table
        );

        AnimeDetailed {
            id: model.id,
            title,
            provider_metadata,
            score: model.score,
            favorites: model.favorites.map(|v| v as u32),
            synopsis: model.synopsis,
            episodes: model.episodes.map(|v| v as u16),
            status: model.status,
            aired: AiredDates {
                from: model.aired_from,
                to: model.aired_to,
            },
            anime_type: model.anime_type,
            age_restriction: model.age_restriction,
            genres,
            studios,
            source: model.source,
            duration: model.duration,
            image_url: model.image_url,
            banner_image: model.banner_image,
            trailer_url: model.trailer_url,
            composite_score: model.composite_score,
            tier: model.tier,
            quality_metrics: quality_metrics.unwrap_or_default(),
            // episodes_list: Vec::new(), // Removed - field deleted
            // relations: Vec::new(),     // Removed - field deleted
            created_at: model.created_at,
            updated_at: model.updated_at,
            last_synced_at: model.last_synced_at,
        }
    }

    // Helper: Convert AnimeDetailed to NewAnime for insertion
    fn entity_to_new_model(entity: &AnimeDetailed) -> NewAnime {
        NewAnime {
            id: entity.id,
            title_english: entity.title.english.clone(),
            title_japanese: entity.title.japanese.clone(),
            score: entity.score,
            favorites: entity.favorites.map(|v| v as i32),
            synopsis: entity.synopsis.clone(),
            episodes: entity.episodes.map(|v| v as i32),
            aired_from: entity.aired.from,
            aired_to: entity.aired.to,
            source: entity.source.clone(),
            duration: entity.duration.clone(),
            image_url: entity.image_url.clone(),
            composite_score: entity.composite_score,
            title_main: entity.title.main.clone(),
            title_romaji: entity.title.romaji.clone(),
            title_native: entity.title.native.clone(),
            title_synonyms: if entity.title.synonyms.is_empty() {
                None
            } else {
                Some(
                    serde_json::to_value(&entity.title.synonyms).unwrap_or(serde_json::Value::Null),
                )
            },
            banner_image: entity.banner_image.clone(),
            trailer_url: entity.trailer_url.clone(),
            tier: entity.tier.clone(),
            quality_metrics: Some(
                serde_json::to_value(&entity.quality_metrics).unwrap_or(serde_json::Value::Null),
            ),
            age_restriction: entity.age_restriction.clone(),
            status: entity.status.clone(),
            anime_type: entity.anime_type.clone(),
            last_synced_at: entity.last_synced_at,
        }
    }

    // Helper: Convert AnimeDetailed to AnimeChangeset for updates
    fn entity_to_changeset(entity: &AnimeDetailed) -> AnimeChangeset {
        AnimeChangeset {
            title_english: entity.title.english.clone(),
            title_japanese: entity.title.japanese.clone(),
            score: entity.score,
            favorites: entity.favorites.map(|v| v as i32),
            synopsis: entity.synopsis.clone(),
            episodes: entity.episodes.map(|v| v as i32),
            aired_from: entity.aired.from,
            aired_to: entity.aired.to,
            source: entity.source.clone(),
            duration: entity.duration.clone(),
            image_url: entity.image_url.clone(),
            composite_score: entity.composite_score,
            updated_at: Utc::now(),
            title_main: entity.title.main.clone(),
            title_romaji: entity.title.romaji.clone(),
            title_native: entity.title.native.clone(),
            title_synonyms: if entity.title.synonyms.is_empty() {
                None
            } else {
                Some(
                    serde_json::to_value(&entity.title.synonyms).unwrap_or(serde_json::Value::Null),
                )
            },
            banner_image: entity.banner_image.clone(),
            trailer_url: entity.trailer_url.clone(),
            tier: entity.tier.clone(),
            quality_metrics: Some(
                serde_json::to_value(&entity.quality_metrics).unwrap_or(serde_json::Value::Null),
            ),
            age_restriction: entity.age_restriction.clone(),
            status: entity.status.clone(),
            anime_type: entity.anime_type.clone(),
            last_synced_at: entity.last_synced_at,
        }
    }
}

// -------------------------------------------------------------------------
// Public API methods first (nicer for readers): CRUD + search + listing
// -------------------------------------------------------------------------

#[async_trait]
impl AnimeRepository for AnimeRepositoryImpl {
    async fn find_by_id(&self, id: &Uuid) -> AppResult<Option<AnimeDetailed>> {
        let db = Arc::clone(&self.db);
        let id = *id;

        let model = task::spawn_blocking(move || -> AppResult<Option<Anime>> {
            let mut conn = db.get_connection()?;
            let m = anime::table
                .filter(anime::id.eq(id))
                .first::<Anime>(&mut conn)
                .optional()?;
            Ok(m)
        })
        .await??;

        match model {
            Some(m) => {
                let v = self.load_anime_batch_with_relations(vec![m]).await?;
                Ok(v.into_iter().next())
            }
            None => Ok(None),
        }
    }

    async fn find_by_external_id(
        &self,
        provider: &AnimeProvider,
        external_id: &str,
    ) -> AppResult<Option<AnimeDetailed>> {
        use crate::infrastructure::database::schema::anime_external_ids;

        let db = Arc::clone(&self.db);
        let provider_code = match provider {
            AnimeProvider::Jikan => "jikan",
            AnimeProvider::AniList => "anilist",
            AnimeProvider::Kitsu => "kitsu",
            AnimeProvider::TMDB => "tmdb",
            AnimeProvider::AniDB => "anidb",
        }
        .to_string();
        let external_id = external_id.to_string();

        let model = task::spawn_blocking(move || -> AppResult<Option<Anime>> {
            let mut conn = db.get_connection()?;
            let anime_id: Option<Uuid> = anime_external_ids::table
                .filter(anime_external_ids::provider_code.eq(&provider_code))
                .filter(anime_external_ids::external_id.eq(&external_id))
                .select(anime_external_ids::anime_id)
                .first::<Uuid>(&mut conn)
                .optional()?;

            if let Some(id) = anime_id {
                let m = anime::table
                    .filter(anime::id.eq(id))
                    .first::<Anime>(&mut conn)
                    .optional()?;
                Ok(m)
            } else {
                Ok(None)
            }
        })
        .await??;

        match model {
            Some(m) => {
                let anime = self.load_anime_batch_with_relations(vec![m]).await?;
                Ok(anime.into_iter().next())
            }
            None => Ok(None),
        }
    }

    async fn search(&self, query: &str, limit: usize) -> AppResult<Vec<AnimeDetailed>> {
        if query.is_empty() {
            return Err(AppError::InvalidInput(
                "Search query cannot be empty".into(),
            ));
        }

        use diesel::dsl::sql;
        use diesel::sql_types::{Bool, Float4, Text};

        let db = Arc::clone(&self.db);
        let q = query.to_string();

        let models = task::spawn_blocking(move || -> AppResult<Vec<Anime>> {
            let mut conn = db.get_connection()?;

            // Enhanced search predicate with all title fields and synonyms
            let pred = sql::<Bool>("(")
                // Primary title fields (main, japanese, english) - higher priority
                .sql("similarity(LOWER(title_main), LOWER(")
                .bind::<Text, _>(&q)
                .sql(")) > 0.25 OR ")
                .sql("similarity(LOWER(title_japanese), LOWER(")
                .bind::<Text, _>(&q)
                .sql(")) > 0.25 OR ")
                .sql("similarity(LOWER(title_english), LOWER(")
                .bind::<Text, _>(&q)
                .sql(")) > 0.25 OR ")
                // Secondary title fields (romaji, native) - standard threshold
                .sql("similarity(LOWER(title_romaji), LOWER(")
                .bind::<Text, _>(&q)
                .sql(")) > 0.3 OR ")
                .sql("similarity(LOWER(title_native), LOWER(")
                .bind::<Text, _>(&q)
                .sql(")) > 0.3 OR ")
                // Exact matches in any title field (case insensitive)
                .sql("LOWER(title_main) LIKE LOWER(")
                .bind::<Text, _>(format!("%{}%", &q))
                .sql(") OR ")
                .sql("LOWER(title_english) LIKE LOWER(")
                .bind::<Text, _>(format!("%{}%", &q))
                .sql(") OR ")
                .sql("LOWER(title_japanese) LIKE LOWER(")
                .bind::<Text, _>(format!("%{}%", &q))
                .sql(") OR ")
                .sql("LOWER(title_romaji) LIKE LOWER(")
                .bind::<Text, _>(format!("%{}%", &q))
                .sql(") OR ")
                .sql("LOWER(title_native) LIKE LOWER(")
                .bind::<Text, _>(format!("%{}%", &q))
                .sql(") OR ")
                // Synonyms search (JSON array contains)
                .sql("EXISTS (")
                .sql("SELECT 1 FROM jsonb_array_elements_text(title_synonyms) AS synonym ")
                .sql("WHERE similarity(LOWER(synonym), LOWER(")
                .bind::<Text, _>(&q)
                .sql(")) > 0.3")
                .sql(")")
                .sql(")");

            // Enhanced ranking with prioritization for main, japanese, and english titles
            let rank = sql::<Float4>("GREATEST(")
                // Primary title fields get higher weight (x1.5)
                .sql("similarity(LOWER(title_main), LOWER(")
                .bind::<Text, _>(&q)
                .sql(")) * 1.5,")
                .sql("similarity(LOWER(title_japanese), LOWER(")
                .bind::<Text, _>(&q)
                .sql(")) * 1.5,")
                .sql("similarity(LOWER(title_english), LOWER(")
                .bind::<Text, _>(&q)
                .sql(")) * 1.5,")
                // Secondary title fields get normal weight
                .sql("similarity(LOWER(title_romaji), LOWER(")
                .bind::<Text, _>(&q)
                .sql(")),")
                .sql("similarity(LOWER(title_native), LOWER(")
                .bind::<Text, _>(&q)
                .sql(")),")
                // Exact match bonus
                .sql("CASE WHEN LOWER(title_main) LIKE LOWER(")
                .bind::<Text, _>(format!("%{}%", &q))
                .sql(") THEN 2.0 ELSE 0 END,")
                .sql("CASE WHEN LOWER(title_english) LIKE LOWER(")
                .bind::<Text, _>(format!("%{}%", &q))
                .sql(") THEN 2.0 ELSE 0 END,")
                .sql("CASE WHEN LOWER(title_japanese) LIKE LOWER(")
                .bind::<Text, _>(format!("%{}%", &q))
                .sql(") THEN 2.0 ELSE 0 END")
                .sql(")");

            let rows = anime::table
                .filter(pred)
                .order((rank.desc(), anime::composite_score.desc()))
                .limit(limit as i64)
                .load::<Anime>(&mut conn)?;
            Ok(rows)
        })
        .await??;

        self.load_anime_batch_with_relations(models).await
    }

    async fn save(&self, anime: &AnimeDetailed) -> AppResult<AnimeDetailed> {
        println!(
            "DEBUG: Repository saving anime: {} (ID: {})",
            anime.title.main, anime.id
        );

        Validator::validate_anime_title(&anime.title.main)?;
        if let Some(score) = anime.score {
            Validator::validate_score(score)?;
        }

        println!("DEBUG: Calling upsert_anime for: {}", anime.title.main);
        let saved_model = self.upsert_anime(anime).await?;

        println!("DEBUG: Upserting genres for anime: {}", anime.title.main);
        self.upsert_genres(saved_model.id, &anime.genres).await?;

        println!("DEBUG: Upserting studios for anime: {}", anime.title.main);
        self.upsert_studios(saved_model.id, &anime.studios).await?;

        println!(
            "DEBUG: Upserting quality metrics for anime: {}",
            anime.title.main
        );
        self.upsert_quality_metrics(saved_model.id, &anime.quality_metrics)
            .await?;

        println!(
            "DEBUG: Building final anime result from saved data: {}",
            anime.title.main
        );

        // Build AnimeDetailed directly from saved data instead of querying DB again
        let result = Self::model_to_entity(
            saved_model,
            anime.genres.clone(),                // We already have the genres
            anime.studios.clone(),               // We already have the studios
            Some(anime.quality_metrics.clone()), // We already have the metrics
        );

        println!(
            "DEBUG: Successfully built anime result: {} (ID: {})",
            result.title.main, result.id
        );
        Ok(result)
    }

    async fn save_batch(&self, anime_list: &[AnimeDetailed]) -> AppResult<Vec<AnimeDetailed>> {
        if anime_list.is_empty() {
            return Ok(vec![]);
        }

        let db = Arc::clone(&self.db);
        let to_upsert = anime_list.to_vec();

        let saved_models = task::spawn_blocking(move || -> AppResult<Vec<Anime>> {
            let mut conn = db.get_connection()?;

            conn.transaction::<Vec<Anime>, AppError, _>(|conn| {
                let mut out = Vec::with_capacity(to_upsert.len());

                for a in &to_upsert {
                    // For batch operations, we'll use a simpler approach - just insert and handle conflicts
                    // TODO: Implement proper external ID based conflict resolution for batch operations
                    let new_row = Self::entity_to_new_model(a);

                    let saved = diesel::insert_into(anime::table)
                        .values(&new_row)
                        .get_result::<Anime>(conn)?;

                    out.push(saved);
                }

                Ok(out)
            })
        })
        .await??;

        for (saved, input) in saved_models.iter().zip(anime_list.iter()) {
            self.upsert_genres(saved.id, &input.genres).await?;
            self.upsert_studios(saved.id, &input.studios).await?;
            self.upsert_quality_metrics(saved.id, &input.quality_metrics)
                .await?;
        }

        self.load_anime_batch_with_relations(saved_models).await
    }

    async fn update(&self, anime: &AnimeDetailed) -> AppResult<AnimeDetailed> {
        if self.find_by_id(&anime.id).await?.is_none() {
            return Err(AppError::NotFound(format!(
                "Anime with ID {} not found",
                anime.id
            )));
        }
        self.save(anime).await
    }

    async fn delete(&self, id: &Uuid) -> AppResult<()> {
        let db = Arc::clone(&self.db);
        let id = *id;

        task::spawn_blocking(move || -> AppResult<()> {
            let mut conn = db.get_connection()?;
            let deleted =
                diesel::delete(anime::table.filter(anime::id.eq(id))).execute(&mut conn)?;
            if deleted == 0 {
                return Err(AppError::NotFound(format!(
                    "Anime with ID {} not found",
                    id
                )));
            }
            Ok(())
        })
        .await?
    }

    async fn find_by_title_variations(
        &self,
        search_title: &str,
    ) -> AppResult<Option<AnimeDetailed>> {
        let search_title_lower = search_title.to_lowercase();
        let search_pattern = format!("%{}%", search_title_lower);
        let exact_pattern = search_title_lower.clone();
        let db = Arc::clone(&self.db);

        let result = task::spawn_blocking(move || {
            let mut conn = db.get_connection()?;

            use diesel::dsl::sql;
            use diesel::sql_types::{Bool, Float4, Text};

            // Enhanced search predicate for title variations with prioritization
            let pred = sql::<Bool>("(")
                // Exact matches first (highest priority)
                .sql("LOWER(title_main) = LOWER(")
                .bind::<Text, _>(&exact_pattern)
                .sql(") OR ")
                .sql("LOWER(title_english) = LOWER(")
                .bind::<Text, _>(&exact_pattern)
                .sql(") OR ")
                .sql("LOWER(title_japanese) = LOWER(")
                .bind::<Text, _>(&exact_pattern)
                .sql(") OR ")
                // Primary title fields (main, japanese, english) - partial matches
                .sql("LOWER(title_main) LIKE LOWER(")
                .bind::<Text, _>(&search_pattern)
                .sql(") OR ")
                .sql("LOWER(title_english) LIKE LOWER(")
                .bind::<Text, _>(&search_pattern)
                .sql(") OR ")
                .sql("LOWER(title_japanese) LIKE LOWER(")
                .bind::<Text, _>(&search_pattern)
                .sql(") OR ")
                // Secondary title fields (romaji, native)
                .sql("LOWER(title_romaji) LIKE LOWER(")
                .bind::<Text, _>(&search_pattern)
                .sql(") OR ")
                .sql("LOWER(title_native) LIKE LOWER(")
                .bind::<Text, _>(&search_pattern)
                .sql(") OR ")
                // Synonyms search (JSON array contains)
                .sql("EXISTS (")
                .sql("SELECT 1 FROM jsonb_array_elements_text(title_synonyms) AS synonym ")
                .sql("WHERE LOWER(synonym) LIKE LOWER(")
                .bind::<Text, _>(&search_pattern)
                .sql(")")
                .sql(")")
                .sql(")");

            // Enhanced ranking for import matching - prioritize exact matches and main/english/japanese titles
            let rank = sql::<Float4>("GREATEST(")
                // Exact matches get highest priority (x3.0)
                .sql("CASE WHEN LOWER(title_main) = LOWER(")
                .bind::<Text, _>(&exact_pattern)
                .sql(") THEN 3.0 ELSE 0 END,")
                .sql("CASE WHEN LOWER(title_english) = LOWER(")
                .bind::<Text, _>(&exact_pattern)
                .sql(") THEN 3.0 ELSE 0 END,")
                .sql("CASE WHEN LOWER(title_japanese) = LOWER(")
                .bind::<Text, _>(&exact_pattern)
                .sql(") THEN 3.0 ELSE 0 END,")
                // Primary title partial matches get high priority (x2.0)
                .sql("CASE WHEN LOWER(title_main) LIKE LOWER(")
                .bind::<Text, _>(&search_pattern)
                .sql(") THEN 2.0 ELSE 0 END,")
                .sql("CASE WHEN LOWER(title_english) LIKE LOWER(")
                .bind::<Text, _>(&search_pattern)
                .sql(") THEN 2.0 ELSE 0 END,")
                .sql("CASE WHEN LOWER(title_japanese) LIKE LOWER(")
                .bind::<Text, _>(&search_pattern)
                .sql(") THEN 2.0 ELSE 0 END,")
                // Secondary title fields get normal priority (x1.0)
                .sql("CASE WHEN LOWER(title_romaji) LIKE LOWER(")
                .bind::<Text, _>(&search_pattern)
                .sql(") THEN 1.0 ELSE 0 END,")
                .sql("CASE WHEN LOWER(title_native) LIKE LOWER(")
                .bind::<Text, _>(&search_pattern)
                .sql(") THEN 1.0 ELSE 0 END")
                .sql(")");

            // Search with enhanced criteria and prioritization
            let anime_models: Vec<Anime> = anime::table
                .filter(pred)
                .order((rank.desc(), anime::composite_score.desc()))
                .limit(1)
                .load::<Anime>(&mut conn)?;
            Ok::<Vec<Anime>, AppError>(anime_models)
        })
        .await?;

        let anime_models = result?;
        if anime_models.is_empty() {
            Ok(None)
        } else {
            // Load the first match with full relations
            let anime_with_relations = self.load_anime_batch_with_relations(anime_models).await?;
            Ok(anime_with_relations.into_iter().next())
        }
    }

    async fn get_all(&self, offset: i64, limit: i64) -> AppResult<Vec<AnimeDetailed>> {
        Validator::validate_pagination(offset, limit)?;

        let db = Arc::clone(&self.db);

        let models = task::spawn_blocking(move || -> AppResult<Vec<Anime>> {
            let mut conn = db.get_connection()?;
            let rows = anime::table
                .offset(offset)
                .limit(limit)
                .order(anime::composite_score.desc())
                .load::<Anime>(&mut conn)?;
            Ok(rows)
        })
        .await??;

        self.load_anime_batch_with_relations(models).await
    }
}

// -----------------------------------------------------------------------------
// Private helpers (kept after the public API for readability)
// -----------------------------------------------------------------------------

impl AnimeRepositoryImpl {
    async fn load_anime_batch_with_relations(
        &self,
        anime_models: Vec<Anime>,
    ) -> AppResult<Vec<AnimeDetailed>> {
        if anime_models.is_empty() {
            return Ok(Vec::new());
        }

        let db = Arc::clone(&self.db);

        let results = task::spawn_blocking(move || -> AppResult<Vec<AnimeDetailed>> {
            let mut conn = db.get_connection()?;

            let rows_g: Vec<(AnimeGenre, GenreModel)> = AnimeGenre::belonging_to(&anime_models)
                .inner_join(genres::table)
                .select((anime_genres::all_columns, genres::all_columns))
                .load::<(AnimeGenre, GenreModel)>(&mut conn)?;
            let grouped_g = rows_g.grouped_by(&anime_models);
            let genres_grouped: HashMap<Uuid, Vec<Genre>> = anime_models
                .iter()
                .zip(grouped_g)
                .map(|(a, pairs)| {
                    let v = pairs
                        .into_iter()
                        .map(|(_, g)| Genre {
                            id: g.id,
                            name: g.name,
                        })
                        .collect::<Vec<_>>();
                    (a.id, v)
                })
                .collect();

            let rows_s: Vec<(AnimeStudio, StudioModel)> = AnimeStudio::belonging_to(&anime_models)
                .inner_join(studios::table)
                .select((anime_studios::all_columns, studios::all_columns))
                .load::<(AnimeStudio, StudioModel)>(&mut conn)?;
            let grouped_s = rows_s.grouped_by(&anime_models);
            let studios_grouped: HashMap<Uuid, Vec<String>> = anime_models
                .iter()
                .zip(grouped_s)
                .map(|(a, pairs)| {
                    let v = pairs.into_iter().map(|(_, s)| s.name).collect::<Vec<_>>();
                    (a.id, v)
                })
                .collect();

            let metrics: Vec<QualityMetricsModel> =
                QualityMetricsModel::belonging_to(&anime_models)
                    .load::<QualityMetricsModel>(&mut conn)?;
            let grouped_m = metrics.grouped_by(&anime_models);

            let out = anime_models
                .into_iter()
                .zip(grouped_m)
                .map(|(m, mvec)| {
                    let genres = genres_grouped.get(&m.id).cloned().unwrap_or_default();
                    let studios = studios_grouped.get(&m.id).cloned().unwrap_or_default();
                    let quality_metrics = mvec
                        .into_iter()
                        .next()
                        .map(|qm| QualityMetrics {
                            popularity_score: qm.popularity_score,
                            engagement_score: qm.engagement_score,
                            consistency_score: qm.consistency_score,
                            audience_reach_score: qm.audience_reach_score,
                        })
                        .unwrap_or_default();

                    Self::model_to_entity(m, genres, studios, Some(quality_metrics))
                })
                .collect::<Vec<_>>();

            Ok(out)
        })
        .await??;

        Ok(results)
    }

    /// Upsert Anime and return the saved row.
    async fn upsert_anime(&self, anime: &AnimeDetailed) -> AppResult<Anime> {
        let db = Arc::clone(&self.db);
        let new_anime = Self::entity_to_new_model(anime);
        let changes = Self::entity_to_changeset(anime);
        let anime_clone = anime.clone();

        task::spawn_blocking(move || -> AppResult<Anime> {
            let mut conn = db.get_connection()?;

            conn.transaction::<Anime, AppError, _>(|conn| {
                // First, check if anime already exists by external ID
                let existing_anime_id = {
                    use crate::infrastructure::database::schema::anime_external_ids;

                    // Get primary external ID, handle case where it might not be set or be "0"
                    if let Some(primary_external_id) = anime_clone
                        .provider_metadata
                        .get_external_id(&anime_clone.provider_metadata.primary_provider)
                    {
                        // Skip if external ID is "0" or empty (invalid)
                        if primary_external_id.is_empty() || primary_external_id == "0" {
                            None
                        } else {
                            let provider_code = match anime_clone.provider_metadata.primary_provider
                            {
                                AnimeProvider::Jikan => "jikan",
                                AnimeProvider::AniList => "anilist",
                                AnimeProvider::Kitsu => "kitsu",
                                AnimeProvider::TMDB => "tmdb",
                                AnimeProvider::AniDB => "anidb",
                            };

                            anime_external_ids::table
                                .filter(anime_external_ids::provider_code.eq(provider_code))
                                .filter(anime_external_ids::external_id.eq(primary_external_id))
                                .select(anime_external_ids::anime_id)
                                .first::<Uuid>(conn)
                                .optional()?
                        }
                    } else {
                        None
                    }
                };

                let saved_anime = if let Some(existing_id) = existing_anime_id {
                    // Update existing anime
                    diesel::update(anime::table.filter(anime::id.eq(existing_id)))
                        .set(&changes)
                        .get_result::<Anime>(conn)?
                } else {
                    // Insert new anime
                    diesel::insert_into(anime::table)
                        .values(&new_anime)
                        .get_result::<Anime>(conn)?
                };

                // Update external IDs if they exist and are valid
                if anime_clone
                    .provider_metadata
                    .external_ids
                    .iter()
                    .any(|(_, id)| !id.is_empty() && id != "0")
                {
                    Self::upsert_external_ids_blocking(
                        conn,
                        saved_anime.id,
                        &anime_clone.provider_metadata,
                    )?;
                }

                Ok(saved_anime)
            })
        })
        .await?
    }

    fn upsert_external_ids_blocking(
        conn: &mut diesel::PgConnection,
        anime_id: Uuid,
        provider_metadata: &ProviderMetadata,
    ) -> AppResult<()> {
        use crate::infrastructure::database::schema::anime_external_ids;
        use diesel::prelude::*;

        // Insert or update external IDs
        for (provider, external_id) in &provider_metadata.external_ids {
            // Skip invalid external IDs
            if external_id.is_empty() || external_id == "0" {
                continue;
            }

            let provider_code = match provider {
                AnimeProvider::Jikan => "jikan",
                AnimeProvider::AniList => "anilist",
                AnimeProvider::Kitsu => "kitsu",
                AnimeProvider::TMDB => "tmdb",
                AnimeProvider::AniDB => "anidb",
            };

            let is_primary = provider == &provider_metadata.primary_provider;

            diesel::insert_into(anime_external_ids::table)
                .values((
                    anime_external_ids::anime_id.eq(anime_id),
                    anime_external_ids::provider_code.eq(provider_code),
                    anime_external_ids::external_id.eq(external_id),
                    anime_external_ids::is_primary.eq(is_primary),
                ))
                .on_conflict((
                    anime_external_ids::anime_id,
                    anime_external_ids::provider_code,
                ))
                .do_update()
                .set((
                    anime_external_ids::external_id.eq(external_id),
                    anime_external_ids::is_primary.eq(is_primary),
                    anime_external_ids::last_synced.eq(chrono::Utc::now()),
                ))
                .execute(conn)?;
        }

        Ok(())
    }

    async fn upsert_genres(&self, anime_id: Uuid, genres_in: &[Genre]) -> AppResult<()> {
        let db = Arc::clone(&self.db);
        // Create owned vector for closure, but more efficiently
        let genres = Vec::from(genres_in);

        task::spawn_blocking(move || -> AppResult<()> {
            let mut conn = db.get_connection()?;

            conn.transaction::<_, AppError, _>(|conn| {
                diesel::delete(anime_genres::table.filter(anime_genres::anime_id.eq(anime_id)))
                    .execute(conn)?;

                for g in genres {
                    println!("DEBUG: Processing genre: {}", g.name);

                    let new_g = NewGenre {
                        id: g.id,
                        name: g.name.clone(),
                    };

                    // Use name-based conflict resolution with better error handling
                    let genre_id = match diesel::insert_into(genres::table)
                        .values(&new_g)
                        .on_conflict(genres::name)
                        .do_update()
                        .set(genres::name.eq(&g.name))
                        .returning(genres::id)
                        .get_result::<Uuid>(conn)
                    {
                        Ok(id) => {
                            println!("DEBUG: Successfully upserted genre: {} (ID: {})", g.name, id);
                            id
                        }
                        Err(e) => {
                            println!("DEBUG: Genre upsert failed, trying fallback for '{}': {}", g.name, e);
                            // Fallback: find existing genre by name
                            match genres::table
                                .filter(genres::name.eq(&g.name))
                                .select(genres::id)
                                .first::<Uuid>(conn)
                            {
                                Ok(existing_id) => {
                                    println!("DEBUG: Found existing genre: {} (ID: {})", g.name, existing_id);
                                    existing_id
                                }
                                Err(find_err) => {
                                    println!("DEBUG: Failed to find genre '{}': {}", g.name, find_err);
                                    return Err(AppError::DatabaseError(format!(
                                        "Failed to upsert genre '{}': insert failed: {}, find failed: {}",
                                        g.name, e, find_err
                                    )));
                                }
                            }
                        }
                    };

                    match diesel::insert_into(anime_genres::table)
                        .values(NewAnimeGenre { anime_id, genre_id })
                        .on_conflict_do_nothing()
                        .execute(conn)
                    {
                        Ok(rows_affected) => {
                            println!("DEBUG: Associated genre {} with anime (rows affected: {})", g.name, rows_affected);
                        }
                        Err(e) => {
                            println!("DEBUG: Failed to associate genre {} with anime: {}", g.name, e);
                            return Err(AppError::DatabaseError(format!(
                                "Failed to associate genre '{}' with anime: {}", g.name, e
                            )));
                        }
                    }
                }

                Ok(())
            })
        })
        .await
        ?
    }

    async fn upsert_studios(&self, anime_id: Uuid, studio_names: &[String]) -> AppResult<()> {
        let db = Arc::clone(&self.db);
        let studs = studio_names.to_vec();

        task::spawn_blocking(move || -> AppResult<()> {
            let mut conn = db.get_connection()?;

            conn.transaction::<_, AppError, _>(|conn| {
                diesel::delete(anime_studios::table.filter(anime_studios::anime_id.eq(anime_id)))
                    .execute(conn)?;

                for name in studs {
                    let new_s = NewStudio {
                        id: Uuid::new_v4(),
                        name: name.clone(),
                    };

                    // NOTE: ON CONFLICT DO NOTHING RETURNING returns 0 rows on conflict;
                    // we fall back to SELECT in that case.
                    let studio_id = diesel::insert_into(studios::table)
                        .values(&new_s)
                        .on_conflict(studios::name)
                        .do_nothing()
                        .returning(studios::id)
                        .get_result::<Uuid>(conn)
                        .or_else(|_| {
                            studios::table
                                .filter(studios::name.eq(&name))
                                .select(studios::id)
                                .first::<Uuid>(conn)
                        })?;

                    diesel::insert_into(anime_studios::table)
                        .values(NewAnimeStudio {
                            anime_id,
                            studio_id,
                        })
                        .on_conflict_do_nothing()
                        .execute(conn)?;
                }

                Ok(())
            })
        })
        .await?
    }

    async fn upsert_quality_metrics(
        &self,
        anime_id: Uuid,
        metrics: &QualityMetrics,
    ) -> AppResult<()> {
        let db = Arc::clone(&self.db);
        let m = metrics.clone();

        task::spawn_blocking(move || -> AppResult<()> {
            let mut conn = db.get_connection()?;

            let new_metrics = NewQualityMetrics {
                id: Uuid::new_v4(),
                anime_id,
                popularity_score: m.popularity_score,
                engagement_score: m.engagement_score,
                consistency_score: m.consistency_score,
                audience_reach_score: m.audience_reach_score,
            };

            let changes = QualityMetricsChangeset {
                popularity_score: m.popularity_score,
                engagement_score: m.engagement_score,
                consistency_score: m.consistency_score,
                audience_reach_score: m.audience_reach_score,
                updated_at: chrono::Utc::now(),
            };

            diesel::insert_into(quality_metrics::table)
                .values(&new_metrics)
                .on_conflict(quality_metrics::anime_id)
                .do_update()
                .set(&changes)
                .execute(&mut conn)?;

            Ok(())
        })
        .await?
    }
}
