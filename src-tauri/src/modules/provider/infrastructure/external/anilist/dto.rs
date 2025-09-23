use serde::{Deserialize, Serialize};
use serde_json::Value;

/// AniList GraphQL Response Wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListResponse<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<AniListError>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListError {
    pub message: String,
    pub status: Option<i32>,
    pub locations: Option<Vec<AniListErrorLocation>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListErrorLocation {
    pub line: i32,
    pub column: i32,
}

/// AniList GraphQL Request
#[derive(Debug, Clone, Serialize)]
pub struct AniListRequest {
    pub query: String,
    pub variables: Option<Value>,
}

/// Single Media Query Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaResponse {
    #[serde(rename = "Media")]
    pub media: Option<AniListMedia>,
}

/// Multiple Media Query Response (for search, batch queries)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageResponse {
    #[serde(rename = "Page")]
    pub page: AniListPage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListPage {
    #[serde(rename = "pageInfo")]
    pub page_info: PageInfo,
    pub media: Vec<AniListMedia>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageInfo {
    pub total: Option<i32>,
    #[serde(rename = "currentPage")]
    pub current_page: Option<i32>,
    #[serde(rename = "lastPage")]
    pub last_page: Option<i32>,
    #[serde(rename = "hasNextPage")]
    pub has_next_page: Option<bool>,
    #[serde(rename = "perPage")]
    pub per_page: Option<i32>,
}

/// AniList Media Object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListMedia {
    pub id: i32,
    #[serde(rename = "idMal")]
    pub id_mal: Option<i32>,
    pub title: AniListTitle,
    pub description: Option<String>,
    #[serde(rename = "startDate")]
    pub start_date: Option<AniListDate>,
    #[serde(rename = "endDate")]
    pub end_date: Option<AniListDate>,
    pub season: Option<String>,
    #[serde(rename = "seasonYear")]
    pub season_year: Option<i32>,
    #[serde(rename = "type")]
    pub media_type: Option<String>,
    pub format: Option<String>,
    pub status: Option<String>,
    pub episodes: Option<i32>,
    pub duration: Option<i32>,
    #[serde(rename = "coverImage")]
    pub cover_image: Option<AniListCoverImage>,
    #[serde(rename = "bannerImage")]
    pub banner_image: Option<String>,
    pub genres: Vec<String>,
    pub synonyms: Vec<String>,
    #[serde(rename = "averageScore")]
    pub average_score: Option<i32>,
    #[serde(rename = "meanScore")]
    pub mean_score: Option<i32>,
    pub popularity: Option<i32>,
    pub favourites: Option<i32>,
    pub source: Option<String>,
    #[serde(rename = "isAdult")]
    pub is_adult: Option<bool>,
    #[serde(rename = "countryOfOrigin")]
    pub country_of_origin: Option<String>,
    #[serde(rename = "externalLinks")]
    pub external_links: Option<Vec<AniListExternalLink>>,
    pub studios: Option<AniListStudiosConnection>,
    pub trailer: Option<AniListTrailer>,
    pub tags: Option<Vec<AniListTag>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListTitle {
    pub romaji: Option<String>,
    pub english: Option<String>,
    pub native: Option<String>,
    #[serde(rename = "userPreferred")]
    pub user_preferred: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListDate {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub day: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListCoverImage {
    #[serde(rename = "extraLarge")]
    pub extra_large: Option<String>,
    pub large: Option<String>,
    pub medium: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListExternalLink {
    pub id: i32,
    pub url: String,
    pub site: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListStudiosConnection {
    pub edges: Vec<AniListStudioEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListStudioEdge {
    #[serde(rename = "isMain")]
    pub is_main: Option<bool>,
    pub node: AniListStudio,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListStudio {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListTrailer {
    pub id: Option<String>,
    pub site: Option<String>,
    pub thumbnail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListTag {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub rank: Option<i32>,
    #[serde(rename = "isGeneralSpoiler")]
    pub is_general_spoiler: Option<bool>,
    #[serde(rename = "isMediaSpoiler")]
    pub is_media_spoiler: Option<bool>,
    #[serde(rename = "isAdult")]
    pub is_adult: Option<bool>,
}
