// @generated automatically by Diesel CLI.

diesel::table! {
    anime (id) {
        id -> Uuid,
        mal_id -> Nullable<Int4>,
        #[max_length = 255]
        title -> Varchar,
        #[max_length = 255]
        title_english -> Nullable<Varchar>,
        #[max_length = 255]
        title_japanese -> Nullable<Varchar>,
        score -> Nullable<Float4>,
        scored_by -> Nullable<Int4>,
        rank -> Nullable<Int4>,
        popularity -> Nullable<Int4>,
        members -> Nullable<Int4>,
        favorites -> Nullable<Int4>,
        synopsis -> Nullable<Text>,
        episodes -> Nullable<Int4>,
        #[max_length = 50]
        status -> Varchar,
        aired_from -> Nullable<Timestamptz>,
        aired_to -> Nullable<Timestamptz>,
        #[max_length = 50]
        anime_type -> Varchar,
        #[max_length = 50]
        rating -> Nullable<Varchar>,
        #[max_length = 100]
        source -> Nullable<Varchar>,
        #[max_length = 50]
        duration -> Nullable<Varchar>,
        image_url -> Nullable<Text>,
        mal_url -> Nullable<Text>,
        composite_score -> Float4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    anime_genres (anime_id, genre_id) {
        anime_id -> Uuid,
        genre_id -> Uuid,
    }
}

diesel::table! {
    anime_studios (anime_id, studio_id) {
        anime_id -> Uuid,
        studio_id -> Uuid,
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
    }
}

diesel::table! {
    genres (id) {
        id -> Uuid,
        mal_id -> Nullable<Int4>,
        #[max_length = 100]
        name -> Varchar,
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

diesel::joinable!(anime_genres -> anime (anime_id));
diesel::joinable!(anime_genres -> genres (genre_id));
diesel::joinable!(anime_studios -> anime (anime_id));
diesel::joinable!(anime_studios -> studios (studio_id));
diesel::joinable!(collection_anime -> anime (anime_id));
diesel::joinable!(collection_anime -> collections (collection_id));
diesel::joinable!(quality_metrics -> anime (anime_id));

diesel::allow_tables_to_appear_in_same_query!(
    anime,
    anime_genres,
    anime_studios,
    collection_anime,
    collections,
    genres,
    quality_metrics,
    studios,
);
