use chrono::Utc;
use diesel::prelude::*;
use std::sync::Arc;
use tokio::task;
use uuid::Uuid;

use crate::modules::anime::domain::{
    entities::anime_detailed::AnimeDetailed,
    repositories::anime_repository::AnimeWithRelationMetadata, value_objects::AnimeRelationType,
};
use crate::schema::anime_relations;
use crate::shared::errors::{AppError, AppResult};
use crate::shared::Database;
use crate::{log_debug, log_error, log_warn};

use super::anime_repository_impl::AnimeRepositoryImpl;

/// Helper function to get the inverse relation type for bidirectional relations
pub fn inverse_relation_type(relation_type: &str) -> String {
    match relation_type.to_lowercase().as_str() {
        // Semantic categories (symmetric or have clear inverses)
        "mainstory" => "mainstory".to_string(),
        "sidestory" => "mainstory".to_string(),
        "movie" | "movies" => "mainstory".to_string(),
        "ova" | "ovaspecial" | "ova_special" => "mainstory".to_string(),

        // AniList relation types (have clear inverses)
        "sequel" => "prequel".to_string(),
        "prequel" => "sequel".to_string(),
        "side_story" => "parent_story".to_string(),
        "parent_story" => "side_story".to_string(),
        "spin_off" => "parent_story".to_string(),
        "alternative" => "alternative".to_string(),
        "summary" => "full_story".to_string(),
        "full_story" => "summary".to_string(),
        "special" => "parent_story".to_string(),
        "same_setting" => "same_setting".to_string(),
        "shared_character" => "shared_character".to_string(),

        // Default
        _ => "other".to_string(),
    }
}

pub struct AnimeRelationsRepositoryImpl {
    db: Arc<Database>,
    anime_repository: Arc<AnimeRepositoryImpl>,
}

impl AnimeRelationsRepositoryImpl {
    pub fn new(db: Arc<Database>, anime_repository: Arc<AnimeRepositoryImpl>) -> Self {
        Self {
            db,
            anime_repository,
        }
    }

    /// Get relations for an anime from database
    pub async fn find_relations(&self, anime_id: &Uuid) -> AppResult<Vec<(Uuid, String)>> {
        let db = Arc::clone(&self.db);
        let anime_id = *anime_id;

        task::spawn_blocking(move || -> AppResult<Vec<(Uuid, String)>> {
            let mut conn = db.get_connection()?;

            let relations: Vec<(Uuid, AnimeRelationType)> = anime_relations::table
                .filter(anime_relations::anime_id.eq(anime_id))
                .select((
                    anime_relations::related_anime_id,
                    anime_relations::relation_type,
                ))
                .load::<(Uuid, AnimeRelationType)>(&mut conn)?;

            // Convert AnimeRelationType to String
            let converted_relations: Vec<(Uuid, String)> = relations
                .into_iter()
                .map(|(uuid, rel_type)| (uuid, rel_type.to_string()))
                .collect();

            Ok(converted_relations)
        })
        .await?
    }

    /// Get bidirectional relations (both A->B and B->A)
    pub async fn find_bidirectional_relations(
        &self,
        anime_id: &Uuid,
    ) -> AppResult<Vec<(Uuid, String)>> {
        let db = Arc::clone(&self.db);
        let anime_id = *anime_id;

        task::spawn_blocking(move || -> AppResult<Vec<(Uuid, String)>> {
            let mut conn = db.get_connection()?;

            // Get both forward (A->B) and reverse (B->A) relations
            let forward_relations: Vec<(Uuid, AnimeRelationType)> = anime_relations::table
                .filter(anime_relations::anime_id.eq(anime_id))
                .select((
                    anime_relations::related_anime_id,
                    anime_relations::relation_type,
                ))
                .load::<(Uuid, AnimeRelationType)>(&mut conn)?;

            let reverse_relations: Vec<(Uuid, AnimeRelationType)> = anime_relations::table
                .filter(anime_relations::related_anime_id.eq(anime_id))
                .select((anime_relations::anime_id, anime_relations::relation_type))
                .load::<(Uuid, AnimeRelationType)>(&mut conn)?;

            // Combine both directions
            let all_relations: Vec<(Uuid, String)> = forward_relations
                .into_iter()
                .chain(reverse_relations.into_iter())
                .map(|(uuid, rel_type)| (uuid, rel_type.to_string()))
                .collect();

            Ok(all_relations)
        })
        .await?
    }

