// src/infrastructure/database/repositories/collection_repository_impl.rs

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use diesel::prelude::*;
use tokio::task;
use uuid::Uuid;

use crate::domain::{
    entities::{Anime, Collection, CollectionAnime, Genre},
    repositories::CollectionRepository,
    value_objects::{AnimeTier, QualityMetrics},
};
use crate::infrastructure::database::{
    connection::Database,
    models::{
        AnimeGenre, AnimeModel, AnimeStudio, CollectionAnimeChangeset, CollectionAnimeModel,
        CollectionChangeset, CollectionModel, GenreModel, NewCollection, NewCollectionAnime,
        QualityMetricsModel, StudioModel,
    },
    // Keep schema imports minimal; only what we actually reference here.
    schema::{anime, anime_genres, anime_studios, collection_anime, collections, genres, studios},
};
use crate::shared::errors::{AppError, AppResult};

pub struct CollectionRepositoryImpl {
    db: Arc<Database>,
}

impl CollectionRepositoryImpl {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl CollectionRepository for CollectionRepositoryImpl {
    // -------------------------------------------------------------------------
    // Public API (listed first for readability)
    // -------------------------------------------------------------------------

    async fn find_by_id(&self, id: &Uuid) -> AppResult<Option<Collection>> {
        let db = Arc::clone(&self.db);
        let id = *id;

        let model = task::spawn_blocking(move || -> AppResult<Option<CollectionModel>> {
            let mut conn = db.get_connection()?;
            let m = collections::table
                .filter(collections::id.eq(id))
                .first::<CollectionModel>(&mut conn)
                .optional()?;
            Ok(m)
        })
        .await
        .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))??;

