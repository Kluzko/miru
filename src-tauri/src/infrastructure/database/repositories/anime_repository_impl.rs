use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use diesel::prelude::*;
use tokio::task;
use uuid::Uuid;

use crate::domain::{
    entities::{AiredDates, Anime, Genre},
    repositories::AnimeRepository,
    value_objects::{AnimeTier, QualityMetrics},
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
}

// -------------------------------------------------------------------------
// Public API methods first (nicer for readers): CRUD + search + listing
// -------------------------------------------------------------------------

#[async_trait]
impl AnimeRepository for AnimeRepositoryImpl {
    async fn find_by_id(&self, id: &Uuid) -> AppResult<Option<Anime>> {
        let db = Arc::clone(&self.db);
        let id = *id;

        let model = task::spawn_blocking(move || -> AppResult<Option<AnimeModel>> {
            let mut conn = db.get_connection()?;
            let m = anime::table
                .filter(anime::id.eq(id))
                .first::<AnimeModel>(&mut conn)
                .optional()?;
            Ok(m)
        })
        .await
        .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))??;

        match model {
            Some(m) => {
                let v = self.load_anime_batch_with_relations(vec![m]).await?;
                Ok(v.into_iter().next())
            }
            None => Ok(None),
        }
    }

    async fn find_by_mal_id(&self, mal_id: i32) -> AppResult<Option<Anime>> {
        Validator::validate_mal_id(mal_id)?;

        let db = Arc::clone(&self.db);

        let model = task::spawn_blocking(move || -> AppResult<Option<AnimeModel>> {
            let mut conn = db.get_connection()?;
            let m = anime::table
                .filter(anime::mal_id.eq(mal_id))
                .first::<AnimeModel>(&mut conn)
                .optional()?;
            Ok(m)
        })
        .await
        .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))??;

        match model {
            Some(m) => {
                let anime = self.load_anime_batch_with_relations(vec![m]).await?;
                Ok(anime.into_iter().next())
            }
            None => Ok(None),
        }
    }

    async fn search(&self, query: &str, limit: usize) -> AppResult<Vec<Anime>> {
        if query.is_empty() {
            return Err(AppError::InvalidInput(
                "Search query cannot be empty".into(),
            ));
        }

        use diesel::dsl::sql;
        use diesel::sql_types::{Bool, Float4, Text};

        let db = Arc::clone(&self.db);
        let q = query.to_string();

        let models = task::spawn_blocking(move || -> AppResult<Vec<AnimeModel>> {
            let mut conn = db.get_connection()?;

            // Build fragments with bound params (safe; no interpolation).
            let pred = sql::<Bool>("similarity(LOWER(title), LOWER(")
                .bind::<Text, _>(&q)
                .sql(")) > 0.3 OR similarity(LOWER(title_english), LOWER(")
                .bind::<Text, _>(&q)
                .sql(")) > 0.3 OR similarity(LOWER(title_japanese), LOWER(")
                .bind::<Text, _>(&q)
                .sql(")) > 0.3");

            let rank = sql::<Float4>("GREATEST(")
                .sql("similarity(LOWER(title), LOWER(")
                .bind::<Text, _>(&q)
                .sql(")),")
                .sql("similarity(LOWER(title_english), LOWER(")
                .bind::<Text, _>(&q)
                .sql(")),")
                .sql("similarity(LOWER(title_japanese), LOWER(")
                .bind::<Text, _>(&q)
                .sql("))")
                .sql(")");

            let rows = anime::table
                .filter(pred)
                .order((rank.desc(), anime::composite_score.desc()))
                .limit(limit as i64)
                .load::<AnimeModel>(&mut conn)?;
            Ok(rows)
        })
        .await
        .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))??;

        self.load_anime_batch_with_relations(models).await
    }

    async fn save(&self, anime: &Anime) -> AppResult<Anime> {
        Validator::validate_anime_title(&anime.title)?;
        if let Some(score) = anime.score {
            Validator::validate_score(score)?;
        }

        let saved_model = self.upsert_anime(anime).await?;

        self.upsert_genres(saved_model.id, &anime.genres).await?;
        self.upsert_studios(saved_model.id, &anime.studios).await?;
        self.upsert_quality_metrics(saved_model.id, &anime.quality_metrics)
            .await?;

        self.find_by_id(&saved_model.id)
            .await?
            .ok_or_else(|| AppError::InternalError("Failed to retrieve saved anime".to_string()))
    }

    async fn save_batch(&self, anime_list: &[Anime]) -> AppResult<Vec<Anime>> {
        if anime_list.is_empty() {
            return Ok(vec![]);
        }

        let db = Arc::clone(&self.db);
        let to_upsert = anime_list.to_vec();

        let saved_models = task::spawn_blocking(move || -> AppResult<Vec<AnimeModel>> {
            let mut conn = db.get_connection()?;

            conn.transaction::<Vec<AnimeModel>, AppError, _>(|conn| {
                let mut out = Vec::with_capacity(to_upsert.len());

                for a in &to_upsert {
                    let new_row = NewAnime {
                        id: a.id,
                        mal_id: a.mal_id,
                        title: a.title.clone(),
                        title_english: a.title_english.clone(),
                        title_japanese: a.title_japanese.clone(),
                        score: a.score,
                        scored_by: a.scored_by,
                        rank: a.rank,
                        popularity: a.popularity,
                        members: a.members,
                        favorites: a.favorites,
                        synopsis: a.synopsis.clone(),
                        episodes: a.episodes,
                        status: a.status.to_string(),
                        aired_from: a.aired.from,
                        aired_to: a.aired.to,
                        anime_type: a.anime_type.to_string(),
                        rating: a.rating.clone(),
                        source: a.source.clone(),
                        duration: a.duration.clone(),
                        image_url: a.image_url.clone(),
                        mal_url: a.mal_url.clone(),
                        composite_score: a.composite_score,
                    };

                    let changes = AnimeChangeset {
                        mal_id: a.mal_id,
                        title: a.title.clone(),
                        title_english: a.title_english.clone(),
                        title_japanese: a.title_japanese.clone(),
                        score: a.score,
                        scored_by: a.scored_by,
                        rank: a.rank,
                        popularity: a.popularity,
                        members: a.members,
                        favorites: a.favorites,
                        synopsis: a.synopsis.clone(),
                        episodes: a.episodes,
                        status: a.status.to_string(),
                        aired_from: a.aired.from,
                        aired_to: a.aired.to,
                        anime_type: a.anime_type.to_string(),
                        rating: a.rating.clone(),
                        source: a.source.clone(),
                        duration: a.duration.clone(),
                        image_url: a.image_url.clone(),
                        mal_url: a.mal_url.clone(),
                        composite_score: a.composite_score,
                        updated_at: chrono::Utc::now(),
                    };

                    let saved = diesel::insert_into(anime::table)
                        .values(&new_row)
                        .on_conflict(anime::mal_id)
                        .do_update()
                        .set(&changes)
                        .get_result::<AnimeModel>(conn)?;

                    out.push(saved);
                }

                Ok(out)
            })
        })
        .await
        .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))??;

        for (saved, input) in saved_models.iter().zip(anime_list.iter()) {
            self.upsert_genres(saved.id, &input.genres).await?;
            self.upsert_studios(saved.id, &input.studios).await?;
            self.upsert_quality_metrics(saved.id, &input.quality_metrics)
                .await?;
        }

        self.load_anime_batch_with_relations(saved_models).await
    }

    async fn update(&self, anime: &Anime) -> AppResult<Anime> {
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
        .await
        .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))?
    }

    async fn get_all(&self, offset: i64, limit: i64) -> AppResult<Vec<Anime>> {
        Validator::validate_pagination(offset, limit)?;

        let db = Arc::clone(&self.db);

        let models = task::spawn_blocking(move || -> AppResult<Vec<AnimeModel>> {
            let mut conn = db.get_connection()?;
            let rows = anime::table
                .offset(offset)
                .limit(limit)
                .order(anime::composite_score.desc())
                .load::<AnimeModel>(&mut conn)?;
            Ok(rows)
        })
        .await
        .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))??;

        self.load_anime_batch_with_relations(models).await
    }
}

