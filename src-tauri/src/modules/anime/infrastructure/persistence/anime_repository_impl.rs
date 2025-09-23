use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use diesel::prelude::*;
use tokio::task;
use uuid::Uuid;

// use crate::shared::utils::logger::{LogContext, TimedOperation};
use crate::{log_debug, log_error, log_warn};

use super::anime_model::*;
use crate::modules::anime::domain::{
    entities::{
        anime_detailed::{AiredDates, AnimeDetailed},
        genre::Genre,
    },
    repositories::anime_repository::AnimeRepository,
    value_objects::{anime_title::AnimeTitle, quality_metrics::QualityMetrics},
};
use crate::modules::provider::{AnimeProvider, ProviderMetadata};
use crate::schema::{anime, anime_genres, anime_studios, genres, quality_metrics, studios};
use crate::shared::Database;
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
        use crate::schema::anime_external_ids;

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
        log_debug!(
            "Repository saving anime: {} (ID: {})",
            anime.title.main,
            anime.id
        );

        Validator::validate_anime_title(&anime.title.main)?;
        if let Some(score) = anime.score {
            Validator::validate_score(score)?;
        }

        log_debug!("Calling upsert_anime for: {}", anime.title.main);
        let saved_model = self.upsert_anime(anime).await?;

        log_debug!("Upserting genres for anime: {}", anime.title.main);
        self.upsert_genres(saved_model.id, &anime.genres).await?;

        log_debug!("Upserting studios for anime: {}", anime.title.main);
        self.upsert_studios(saved_model.id, &anime.studios).await?;

        log_debug!("Upserting quality metrics for anime: {}", anime.title.main);
        self.upsert_quality_metrics(saved_model.id, &anime.quality_metrics)
            .await?;

        log_debug!(
            "Building final anime result from saved data: {}",
            anime.title.main
        );

        // Build AnimeDetailed directly from saved data instead of querying DB again
        let result = Self::model_to_entity(
            saved_model,
            anime.genres.clone(),                // We already have the genres
            anime.studios.clone(),               // We already have the studios
            Some(anime.quality_metrics.clone()), // We already have the metrics
        );

        log_debug!(
            "Successfully built anime result: {} (ID: {})",
            result.title.main,
            result.id
        );
        Ok(result)
    }

    async fn save_batch(&self, anime_list: &[AnimeDetailed]) -> AppResult<Vec<AnimeDetailed>> {
        if anime_list.is_empty() {
            return Ok(vec![]);
        }

        log_debug!(
            "Starting bulk save operation for {} anime",
            anime_list.len()
        );
        let batch_start = std::time::Instant::now();

        let db = Arc::clone(&self.db);
        let to_upsert = anime_list.to_vec();

        let saved_models = task::spawn_blocking(move || -> AppResult<Vec<Anime>> {
            let mut conn = db.get_connection()?;

            conn.transaction::<Vec<Anime>, AppError, _>(|conn| {
                log_debug!("Starting database transaction for bulk anime save");
                let transaction_start = std::time::Instant::now();

                // Step 1: Bulk upsert anime records
                log_debug!(
                    "Preparing bulk anime insert for {} records",
                    to_upsert.len()
                );
                let new_anime_records: Vec<NewAnime> = to_upsert
                    .iter()
                    .map(|a| Self::entity_to_new_model(a))
                    .collect();

                // Use bulk insert with ON CONFLICT DO UPDATE for anime records
                let upsert_start = std::time::Instant::now();
                let saved_anime: Vec<Anime> = diesel::insert_into(anime::table)
                    .values(&new_anime_records)
                    .on_conflict(anime::id)
                    .do_update()
                    .set((
                        anime::title_english.eq(diesel::upsert::excluded(anime::title_english)),
                        anime::title_japanese.eq(diesel::upsert::excluded(anime::title_japanese)),
                        anime::score.eq(diesel::upsert::excluded(anime::score)),
                        anime::favorites.eq(diesel::upsert::excluded(anime::favorites)),
                        anime::synopsis.eq(diesel::upsert::excluded(anime::synopsis)),
                        anime::episodes.eq(diesel::upsert::excluded(anime::episodes)),
                        anime::aired_from.eq(diesel::upsert::excluded(anime::aired_from)),
                        anime::aired_to.eq(diesel::upsert::excluded(anime::aired_to)),
                        anime::source.eq(diesel::upsert::excluded(anime::source)),
                        anime::duration.eq(diesel::upsert::excluded(anime::duration)),
                        anime::image_url.eq(diesel::upsert::excluded(anime::image_url)),
                        anime::composite_score.eq(diesel::upsert::excluded(anime::composite_score)),
                        anime::updated_at.eq(chrono::Utc::now()),
                        anime::title_main.eq(diesel::upsert::excluded(anime::title_main)),
                        anime::title_romaji.eq(diesel::upsert::excluded(anime::title_romaji)),
                        anime::title_native.eq(diesel::upsert::excluded(anime::title_native)),
                        anime::title_synonyms.eq(diesel::upsert::excluded(anime::title_synonyms)),
                        anime::banner_image.eq(diesel::upsert::excluded(anime::banner_image)),
                        anime::trailer_url.eq(diesel::upsert::excluded(anime::trailer_url)),
                        anime::tier.eq(diesel::upsert::excluded(anime::tier)),
                        anime::quality_metrics.eq(diesel::upsert::excluded(anime::quality_metrics)),
                        anime::status.eq(diesel::upsert::excluded(anime::status)),
                        anime::anime_type.eq(diesel::upsert::excluded(anime::anime_type)),
                        anime::age_restriction.eq(diesel::upsert::excluded(anime::age_restriction)),
                        anime::last_synced_at.eq(diesel::upsert::excluded(anime::last_synced_at)),
                    ))
                    .get_results::<Anime>(conn)?;

                log_debug!(
                    "Bulk anime upsert completed in {:.2}ms for {} records",
                    upsert_start.elapsed().as_secs_f64() * 1000.0,
                    saved_anime.len()
                );

                // Step 2: Bulk handle external IDs for all anime
                let external_ids_start = std::time::Instant::now();
                Self::bulk_upsert_external_ids_blocking(conn, &saved_anime, &to_upsert)?;
                log_debug!(
                    "Bulk external IDs processing completed in {:.2}ms",
                    external_ids_start.elapsed().as_secs_f64() * 1000.0
                );

                log_debug!(
                    "Database transaction completed in {:.2}ms",
                    transaction_start.elapsed().as_secs_f64() * 1000.0
                );

                Ok(saved_anime)
            })
        })
        .await??;

        // Step 3: Bulk process related data (genres, studios, quality_metrics)
        let relations_start = std::time::Instant::now();

        log_debug!(
            "Starting bulk relations processing for {} anime",
            saved_models.len()
        );
        self.bulk_upsert_all_relations(&saved_models, anime_list)
            .await?;

        log_debug!(
            "Bulk relations processing completed in {:.2}ms",
            relations_start.elapsed().as_secs_f64() * 1000.0
        );

        // Step 4: Load final results with relations
        let load_start = std::time::Instant::now();
        let final_results = self.load_anime_batch_with_relations(saved_models).await?;

        log_debug!(
            "Final data loading completed in {:.2}ms",
            load_start.elapsed().as_secs_f64() * 1000.0
        );

        log_debug!(
            "Bulk save operation completed in {:.2}ms for {} anime (avg: {:.2}ms per anime)",
            batch_start.elapsed().as_secs_f64() * 1000.0,
            final_results.len(),
            batch_start.elapsed().as_secs_f64() * 1000.0 / final_results.len() as f64
        );

        Ok(final_results)
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
        log_debug!(
            "SEARCHING for existing anime with title: '{}'",
            search_title
        );
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
            log_debug!(
                "SEARCH RESULT: No existing anime found for title: '{}'",
                search_title
            );
            Ok(None)
        } else {
            log_debug!("SEARCH RESULT: Found {} existing anime for title: '{}', using first match with ID: {}",
                      anime_models.len(), search_title, anime_models[0].id);
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

            log_debug!(
                "Starting database transaction for anime: {}",
                anime_clone.title.main
            );
            conn.transaction::<Anime, AppError, _>(|conn| {
                // First, check if anime already exists by external ID
                let existing_anime_id = {
                    use crate::schema::anime_external_ids;

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
                    log_debug!(
                        "UPDATING existing anime with ID: {} for title: {}",
                        existing_id,
                        anime_clone.title.main
                    );
                    diesel::update(anime::table.filter(anime::id.eq(existing_id)))
                        .set(&changes)
                        .get_result::<Anime>(conn)?
                } else {
                    // Insert new anime
                    log_debug!("INSERTING new anime for title: {}", anime_clone.title.main);
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

                log_debug!(
                    "DATABASE TRANSACTION COMPLETED for anime: {} (ID: {})",
                    anime_clone.title.main,
                    saved_anime.id
                );
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
        use crate::schema::anime_external_ids;
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

    /// Bulk upsert external IDs for multiple anime - optimized for batch operations
    fn bulk_upsert_external_ids_blocking(
        conn: &mut diesel::PgConnection,
        saved_anime: &[Anime],
        original_anime: &[AnimeDetailed],
    ) -> AppResult<()> {
        use crate::schema::anime_external_ids;
        use diesel::prelude::*;

        // Collect all external ID records to insert
        let mut external_id_records = Vec::new();
        let current_time = chrono::Utc::now();

        for (saved, original) in saved_anime.iter().zip(original_anime.iter()) {
            for (provider, external_id) in &original.provider_metadata.external_ids {
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

                let is_primary = provider == &original.provider_metadata.primary_provider;

                external_id_records.push((
                    anime_external_ids::anime_id.eq(saved.id),
                    anime_external_ids::provider_code.eq(provider_code),
                    anime_external_ids::external_id.eq(external_id.clone()),
                    anime_external_ids::is_primary.eq(is_primary),
                    anime_external_ids::last_synced.eq(current_time),
                ));
            }
        }

        // Bulk insert external IDs if any exist
        if !external_id_records.is_empty() {
            log_debug!(
                "Bulk upserting {} external ID records",
                external_id_records.len()
            );
            diesel::insert_into(anime_external_ids::table)
                .values(&external_id_records)
                .on_conflict((
                    anime_external_ids::anime_id,
                    anime_external_ids::provider_code,
                ))
                .do_update()
                .set((
                    anime_external_ids::external_id
                        .eq(diesel::upsert::excluded(anime_external_ids::external_id)),
                    anime_external_ids::is_primary
                        .eq(diesel::upsert::excluded(anime_external_ids::is_primary)),
                    anime_external_ids::last_synced.eq(current_time),
                ))
                .execute(conn)?;
        }

        Ok(())
    }

    /// Bulk process all relations (genres, studios, quality_metrics) for multiple anime
    async fn bulk_upsert_all_relations(
        &self,
        saved_anime: &[Anime],
        original_anime: &[AnimeDetailed],
    ) -> AppResult<()> {
        // Process all relations in parallel for better performance
        let genres_task = self.bulk_upsert_genres(saved_anime, original_anime);
        let studios_task = self.bulk_upsert_studios(saved_anime, original_anime);
        let metrics_task = self.bulk_upsert_quality_metrics(saved_anime, original_anime);

        // Wait for all operations to complete
        tokio::try_join!(genres_task, studios_task, metrics_task)?;

        Ok(())
    }

    /// Bulk upsert genres for multiple anime
    async fn bulk_upsert_genres(
        &self,
        saved_anime: &[Anime],
        original_anime: &[AnimeDetailed],
    ) -> AppResult<()> {
        let db = Arc::clone(&self.db);
        let anime_pairs: Vec<(Uuid, Vec<Genre>)> = saved_anime
            .iter()
            .zip(original_anime.iter())
            .map(|(saved, original)| (saved.id, original.genres.clone()))
            .collect();

        task::spawn_blocking(move || -> AppResult<()> {
            let mut conn = db.get_connection()?;

            conn.transaction::<_, AppError, _>(|conn| {
                // Step 1: Collect all unique genres to upsert
                let mut all_genres = std::collections::HashMap::new();
                for (_, genres) in &anime_pairs {
                    for genre in genres {
                        all_genres.insert(genre.name.clone(), genre.clone());
                    }
                }

                // Step 2: Bulk upsert all unique genres
                if !all_genres.is_empty() {
                    let genre_records: Vec<NewGenre> = all_genres
                        .values()
                        .map(|g| NewGenre {
                            id: g.id,
                            name: g.name.clone(),
                        })
                        .collect();

                    log_debug!("Bulk upserting {} unique genres", genre_records.len());
                    diesel::insert_into(genres::table)
                        .values(&genre_records)
                        .on_conflict(genres::name)
                        .do_update()
                        .set(genres::name.eq(diesel::upsert::excluded(genres::name)))
                        .execute(conn)?;
                }

                // Step 3: Get genre IDs for association
                let genre_name_to_id: std::collections::HashMap<String, Uuid> =
                    if !all_genres.is_empty() {
                        let names: Vec<String> = all_genres.keys().cloned().collect();
                        genres::table
                            .filter(genres::name.eq_any(&names))
                            .select((genres::name, genres::id))
                            .load::<(String, Uuid)>(conn)?
                            .into_iter()
                            .collect()
                    } else {
                        std::collections::HashMap::new()
                    };

                // Step 4: Clear existing associations and create new ones in bulk
                let anime_ids: Vec<Uuid> = anime_pairs.iter().map(|(id, _)| *id).collect();

                if !anime_ids.is_empty() {
                    diesel::delete(
                        anime_genres::table.filter(anime_genres::anime_id.eq_any(&anime_ids)),
                    )
                    .execute(conn)?;

                    let mut association_records = Vec::new();
                    for (anime_id, genres) in &anime_pairs {
                        for genre in genres {
                            if let Some(genre_id) = genre_name_to_id.get(&genre.name) {
                                association_records.push(NewAnimeGenre {
                                    anime_id: *anime_id,
                                    genre_id: *genre_id,
                                });
                            }
                        }
                    }

                    if !association_records.is_empty() {
                        log_debug!(
                            "Bulk inserting {} anime-genre associations",
                            association_records.len()
                        );
                        diesel::insert_into(anime_genres::table)
                            .values(&association_records)
                            .on_conflict_do_nothing()
                            .execute(conn)?;
                    }
                }

                Ok(())
            })
        })
        .await?
    }

    /// Bulk upsert studios for multiple anime
    async fn bulk_upsert_studios(
        &self,
        saved_anime: &[Anime],
        original_anime: &[AnimeDetailed],
    ) -> AppResult<()> {
        let db = Arc::clone(&self.db);
        let anime_pairs: Vec<(Uuid, Vec<String>)> = saved_anime
            .iter()
            .zip(original_anime.iter())
            .map(|(saved, original)| (saved.id, original.studios.clone()))
            .collect();

        task::spawn_blocking(move || -> AppResult<()> {
            let mut conn = db.get_connection()?;

            conn.transaction::<_, AppError, _>(|conn| {
                // Step 1: Collect all unique studios to upsert
                let mut all_studios = std::collections::HashSet::new();
                for (_, studios) in &anime_pairs {
                    for studio in studios {
                        all_studios.insert(studio.clone());
                    }
                }

                // Step 2: Bulk upsert all unique studios
                if !all_studios.is_empty() {
                    let studio_records: Vec<NewStudio> = all_studios
                        .iter()
                        .map(|name| NewStudio {
                            id: Uuid::new_v4(),
                            name: name.clone(),
                        })
                        .collect();

                    log_debug!("Bulk upserting {} unique studios", studio_records.len());
                    diesel::insert_into(studios::table)
                        .values(&studio_records)
                        .on_conflict(studios::name)
                        .do_nothing()
                        .execute(conn)?;
                }

                // Step 3: Get studio IDs for association
                let studio_name_to_id: std::collections::HashMap<String, Uuid> =
                    if !all_studios.is_empty() {
                        let names: Vec<String> = all_studios.into_iter().collect();
                        studios::table
                            .filter(studios::name.eq_any(&names))
                            .select((studios::name, studios::id))
                            .load::<(String, Uuid)>(conn)?
                            .into_iter()
                            .collect()
                    } else {
                        std::collections::HashMap::new()
                    };

                // Step 4: Clear existing associations and create new ones in bulk
                let anime_ids: Vec<Uuid> = anime_pairs.iter().map(|(id, _)| *id).collect();

                if !anime_ids.is_empty() {
                    diesel::delete(
                        anime_studios::table.filter(anime_studios::anime_id.eq_any(&anime_ids)),
                    )
                    .execute(conn)?;

                    let mut association_records = Vec::new();
                    for (anime_id, studios) in &anime_pairs {
                        for studio_name in studios {
                            if let Some(studio_id) = studio_name_to_id.get(studio_name) {
                                association_records.push(NewAnimeStudio {
                                    anime_id: *anime_id,
                                    studio_id: *studio_id,
                                });
                            }
                        }
                    }

                    if !association_records.is_empty() {
                        log_debug!(
                            "Bulk inserting {} anime-studio associations",
                            association_records.len()
                        );
                        diesel::insert_into(anime_studios::table)
                            .values(&association_records)
                            .on_conflict_do_nothing()
                            .execute(conn)?;
                    }
                }

                Ok(())
            })
        })
        .await?
    }

    /// Bulk upsert quality metrics for multiple anime
    async fn bulk_upsert_quality_metrics(
        &self,
        saved_anime: &[Anime],
        original_anime: &[AnimeDetailed],
    ) -> AppResult<()> {
        let db = Arc::clone(&self.db);
        let anime_pairs: Vec<(Uuid, QualityMetrics)> = saved_anime
            .iter()
            .zip(original_anime.iter())
            .map(|(saved, original)| (saved.id, original.quality_metrics.clone()))
            .collect();

        task::spawn_blocking(move || -> AppResult<()> {
            let mut conn = db.get_connection()?;

            if anime_pairs.is_empty() {
                return Ok(());
            }

            let metrics_records: Vec<NewQualityMetrics> = anime_pairs
                .iter()
                .map(|(anime_id, metrics)| NewQualityMetrics {
                    id: Uuid::new_v4(),
                    anime_id: *anime_id,
                    popularity_score: metrics.popularity_score,
                    engagement_score: metrics.engagement_score,
                    consistency_score: metrics.consistency_score,
                    audience_reach_score: metrics.audience_reach_score,
                })
                .collect();

            log_debug!(
                "Bulk upserting {} quality metrics records",
                metrics_records.len()
            );

            let current_time = chrono::Utc::now();
            diesel::insert_into(quality_metrics::table)
                .values(&metrics_records)
                .on_conflict(quality_metrics::anime_id)
                .do_update()
                .set((
                    quality_metrics::popularity_score
                        .eq(diesel::upsert::excluded(quality_metrics::popularity_score)),
                    quality_metrics::engagement_score
                        .eq(diesel::upsert::excluded(quality_metrics::engagement_score)),
                    quality_metrics::consistency_score
                        .eq(diesel::upsert::excluded(quality_metrics::consistency_score)),
                    quality_metrics::audience_reach_score.eq(diesel::upsert::excluded(
                        quality_metrics::audience_reach_score,
                    )),
                    quality_metrics::updated_at.eq(current_time),
                ))
                .execute(&mut conn)?;

            Ok(())
        })
        .await?
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
                    log_debug!("Processing genre: {}", g.name);

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
                            log_debug!("Successfully upserted genre: {} (ID: {})", g.name, id);
                            id
                        }
                        Err(e) => {
                            log_warn!("Genre upsert failed, trying fallback for '{}': {}", g.name, e);
                            // Fallback: find existing genre by name
                            match genres::table
                                .filter(genres::name.eq(&g.name))
                                .select(genres::id)
                                .first::<Uuid>(conn)
                            {
                                Ok(existing_id) => {
                                    log_debug!("Found existing genre: {} (ID: {})", g.name, existing_id);
                                    existing_id
                                }
                                Err(find_err) => {
                                    log_error!("Failed to find genre '{}': {}", g.name, find_err);
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
                            log_debug!("Associated genre {} with anime (rows affected: {})", g.name, rows_affected);
                        }
                        Err(e) => {
                            log_error!("Failed to associate genre {} with anime: {}", g.name, e);
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
