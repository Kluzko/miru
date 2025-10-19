use diesel::prelude::*;
use std::sync::Arc;
use tokio::task;
use uuid::Uuid;

use crate::modules::media::domain::entities::{AnimeImage, NewAnimeImage};
use crate::modules::media::domain::repositories::AnimeImageRepository;
use crate::modules::media::domain::value_objects::{AnimeProvider, ImageType};
use crate::schema::anime_images;
use crate::shared::Database;

pub struct AnimeImageRepositoryImpl {
    db: Arc<Database>,
}

impl AnimeImageRepositoryImpl {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}

impl AnimeImageRepository for AnimeImageRepositoryImpl {
    fn find_by_id(&self, id: Uuid) -> Result<Option<AnimeImage>, String> {
        let db = Arc::clone(&self.db);
        task::block_in_place(move || {
            let mut conn = db
                .get_connection()
                .map_err(|e| format!("Database connection error: {}", e))?;

            anime_images::table
                .find(id)
                .first::<AnimeImage>(&mut conn)
                .optional()
                .map_err(|e| format!("Failed to find image: {}", e))
        })
    }

    fn find_by_anime_id(&self, anime_id: Uuid) -> Result<Vec<AnimeImage>, String> {
        let db = Arc::clone(&self.db);
        task::block_in_place(move || {
            let mut conn = db
                .get_connection()
                .map_err(|e| format!("Database connection error: {}", e))?;

            anime_images::table
                .filter(anime_images::anime_id.eq(anime_id))
                .order(anime_images::is_primary.desc())
                .then_order_by(anime_images::vote_average.desc())
                .load::<AnimeImage>(&mut conn)
                .map_err(|e| format!("Failed to load images: {}", e))
        })
    }

    fn find_by_anime_and_type(
        &self,
        anime_id: Uuid,
        image_type: ImageType,
    ) -> Result<Vec<AnimeImage>, String> {
        let db = Arc::clone(&self.db);
        task::block_in_place(move || {
            let mut conn = db
                .get_connection()
                .map_err(|e| format!("Database connection error: {}", e))?;

            anime_images::table
                .filter(anime_images::anime_id.eq(anime_id))
                .filter(anime_images::image_type.eq(image_type))
                .order(anime_images::is_primary.desc())
                .then_order_by(anime_images::vote_average.desc())
                .load::<AnimeImage>(&mut conn)
                .map_err(|e| format!("Failed to load images: {}", e))
        })
    }

    fn find_primary_by_type(
        &self,
        anime_id: Uuid,
        image_type: ImageType,
    ) -> Result<Option<AnimeImage>, String> {
        let db = Arc::clone(&self.db);
        task::block_in_place(move || {
            let mut conn = db
                .get_connection()
                .map_err(|e| format!("Database connection error: {}", e))?;

            anime_images::table
                .filter(anime_images::anime_id.eq(anime_id))
                .filter(anime_images::image_type.eq(image_type))
                .filter(anime_images::is_primary.eq(true))
                .first::<AnimeImage>(&mut conn)
                .optional()
                .map_err(|e| format!("Failed to find primary image: {}", e))
        })
    }

    fn find_all_primary(&self, anime_id: Uuid) -> Result<Vec<AnimeImage>, String> {
        let db = Arc::clone(&self.db);
        task::block_in_place(move || {
            let mut conn = db
                .get_connection()
                .map_err(|e| format!("Database connection error: {}", e))?;

            anime_images::table
                .filter(anime_images::anime_id.eq(anime_id))
                .filter(anime_images::is_primary.eq(true))
                .load::<AnimeImage>(&mut conn)
                .map_err(|e| format!("Failed to load primary images: {}", e))
        })
    }

    fn find_by_provider(
        &self,
        anime_id: Uuid,
        provider: AnimeProvider,
    ) -> Result<Vec<AnimeImage>, String> {
        let db = Arc::clone(&self.db);
        task::block_in_place(move || {
            let mut conn = db
                .get_connection()
                .map_err(|e| format!("Database connection error: {}", e))?;

            anime_images::table
                .filter(anime_images::anime_id.eq(anime_id))
                .filter(anime_images::provider.eq(provider))
                .order(anime_images::vote_average.desc())
                .load::<AnimeImage>(&mut conn)
                .map_err(|e| format!("Failed to load images by provider: {}", e))
        })
    }

    fn find_best_quality(
        &self,
        anime_id: Uuid,
        image_type: ImageType,
        limit: i64,
    ) -> Result<Vec<AnimeImage>, String> {
        let db = Arc::clone(&self.db);
        task::block_in_place(move || {
            let mut conn = db
                .get_connection()
                .map_err(|e| format!("Database connection error: {}", e))?;

            anime_images::table
                .filter(anime_images::anime_id.eq(anime_id))
                .filter(anime_images::image_type.eq(image_type))
                .filter(anime_images::vote_average.is_not_null())
                .order(anime_images::vote_average.desc())
                .limit(limit)
                .load::<AnimeImage>(&mut conn)
                .map_err(|e| format!("Failed to load best quality images: {}", e))
        })
    }

