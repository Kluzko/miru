#![allow(unused)]
use serde::{Deserialize, Serialize};

// Response envelopes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TmdbSearchResponse {
    pub page: u32,
    pub results: Vec<TvShow>,
    pub total_pages: u32,
    pub total_results: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TmdbError {
    pub status_code: u16,
    pub status_message: String,
    #[serde(default)]
    pub success: Option<bool>,
}

// Core TV Show types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TvShow {
    pub id: u32,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub original_name: Option<String>,
    #[serde(default)]
    pub original_language: Option<String>,
    #[serde(default)]
    pub overview: Option<String>,
    #[serde(default)]
    pub poster_path: Option<String>,
    #[serde(default)]
    pub backdrop_path: Option<String>,
    #[serde(default)]
    pub first_air_date: Option<String>,
    #[serde(default)]
    pub vote_average: Option<f32>,
    #[serde(default)]
    pub vote_count: Option<u32>,
    #[serde(default)]
    pub popularity: Option<f32>,
    #[serde(default)]
    pub genre_ids: Option<Vec<u32>>,
    #[serde(default)]
    pub origin_country: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TvShowDetails {
    pub id: u32,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub original_name: Option<String>,
    #[serde(default)]
    pub original_language: Option<String>,
    #[serde(default)]
    pub overview: Option<String>,
    #[serde(default)]
    pub poster_path: Option<String>,
    #[serde(default)]
    pub backdrop_path: Option<String>,
    #[serde(default)]
    pub first_air_date: Option<String>,
    #[serde(default)]
    pub last_air_date: Option<String>,
    #[serde(default)]
    pub vote_average: Option<f32>,
    #[serde(default)]
    pub vote_count: Option<u32>,
    #[serde(default)]
    pub popularity: Option<f32>,
    #[serde(default)]
    pub genres: Option<Vec<Genre>>,
    #[serde(default)]
    pub origin_country: Option<Vec<String>>,
    #[serde(default)]
    pub status: Option<String>, // "Returning Series", "Ended", "Canceled"
    #[serde(default)]
    pub r#type: Option<String>, // "Scripted", "Documentary", "Animation"
    #[serde(default)]
    pub number_of_episodes: Option<u32>,
    #[serde(default)]
    pub number_of_seasons: Option<u32>,
    #[serde(default)]
    pub episode_run_time: Option<Vec<u32>>,
    #[serde(default)]
    pub networks: Option<Vec<Network>>,
    #[serde(default)]
    pub production_companies: Option<Vec<ProductionCompany>>,
    #[serde(default)]
    pub created_by: Option<Vec<Creator>>,
    #[serde(default)]
    pub tagline: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub in_production: Option<bool>,
    #[serde(default)]
    pub languages: Option<Vec<String>>,
    #[serde(default)]
    pub last_episode_to_air: Option<Episode>,
    #[serde(default)]
    pub next_episode_to_air: Option<Episode>,
    #[serde(default)]
    pub seasons: Option<Vec<Season>>,
}

// External IDs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExternalIds {
    #[serde(default)]
    pub imdb_id: Option<String>,
    #[serde(default)]
    pub freebase_mid: Option<String>,
    #[serde(default)]
    pub freebase_id: Option<String>,
    #[serde(default)]
    pub tvdb_id: Option<u32>,
    #[serde(default)]
    pub tvrage_id: Option<u32>,
    #[serde(default)]
    pub wikidata_id: Option<String>,
    #[serde(default)]
    pub facebook_id: Option<String>,
    #[serde(default)]
    pub instagram_id: Option<String>,
    #[serde(default)]
    pub twitter_id: Option<String>,
}