        match model {
            Some(m) => {
                let out = self.load_collections_with_anime_ids(vec![m]).await?;
                Ok(out.into_iter().next())
            }
            None => Ok(None),
        }
    }

    async fn find_by_name(&self, name: &str) -> AppResult<Option<Collection>> {
        use diesel::dsl::sql;
        use diesel::sql_types::{Bool, Text};

        let db = Arc::clone(&self.db);
        let needle = name.to_string();

        let model = task::spawn_blocking(move || -> AppResult<Option<CollectionModel>> {
            let mut conn = db.get_connection()?;

            // Case-insensitive equality with bindings (no interpolation).
            let pred = sql::<Bool>("LOWER(name) = LOWER(")
                .bind::<Text, _>(&needle)
                .sql(")");

            let m = collections::table
                .filter(pred)
                .first::<CollectionModel>(&mut conn)
                .optional()?;
            Ok(m)
        })
        .await
        .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))??;

        match model {
            Some(m) => {
                let out = self.load_collections_with_anime_ids(vec![m]).await?;
                Ok(out.into_iter().next())
            }
            None => Ok(None),
        }
    }

    async fn get_all(&self) -> AppResult<Vec<Collection>> {
        let db = Arc::clone(&self.db);

        let models = task::spawn_blocking(move || -> AppResult<Vec<CollectionModel>> {
            let mut conn = db.get_connection()?;
            let rows = collections::table
                .order(collections::created_at.desc())
                .load::<CollectionModel>(&mut conn)?;
            Ok(rows)
        })
        .await
        .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))??;

        self.load_collections_with_anime_ids(models).await
    }

    async fn save(&self, collection: &Collection) -> AppResult<Collection> {
        let saved = self.upsert_collection(collection).await?;
        self.find_by_id(&saved.id)
            .await?
            .ok_or_else(|| AppError::InternalError("Failed to retrieve saved collection".into()))
    }

    async fn update(&self, collection: &Collection) -> AppResult<Collection> {
        if self.find_by_id(&collection.id).await?.is_none() {
            return Err(AppError::NotFound(format!(
                "Collection with ID {} not found",
                collection.id
            )));
        }
        self.save(collection).await
    }

    async fn delete(&self, id: &Uuid) -> AppResult<()> {
        let db = Arc::clone(&self.db);
        let id = *id;

        task::spawn_blocking(move || -> AppResult<()> {
            let mut conn = db.get_connection()?;
            let n = diesel::delete(collections::table.filter(collections::id.eq(id)))
                .execute(&mut conn)?;
            if n == 0 {
                return Err(AppError::NotFound(format!(
                    "Collection with ID {} not found",
                    id
                )));
            }
            Ok(())
        })
        .await
        .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))?
    }

    async fn add_anime_to_collection(
        &self,
        collection_id: &Uuid,
        anime_id: &Uuid,
        user_score: Option<f32>,
        notes: Option<String>,
    ) -> AppResult<()> {
        let db = Arc::clone(&self.db);
        let collection_id = *collection_id;
        let anime_id = *anime_id;

        task::spawn_blocking(move || -> AppResult<()> {
            let mut conn = db.get_connection()?;

            let new_entry = NewCollectionAnime {
                collection_id,
                anime_id,
                user_score,
                notes,
            };

            // relies on UNIQUE (collection_id, anime_id) in DB
            diesel::insert_into(collection_anime::table)
                .values(&new_entry)
                .on_conflict((collection_anime::collection_id, collection_anime::anime_id))
                .do_nothing()
                .execute(&mut conn)?;

            Ok(())
        })
        .await
        .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))?
    }

    async fn remove_anime_from_collection(
        &self,
        collection_id: &Uuid,
        anime_id: &Uuid,
    ) -> AppResult<()> {
        let db = Arc::clone(&self.db);
        let collection_id = *collection_id;
        let anime_id = *anime_id;

        task::spawn_blocking(move || -> AppResult<()> {
            let mut conn = db.get_connection()?;

            let n = diesel::delete(
                collection_anime::table
                    .filter(collection_anime::collection_id.eq(collection_id))
                    .filter(collection_anime::anime_id.eq(anime_id)),
            )
            .execute(&mut conn)?;

            if n == 0 {
                return Err(AppError::NotFound("Anime not found in collection".into()));
            }
            Ok(())
        })
        .await
        .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))?
    }

    async fn get_collection_anime(&self, collection_id: &Uuid) -> AppResult<Vec<Anime>> {
        let db = Arc::clone(&self.db);
        let collection_id = *collection_id;

        let anime_models = task::spawn_blocking(move || -> AppResult<Vec<AnimeModel>> {
            let mut conn = db.get_connection()?;

            // join + select only anime columns; maintain ordering by added_at
            let rows = collection_anime::table
                .inner_join(anime::table.on(anime::id.eq(collection_anime::anime_id)))
                .filter(collection_anime::collection_id.eq(collection_id))
                .select(anime::all_columns)
                .order(collection_anime::added_at.desc())
                .load::<AnimeModel>(&mut conn)?;
            Ok(rows)
        })
        .await
        .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))??;

        self.load_anime_batch_with_relations(anime_models).await
    }

    async fn get_collection_entry(
        &self,
        collection_id: &Uuid,
        anime_id: &Uuid,
    ) -> AppResult<Option<CollectionAnime>> {
        let db = Arc::clone(&self.db);
        let collection_id = *collection_id;
        let anime_id = *anime_id;

        let entry = task::spawn_blocking(move || -> AppResult<Option<CollectionAnime>> {
            let mut conn = db.get_connection()?;

            let model = collection_anime::table
                .filter(collection_anime::collection_id.eq(collection_id))
                .filter(collection_anime::anime_id.eq(anime_id))
                .first::<CollectionAnimeModel>(&mut conn)
                .optional()?;

            Ok(model.map(|m| CollectionAnime {
                collection_id: m.collection_id,
                anime_id: m.anime_id,
                added_at: m.added_at,
                user_score: m.user_score,
                notes: m.notes,
            }))
        })
        .await
        .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))??;

        Ok(entry)
    }

    async fn update_collection_entry(&self, entry: &CollectionAnime) -> AppResult<()> {
        let db = Arc::clone(&self.db);
        let entry = entry.clone();

        task::spawn_blocking(move || -> AppResult<()> {
            let mut conn = db.get_connection()?;

            let changes = CollectionAnimeChangeset {
                user_score: entry.user_score,
                notes: entry.notes,
            };

            diesel::update(
                collection_anime::table
                    .filter(collection_anime::collection_id.eq(entry.collection_id))
                    .filter(collection_anime::anime_id.eq(entry.anime_id)),
            )
            .set(&changes)
            .execute(&mut conn)?;

            Ok(())
        })
        .await
        .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))?
    }
}

// -----------------------------------------------------------------------------
// Private helpers (kept after public API for readability)
// -----------------------------------------------------------------------------