    fn create(&self, image: NewAnimeImage) -> Result<AnimeImage, String> {
        let db = Arc::clone(&self.db);
        task::block_in_place(move || {
            let mut conn = db
                .get_connection()
                .map_err(|e| format!("Database connection error: {}", e))?;

            diesel::insert_into(anime_images::table)
                .values(&image)
                .get_result::<AnimeImage>(&mut conn)
                .map_err(|e| format!("Failed to create image: {}", e))
        })
    }

    fn create_many(&self, images: Vec<NewAnimeImage>) -> Result<Vec<AnimeImage>, String> {
        if images.is_empty() {
            return Ok(Vec::new());
        }

        let db = Arc::clone(&self.db);
        task::block_in_place(move || {
            let mut conn = db
                .get_connection()
                .map_err(|e| format!("Database connection error: {}", e))?;

            diesel::insert_into(anime_images::table)
                .values(&images)
                .get_results::<AnimeImage>(&mut conn)
                .map_err(|e| format!("Failed to create images: {}", e))
        })
    }

    fn update(&self, id: Uuid, image: NewAnimeImage) -> Result<AnimeImage, String> {
        let db = Arc::clone(&self.db);
        task::block_in_place(move || {
            let mut conn = db
                .get_connection()
                .map_err(|e| format!("Database connection error: {}", e))?;

            diesel::update(anime_images::table.find(id))
                .set(&image)
                .get_result::<AnimeImage>(&mut conn)
                .map_err(|e| format!("Failed to update image: {}", e))
        })
    }

    fn delete(&self, id: Uuid) -> Result<bool, String> {
        let db = Arc::clone(&self.db);
        task::block_in_place(move || {
            let mut conn = db
                .get_connection()
                .map_err(|e| format!("Database connection error: {}", e))?;

            let deleted = diesel::delete(anime_images::table.find(id))
                .execute(&mut conn)
                .map_err(|e| format!("Failed to delete image: {}", e))?;

            Ok(deleted > 0)
        })
    }

    fn delete_by_anime_id(&self, anime_id: Uuid) -> Result<usize, String> {
        let db = Arc::clone(&self.db);
        task::block_in_place(move || {
            let mut conn = db
                .get_connection()
                .map_err(|e| format!("Database connection error: {}", e))?;

            diesel::delete(anime_images::table.filter(anime_images::anime_id.eq(anime_id)))
                .execute(&mut conn)
                .map_err(|e| format!("Failed to delete images: {}", e))
        })
    }

    fn delete_by_provider(&self, anime_id: Uuid, provider: AnimeProvider) -> Result<usize, String> {
        let db = Arc::clone(&self.db);
        task::block_in_place(move || {
            let mut conn = db
                .get_connection()
                .map_err(|e| format!("Database connection error: {}", e))?;

            diesel::delete(
                anime_images::table
                    .filter(anime_images::anime_id.eq(anime_id))
                    .filter(anime_images::provider.eq(provider)),
            )
            .execute(&mut conn)
            .map_err(|e| format!("Failed to delete images by provider: {}", e))
        })
    }

    fn set_primary(&self, id: Uuid) -> Result<AnimeImage, String> {
        let db = Arc::clone(&self.db);
        task::block_in_place(move || {
            let mut conn = db
                .get_connection()
                .map_err(|e| format!("Database connection error: {}", e))?;

            // First, get the image to know its anime_id and type
            let image = anime_images::table
                .find(id)
                .first::<AnimeImage>(&mut conn)
                .map_err(|e| format!("Failed to find image: {}", e))?;

            // Unset all other primary images of the same type for this anime
            diesel::update(
                anime_images::table
                    .filter(anime_images::anime_id.eq(image.anime_id))
                    .filter(anime_images::image_type.eq(image.image_type))
                    .filter(anime_images::is_primary.eq(true)),
            )
            .set(anime_images::is_primary.eq(false))
            .execute(&mut conn)
            .map_err(|e| format!("Failed to unset primary images: {}", e))?;

            // Set this image as primary
            diesel::update(anime_images::table.find(id))
                .set(anime_images::is_primary.eq(true))
                .get_result::<AnimeImage>(&mut conn)
                .map_err(|e| format!("Failed to set primary image: {}", e))
        })
    }

    fn exists_by_url(&self, anime_id: Uuid, url: &str) -> Result<bool, String> {
        let db = Arc::clone(&self.db);
        let url = url.to_string();
        task::block_in_place(move || {
            let mut conn = db
                .get_connection()
                .map_err(|e| format!("Database connection error: {}", e))?;

            use diesel::dsl::exists;
            use diesel::select;

            select(exists(
                anime_images::table
                    .filter(anime_images::anime_id.eq(anime_id))
                    .filter(anime_images::url.eq(url)),
            ))
            .get_result::<bool>(&mut conn)
            .map_err(|e| format!("Failed to check if image exists: {}", e))
        })
    }

    fn count_by_anime_id(&self, anime_id: Uuid) -> Result<i64, String> {
        let db = Arc::clone(&self.db);
        task::block_in_place(move || {
            let mut conn = db
                .get_connection()
                .map_err(|e| format!("Database connection error: {}", e))?;

            anime_images::table
                .filter(anime_images::anime_id.eq(anime_id))
                .count()
                .get_result::<i64>(&mut conn)
                .map_err(|e| format!("Failed to count images: {}", e))
        })
    }
}
