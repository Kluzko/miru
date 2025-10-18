use diesel::prelude::*;
use std::sync::Arc;
use tokio::task;

use crate::modules::anime::domain::entities::anime_detailed::AnimeDetailed;
use crate::modules::anime::infrastructure::models::Anime;
use crate::schema::anime;
use crate::shared::domain::value_objects::AnimeProvider;
use crate::shared::errors::{AppError, AppResult};
use crate::shared::utils::Validator;
use crate::shared::Database;

use super::anime_repository_impl::AnimeRepositoryImpl;

/// Specification for complex anime searches (following Specification Pattern)
#[derive(Debug, Clone, Default)]
pub struct AnimeSearchSpecification {
    pub title_contains: Option<String>,
    pub min_score: Option<f32>,
    pub max_score: Option<f32>,
    pub providers: Option<Vec<AnimeProvider>>,
    pub genres: Option<Vec<String>>,
    pub year: Option<i32>,
    pub status: Option<String>,
}

pub struct AnimeQueryRepositoryImpl {
    db: Arc<Database>,
    anime_repository: Arc<AnimeRepositoryImpl>,
}

impl AnimeQueryRepositoryImpl {
    pub fn new(db: Arc<Database>, anime_repository: Arc<AnimeRepositoryImpl>) -> Self {
        Self {
            db,
            anime_repository,
        }
    }

    /// Find anime by multiple criteria (Specification Pattern)
    pub async fn find_by_criteria(
        &self,
        specification: AnimeSearchSpecification,
        offset: i64,
        limit: i64,
    ) -> AppResult<Vec<AnimeDetailed>> {
        Validator::validate_pagination(offset, limit)?;

        let db = Arc::clone(&self.db);

        let models = task::spawn_blocking(move || -> AppResult<Vec<Anime>> {
            let mut conn = db.get_connection()?;

            // Start with base query
            let mut query = anime::table.into_boxed();

            // Apply title filter
            if let Some(title) = specification.title_contains {
                let pattern = format!("%{}%", title.to_lowercase());
                query = query.filter(
                    anime::title_main
                        .ilike(pattern.clone())
                        .or(anime::title_english.ilike(pattern.clone()))
                        .or(anime::title_japanese.ilike(pattern.clone()))
                        .or(anime::title_romaji.ilike(pattern)),
                );
            }

            // Apply score filters
            if let Some(min_score) = specification.min_score {
                query = query.filter(anime::score.ge(min_score));
            }
            if let Some(max_score) = specification.max_score {
                query = query.filter(anime::score.le(max_score));
            }

            // Apply status filter
            if let Some(status) = specification.status {
                use crate::modules::anime::domain::value_objects::AnimeStatus;
                // Parse string to AnimeStatus enum
                if let Ok(status_enum) = status.parse::<AnimeStatus>() {
                    query = query.filter(anime::status.eq(status_enum));
                }
            }

            // Apply year filter (based on aired_from)
            if let Some(year) = specification.year {
                use diesel::dsl::sql;
                use diesel::sql_types::Integer;

                query = query.filter(
                    sql::<diesel::sql_types::Bool>("EXTRACT(YEAR FROM aired_from) = ")
                        .bind::<Integer, _>(year),
                );
            }

            // Execute query with pagination
            let rows = query
                .offset(offset)
                .limit(limit)
                .order(anime::composite_score.desc())
                .load::<Anime>(&mut conn)?;

            Ok(rows)
        })
        .await??;

        // Load full anime details with relations
        self.anime_repository
            .load_anime_batch_with_relations(models)
            .await
    }

    /// Count anime matching criteria
    pub async fn count_by_criteria(
        &self,
        specification: AnimeSearchSpecification,
    ) -> AppResult<u64> {
        let db = Arc::clone(&self.db);

        task::spawn_blocking(move || -> AppResult<u64> {
            let mut conn = db.get_connection()?;

            // Start with base query
            let mut query = anime::table.into_boxed();

            // Apply title filter
            if let Some(title) = specification.title_contains {
                let pattern = format!("%{}%", title.to_lowercase());
                query = query.filter(
                    anime::title_main
                        .ilike(pattern.clone())
                        .or(anime::title_english.ilike(pattern.clone()))
                        .or(anime::title_japanese.ilike(pattern.clone()))
                        .or(anime::title_romaji.ilike(pattern)),
                );
            }

            // Apply score filters
            if let Some(min_score) = specification.min_score {
                query = query.filter(anime::score.ge(min_score));
            }
            if let Some(max_score) = specification.max_score {
                query = query.filter(anime::score.le(max_score));
            }

            // Apply status filter
            if let Some(status) = specification.status {
                use crate::modules::anime::domain::value_objects::AnimeStatus;
                // Parse string to AnimeStatus enum
                if let Ok(status_enum) = status.parse::<AnimeStatus>() {
                    query = query.filter(anime::status.eq(status_enum));
                }
            }

            // Apply year filter (based on aired_from)
            if let Some(year) = specification.year {
                use diesel::dsl::sql;
                use diesel::sql_types::Integer;

                query = query.filter(
                    sql::<diesel::sql_types::Bool>("EXTRACT(YEAR FROM aired_from) = ")
                        .bind::<Integer, _>(year),
                );
            }

            // Count results
            let count = query.count().get_result::<i64>(&mut conn)?;

            Ok(count as u64)
        })
        .await?
    }

