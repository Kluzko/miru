// src/infrastructure/database/repositories/collection_repository_impl.rs

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use diesel::prelude::*;
use tokio::task;
use uuid::Uuid;

use crate::domain::{
    entities::{
        anime_detailed::{AiredDates, AnimeDetailed},
        Collection, CollectionAnime, Genre,
    },
    repositories::CollectionRepository,
    value_objects::{AnimeProvider, AnimeTitle, ProviderMetadata, QualityMetrics},
};
use crate::infrastructure::database::{
    connection::Database,
    models::{
        Anime, AnimeGenre, AnimeStudio, CollectionAnime as CollectionAnimeModel,
        CollectionAnimeChangeset, CollectionChangeset, CollectionModel, GenreModel, NewCollection,
        NewCollectionAnime, QualityMetricsModel, StudioModel,
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

    // Helper: Convert Anime to AnimeDetailed (same as in anime repository)
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
            scored_by: model.scored_by.map(|v| v as u32),
            rank: model.rank.map(|v| v as u32),
            popularity: model.popularity.map(|v| v as u32),
            members: model.members.map(|v| v as u32),
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
        }
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
        ??;

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
        ??;

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
        ??;

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
        ?
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
        ?
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
        ?
    }

    async fn get_collection_anime(&self, collection_id: &Uuid) -> AppResult<Vec<AnimeDetailed>> {
        let db = Arc::clone(&self.db);
        let collection_id = *collection_id;

        let anime_models = task::spawn_blocking(move || -> AppResult<Vec<Anime>> {
            let mut conn = db.get_connection()?;

            // join + select only anime columns; maintain ordering by added_at
            let rows = collection_anime::table
                .inner_join(anime::table.on(anime::id.eq(collection_anime::anime_id)))
                .filter(collection_anime::collection_id.eq(collection_id))
                .select(anime::all_columns)
                .order(collection_anime::added_at.desc())
                .load::<Anime>(&mut conn)?;
            Ok(rows)
        })
        .await
        ??;

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
        ??;

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
        ?
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
        ??;

        Ok(results)
    }

    /// Eager-load Anime relations in batch (genres, studios, metrics).
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

                    Self::model_to_entity(m, genres, studios, Some(quality_metrics))
                })
                .collect::<Vec<_>>();

            Ok(out)
        })
        .await
        ??;

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
        ?
    }
}