impl CollectionRepositoryImpl {
    /// Load collections with their anime IDs, grouped and ordered by added_at desc.
    async fn load_collections_with_anime_ids(
        &self,
        collection_models: Vec<CollectionModel>,
    ) -> AppResult<Vec<Collection>> {
        if collection_models.is_empty() {
            return Ok(Vec::new());
        }

        let db = Arc::clone(&self.db);

        let results = task::spawn_blocking(move || -> AppResult<Vec<Collection>> {
            let mut conn = db.get_connection()?;

            let links: Vec<CollectionAnimeModel> =
                CollectionAnimeModel::belonging_to(&collection_models)
                    .order(collection_anime::added_at.desc())
                    .load::<CollectionAnimeModel>(&mut conn)?;

            let grouped = links.grouped_by(&collection_models);

            let mut by_coll: HashMap<Uuid, Vec<Uuid>> =
                HashMap::with_capacity(collection_models.len());

            for (coll, entries) in collection_models.iter().zip(grouped) {
                let ids = entries.into_iter().map(|e| e.anime_id).collect::<Vec<_>>();
                by_coll.insert(coll.id, ids);
            }

            let out = collection_models
                .into_iter()
                .map(|m| Collection {
                    id: m.id,
                    name: m.name,
                    description: m.description,
                    anime_ids: by_coll.remove(&m.id).unwrap_or_default(),
                    created_at: m.created_at,
                    updated_at: m.updated_at,
                })
                .collect();

            Ok(out)
        })
        .await
        .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))??;

        Ok(results)
    }

    /// Eager-load Anime relations in batch (genres, studios, metrics).
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

            // GENRES
            let pairs_g: Vec<(AnimeGenre, GenreModel)> = AnimeGenre::belonging_to(&anime_models)
                .inner_join(genres::table)
                .select((anime_genres::all_columns, genres::all_columns))
                .load::<(AnimeGenre, GenreModel)>(&mut conn)?;
            let grouped_g = pairs_g.grouped_by(&anime_models);
            let genres_by_anime: HashMap<Uuid, Vec<Genre>> = anime_models
                .iter()
                .zip(grouped_g)
                .map(|(a, pairs)| {
                    (
                        a.id,
                        pairs
                            .into_iter()
                            .map(|(_, g)| Genre {
                                id: g.id,
                                mal_id: g.mal_id,
                                name: g.name,
                            })
                            .collect(),
                    )
                })
                .collect();

            // STUDIOS
            let pairs_s: Vec<(AnimeStudio, StudioModel)> = AnimeStudio::belonging_to(&anime_models)
                .inner_join(studios::table)
                .select((anime_studios::all_columns, studios::all_columns))
                .load::<(AnimeStudio, StudioModel)>(&mut conn)?;
            let grouped_s = pairs_s.grouped_by(&anime_models);
            let studios_by_anime: HashMap<Uuid, Vec<String>> = anime_models
                .iter()
                .zip(grouped_s)
                .map(|(a, pairs)| (a.id, pairs.into_iter().map(|(_, s)| s.name).collect()))
                .collect();

            // METRICS
            let metrics: Vec<QualityMetricsModel> =
                QualityMetricsModel::belonging_to(&anime_models)
                    .load::<QualityMetricsModel>(&mut conn)?;
            let grouped_m = metrics.grouped_by(&anime_models);

            // BUILD
            let out = anime_models
                .into_iter()
                .zip(grouped_m)
                .map(|(m, mvec)| {
                    let genres = genres_by_anime.get(&m.id).cloned().unwrap_or_default();
                    let studios = studios_by_anime.get(&m.id).cloned().unwrap_or_default();
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
                        aired: crate::domain::entities::AiredDates {
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

    /// Upsert collection by ID and return saved row.
    async fn upsert_collection(&self, collection: &Collection) -> AppResult<CollectionModel> {
        let db = Arc::clone(&self.db);

        let new_row = NewCollection {
            id: collection.id,
            name: collection.name.clone(),
            description: collection.description.clone(),
        };
        let changes = CollectionChangeset {
            name: collection.name.clone(),
            description: collection.description.clone(),
            updated_at: chrono::Utc::now(),
        };

        task::spawn_blocking(move || -> AppResult<CollectionModel> {
            let mut conn = db.get_connection()?;

            let saved = diesel::insert_into(collections::table)
                .values(&new_row)
                .on_conflict(collections::id)
                .do_update()
                .set(&changes)
                .get_result::<CollectionModel>(&mut conn)?;

            Ok(saved)
        })
        .await
        .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))?
    }
}