    /// Save relations for an anime to database (with bidirectional support)
    pub async fn save_relations(
        &self,
        anime_id: &Uuid,
        relations: &[(Uuid, String)],
    ) -> AppResult<()> {
        log_debug!(
            "Starting save_relations for anime {} with {} relations",
            anime_id,
            relations.len()
        );

        let db = Arc::clone(&self.db);
        let anime_id = *anime_id;
        let relations_data = relations.to_vec();

        task::spawn_blocking(move || -> AppResult<()> {
            use crate::schema::anime;

            let mut conn = db.get_connection()?;
            log_debug!("Database connection acquired for save_relations");

            conn.transaction::<_, AppError, _>(|conn| {
                log_debug!(
                    "Starting transaction for {} relations",
                    relations_data.len()
                );

                for (index, (related_id, relation_type)) in relations_data.iter().enumerate() {
                    log_debug!(
                        "Processing relation {}/{}: {} -> {} (type: {})",
                        index + 1,
                        relations_data.len(),
                        anime_id,
                        related_id,
                        relation_type
                    );

                    // Verify that the related anime exists
                    let exists = anime::table
                        .filter(anime::id.eq(*related_id))
                        .count()
                        .get_result::<i64>(conn)?;

                    if exists == 0 {
                        log_error!(
                            "Related anime {} does not exist in database for relation {}",
                            related_id,
                            relation_type
                        );
                        return Err(AppError::InvalidInput(format!(
                            "Related anime {} does not exist in database",
                            related_id
                        )));
                    }

                    log_debug!(
                        "Related anime {} exists, proceeding with relation save",
                        related_id
                    );

                    // Convert string to enum
                    let relation_type_enum = match relation_type.to_lowercase().as_str() {
                        // Semantic franchise categories (preferred - absolute categorization)
                        "mainstory" => AnimeRelationType::MainStory,
                        "sidestory" => AnimeRelationType::SideStory,
                        "movie" | "movies" => AnimeRelationType::Movies,
                        "ova" | "ovaspecial" => AnimeRelationType::OvaSpecial,

                        // AniList relation types (legacy - relative relations)
                        "sequel" => AnimeRelationType::Sequel,
                        "prequel" => AnimeRelationType::Prequel,
                        "side_story" => AnimeRelationType::SideStory,
                        "spin_off" => AnimeRelationType::SpinOff,
                        "alternative" => AnimeRelationType::Alternative,
                        "summary" => AnimeRelationType::Summary,
                        "special" => AnimeRelationType::Special,
                        "parent_story" => AnimeRelationType::ParentStory,
                        "full_story" => AnimeRelationType::FullStory,
                        "same_setting" => AnimeRelationType::SameSetting,
                        "shared_character" => AnimeRelationType::SharedCharacter,
                        _ => AnimeRelationType::Other,
                    };

                    log_debug!(
                        "Attempting to insert/update relation: {} -> {} ({})",
                        anime_id,
                        related_id,
                        relation_type_enum
                    );

                    let result = diesel::insert_into(anime_relations::table)
                        .values((
                            anime_relations::anime_id.eq(anime_id),
                            anime_relations::related_anime_id.eq(*related_id),
                            anime_relations::relation_type.eq(relation_type_enum),
                            anime_relations::synced_at.eq(Utc::now()),
                        ))
                        .on_conflict((
                            anime_relations::anime_id,
                            anime_relations::related_anime_id,
                            anime_relations::relation_type,
                        ))
                        .do_update()
                        .set((
                            anime_relations::relation_type.eq(relation_type_enum),
                            anime_relations::synced_at.eq(Utc::now()),
                        ))
                        .execute(conn);

                    match result {
                        Ok(rows_affected) => {
                            log_debug!(
                                "Successfully saved FORWARD relation {}/{}: {} rows affected",
                                index + 1,
                                relations_data.len(),
                                rows_affected
                            );
                        }
                        Err(e) => {
                            log_error!(
                                "Failed to save forward relation {}/{}: {}",
                                index + 1,
                                relations_data.len(),
                                e
                            );
                            return Err(AppError::DatabaseError(e.to_string()));
                        }
                    }

                    // Insert REVERSE relation (B → A) for bidirectional navigation
                    let inverse_type_str = inverse_relation_type(relation_type);
                    let inverse_type_enum = match inverse_type_str.to_lowercase().as_str() {
                        "mainstory" => AnimeRelationType::MainStory,
                        "sidestory" => AnimeRelationType::SideStory,
                        "movie" | "movies" => AnimeRelationType::Movies,
                        "ova" | "ovaspecial" | "ova_special" => AnimeRelationType::OvaSpecial,
                        "sequel" => AnimeRelationType::Sequel,
                        "prequel" => AnimeRelationType::Prequel,
                        "side_story" => AnimeRelationType::SideStory,
                        "spin_off" => AnimeRelationType::SpinOff,
                        "alternative" => AnimeRelationType::Alternative,
                        "summary" => AnimeRelationType::Summary,
                        "special" => AnimeRelationType::Special,
                        "parent_story" => AnimeRelationType::ParentStory,
                        "full_story" => AnimeRelationType::FullStory,
                        "same_setting" => AnimeRelationType::SameSetting,
                        "shared_character" => AnimeRelationType::SharedCharacter,
                        _ => AnimeRelationType::Other,
                    };

                    log_debug!(
                        "Attempting to insert/update REVERSE relation: {} -> {} ({})",
                        related_id,
                        anime_id,
                        inverse_type_enum
                    );

                    let reverse_result = diesel::insert_into(anime_relations::table)
                        .values((
                            anime_relations::anime_id.eq(*related_id),
                            anime_relations::related_anime_id.eq(anime_id),
                            anime_relations::relation_type.eq(inverse_type_enum),
                            anime_relations::synced_at.eq(Utc::now()),
                        ))
                        .on_conflict((
                            anime_relations::anime_id,
                            anime_relations::related_anime_id,
                            anime_relations::relation_type,
                        ))
                        .do_update()
                        .set((
                            anime_relations::relation_type.eq(inverse_type_enum),
                            anime_relations::synced_at.eq(Utc::now()),
                        ))
                        .execute(conn);

                    match reverse_result {
                        Ok(rows_affected) => {
                            log_debug!(
                                "Successfully saved REVERSE relation {}/{}: {} rows affected",
                                index + 1,
                                relations_data.len(),
                                rows_affected
                            );
                        }
                        Err(e) => {
                            log_error!(
                                "Failed to save reverse relation {}/{}: {}",
                                index + 1,
                                relations_data.len(),
                                e
                            );
                            return Err(AppError::DatabaseError(e.to_string()));
                        }
                    }
                }

                log_debug!(
                    "Transaction completed successfully for {} relations",
                    relations_data.len()
                );
                Ok(())
            })
        })
        .await?
    }

