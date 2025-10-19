use diesel::prelude::*;
use std::sync::Arc;
use tokio::task;
use uuid::Uuid;

use crate::modules::media::domain::entities::{AnimeVideo, NewAnimeVideo};
use crate::modules::media::domain::repositories::AnimeVideoRepository;
use crate::modules::media::domain::value_objects::{AnimeProvider, VideoType};
use crate::schema::anime_videos;
use crate::shared::Database;

pub struct AnimeVideoRepositoryImpl {
    db: Arc<Database>,
}

impl AnimeVideoRepositoryImpl {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}

impl AnimeVideoRepository for AnimeVideoRepositoryImpl {
    fn find_by_id(&self, id: Uuid) -> Result<Option<AnimeVideo>, String> {
        let db = Arc::clone(&self.db);
        task::block_in_place(move || {
            let mut conn = db
                .get_connection()
                .map_err(|e| format!("Database connection error: {}", e))?;

            anime_videos::table
                .find(id)
                .first::<AnimeVideo>(&mut conn)
                .optional()
                .map_err(|e| format!("Failed to find video: {}", e))
        })
    }

    fn find_by_anime_id(&self, anime_id: Uuid) -> Result<Vec<AnimeVideo>, String> {
        let db = Arc::clone(&self.db);
        task::block_in_place(move || {
            let mut conn = db
                .get_connection()
                .map_err(|e| format!("Database connection error: {}", e))?;

            anime_videos::table
                .filter(anime_videos::anime_id.eq(anime_id))
                .order(anime_videos::is_official.desc())
                .then_order_by(anime_videos::published_at.desc())
                .load::<AnimeVideo>(&mut conn)
                .map_err(|e| format!("Failed to load videos: {}", e))
        })
    }

    fn find_by_anime_and_type(
        &self,
        anime_id: Uuid,
        video_type: VideoType,
    ) -> Result<Vec<AnimeVideo>, String> {
        let db = Arc::clone(&self.db);
        task::block_in_place(move || {
            let mut conn = db
                .get_connection()
                .map_err(|e| format!("Database connection error: {}", e))?;

            anime_videos::table
                .filter(anime_videos::anime_id.eq(anime_id))
                .filter(anime_videos::video_type.eq(video_type))
                .order(anime_videos::is_official.desc())
                .then_order_by(anime_videos::published_at.desc())
                .load::<AnimeVideo>(&mut conn)
                .map_err(|e| format!("Failed to load videos by type: {}", e))
        })
    }

    fn find_official(&self, anime_id: Uuid) -> Result<Vec<AnimeVideo>, String> {
        let db = Arc::clone(&self.db);
        task::block_in_place(move || {
            let mut conn = db
                .get_connection()
                .map_err(|e| format!("Database connection error: {}", e))?;

            anime_videos::table
                .filter(anime_videos::anime_id.eq(anime_id))
                .filter(anime_videos::is_official.eq(true))
                .order(anime_videos::published_at.desc())
                .load::<AnimeVideo>(&mut conn)
                .map_err(|e| format!("Failed to load official videos: {}", e))
        })
    }

    fn find_by_provider(
        &self,
        anime_id: Uuid,
        provider: AnimeProvider,
    ) -> Result<Vec<AnimeVideo>, String> {
        let db = Arc::clone(&self.db);
        task::block_in_place(move || {
            let mut conn = db
                .get_connection()
                .map_err(|e| format!("Database connection error: {}", e))?;

            anime_videos::table
                .filter(anime_videos::anime_id.eq(anime_id))
                .filter(anime_videos::provider.eq(provider))
                .order(anime_videos::published_at.desc())
                .load::<AnimeVideo>(&mut conn)
                .map_err(|e| format!("Failed to load videos by provider: {}", e))
        })
    }

    fn find_promotional(&self, anime_id: Uuid) -> Result<Vec<AnimeVideo>, String> {
        let db = Arc::clone(&self.db);
        task::block_in_place(move || {
            let mut conn = db
                .get_connection()
                .map_err(|e| format!("Database connection error: {}", e))?;

            anime_videos::table
                .filter(anime_videos::anime_id.eq(anime_id))
                .filter(
                    anime_videos::video_type
                        .eq(VideoType::Trailer)
                        .or(anime_videos::video_type.eq(VideoType::Teaser))
                        .or(anime_videos::video_type.eq(VideoType::PV))
                        .or(anime_videos::video_type.eq(VideoType::CM))
                        .or(anime_videos::video_type.eq(VideoType::Featurette)),
                )
                .order(anime_videos::published_at.desc())
                .load::<AnimeVideo>(&mut conn)
                .map_err(|e| format!("Failed to load promotional videos: {}", e))
        })
    }

    fn find_content(&self, anime_id: Uuid) -> Result<Vec<AnimeVideo>, String> {
        let db = Arc::clone(&self.db);
        task::block_in_place(move || {
            let mut conn = db
                .get_connection()
                .map_err(|e| format!("Database connection error: {}", e))?;

            anime_videos::table
                .filter(anime_videos::anime_id.eq(anime_id))
                .filter(
                    anime_videos::video_type
                        .eq(VideoType::Opening)
                        .or(anime_videos::video_type.eq(VideoType::Ending))
                        .or(anime_videos::video_type.eq(VideoType::Clip)),
                )
                .order(anime_videos::published_at.desc())
                .load::<AnimeVideo>(&mut conn)
                .map_err(|e| format!("Failed to load content videos: {}", e))
        })
    }

