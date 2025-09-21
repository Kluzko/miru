use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JikanAnimeResponse {
    pub data: JikanAnimeData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JikanAnimeListResponse {
    pub data: Vec<JikanAnimeData>,
    pub pagination: Option<JikanPagination>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JikanPagination {
    pub last_visible_page: i32,
    pub has_next_page: bool,
    pub current_page: i32,
    pub items: JikanPaginationItems,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JikanPaginationItems {
    pub count: i32,
    pub total: i32,
    pub per_page: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JikanAnimeData {
    pub mal_id: i32,
    pub url: String,
    pub images: JikanImages,
    pub trailer: Option<JikanTrailer>,
    pub approved: bool,
    pub titles: Vec<JikanTitle>,
    pub title: String,
    pub title_english: Option<String>,
    pub title_japanese: Option<String>,
    pub title_synonyms: Vec<String>,
    #[serde(rename = "type")]
    pub anime_type: Option<String>,
    pub source: Option<String>,
    pub episodes: Option<i32>,
    pub status: Option<String>,
    pub airing: bool,
    pub aired: JikanAired,
    pub duration: Option<String>,
    pub rating: Option<String>,
    pub score: Option<f32>,
    pub scored_by: Option<i32>,
    pub rank: Option<i32>,
    pub popularity: Option<i32>,
    pub members: Option<i32>,
    pub favorites: Option<i32>,
    pub synopsis: Option<String>,
    pub background: Option<String>,
    pub season: Option<String>,
    pub year: Option<i32>,
    pub broadcast: Option<JikanBroadcast>,
    pub producers: Vec<JikanEntity>,
    pub licensors: Vec<JikanEntity>,
    pub studios: Vec<JikanEntity>,
    pub genres: Vec<JikanEntity>,
    pub explicit_genres: Vec<JikanEntity>,
    pub themes: Vec<JikanEntity>,
    pub demographics: Vec<JikanEntity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JikanImages {
    pub jpg: JikanImageSet,
    pub webp: Option<JikanImageSet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JikanImageSet {
    pub image_url: Option<String>,
    pub small_image_url: Option<String>,
    pub large_image_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JikanTrailer {
    pub youtube_id: Option<String>,
    pub url: Option<String>,
    pub embed_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JikanTitle {
    #[serde(rename = "type")]
    pub title_type: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JikanAired {
    pub from: Option<String>,
    pub to: Option<String>,
    pub prop: Option<JikanAiredProp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JikanAiredProp {
    pub from: JikanDateProp,
    pub to: JikanDateProp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JikanDateProp {
    pub day: Option<i32>,
    pub month: Option<i32>,
    pub year: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JikanBroadcast {
    pub day: Option<String>,
    pub time: Option<String>,
    pub timezone: Option<String>,
    pub string: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JikanEntity {
    pub mal_id: i32,
    #[serde(rename = "type")]
    pub entity_type: String,
    pub name: String,
    pub url: String,
}

// Search request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JikanSearchParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub q: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sfw: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genres: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_score: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_score: Option<f32>,
}

impl Default for JikanSearchParams {
    fn default() -> Self {
        Self {
            q: None,
            page: Some(1),
            limit: Some(25),
            order_by: Some("members".to_string()),
            sort: Some("desc".to_string()),
            sfw: Some(true),
            genres: None,
            status: None,
            rating: None,
            min_score: None,
            max_score: None,
        }
    }
}