    /// Delete all relations for an anime
    pub async fn delete_relations(&self, anime_id: &Uuid) -> AppResult<()> {
        let db = Arc::clone(&self.db);
        let anime_id = *anime_id;

        task::spawn_blocking(move || -> AppResult<()> {
            let mut conn = db.get_connection()?;

            conn.transaction::<_, AppError, _>(|conn| {
                // Delete forward relations (A -> B)
                let forward_deleted = diesel::delete(
                    anime_relations::table.filter(anime_relations::anime_id.eq(anime_id)),
                )
                .execute(conn)?;

                // Delete reverse relations (B -> A)
                let reverse_deleted = diesel::delete(
                    anime_relations::table.filter(anime_relations::related_anime_id.eq(anime_id)),
                )
                .execute(conn)?;

                log_debug!(
                    "Deleted {} forward and {} reverse relations for anime {}",
                    forward_deleted,
                    reverse_deleted,
                    anime_id
                );

                Ok(())
            })
        })
        .await?
    }

    /// Get anime with their relation metadata - batch fetch with full anime details
    pub async fn get_anime_with_relations(
        &self,
        anime_id: &Uuid,
    ) -> AppResult<Vec<AnimeWithRelationMetadata>> {
        log_debug!("Getting anime with relations for anime_id: {}", anime_id);

        // First get the relation metadata (relation_type, synced_at)
        let db = Arc::clone(&self.db);
        let anime_id = *anime_id;

        let relation_data: Vec<(Uuid, AnimeRelationType, Option<chrono::DateTime<Utc>>)> =
            task::spawn_blocking(
                move || -> AppResult<Vec<(Uuid, AnimeRelationType, Option<chrono::DateTime<Utc>>)>> {
                    let mut conn = db.get_connection()?;

                    let results = anime_relations::table
                        .filter(anime_relations::anime_id.eq(anime_id))
                        .select((
                            anime_relations::related_anime_id,
                            anime_relations::relation_type,
                            anime_relations::synced_at,
                        ))
                        .load::<(Uuid, AnimeRelationType, Option<chrono::DateTime<Utc>>)>(&mut conn)?;

                    Ok(results)
                },
            )
            .await??;

        log_debug!("Found {} relations", relation_data.len());

        // Then fetch each anime individually (reusing existing find_by_id method)
        let mut anime_with_relations = Vec::new();

        for (related_anime_id, relation_type, synced_at) in relation_data {
            use crate::modules::anime::domain::repositories::anime_repository::AnimeRepository;
            if let Ok(Some(anime)) = self.anime_repository.find_by_id(&related_anime_id).await {
                anime_with_relations.push(AnimeWithRelationMetadata {
                    anime,
                    relation_type: relation_type.to_string(),
                    synced_at: synced_at.unwrap_or_else(|| Utc::now()),
                });
            } else {
                log_warn!("Related anime {} not found", related_anime_id);
            }
        }

        log_debug!(
            "Successfully loaded {} anime with relations",
            anime_with_relations.len()
        );

        Ok(anime_with_relations)
    }

    /// Load anime batch with their relations (used by services)
    pub async fn load_anime_batch_with_relations(
        &self,
        anime_ids: Vec<Uuid>,
    ) -> AppResult<Vec<AnimeDetailed>> {
        if anime_ids.is_empty() {
            return Ok(Vec::new());
        }

        log_debug!("Loading batch of {} anime with relations", anime_ids.len());

        // Fetch all anime using the anime repository
        let mut results = Vec::new();

        for anime_id in anime_ids {
            use crate::modules::anime::domain::repositories::anime_repository::AnimeRepository;
            if let Ok(Some(anime)) = self.anime_repository.find_by_id(&anime_id).await {
                results.push(anime);
            } else {
                log_warn!("Anime {} not found during batch load", anime_id);
            }
        }

        log_debug!("Successfully loaded {} anime in batch", results.len());

        Ok(results)
    }
}