    /// Find anime by title variations (exact and fuzzy matching)
    pub async fn find_by_title_variations(
        &self,
        search_title: &str,
    ) -> AppResult<Option<AnimeDetailed>> {
        // Delegate to anime repository implementation
        use crate::modules::anime::domain::repositories::anime_repository::AnimeRepository;
        self.anime_repository
            .find_by_title_variations(search_title)
            .await
    }

    /// Get all anime with pagination
    pub async fn get_all(&self, offset: i64, limit: i64) -> AppResult<Vec<AnimeDetailed>> {
        // Delegate to anime repository implementation
        use crate::modules::anime::domain::repositories::anime_repository::AnimeRepository;
        self.anime_repository.get_all(offset, limit).await
    }

    /// Advanced search with fuzzy matching and ranking
    pub async fn advanced_search(
        &self,
        query: &str,
        limit: usize,
    ) -> AppResult<Vec<AnimeDetailed>> {
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

        self.anime_repository
            .load_anime_batch_with_relations(models)
            .await
    }

    /// Search by genre
    pub async fn find_by_genre(
        &self,
        genre_name: &str,
        offset: i64,
        limit: i64,
    ) -> AppResult<Vec<AnimeDetailed>> {
        Validator::validate_pagination(offset, limit)?;

        let db = Arc::clone(&self.db);
        let genre = genre_name.to_string();

        let models = task::spawn_blocking(move || -> AppResult<Vec<Anime>> {
            use crate::schema::{anime_genres, genres};

            let mut conn = db.get_connection()?;

            // Join with genres table to filter by genre name
            let rows = anime::table
                .inner_join(anime_genres::table.inner_join(genres::table))
                .filter(genres::name.eq(&genre))
                .select(anime::all_columns)
                .distinct()
                .offset(offset)
                .limit(limit)
                .order(anime::composite_score.desc())
                .load::<Anime>(&mut conn)?;

            Ok(rows)
        })
        .await??;

        self.anime_repository
            .load_anime_batch_with_relations(models)
            .await
    }

    /// Search by studio
    pub async fn find_by_studio(
        &self,
        studio_name: &str,
        offset: i64,
        limit: i64,
    ) -> AppResult<Vec<AnimeDetailed>> {
        Validator::validate_pagination(offset, limit)?;

        let db = Arc::clone(&self.db);
        let studio = studio_name.to_string();

        let models = task::spawn_blocking(move || -> AppResult<Vec<Anime>> {
            use crate::schema::{anime_studios, studios};

            let mut conn = db.get_connection()?;

            // Join with studios table to filter by studio name
            let rows = anime::table
                .inner_join(anime_studios::table.inner_join(studios::table))
                .filter(studios::name.eq(&studio))
                .select(anime::all_columns)
                .distinct()
                .offset(offset)
                .limit(limit)
                .order(anime::composite_score.desc())
                .load::<Anime>(&mut conn)?;

            Ok(rows)
        })
        .await??;

        self.anime_repository
            .load_anime_batch_with_relations(models)
            .await
    }

    /// Get top-rated anime
    pub async fn find_top_rated(&self, offset: i64, limit: i64) -> AppResult<Vec<AnimeDetailed>> {
        Validator::validate_pagination(offset, limit)?;

        let db = Arc::clone(&self.db);

        let models = task::spawn_blocking(move || -> AppResult<Vec<Anime>> {
            let mut conn = db.get_connection()?;

            let rows = anime::table
                .filter(anime::score.is_not_null())
                .offset(offset)
                .limit(limit)
                .order(anime::score.desc())
                .load::<Anime>(&mut conn)?;

            Ok(rows)
        })
        .await??;

        self.anime_repository
            .load_anime_batch_with_relations(models)
            .await
    }

    /// Get recently updated anime
    pub async fn find_recently_updated(
        &self,
        offset: i64,
        limit: i64,
    ) -> AppResult<Vec<AnimeDetailed>> {
        Validator::validate_pagination(offset, limit)?;

        let db = Arc::clone(&self.db);

        let models = task::spawn_blocking(move || -> AppResult<Vec<Anime>> {
            let mut conn = db.get_connection()?;

            let rows = anime::table
                .offset(offset)
                .limit(limit)
                .order(anime::updated_at.desc())
                .load::<Anime>(&mut conn)?;

            Ok(rows)
        })
        .await??;

        self.anime_repository
            .load_anime_batch_with_relations(models)
            .await
    }
}
