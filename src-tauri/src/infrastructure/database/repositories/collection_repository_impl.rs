use crate::domain::{
    entities::{Anime, Collection, CollectionAnime, Genre},
    repositories::CollectionRepository,
    value_objects::AnimeTier,
};
use crate::infrastructure::database::{
    connection::Database,
    models::{AnimeModel, CollectionAnimeModel, CollectionModel, GenreModel},
    schema,
};
use crate::shared::errors::{AppError, AppResult};
use async_trait::async_trait;
use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

pub struct CollectionRepositoryImpl {
    db: Arc<Database>,
}

impl CollectionRepositoryImpl {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    fn model_to_entity(&self, model: CollectionModel, anime_ids: Vec<Uuid>) -> Collection {
        Collection {
            id: model.id,
            name: model.name,
            description: model.description,
            anime_ids,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }

    fn entity_to_model(&self, collection: &Collection) -> CollectionModel {
        CollectionModel {
            id: collection.id,
            name: collection.name.clone(),
            description: collection.description.clone(),
            created_at: collection.created_at,
            updated_at: collection.updated_at,
        }
    }

    async fn get_collection_anime_ids(&self, collection_id: &Uuid) -> AppResult<Vec<Uuid>> {
        use schema::collection_anime;

        let mut conn = self.db.get_connection()?;

        let anime_ids = collection_anime::table
            .filter(collection_anime::collection_id.eq(collection_id))
            .select(collection_anime::anime_id)
            .load::<Uuid>(&mut conn)?;

        Ok(anime_ids)
    }
}

#[async_trait]
impl CollectionRepository for CollectionRepositoryImpl {
    async fn find_by_id(&self, id: &Uuid) -> AppResult<Option<Collection>> {
        use schema::collections;

        let mut conn = self.db.get_connection()?;

        let model = collections::table
            .filter(collections::id.eq(id))
            .first::<CollectionModel>(&mut conn)
            .optional()?;

        match model {
            Some(m) => {
                let anime_ids = self.get_collection_anime_ids(&m.id).await?;
                Ok(Some(self.model_to_entity(m, anime_ids)))
            }
            None => Ok(None),
        }
    }

    async fn find_by_name(&self, name: &str) -> AppResult<Option<Collection>> {
        use schema::collections;

        let mut conn = self.db.get_connection()?;

        let model = collections::table
            .filter(collections::name.eq(name))
            .first::<CollectionModel>(&mut conn)
            .optional()?;

        match model {
            Some(m) => {
                let anime_ids = self.get_collection_anime_ids(&m.id).await?;
                Ok(Some(self.model_to_entity(m, anime_ids)))
            }
            None => Ok(None),
        }
    }

    async fn get_all(&self) -> AppResult<Vec<Collection>> {
        use schema::collections;

        let mut conn = self.db.get_connection()?;

        let models = collections::table
            .order(collections::created_at.desc())
            .load::<CollectionModel>(&mut conn)?;

        let mut collections = Vec::new();
        for model in models {
            let anime_ids = self.get_collection_anime_ids(&model.id).await?;
            collections.push(self.model_to_entity(model, anime_ids));
        }

        Ok(collections)
    }

    async fn save(&self, collection: &Collection) -> AppResult<Collection> {
        use schema::collections;

        let mut conn = self.db.get_connection()?;
        let model = self.entity_to_model(collection);

        diesel::insert_into(collections::table)
            .values(&model)
            .execute(&mut conn)?;

        Ok(collection.clone())
    }

    async fn update(&self, collection: &Collection) -> AppResult<Collection> {
        use schema::collections;

        let mut conn = self.db.get_connection()?;
        let model = self.entity_to_model(collection);

        diesel::update(collections::table)
            .filter(collections::id.eq(collection.id))
            .set(&model)
            .execute(&mut conn)?;

        Ok(collection.clone())
    }

    async fn delete(&self, id: &Uuid) -> AppResult<()> {
        use schema::collections;

        let mut conn = self.db.get_connection()?;

        let deleted_count =
            diesel::delete(collections::table.filter(collections::id.eq(id))).execute(&mut conn)?;

        if deleted_count == 0 {
            return Err(AppError::NotFound(format!(
                "Collection with ID {} not found",
                id
            )));
        }

        Ok(())
    }

    async fn add_anime_to_collection(
        &self,
        collection_id: &Uuid,
        anime_id: &Uuid,
        user_score: Option<f32>,
        notes: Option<String>,
    ) -> AppResult<()> {
        use schema::collection_anime;

        let mut conn = self.db.get_connection()?;

        let model = CollectionAnimeModel {
            collection_id: *collection_id,
            anime_id: *anime_id,
            added_at: chrono::Utc::now(),
            user_score,
            notes,
        };

        diesel::insert_into(collection_anime::table)
            .values(&model)
            .execute(&mut conn)?;

        Ok(())
    }

