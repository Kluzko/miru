// Jikan v4 API models for Rust
// Clean implementation based on https://docs.api.jikan.moe/

#![allow(unused)]

use serde::{Deserialize, Serialize};

// Response envelopes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JikanItem<T> {
    pub data: T,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JikanList<T> {
    pub data: Vec<T>,
    #[serde(default)]
    pub pagination: Option<Pagination>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pagination {
    pub last_visible_page: u32,
    pub has_next_page: bool,
    #[serde(default)]
    pub items: Option<PaginationItems>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaginationItems {
    pub count: u32,
    pub total: u32,
    pub per_page: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JikanError {
    pub status: u16,
    pub r#type: String,
    pub message: String,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub report_url: Option<String>,
}

// Shared primitives
pub type MalId = u32;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MalEntity {
    pub mal_id: MalId,
    pub r#type: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MalEntityWithImages {
    pub mal_id: MalId,
    pub url: String,
    #[serde(default)]
    pub images: Option<Images>,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Images {
    #[serde(default)]
    pub jpg: Option<ImageUrls>,
    #[serde(default)]
    pub webp: Option<ImageUrls>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageUrls {
    #[serde(default)]
    pub image_url: Option<String>,
    #[serde(default)]
    pub small_image_url: Option<String>,
    #[serde(default)]
    pub large_image_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trailer {
    #[serde(default)]
    pub youtube_id: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub embed_url: Option<String>,
    #[serde(default)]
    pub images: Option<TrailerImages>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrailerImages {
    #[serde(default)]
    pub image_url: Option<String>,
    #[serde(default)]
    pub small_image_url: Option<String>,
    #[serde(default)]
    pub medium_image_url: Option<String>,
    #[serde(default)]
    pub large_image_url: Option<String>,
    #[serde(default)]
    pub maximum_image_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TitleEntry {
    pub r#type: String, // "Default", "English", "Japanese", "Synonym"
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Aired {
    #[serde(default)]
    pub from: Option<String>, // ISO8601 UTC
    #[serde(default)]
    pub to: Option<String>, // ISO8601 UTC
    #[serde(default)]
    pub prop: Option<AiredProp>,
    #[serde(default)]
    pub string: Option<String>, // Human-readable date range
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiredProp {
    pub from: PartialDate,
    pub to: PartialDate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PartialDate {
    #[serde(default)]
    pub day: Option<u8>,
    #[serde(default)]
    pub month: Option<u8>,
    #[serde(default)]
    pub year: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Broadcast {
    #[serde(default)]
    pub day: Option<String>, // "Mondays"
    #[serde(default)]
    pub time: Option<String>, // "17:00"
    #[serde(default)]
    pub timezone: Option<String>, // "Asia/Tokyo"
    #[serde(default)]
    pub string: Option<String>, // human-readable
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExternalLink {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamingLink {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeSongs {
    #[serde(default)]
    pub openings: Option<Vec<String>>, // OPs
    #[serde(default)]
    pub endings: Option<Vec<String>>, // EDs
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelationGroup {
    pub relation: String, // "Sequel", "Prequel"
    pub entry: Vec<MalEntity>,
}

// Anime
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Anime {
    pub mal_id: MalId,
    pub url: String,
    #[serde(default)]
    pub images: Option<Images>,
    #[serde(default)]
    pub trailer: Option<Trailer>,
    pub approved: bool,
    #[serde(default)]
    pub titles: Option<Vec<TitleEntry>>, // present in /anime/{id}/full
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub title_english: Option<String>,
    #[serde(default)]
    pub title_japanese: Option<String>,
    #[serde(default)]
    pub title_synonyms: Option<Vec<String>>,
    #[serde(default)]
    pub r#type: Option<String>, // TV, Movie, OVA, etc.
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub episodes: Option<i32>,
    #[serde(default)]
    pub status: Option<String>, // Finished Airing, Currently Airing
    pub airing: bool,
    #[serde(default)]
    pub aired: Option<Aired>,
    #[serde(default)]
    pub duration: Option<String>,
    #[serde(default)]
    pub rating: Option<String>, // G, PG-13, R, etc.
    #[serde(default)]
    pub score: Option<f32>,
    #[serde(default)]
    pub scored_by: Option<i32>,
    #[serde(default)]
    pub rank: Option<i32>,
    #[serde(default)]
    pub popularity: Option<i32>,
    #[serde(default)]
    pub members: Option<i32>,
    #[serde(default)]
    pub favorites: Option<i32>,
    #[serde(default)]
    pub synopsis: Option<String>,
    #[serde(default)]
    pub background: Option<String>,
    #[serde(default)]
    pub season: Option<String>, // winter, spring, summer, fall
    #[serde(default)]
    pub year: Option<i32>,
    #[serde(default)]
    pub broadcast: Option<Broadcast>,
    #[serde(default)]
    pub producers: Option<Vec<MalEntity>>,
    #[serde(default)]
    pub licensors: Option<Vec<MalEntity>>,
    #[serde(default)]
    pub studios: Option<Vec<MalEntity>>,
    #[serde(default)]
    pub genres: Option<Vec<MalEntity>>,
    #[serde(default)]
    pub explicit_genres: Option<Vec<MalEntity>>,
    #[serde(default)]
    pub themes: Option<Vec<MalEntity>>,
    #[serde(default)]
    pub demographics: Option<Vec<MalEntity>>,
    #[serde(default)]
    pub external: Option<Vec<ExternalLink>>, // Add external links
    #[serde(default)]
    pub streaming: Option<Vec<StreamingLink>>, // Add streaming links
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimeFull {
    #[serde(flatten)]
    pub core: Anime,
    #[serde(default)]
    pub explicit_genres: Option<Vec<MalEntity>>,
    #[serde(default)]
    pub themes: Option<Vec<MalEntity>>,
    #[serde(default)]
    pub demographics: Option<Vec<MalEntity>>,
    #[serde(default)]
    pub relations: Option<Vec<RelationGroup>>,
    #[serde(default)]
    pub theme: Option<ThemeSongs>, // opening & ending songs
    #[serde(default)]
    pub external: Option<Vec<ExternalLink>>,
    #[serde(default)]
    pub streaming: Option<Vec<StreamingLink>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimeCharacterEdge {
    pub character: MalEntityWithImages,
    pub role: String, // Main / Supporting
    #[serde(default)]
    pub voice_actors: Option<Vec<VoiceActorEdge>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VoiceActorEdge {
    pub person: MalEntityWithImages,
    pub language: String, // "Japanese"
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimeStaffEdge {
    pub person: MalEntityWithImages,
    pub positions: Vec<String>, // array of roles
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimeEpisode {
    pub mal_id: MalId,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub title_japanese: Option<String>,
    #[serde(default)]
    pub title_romanji: Option<String>,
    #[serde(default)]
    pub aired: Option<String>, // ISO8601
    #[serde(default)]
    pub score: Option<f32>,
    #[serde(default)]
    pub filler: Option<bool>,
    #[serde(default)]
    pub recap: Option<bool>,
    #[serde(default)]
    pub forum_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimeNewsItem {
    pub mal_id: MalId,
    pub url: String,
    pub title: String,
    #[serde(default)]
    pub date: Option<String>,
    #[serde(default)]
    pub author_username: Option<String>,
    #[serde(default)]
    pub author_url: Option<String>,
    #[serde(default)]
    pub forum_url: Option<String>,
    #[serde(default)]
    pub images: Option<Images>,
    #[serde(default)]
    pub comments: Option<u32>,
    #[serde(default)]
    pub excerpt: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimeVideos {
    #[serde(default)]
    pub promo: Option<Vec<PromoVideo>>,
    #[serde(default)]
    pub episodes: Option<Vec<EpisodeVideo>>,
    #[serde(default)]
    pub music_videos: Option<Vec<MusicVideo>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromoVideo {
    pub title: String,
    #[serde(default)]
    pub trailer: Option<Trailer>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpisodeVideo {
    pub mal_id: MalId,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub images: Option<Images>,
    #[serde(default)]
    pub title: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MusicVideo {
    pub title: String,
    #[serde(default)]
    pub video: Option<Trailer>,
    #[serde(default)]
    pub meta: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PicturesVariants {
    pub data: Vec<Images>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimeStatistics {
    pub watching: u32,
    pub completed: u32,
    pub on_hold: u32,
    pub dropped: u32,
    pub plan_to_watch: u32,
    pub total: u32,
    #[serde(default)]
    pub scores: Option<Vec<ScoreDistribution>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoreDistribution {
    pub score: u8,
    pub votes: u32,
    pub percentage: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MoreInfo {
    pub moreinfo: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntryRecommendation {
    pub mal_id: MalId,
    pub url: String,
    pub images: Images,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimeUserUpdate {
    pub user: UserMeta,
    #[serde(default)]
    pub score: Option<u8>,
    #[serde(default)]
    pub status: Option<String>, // "watching"
    #[serde(default)]
    pub episodes_seen: Option<u32>,
    #[serde(default)]
    pub episodes_total: Option<u32>,
    #[serde(default)]
    pub date: Option<String>,
}

// User types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserMeta {
    pub username: String,
    pub url: String,
    #[serde(default)]
    pub images: Option<Images>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserProfile {
    #[serde(flatten)]
    pub user: UserMeta,
    #[serde(default)]
    pub last_online: Option<String>,
    #[serde(default)]
    pub gender: Option<String>,
    #[serde(default)]
    pub birthday: Option<String>,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub joined: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserStatistics {
    #[serde(default)]
    pub anime: Option<UserAnimeStats>,
    #[serde(default)]
    pub manga: Option<UserMangaStats>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserAnimeStats {
    pub days_watched: f32,
    pub mean_score: f32,
    pub watching: u32,
    pub completed: u32,
    pub on_hold: u32,
    pub dropped: u32,
    pub plan_to_watch: u32,
    pub total_entries: u32,
    pub rewatched: u32,
    pub episodes_watched: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserMangaStats {
    pub days_read: f32,
    pub mean_score: f32,
    pub reading: u32,
    pub completed: u32,
    pub on_hold: u32,
    pub dropped: u32,
    pub plan_to_read: u32,
    pub total_entries: u32,
    pub reread: u32,
    pub chapters_read: u32,
    pub volumes_read: u32,
}

// Manga (similar structure to Anime)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Published {
    #[serde(default)]
    pub from: Option<String>,
    #[serde(default)]
    pub to: Option<String>,
    #[serde(default)]
    pub prop: Option<AiredProp>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Manga {
    pub mal_id: MalId,
    pub url: String,
    #[serde(default)]
    pub images: Option<Images>,
    pub approved: bool,
    #[serde(default)]
    pub titles: Option<Vec<TitleEntry>>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub title_english: Option<String>,
    #[serde(default)]
    pub title_japanese: Option<String>,
    #[serde(default)]
    pub title_synonyms: Option<Vec<String>>,
    #[serde(default)]
    pub r#type: Option<String>, // Manga, Novel, One-shot, etc
    #[serde(default)]
    pub chapters: Option<i32>,
    #[serde(default)]
    pub volumes: Option<i32>,
    #[serde(default)]
    pub status: Option<String>, // Publishing, Finished
    #[serde(default)]
    pub published: Option<Published>,
    #[serde(default)]
    pub score: Option<f32>,
    #[serde(default)]
    pub scored_by: Option<i32>,
    #[serde(default)]
    pub rank: Option<i32>,
    #[serde(default)]
    pub popularity: Option<i32>,
    #[serde(default)]
    pub members: Option<i32>,
    #[serde(default)]
    pub favorites: Option<i32>,
    #[serde(default)]
    pub synopsis: Option<String>,
    #[serde(default)]
    pub background: Option<String>,
    #[serde(default)]
    pub authors: Option<Vec<MalEntity>>,
    #[serde(default)]
    pub serializations: Option<Vec<MalEntity>>, // magazines
    #[serde(default)]
    pub genres: Option<Vec<MalEntity>>,
}

// Characters & People
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Character {
    pub mal_id: MalId,
    pub url: String,
    #[serde(default)]
    pub images: Option<Images>,
    pub name: String,
    #[serde(default)]
    pub name_kanji: Option<String>,
    #[serde(default)]
    pub nicknames: Option<Vec<String>>,
    #[serde(default)]
    pub favorites: Option<i32>,
    #[serde(default)]
    pub about: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Person {
    pub mal_id: MalId,
    pub url: String,
    #[serde(default)]
    pub images: Option<Images>,
    pub name: String,
    #[serde(default)]
    pub given_name: Option<String>,
    #[serde(default)]
    pub family_name: Option<String>,
    #[serde(default)]
    pub alternate_names: Option<Vec<String>>,
    #[serde(default)]
    pub birthday: Option<String>,
    #[serde(default)]
    pub website_url: Option<String>,
    #[serde(default)]
    pub favorites: Option<i32>,
    #[serde(default)]
    pub about: Option<String>,
}

// Search parameters
#[derive(Debug, Default, Clone)]
pub struct JikanSearchParams {
    pub q: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub r#type: Option<String>, // tv, movie, ova, special, ona, music
    pub score: Option<f32>,
    pub min_score: Option<f32>,
    pub max_score: Option<f32>,
    pub status: Option<String>, // airing, complete, upcoming
    pub rating: Option<String>, // g, pg, pg13, r17, r, rx
    pub genre: Option<String>,
    pub genre_exclude: Option<String>,
    pub order_by: Option<String>, // title, start_date, end_date, episodes, score, scored_by, rank, popularity, members, favorites
    pub sort: Option<String>,     // desc, asc
    pub letter: Option<String>,
    pub producer: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

impl JikanSearchParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn query(mut self, q: impl Into<String>) -> Self {
        self.q = Some(q.into());
        self
    }

    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn anime_type(mut self, anime_type: impl Into<String>) -> Self {
        self.r#type = Some(anime_type.into());
        self
    }

    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub fn min_score(mut self, score: f32) -> Self {
        self.min_score = Some(score);
        self
    }

    pub fn order_by(mut self, order: impl Into<String>) -> Self {
        self.order_by = Some(order.into());
        self
    }

    pub fn to_query_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();

        if let Some(ref q) = self.q {
            params.push(("q".to_string(), q.clone()));
        }
        if let Some(page) = self.page {
            params.push(("page".to_string(), page.to_string()));
        }
        if let Some(limit) = self.limit {
            params.push(("limit".to_string(), limit.to_string()));
        }
        if let Some(ref r#type) = self.r#type {
            params.push(("type".to_string(), r#type.clone()));
        }
        if let Some(score) = self.score {
            params.push(("score".to_string(), score.to_string()));
        }
        if let Some(min_score) = self.min_score {
            params.push(("min_score".to_string(), min_score.to_string()));
        }
        if let Some(max_score) = self.max_score {
            params.push(("max_score".to_string(), max_score.to_string()));
        }
        if let Some(ref status) = self.status {
            params.push(("status".to_string(), status.clone()));
        }
        if let Some(ref rating) = self.rating {
            params.push(("rating".to_string(), rating.clone()));
        }
        if let Some(ref order_by) = self.order_by {
            params.push(("order_by".to_string(), order_by.clone()));
        }
        if let Some(ref sort) = self.sort {
            params.push(("sort".to_string(), sort.clone()));
        }

        params
    }
}
