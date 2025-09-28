//! AniList GraphQL models - Simplified for clean implementation

#![allow(unused)]

use serde::{Deserialize, Serialize};

// Basic enums
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MediaType {
    Anime,
    Manga,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MediaFormat {
    Tv,
    TvShort,
    Movie,
    Special,
    Ova,
    Ona,
    Music,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MediaStatus {
    Finished,
    Releasing,
    NotYetReleased,
    Cancelled,
    Hiatus,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MediaSeason {
    Winter,
    Spring,
    Summer,
    Fall,
    #[serde(other)]
    Unknown,
}

// Date structures
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct FuzzyDate {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub day: Option<i32>,
}

// Main Media type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    pub id: Option<i32>,
    pub id_mal: Option<i32>,
    pub title: Option<MediaTitle>,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub media_type: Option<MediaType>,
    pub format: Option<MediaFormat>,
    pub status: Option<MediaStatus>,
    pub episodes: Option<i32>,
    pub duration: Option<i32>,
    pub source: Option<String>,
    pub country_of_origin: Option<String>,
    pub hashtag: Option<String>,
    pub trailer: Option<MediaTrailer>,
    pub updated_at: Option<i64>,
    pub cover_image: Option<MediaCoverImage>,
    pub banner_image: Option<String>,
    pub genres: Option<Vec<String>>,
    pub synonyms: Option<Vec<String>>,
    pub average_score: Option<i32>,
    pub mean_score: Option<i32>,
    pub popularity: Option<i32>,
    pub favourites: Option<i32>,
    pub tags: Option<Vec<MediaTag>>,
    pub studios: Option<StudioConnection>,
    pub is_adult: Option<bool>,
    pub external_links: Option<Vec<MediaExternalLink>>,
    pub streaming_episodes: Option<Vec<MediaStreamingEpisode>>,
    pub start_date: Option<FuzzyDate>,
    pub end_date: Option<FuzzyDate>,
    pub season: Option<MediaSeason>,
    pub season_year: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct MediaTitle {
    pub romaji: Option<String>,
    pub english: Option<String>,
    pub native: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct MediaTrailer {
    pub id: Option<String>,
    pub site: Option<String>,
    pub thumbnail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct MediaCoverImage {
    pub extra_large: Option<String>,
    pub large: Option<String>,
    pub medium: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct MediaTag {
    pub id: Option<i32>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub rank: Option<i32>,
    pub is_general_spoiler: Option<bool>,
    pub is_media_spoiler: Option<bool>,
    pub is_adult: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct MediaExternalLink {
    pub id: Option<i32>,
    pub url: Option<String>,
    pub site: Option<String>,
    pub site_id: Option<i32>,
    pub language: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct MediaStreamingEpisode {
    pub title: Option<String>,
    pub thumbnail: Option<String>,
    pub url: Option<String>,
    pub site: Option<String>,
}

// Studio structures
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct StudioConnection {
    pub edges: Option<Vec<StudioEdge>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct StudioEdge {
    pub node: Option<Studio>,
    pub is_main: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct Studio {
    pub id: Option<i32>,
    pub name: Option<String>,
}

// Query response structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListResponse<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<AniListError>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListError {
    pub message: String,
    pub status: Option<i32>,
}

// Page wrapper for paginated queries
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Page {
    pub media: Vec<Media>,
    pub page_info: Option<PageInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct PageInfo {
    pub total: Option<i32>,
    pub per_page: Option<i32>,
    pub current_page: Option<i32>,
    pub last_page: Option<i32>,
    pub has_next_page: Option<bool>,
}

// Response wrapper types for GraphQL queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListSearchResponse {
    #[serde(rename = "Page")]
    pub page: Page,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListMediaResponse {
    #[serde(rename = "Media")]
    pub media: Option<Media>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListCharactersResponse {
    #[serde(rename = "Media")]
    pub media: Option<MediaWithCharacters>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListStaffResponse {
    #[serde(rename = "Media")]
    pub media: Option<MediaWithStaff>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaWithCharacters {
    pub characters: CharacterConnection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaWithStaff {
    pub staff: StaffConnection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffConnection {
    pub nodes: Vec<AniListStaff>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterConnection {
    pub nodes: Vec<AniListCharacter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListCharacter {
    pub id: Option<i32>,
    pub name: Option<CharacterName>,
    pub image: Option<CharacterImage>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterName {
    pub full: Option<String>,
    pub native: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterImage {
    pub large: Option<String>,
    pub medium: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListStaff {
    pub id: Option<i32>,
    pub name: Option<StaffName>,
    pub image: Option<StaffImage>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffName {
    pub full: Option<String>,
    pub native: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffImage {
    pub large: Option<String>,
    pub medium: Option<String>,
}

// Additional response types for new functions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListStatisticsResponse {
    #[serde(rename = "Media")]
    pub media: Option<MediaWithStatistics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaWithStatistics {
    pub stats: AniListStatistics,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AniListStatistics {
    pub score_distribution: Option<Vec<ScoreDistribution>>,
    pub status_distribution: Option<Vec<StatusDistribution>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreDistribution {
    pub score: Option<i32>,
    pub amount: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusDistribution {
    pub status: Option<String>,
    pub amount: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListRecommendationsResponse {
    #[serde(rename = "Media")]
    pub media: Option<MediaWithRecommendations>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaWithRecommendations {
    pub recommendations: RecommendationConnection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationConnection {
    pub nodes: Vec<AniListRecommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListRecommendation {
    pub id: Option<i32>,
    pub rating: Option<i32>,
    pub media_recommendation: Option<AniListMedia>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListRelationsResponse {
    #[serde(rename = "Media")]
    pub media: Option<MediaWithRelations>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaWithRelations {
    pub relations: RelationConnection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationConnection {
    pub nodes: Vec<AniListRelation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListRelation {
    pub id: Option<i32>,
    pub relation_type: Option<String>,
    pub node: Option<AniListMedia>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListScheduleResponse {
    #[serde(rename = "Page")]
    pub page: SchedulePage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulePage {
    pub airing_schedules: Option<Vec<AniListSchedule>>,
    pub page_info: Option<PageInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListSchedule {
    pub id: Option<i32>,
    pub airing_at: Option<i64>,
    pub episode: Option<i32>,
    pub media: Option<AniListMedia>,
}

// Search parameters for advanced search
#[derive(Debug, Clone, Default)]
pub struct AniListSearchParams {
    pub search: Option<String>,
    pub per_page: Option<usize>,
    pub page: Option<u32>,
    pub sort: Option<Vec<String>>,
    pub genre_in: Option<Vec<String>>,
    pub genre_not_in: Option<Vec<String>>,
    pub year: Option<i32>,
    pub season: Option<String>,
    pub format_in: Option<Vec<String>>,
    pub status: Option<String>,
    pub min_score: Option<f32>,
}

impl AniListSearchParams {
    pub fn to_json(&self) -> serde_json::Value {
        let mut json = serde_json::Map::new();

        if let Some(search) = &self.search {
            json.insert(
                "search".to_string(),
                serde_json::Value::String(search.clone()),
            );
        }
        if let Some(per_page) = self.per_page {
            json.insert(
                "perPage".to_string(),
                serde_json::Value::Number(serde_json::Number::from(per_page)),
            );
        }
        if let Some(page) = self.page {
            json.insert(
                "page".to_string(),
                serde_json::Value::Number(serde_json::Number::from(page)),
            );
        }
        if let Some(sort) = &self.sort {
            json.insert(
                "sort".to_string(),
                serde_json::Value::Array(
                    sort.iter()
                        .map(|s| serde_json::Value::String(s.clone()))
                        .collect(),
                ),
            );
        }
        if let Some(genres) = &self.genre_in {
            json.insert(
                "genre_in".to_string(),
                serde_json::Value::Array(
                    genres
                        .iter()
                        .map(|g| serde_json::Value::String(g.clone()))
                        .collect(),
                ),
            );
        }
        if let Some(year) = self.year {
            json.insert(
                "seasonYear".to_string(),
                serde_json::Value::Number(serde_json::Number::from(year)),
            );
        }
        if let Some(season) = &self.season {
            json.insert(
                "season".to_string(),
                serde_json::Value::String(season.clone()),
            );
        }
        if let Some(status) = &self.status {
            json.insert(
                "status".to_string(),
                serde_json::Value::String(status.clone()),
            );
        }

        serde_json::Value::Object(json)
    }
}

// Type aliases for consistency with adapter
pub type AniListMedia = Media;
