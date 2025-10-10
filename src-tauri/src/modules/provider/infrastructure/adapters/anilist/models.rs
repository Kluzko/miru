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
    pub nodes: Option<Vec<Studio>>,     // Used in search queries
    pub edges: Option<Vec<StudioEdge>>, // Used in detail queries
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct StudioEdge {
    pub is_main: Option<bool>,
    pub node: Option<Studio>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct Studio {
    pub id: Option<i32>,
    pub name: Option<String>,
    pub is_main: Option<bool>, // For search queries with nodes
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
    pub edges: Vec<AniListStaff>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterConnection {
    pub edges: Vec<AniListCharacter>,
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
    pub edges: Vec<AniListRecommendation>,
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
    pub edges: Vec<AniListRelation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListRelation {
    pub id: Option<i32>,
    #[serde(rename = "relationType")]
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

// Optimized nested relations models for single GraphQL query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListNestedRelationsResponse {
    #[serde(rename = "Media")]
    pub media: Option<MediaWithNestedRelations>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaWithNestedRelations {
    pub id: Option<i32>,
    pub title: Option<MediaTitle>,
    pub relations: NestedRelationConnection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestedRelationConnection {
    pub edges: Vec<NestedAniListRelation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestedAniListRelation {
    pub id: Option<i32>,
    #[serde(rename = "relationType")]
    pub relation_type: Option<String>,
    pub node: Option<MediaWithNestedRelations>,
}

// Complete franchise discovery models for optimized single GraphQL query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListFranchiseDiscoveryResponse {
    #[serde(rename = "Media")]
    pub media: Option<MediaWithFranchiseData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaWithFranchiseData {
    pub id: Option<i32>,
    #[serde(rename = "idMal")]
    pub id_mal: Option<i32>,
    pub title: Option<MediaTitle>,
    #[serde(rename = "type")]
    pub media_type: Option<String>,
    pub format: Option<String>,
    pub status: Option<String>,
    pub episodes: Option<i32>,
    #[serde(rename = "startDate")]
    pub start_date: Option<FuzzyDate>,
    #[serde(rename = "endDate")]
    pub end_date: Option<FuzzyDate>,
    pub relations: Option<FranchiseRelationConnection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FranchiseRelationConnection {
    pub edges: Vec<FranchiseAniListRelation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FranchiseAniListRelation {
    pub id: Option<i32>,
    #[serde(rename = "relationType")]
    pub relation_type: Option<String>,
    pub node: Option<MediaWithFranchiseData>,
}

/// Detailed franchise relation information for testing and verification
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct FranchiseRelation {
    pub id: u32,
    pub title: String,
    pub relation_type: String,
    pub format: Option<String>,
    pub status: Option<String>,
    pub episodes: Option<i32>,
    pub start_year: Option<i32>,
}

/// Categorized franchise information organized by content type
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct CategorizedFranchise {
    pub main_story: Vec<FranchiseRelation>,
    pub side_stories: Vec<FranchiseRelation>,
    pub movies: Vec<FranchiseRelation>,
    pub ovas_specials: Vec<FranchiseRelation>,
    pub other: Vec<FranchiseRelation>,
}

impl CategorizedFranchise {
    pub fn new() -> Self {
        Self {
            main_story: Vec::new(),
            side_stories: Vec::new(),
            movies: Vec::new(),
            ovas_specials: Vec::new(),
            other: Vec::new(),
        }
    }

    pub fn total_count(&self) -> usize {
        self.main_story.len()
            + self.side_stories.len()
            + self.movies.len()
            + self.ovas_specials.len()
            + self.other.len()
    }

    pub fn categorize_relation(&mut self, relation: FranchiseRelation) {
        match self.determine_category(&relation) {
            FranchiseCategory::MainStory => self.main_story.push(relation),
            FranchiseCategory::SideStory => self.side_stories.push(relation),
            FranchiseCategory::Movie => self.movies.push(relation),
            FranchiseCategory::OvaSpecial => self.ovas_specials.push(relation),
            FranchiseCategory::Other => self.other.push(relation),
        }
    }

    fn determine_category(&self, relation: &FranchiseRelation) -> FranchiseCategory {
        // Check format first
        if let Some(format) = &relation.format {
            match format.to_uppercase().as_str() {
                "MOVIE" => return FranchiseCategory::Movie,
                "OVA" | "ONA" => return FranchiseCategory::OvaSpecial,
                "SPECIAL" => return FranchiseCategory::OvaSpecial,
                _ => {}
            }
        }

        // Check relation type
        match relation.relation_type.to_lowercase().as_str() {
            "sequel" | "prequel" | "parent story" | "full story" => FranchiseCategory::MainStory,
            "side story" | "spin off" | "alternative" | "character" => FranchiseCategory::SideStory,
            "adaptation" => {
                // Check if it's main story based on title patterns
                if Self::is_main_story_by_title(&relation.title) {
                    FranchiseCategory::MainStory
                } else {
                    // For adaptation relations, check if it's TV format (likely main story)
                    if let Some(format) = &relation.format {
                        if format.to_uppercase() == "TV" {
                            FranchiseCategory::MainStory
                        } else {
                            FranchiseCategory::Other
                        }
                    } else {
                        FranchiseCategory::Other
                    }
                }
            }
            "other" | "shared character" => {
                // Check if it's main story based on title patterns
                if Self::is_main_story_by_title(&relation.title) {
                    FranchiseCategory::MainStory
                } else {
                    FranchiseCategory::Other
                }
            }
            _ => {
                // Default: if it's TV format, assume main story
                if let Some(format) = &relation.format {
                    if format.to_uppercase() == "TV" {
                        FranchiseCategory::MainStory
                    } else {
                        FranchiseCategory::Other
                    }
                } else {
                    FranchiseCategory::Other
                }
            }
        }
    }

    /// Determines if a title represents a main story entry based on patterns
    fn is_main_story_by_title(title: &str) -> bool {
        let title_lower = title.to_lowercase();

        // Check for explicit season indicators
        if title_lower.contains("season") {
            return true;
        }

        // Check for Roman numerals (I, II, III, IV, V) which usually indicate main story
        if title.contains(" II")
            || title.contains(" III")
            || title.contains(" IV")
            || title.contains(" V")
            || title.ends_with(" I")
        {
            return true;
        }

        // Check for numbered seasons patterns
        if title_lower.contains("2nd")
            || title_lower.contains("3rd")
            || title_lower.contains("4th")
            || title_lower.contains("5th")
        {
            return true;
        }

        // Check for common anime sequel patterns
        if title_lower.contains("part")
            || title_lower.contains("cour")
            || title_lower.contains("final")
            || title_lower.contains("last")
        {
            return true;
        }

        // Check for numbered series (like "Anime 2", "Anime 3")
        let words: Vec<&str> = title.split_whitespace().collect();
        for word in &words {
            if word.parse::<u32>().is_ok() && word.parse::<u32>().unwrap() > 1 {
                return true;
            }
        }

        // Exclude typical side story indicators
        if title_lower.contains("gaiden")
            || title_lower.contains("side")
            || title_lower.contains("special")
            || title_lower.contains("ova")
            || title_lower.contains("omake")
        {
            return false;
        }

        // For DanMachi specifically, check for main story arc patterns
        if title_lower.contains("dungeon ni deai") {
            // Main story arcs usually have specific patterns
            if title_lower.contains("shin shou") || // New Chapter
               title_lower.contains("familia") ||
               (title_lower.matches(char::is_numeric).count() > 0 &&
                !title_lower.contains("onsen") &&
                !title_lower.contains("orion"))
            {
                return true;
            }
        }

        false
    }

    pub fn sort_all_categories(&mut self) {
        Self::sort_by_chronology(&mut self.main_story);
        Self::sort_by_chronology(&mut self.side_stories);
        Self::sort_by_chronology(&mut self.movies);
        Self::sort_by_chronology(&mut self.ovas_specials);
        Self::sort_by_chronology(&mut self.other);
    }

    fn sort_by_chronology(relations: &mut Vec<FranchiseRelation>) {
        relations.sort_by(|a, b| {
            // Primary sort: by year
            match (a.start_year, b.start_year) {
                (Some(year_a), Some(year_b)) => {
                    if year_a != year_b {
                        return year_a.cmp(&year_b);
                    }
                }
                (Some(_), None) => return std::cmp::Ordering::Less,
                (None, Some(_)) => return std::cmp::Ordering::Greater,
                (None, None) => {}
            }

            // Secondary sort: by title (for same year items)
            a.title.cmp(&b.title)
        });
    }
}

#[derive(Debug, Clone)]
enum FranchiseCategory {
    MainStory,
    SideStory,
    Movie,
    OvaSpecial,
    Other,
}

// Type aliases for consistency with adapter
pub type AniListMedia = Media;