// Images
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImagesResponse {
    pub id: u32,
    #[serde(default)]
    pub backdrops: Option<Vec<Image>>,
    #[serde(default)]
    pub logos: Option<Vec<Image>>,
    #[serde(default)]
    pub posters: Option<Vec<Image>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Image {
    pub aspect_ratio: f32,
    pub height: u32,
    pub width: u32,
    #[serde(default)]
    pub iso_639_1: Option<String>,
    pub file_path: String,
    pub vote_average: f32,
    pub vote_count: u32,
}

// Videos
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideosResponse {
    pub id: u32,
    #[serde(default)]
    pub results: Option<Vec<Video>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Video {
    #[serde(default)]
    pub iso_639_1: Option<String>,
    #[serde(default)]
    pub iso_3166_1: Option<String>,
    pub name: String,
    pub key: String,    // YouTube video ID
    pub site: String,   // "YouTube"
    pub size: u32,      // 1080, 720, etc.
    pub r#type: String, // "Trailer", "Teaser", "Clip", "Featurette", "Behind the Scenes", "Opening Credits"
    pub official: bool,
    pub published_at: String,
    pub id: String,
}

// Supporting types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Genre {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Network {
    pub id: u32,
    pub name: String,
    #[serde(default)]
    pub logo_path: Option<String>,
    #[serde(default)]
    pub origin_country: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductionCompany {
    pub id: u32,
    pub name: String,
    #[serde(default)]
    pub logo_path: Option<String>,
    #[serde(default)]
    pub origin_country: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Creator {
    pub id: u32,
    pub name: String,
    #[serde(default)]
    pub credit_id: Option<String>,
    #[serde(default)]
    pub gender: Option<u8>,
    #[serde(default)]
    pub profile_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Episode {
    pub id: u32,
    pub name: String,
    #[serde(default)]
    pub overview: Option<String>,
    pub episode_number: u32,
    pub season_number: u32,
    #[serde(default)]
    pub air_date: Option<String>,
    #[serde(default)]
    pub vote_average: Option<f32>,
    #[serde(default)]
    pub vote_count: Option<u32>,
    #[serde(default)]
    pub still_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Season {
    pub id: u32,
    pub name: String,
    #[serde(default)]
    pub overview: Option<String>,
    pub season_number: u32,
    #[serde(default)]
    pub episode_count: Option<u32>,
    #[serde(default)]
    pub air_date: Option<String>,
    #[serde(default)]
    pub poster_path: Option<String>,
}

// Content Ratings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentRatingsResponse {
    pub id: u32,
    #[serde(default)]
    pub results: Option<Vec<ContentRating>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentRating {
    pub iso_3166_1: String, // Country code (e.g., "US", "JP")
    pub rating: String,     // Rating value (e.g., "TV-14", "PG-13")
}

// Find by external ID
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FindResponse {
    #[serde(default)]
    pub movie_results: Option<Vec<Movie>>,
    #[serde(default)]
    pub tv_results: Option<Vec<TvShow>>,
    #[serde(default)]
    pub tv_episode_results: Option<Vec<Episode>>,
    #[serde(default)]
    pub tv_season_results: Option<Vec<Season>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Movie {
    pub id: u32,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub original_title: Option<String>,
    #[serde(default)]
    pub overview: Option<String>,
    #[serde(default)]
    pub poster_path: Option<String>,
    #[serde(default)]
    pub backdrop_path: Option<String>,
    #[serde(default)]
    pub release_date: Option<String>,
    #[serde(default)]
    pub vote_average: Option<f32>,
    #[serde(default)]
    pub vote_count: Option<u32>,
    #[serde(default)]
    pub popularity: Option<f32>,
}

// Search parameters
#[derive(Debug, Default, Clone)]
pub struct TmdbSearchParams {
    pub query: Option<String>,
    pub page: Option<u32>,
    pub language: Option<String>,
    pub first_air_date_year: Option<u32>,
    pub include_adult: Option<bool>,
}

impl TmdbSearchParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    pub fn language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    pub fn first_air_date_year(mut self, year: u32) -> Self {
        self.first_air_date_year = Some(year);
        self
    }

    pub fn include_adult(mut self, include: bool) -> Self {
        self.include_adult = Some(include);
        self
    }

    pub fn to_query_params(&self, api_key: &str) -> Vec<(String, String)> {
        let mut params = vec![("api_key".to_string(), api_key.to_string())];

        if let Some(ref query) = self.query {
            params.push(("query".to_string(), query.clone()));
        }
        if let Some(page) = self.page {
            params.push(("page".to_string(), page.to_string()));
        }
        if let Some(ref language) = self.language {
            params.push(("language".to_string(), language.clone()));
        }
        if let Some(year) = self.first_air_date_year {
            params.push(("first_air_date_year".to_string(), year.to_string()));
        }
        if let Some(include) = self.include_adult {
            params.push(("include_adult".to_string(), include.to_string()));
        }

        params
    }
}
