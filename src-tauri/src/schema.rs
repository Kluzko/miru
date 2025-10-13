// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "anime_relation_type"))]
    pub struct AnimeRelationType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "anime_status"))]
    pub struct AnimeStatus;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "anime_tier"))]
    pub struct AnimeTier;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "anime_type"))]
    pub struct AnimeType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "job_status"))]
    pub struct JobStatus;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "unified_age_restriction"))]
    pub struct UnifiedAgeRestriction;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "watching_status"))]
    pub struct WatchingStatus;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::AnimeTier;
    use super::sql_types::AnimeStatus;
    use super::sql_types::AnimeType;
    use super::sql_types::UnifiedAgeRestriction;

    anime (id) {
        id -> Uuid,
        #[max_length = 255]
        title_english -> Nullable<Varchar>,
        #[max_length = 255]
        title_japanese -> Nullable<Varchar>,
        score -> Nullable<Float4>,
        favorites -> Nullable<Int4>,
        synopsis -> Nullable<Text>,
        episodes -> Nullable<Int4>,
        aired_from -> Nullable<Timestamptz>,
        aired_to -> Nullable<Timestamptz>,
        #[max_length = 100]
        source -> Nullable<Varchar>,
        #[max_length = 50]
        duration -> Nullable<Varchar>,
        image_url -> Nullable<Text>,
        composite_score -> Float4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        #[max_length = 255]
        title_main -> Varchar,
        #[max_length = 255]
        title_romaji -> Nullable<Varchar>,
        #[max_length = 255]
        title_native -> Nullable<Varchar>,
        title_synonyms -> Nullable<Jsonb>,
        banner_image -> Nullable<Text>,
        trailer_url -> Nullable<Text>,
        tier -> AnimeTier,
        quality_metrics -> Nullable<Jsonb>,
        status -> AnimeStatus,
        anime_type -> AnimeType,
        age_restriction -> Nullable<UnifiedAgeRestriction>,
        last_synced_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    anime_external_ids (anime_id, provider_code) {
        anime_id -> Uuid,
        #[max_length = 20]
        provider_code -> Varchar,
        #[max_length = 255]
        external_id -> Varchar,
        provider_url -> Nullable<Text>,
        is_primary -> Nullable<Bool>,
        last_synced -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    anime_genres (anime_id, genre_id) {
        anime_id -> Uuid,
        genre_id -> Uuid,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::AnimeRelationType;

    anime_relations (id) {
        id -> Uuid,
        anime_id -> Uuid,
        related_anime_id -> Uuid,
        relation_type -> AnimeRelationType,
        created_at -> Nullable<Timestamptz>,
        synced_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    anime_studios (anime_id, studio_id) {
        anime_id -> Uuid,
        studio_id -> Uuid,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::JobStatus;

    background_jobs (id) {
        id -> Uuid,
        #[max_length = 50]
        job_type -> Varchar,
        payload -> Jsonb,
        priority -> Int4,
        status -> JobStatus,
        attempts -> Int4,
        max_attempts -> Int4,
        created_at -> Timestamptz,
        started_at -> Nullable<Timestamptz>,
        completed_at -> Nullable<Timestamptz>,
        error -> Nullable<Text>,
    }
}

diesel::table! {
    collection_anime (collection_id, anime_id) {
        collection_id -> Uuid,
        anime_id -> Uuid,
        added_at -> Timestamptz,
        user_score -> Nullable<Float4>,
        notes -> Nullable<Text>,
    }
}

diesel::table! {
    collections (id) {
        id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
        description -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        #[max_length = 255]
        user_id -> Nullable<Varchar>,
        is_public -> Nullable<Bool>,
    }
}

diesel::table! {
    genres (id) {
        id -> Uuid,
        #[max_length = 100]
        name -> Varchar,
    }
}

diesel::table! {
    providers (code) {
        #[max_length = 20]
        code -> Varchar,
        #[max_length = 100]
        display_name -> Varchar,
        #[max_length = 255]
        api_base_url -> Nullable<Varchar>,
        is_active -> Nullable<Bool>,
    }
}

diesel::table! {
    quality_metrics (id) {
        id -> Uuid,
        anime_id -> Uuid,
        popularity_score -> Float4,
        engagement_score -> Float4,
        consistency_score -> Float4,
        audience_reach_score -> Float4,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    studios (id) {
        id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::WatchingStatus;

    user_anime_data (anime_id, user_id) {
        anime_id -> Uuid,
        #[max_length = 255]
        user_id -> Varchar,
        status -> Nullable<WatchingStatus>,
        personal_rating -> Nullable<Float4>,
        episodes_watched -> Nullable<Int4>,
        rewatched_count -> Nullable<Int4>,
        is_favorite -> Nullable<Bool>,
        notes -> Nullable<Text>,
        tags -> Nullable<Jsonb>,
        start_date -> Nullable<Timestamptz>,
        finish_date -> Nullable<Timestamptz>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::joinable!(anime_external_ids -> anime (anime_id));
diesel::joinable!(anime_external_ids -> providers (provider_code));
diesel::joinable!(anime_genres -> anime (anime_id));
diesel::joinable!(anime_genres -> genres (genre_id));
diesel::joinable!(anime_studios -> anime (anime_id));
diesel::joinable!(anime_studios -> studios (studio_id));
diesel::joinable!(collection_anime -> anime (anime_id));
diesel::joinable!(collection_anime -> collections (collection_id));
diesel::joinable!(quality_metrics -> anime (anime_id));
diesel::joinable!(user_anime_data -> anime (anime_id));

diesel::allow_tables_to_appear_in_same_query!(
    anime,
    anime_external_ids,
    anime_genres,
    anime_relations,
    anime_studios,
    background_jobs,
    collection_anime,
    collections,
    genres,
    providers,
    quality_metrics,
    studios,
    user_anime_data,
);