// -----------------------------------------------------------------------------
// Private helpers (kept after the public API for readability)
// -----------------------------------------------------------------------------

impl AnimeRepositoryImpl {
    async fn load_anime_batch_with_relations(
        &self,
        anime_models: Vec<AnimeModel>,
    ) -> AppResult<Vec<Anime>> {
        if anime_models.is_empty() {
            return Ok(Vec::new());
        }

        let db = Arc::clone(&self.db);

        let results = task::spawn_blocking(move || -> AppResult<Vec<Anime>> {
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
                            mal_id: g.mal_id,
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

                    Anime {
                        id: m.id,
                        mal_id: m.mal_id,
                        title: m.title,
                        title_english: m.title_english,
                        title_japanese: m.title_japanese,
                        score: m.score,
                        scored_by: m.scored_by,
                        rank: m.rank,
                        popularity: m.popularity,
                        members: m.members,
                        favorites: m.favorites,
                        synopsis: m.synopsis,
                        episodes: m.episodes,
                        status: m.status.as_str().into(),
                        aired: AiredDates {
                            from: m.aired_from,
                            to: m.aired_to,
                        },
                        anime_type: m.anime_type.as_str().into(),
                        rating: m.rating,
                        genres,
                        studios,
                        source: m.source,
                        duration: m.duration,
                        image_url: m.image_url,
                        mal_url: m.mal_url,
                        composite_score: m.composite_score,
                        tier: AnimeTier::new(m.composite_score),
                        quality_metrics,
                    }
                })
                .collect::<Vec<_>>();

            Ok(out)
        })
        .await
        .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))??;

        Ok(results)
    }

    /// Upsert Anime (by mal_id) and return the saved row.
    async fn upsert_anime(&self, anime: &Anime) -> AppResult<AnimeModel> {
        let db = Arc::clone(&self.db);

        let new_anime = NewAnime {
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
        };

        let changes = AnimeChangeset {
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
            updated_at: chrono::Utc::now(),
        };

        task::spawn_blocking(move || -> AppResult<AnimeModel> {
            let mut conn = db.get_connection()?;

            let saved = diesel::insert_into(anime::table)
                .values(&new_anime)
                .on_conflict(anime::mal_id)
                .do_update()
                .set(&changes)
                .get_result::<AnimeModel>(&mut conn)?;

            Ok(saved)
        })
        .await
        .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))?
    }

    async fn upsert_genres(&self, anime_id: Uuid, genres_in: &[Genre]) -> AppResult<()> {
        let db = Arc::clone(&self.db);
        let genres_vec = genres_in.to_vec();

        task::spawn_blocking(move || -> AppResult<()> {
            let mut conn = db.get_connection()?;

            conn.transaction::<_, AppError, _>(|conn| {
                diesel::delete(anime_genres::table.filter(anime_genres::anime_id.eq(anime_id)))
                    .execute(conn)?;

                for g in genres_vec {
                    let new_g = NewGenre {
                        id: g.id,
                        mal_id: g.mal_id,
                        name: g.name.clone(),
                    };

                    let genre_id = if g.mal_id.is_some() {
                        diesel::insert_into(genres::table)
                            .values(&new_g)
                            .on_conflict(genres::mal_id)
                            .do_update()
                            .set(genres::name.eq(&g.name))
                            .returning(genres::id)
                            .get_result::<Uuid>(conn)?
                    } else {
                        diesel::insert_into(genres::table)
                            .values(&new_g)
                            .on_conflict_do_nothing()
                            .execute(conn)?;
                        genres::table
                            .filter(genres::name.eq(&g.name))
                            .select(genres::id)
                            .first::<Uuid>(conn)?
                    };

                    diesel::insert_into(anime_genres::table)
                        .values(NewAnimeGenre { anime_id, genre_id })
                        .on_conflict_do_nothing()
                        .execute(conn)?;
                }

                Ok(())
            })
        })
        .await
        .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))?
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
        .await
        .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))?
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
        .await
        .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))?
    }
}