    fn find_by_site(&self, anime_id: Uuid, site: &str) -> Result<Vec<AnimeVideo>, String> {
        let db = Arc::clone(&self.db);
        let site = site.to_string();
        task::block_in_place(move || {
            let mut conn = db
                .get_connection()
                .map_err(|e| format!("Database connection error: {}", e))?;

            anime_videos::table
                .filter(anime_videos::anime_id.eq(anime_id))
                .filter(anime_videos::site.eq(site))
                .order(anime_videos::published_at.desc())
                .load::<AnimeVideo>(&mut conn)
                .map_err(|e| format!("Failed to load videos by site: {}", e))
        })
    }

    fn create(&self, video: NewAnimeVideo) -> Result<AnimeVideo, String> {
        let db = Arc::clone(&self.db);
        task::block_in_place(move || {
            let mut conn = db
                .get_connection()
                .map_err(|e| format!("Database connection error: {}", e))?;

            diesel::insert_into(anime_videos::table)
                .values(&video)
                .get_result::<AnimeVideo>(&mut conn)
                .map_err(|e| format!("Failed to create video: {}", e))
        })
    }

    fn create_many(&self, videos: Vec<NewAnimeVideo>) -> Result<Vec<AnimeVideo>, String> {
        if videos.is_empty() {
            return Ok(Vec::new());
        }

        let db = Arc::clone(&self.db);
        task::block_in_place(move || {
            let mut conn = db
                .get_connection()
                .map_err(|e| format!("Database connection error: {}", e))?;

            diesel::insert_into(anime_videos::table)
                .values(&videos)
                .get_results::<AnimeVideo>(&mut conn)
                .map_err(|e| format!("Failed to create videos: {}", e))
        })
    }

    fn update(&self, id: Uuid, video: NewAnimeVideo) -> Result<AnimeVideo, String> {
        let db = Arc::clone(&self.db);
        task::block_in_place(move || {
            let mut conn = db
                .get_connection()
                .map_err(|e| format!("Database connection error: {}", e))?;

            diesel::update(anime_videos::table.find(id))
                .set(&video)
                .get_result::<AnimeVideo>(&mut conn)
                .map_err(|e| format!("Failed to update video: {}", e))
        })
    }

    fn delete(&self, id: Uuid) -> Result<bool, String> {
        let db = Arc::clone(&self.db);
        task::block_in_place(move || {
            let mut conn = db
                .get_connection()
                .map_err(|e| format!("Database connection error: {}", e))?;

            let deleted = diesel::delete(anime_videos::table.find(id))
                .execute(&mut conn)
                .map_err(|e| format!("Failed to delete video: {}", e))?;

            Ok(deleted > 0)
        })
    }

    fn delete_by_anime_id(&self, anime_id: Uuid) -> Result<usize, String> {
        let db = Arc::clone(&self.db);
        task::block_in_place(move || {
            let mut conn = db
                .get_connection()
                .map_err(|e| format!("Database connection error: {}", e))?;

            diesel::delete(anime_videos::table.filter(anime_videos::anime_id.eq(anime_id)))
                .execute(&mut conn)
                .map_err(|e| format!("Failed to delete videos: {}", e))
        })
    }

    fn delete_by_provider(&self, anime_id: Uuid, provider: AnimeProvider) -> Result<usize, String> {
        let db = Arc::clone(&self.db);
        task::block_in_place(move || {
            let mut conn = db
                .get_connection()
                .map_err(|e| format!("Database connection error: {}", e))?;

            diesel::delete(
                anime_videos::table
                    .filter(anime_videos::anime_id.eq(anime_id))
                    .filter(anime_videos::provider.eq(provider)),
            )
            .execute(&mut conn)
            .map_err(|e| format!("Failed to delete videos by provider: {}", e))
        })
    }

    fn exists_by_key(&self, anime_id: Uuid, site: &str, key: &str) -> Result<bool, String> {
        let db = Arc::clone(&self.db);
        let site = site.to_string();
        let key = key.to_string();
        task::block_in_place(move || {
            let mut conn = db
                .get_connection()
                .map_err(|e| format!("Database connection error: {}", e))?;

            use diesel::dsl::exists;
            use diesel::select;

            select(exists(
                anime_videos::table
                    .filter(anime_videos::anime_id.eq(anime_id))
                    .filter(anime_videos::site.eq(site))
                    .filter(anime_videos::key.eq(key)),
            ))
            .get_result::<bool>(&mut conn)
            .map_err(|e| format!("Failed to check if video exists: {}", e))
        })
    }

    fn count_by_anime_id(&self, anime_id: Uuid) -> Result<i64, String> {
        let db = Arc::clone(&self.db);
        task::block_in_place(move || {
            let mut conn = db
                .get_connection()
                .map_err(|e| format!("Database connection error: {}", e))?;

            anime_videos::table
                .filter(anime_videos::anime_id.eq(anime_id))
                .count()
                .get_result::<i64>(&mut conn)
                .map_err(|e| format!("Failed to count videos: {}", e))
        })
    }
}
