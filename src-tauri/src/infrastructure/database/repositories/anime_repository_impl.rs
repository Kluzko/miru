use crate::domain::{
    entities::{AiredDates, Anime, Genre},
    repositories::AnimeRepository,
    value_objects::{AnimeTier, QualityMetrics},
};
use crate::infrastructure::database::{connection::Database, models::*, schema};
use crate::shared::{
    errors::{AppError, AppResult},
    utils::Validator,
};
use async_trait::async_trait;
use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

pub struct AnimeRepositoryImpl {
    db: Arc<Database>,
}

impl AnimeRepositoryImpl {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    fn model_to_entity(
        &self,
        model: AnimeModel,
        genres: Vec<Genre>,
        studios: Vec<String>,
        metrics: Option<QualityMetricsModel>,
    ) -> Anime {
        let quality_metrics = if let Some(m) = metrics {
            QualityMetrics {
                popularity_score: m.popularity_score,
                engagement_score: m.engagement_score,
                consistency_score: m.consistency_score,
                audience_reach_score: m.audience_reach_score,
            }
        } else {
            QualityMetrics::default()
        };

        Anime {
            id: model.id,
            mal_id: model.mal_id,
            title: model.title,
            title_english: model.title_english,
            title_japanese: model.title_japanese,
            score: model.score,
            scored_by: model.scored_by,
            rank: model.rank,
            popularity: model.popularity,
            members: model.members,
            favorites: model.favorites,
            synopsis: model.synopsis,
            episodes: model.episodes,
            status: model.status.as_str().into(),
            aired: AiredDates {
                from: model.aired_from,
                to: model.aired_to,
            },
            anime_type: model.anime_type.as_str().into(),
            rating: model.rating,
            genres,
            studios,
            source: model.source,
            duration: model.duration,
            image_url: model.image_url,
            mal_url: model.mal_url,
            composite_score: model.composite_score,
            tier: AnimeTier::new(model.composite_score),
            quality_metrics,
        }
    }