    async fn remove_anime_from_collection(
        &self,
        collection_id: &Uuid,
        anime_id: &Uuid,
    ) -> AppResult<()> {
        use schema::collection_anime;

        let mut conn = self.db.get_connection()?;

        let deleted_count = diesel::delete(
            collection_anime::table
                .filter(collection_anime::collection_id.eq(collection_id))
                .filter(collection_anime::anime_id.eq(anime_id)),
        )
        .execute(&mut conn)?;

        if deleted_count == 0 {
            return Err(AppError::NotFound(
                "Anime not found in collection".to_string(),
            ));
        }

        Ok(())
    }

    async fn get_collection_anime(&self, collection_id: &Uuid) -> AppResult<Vec<Anime>> {
        use schema::{anime, anime_genres, collection_anime, genres, quality_metrics};

        let mut conn = self.db.get_connection()?;

        // Get anime models that belong to the collection
        let anime_models = collection_anime::table
            .inner_join(anime::table.on(anime::id.eq(collection_anime::anime_id)))
            .filter(collection_anime::collection_id.eq(collection_id))
            .select(anime::all_columns)
            .order(collection_anime::added_at.desc())
            .load::<AnimeModel>(&mut conn)?;

        let mut anime_list = Vec::new();

        for anime_model in anime_models {
            // Get genres for each anime
            let genre_models = anime_genres::table
                .inner_join(genres::table.on(genres::id.eq(anime_genres::genre_id)))
                .filter(anime_genres::anime_id.eq(&anime_model.id))
                .select(genres::all_columns)
                .load::<GenreModel>(&mut conn)?;

            let genres: Vec<Genre> = genre_models
                .into_iter()
                .map(|g| Genre {
                    id: g.id,
                    mal_id: g.mal_id,
                    name: g.name,
                })
                .collect();

            // Get studios (simplified - just returning empty for now)
            let studios = Vec::new();

            // Get quality metrics
            let metrics = quality_metrics::table
                .filter(quality_metrics::anime_id.eq(&anime_model.id))
                .first::<crate::infrastructure::database::models::QualityMetricsModel>(&mut conn)
                .optional()?;

            let quality_metrics = if let Some(m) = metrics {
                crate::domain::value_objects::QualityMetrics {
                    popularity_score: m.popularity_score,
                    engagement_score: m.engagement_score,
                    consistency_score: m.consistency_score,
                    audience_reach_score: m.audience_reach_score,
                }
            } else {
                crate::domain::value_objects::QualityMetrics::default()
            };

            anime_list.push(Anime {
                id: anime_model.id,
                mal_id: anime_model.mal_id,
                title: anime_model.title,
                title_english: anime_model.title_english,
                title_japanese: anime_model.title_japanese,
                score: anime_model.score,
                scored_by: anime_model.scored_by,
                rank: anime_model.rank,
                popularity: anime_model.popularity,
                members: anime_model.members,
                favorites: anime_model.favorites,
                synopsis: anime_model.synopsis,
                episodes: anime_model.episodes,
                status: anime_model.status.as_str().into(),
                aired: crate::domain::entities::AiredDates {
                    from: anime_model.aired_from,
                    to: anime_model.aired_to,
                },
                anime_type: anime_model.anime_type.as_str().into(),
                rating: anime_model.rating,
                genres,
                studios,
                source: anime_model.source,
                duration: anime_model.duration,
                image_url: anime_model.image_url,
                mal_url: anime_model.mal_url,
                composite_score: anime_model.composite_score,
                tier: AnimeTier::new(anime_model.composite_score),
                quality_metrics,
            });
        }

        Ok(anime_list)
    }

    async fn get_collection_entry(
        &self,
        collection_id: &Uuid,
        anime_id: &Uuid,
    ) -> AppResult<Option<CollectionAnime>> {
        use schema::collection_anime;

        let mut conn = self.db.get_connection()?;

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
    }

    async fn update_collection_entry(&self, entry: &CollectionAnime) -> AppResult<()> {
        use schema::collection_anime;

        let mut conn = self.db.get_connection()?;

        let model = CollectionAnimeModel {
            collection_id: entry.collection_id,
            anime_id: entry.anime_id,
            added_at: entry.added_at,
            user_score: entry.user_score,
            notes: entry.notes.clone(),
        };

        diesel::update(
            collection_anime::table
                .filter(collection_anime::collection_id.eq(entry.collection_id))
                .filter(collection_anime::anime_id.eq(entry.anime_id)),
        )
        .set(&model)
        .execute(&mut conn)?;

        Ok(())
    }
}