    fn entity_to_model(&self, anime: &Anime) -> AnimeModel {
        AnimeModel {
            id: anime.id,
            mal_id: anime.mal_id,
            title: anime.title.clone(),
            title_english: anime.title_english.clone(),
            title_japanese: anime.title_japanese.clone(),
            score: anime.score,
            scored_by: anime.scored_by,
            rank: anime.rank,
            popularity: anime.popularity,
            members: anime.members,
            favorites: anime.favorites,
            synopsis: anime.synopsis.clone(),
            episodes: anime.episodes,
            status: anime.status.to_string(),
            aired_from: anime.aired.from,
            aired_to: anime.aired.to,
            anime_type: anime.anime_type.to_string(),
            rating: anime.rating.clone(),
            source: anime.source.clone(),
            duration: anime.duration.clone(),
            image_url: anime.image_url.clone(),
            mal_url: anime.mal_url.clone(),
            composite_score: anime.composite_score,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    async fn get_anime_genres(&self, anime_id: &Uuid) -> AppResult<Vec<Genre>> {
        use schema::{anime_genres, genres};

        let mut conn = self.db.get_connection()?;

        let genre_models = anime_genres::table
            .inner_join(genres::table.on(genres::id.eq(anime_genres::genre_id)))
            .filter(anime_genres::anime_id.eq(anime_id))
            .select(genres::all_columns)
            .load::<GenreModel>(&mut conn)?;

        Ok(genre_models
            .into_iter()
            .map(|g| Genre {
                id: g.id,
                mal_id: g.mal_id,
                name: g.name,
            })
            .collect())
    }

    async fn get_anime_studios(&self, anime_id: &Uuid) -> AppResult<Vec<String>> {
        use schema::{anime_studios, studios};

        let mut conn = self.db.get_connection()?;

        let studio_names = anime_studios::table
            .inner_join(studios::table.on(studios::id.eq(anime_studios::studio_id)))
            .filter(anime_studios::anime_id.eq(anime_id))
            .select(studios::name)
            .load::<String>(&mut conn)?;

        Ok(studio_names)
    }

    async fn get_quality_metrics(&self, anime_id: &Uuid) -> AppResult<Option<QualityMetricsModel>> {
        use schema::quality_metrics;

        let mut conn = self.db.get_connection()?;

        quality_metrics::table
            .filter(quality_metrics::anime_id.eq(anime_id))
            .first::<QualityMetricsModel>(&mut conn)
            .optional()
            .map_err(AppError::from)
    }

    fn save_anime_with_relations(
        &self,
        conn: &mut diesel::PgConnection,
        anime: &Anime,
    ) -> AppResult<Uuid> {
        use schema::{
            anime as anime_table, anime_genres, anime_studios, genres, quality_metrics, studios,
        };

        // Determine the actual anime ID to use
        let anime_id = if let Some(mal_id) = anime.mal_id {
            // Check if anime with this mal_id already exists
            let existing_id = anime_table::table
                .filter(anime_table::mal_id.eq(mal_id))
                .select(anime_table::id)
                .first::<Uuid>(conn)
                .optional()?;

            if let Some(id) = existing_id {
                // Update existing anime
                let mut model = self.entity_to_model(anime);
                model.id = id; // Use the existing ID

                diesel::update(anime_table::table)
                    .filter(anime_table::id.eq(id))
                    .set(&model)
                    .execute(conn)?;

                id
            } else {
                // Insert new anime
                let model = self.entity_to_model(anime);
                diesel::insert_into(anime_table::table)
                    .values(&model)
                    .execute(conn)?;

                anime.id
            }
        } else {
            // No mal_id, use regular upsert by ID
            let model = self.entity_to_model(anime);
            diesel::insert_into(anime_table::table)
                .values(&model)
                .on_conflict(anime_table::id)
                .do_update()
                .set(&model)
                .execute(conn)?;

            anime.id
        };

        // Clear existing relationships
        diesel::delete(anime_genres::table.filter(anime_genres::anime_id.eq(&anime_id)))
            .execute(conn)?;
        diesel::delete(anime_studios::table.filter(anime_studios::anime_id.eq(&anime_id)))
            .execute(conn)?;

        // Save genres
        for genre in &anime.genres {
            // Get or create genre
            let genre_id = if let Some(mal_id) = genre.mal_id {
                // Try to find by mal_id
                let existing = genres::table
                    .filter(genres::mal_id.eq(mal_id))
                    .select(genres::id)
                    .first::<Uuid>(conn)
                    .optional()?;

                if let Some(id) = existing {
                    id
                } else {
                    // Create new genre
                    diesel::insert_into(genres::table)
                        .values(&GenreModel {
                            id: Uuid::new_v4(),
                            mal_id: Some(mal_id),
                            name: genre.name.clone(),
                        })
                        .returning(genres::id)
                        .get_result::<Uuid>(conn)?
                }
            } else {
                // Try to find by name
                let existing = genres::table
                    .filter(genres::name.eq(&genre.name))
                    .select(genres::id)
                    .first::<Uuid>(conn)
                    .optional()?;

                if let Some(id) = existing {
                    id
                } else {
                    // Create new genre
                    diesel::insert_into(genres::table)
                        .values(&GenreModel {
                            id: Uuid::new_v4(),
                            mal_id: None,
                            name: genre.name.clone(),
                        })
                        .returning(genres::id)
                        .get_result::<Uuid>(conn)?
                }
            };

            // Create anime-genre relationship
            diesel::insert_into(anime_genres::table)
                .values(&AnimeGenreModel { anime_id, genre_id })
                .execute(conn)?;
        }

        // Save studios
        for studio_name in &anime.studios {
            // Get or create studio
            let studio_id = studios::table
                .filter(studios::name.eq(studio_name))
                .select(studios::id)
                .first::<Uuid>(conn)
                .optional()?
                .unwrap_or_else(|| {
                    // Create new studio if it doesn't exist
                    let new_id = Uuid::new_v4();
                    diesel::insert_into(studios::table)
                        .values(&StudioModel {
                            id: new_id,
                            name: studio_name.clone(),
                        })
                        .execute(conn)
                        .ok();
                    new_id
                });

            // Create anime-studio relationship
            diesel::insert_into(anime_studios::table)
                .values(&AnimeStudioModel {
                    anime_id,
                    studio_id,
                })
                .execute(conn)?;
        }

        // Save quality metrics
        let metrics_model = QualityMetricsModel {
            id: Uuid::new_v4(),
            anime_id,
            popularity_score: anime.quality_metrics.popularity_score,
            engagement_score: anime.quality_metrics.engagement_score,
            consistency_score: anime.quality_metrics.consistency_score,
            audience_reach_score: anime.quality_metrics.audience_reach_score,
            updated_at: chrono::Utc::now(),
        };

        diesel::insert_into(quality_metrics::table)
            .values(&metrics_model)
            .on_conflict(quality_metrics::anime_id)
            .do_update()
            .set(&metrics_model)
            .execute(conn)?;

        Ok(anime_id)
    }
}

#[async_trait]
impl AnimeRepository for AnimeRepositoryImpl {
    async fn find_by_id(&self, id: &Uuid) -> AppResult<Option<Anime>> {
        use schema::anime;

        let mut conn = self.db.get_connection()?;

        let model = anime::table
            .filter(anime::id.eq(id))
            .first::<AnimeModel>(&mut conn)
            .optional()?;

        match model {
            Some(m) => {
                let genres = self.get_anime_genres(&m.id).await?;
                let studios = self.get_anime_studios(&m.id).await?;
                let metrics = self.get_quality_metrics(&m.id).await?;
                Ok(Some(self.model_to_entity(m, genres, studios, metrics)))
            }
            None => Ok(None),
        }
    }

    async fn find_by_mal_id(&self, mal_id: i32) -> AppResult<Option<Anime>> {
        use schema::anime;

        Validator::validate_mal_id(mal_id)?;

        let mut conn = self.db.get_connection()?;

        let model = anime::table
            .filter(anime::mal_id.eq(mal_id))
            .first::<AnimeModel>(&mut conn)
            .optional()?;

        match model {
            Some(m) => {
                let genres = self.get_anime_genres(&m.id).await?;
                let studios = self.get_anime_studios(&m.id).await?;
                let metrics = self.get_quality_metrics(&m.id).await?;
                Ok(Some(self.model_to_entity(m, genres, studios, metrics)))
            }
            None => Ok(None),
        }
    }

    async fn search(&self, query: &str, limit: usize) -> AppResult<Vec<Anime>> {
        use schema::anime;

        if query.is_empty() {
            return Err(AppError::InvalidInput(
                "Search query cannot be empty".to_string(),
            ));
        }

        let mut conn = self.db.get_connection()?;
        let pattern = format!("%{}%", query.to_lowercase());

        let models = anime::table
            .filter(
                anime::title
                    .ilike(&pattern)
                    .or(anime::title_english.ilike(&pattern))
                    .or(anime::title_japanese.ilike(&pattern)),
            )
            .limit(limit as i64)
            .order(anime::composite_score.desc())
            .load::<AnimeModel>(&mut conn)?;

        let mut results = Vec::new();
        for model in models {
            let genres = self.get_anime_genres(&model.id).await?;
            let studios = self.get_anime_studios(&model.id).await?;
            let metrics = self.get_quality_metrics(&model.id).await?;
            results.push(self.model_to_entity(model, genres, studios, metrics));
        }

        Ok(results)
    }

    async fn save(&self, anime: &Anime) -> AppResult<Anime> {
        Validator::validate_anime_title(&anime.title)?;
        if let Some(score) = anime.score {
            Validator::validate_score(score)?;
        }

        let mut conn = self.db.get_connection()?;

        // Use transaction for atomicity
        let saved_id =
            conn.transaction::<_, AppError, _>(|conn| self.save_anime_with_relations(conn, anime))?;

        // Return the saved anime with the correct ID
        if saved_id != anime.id {
            // The anime was updated with an existing ID
            self.find_by_id(&saved_id).await?.ok_or_else(|| {
                AppError::InternalError("Failed to retrieve saved anime".to_string())
            })
        } else {
            Ok(anime.clone())
        }
    }

    async fn save_batch(&self, anime_list: &[Anime]) -> AppResult<Vec<Anime>> {
        let mut results = Vec::new();

        for anime in anime_list {
            match self.save(anime).await {
                Ok(saved) => results.push(saved),
                Err(e) => {
                    // Log error but continue with other anime
                    eprintln!("Failed to save anime {}: {}", anime.title, e);
                }
            }
        }

        if results.is_empty() && !anime_list.is_empty() {
            return Err(AppError::DatabaseError(
                "Failed to save any anime".to_string(),
            ));
        }

        Ok(results)
    }

    async fn update(&self, anime: &Anime) -> AppResult<Anime> {
        // Check if anime exists
        if self.find_by_id(&anime.id).await?.is_none() {
            return Err(AppError::NotFound(format!(
                "Anime with ID {} not found",
                anime.id
            )));
        }

        self.save(anime).await
    }

    async fn delete(&self, id: &Uuid) -> AppResult<()> {
        use schema::anime;

        let mut conn = self.db.get_connection()?;

        let deleted_count =
            diesel::delete(anime::table.filter(anime::id.eq(id))).execute(&mut conn)?;

        if deleted_count == 0 {
            return Err(AppError::NotFound(format!(
                "Anime with ID {} not found",
                id
            )));
        }

        Ok(())
    }

    async fn get_all(&self, offset: i64, limit: i64) -> AppResult<Vec<Anime>> {
        use schema::anime;

        Validator::validate_pagination(offset, limit)?;

        let mut conn = self.db.get_connection()?;

        let models = anime::table
            .offset(offset)
            .limit(limit)
            .order(anime::composite_score.desc())
            .load::<AnimeModel>(&mut conn)?;

        let mut results = Vec::new();
        for model in models {
            let genres = self.get_anime_genres(&model.id).await?;
            let studios = self.get_anime_studios(&model.id).await?;
            let metrics = self.get_quality_metrics(&model.id).await?;
            results.push(self.model_to_entity(model, genres, studios, metrics));
        }

        Ok(results)
    }
}
