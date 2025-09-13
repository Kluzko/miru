/// AniList GraphQL Query Definitions
/// Organized collection of GraphQL queries for different use cases
use serde_json::{json, Value};

pub struct AniListQueries;

impl AniListQueries {
    /// Search query with comprehensive fields for search results
    pub fn search_anime() -> &'static str {
        r#"
            query ($search: String, $perPage: Int, $type: MediaType) {
                Page(page: 1, perPage: $perPage) {
                    pageInfo {
                        total
                        currentPage
                        lastPage
                        hasNextPage
                        perPage
                    }
                    media(search: $search, type: $type, sort: [POPULARITY_DESC, SCORE_DESC]) {
                        id
                        idMal
                        title {
                            romaji
                            english
                            native
                            userPreferred
                        }
                        description
                        startDate {
                            year
                            month
                            day
                        }
                        endDate {
                            year
                            month
                            day
                        }
                        season
                        seasonYear
                        type
                        format
                        status
                        episodes
                        duration
                        coverImage {
                            extraLarge
                            large
                            medium
                            color
                        }
                        bannerImage
                        genres
                        synonyms
                        averageScore
                        meanScore
                        popularity
                        favourites
                        source
                        isAdult
                        countryOfOrigin
                        studios {
                            nodes {
                                id
                                name
                            }
                        }
                        trailer {
                            id
                            site
                            thumbnail
                        }
                    }
                }
            }
        "#
    }

    /// Variables for search query
    pub fn search_variables(query: &str, limit: usize) -> Value {
        json!({
            "search": query.trim(),
            "perPage": limit.min(50),
            "type": "ANIME"
        })
    }

    /// Get anime by AniList ID (if ever needed)
    pub fn get_by_id() -> &'static str {
        r#"
            query ($id: Int, $type: MediaType) {
                Media(id: $id, type: $type) {
                    id
                    idMal
                    title {
                        romaji
                        english
                        native
                        userPreferred
                    }
                    description
                    startDate {
                        year
                        month
                        day
                    }
                    endDate {
                        year
                        month
                        day
                    }
                    season
                    seasonYear
                    type
                    format
                    status
                    episodes
                    duration
                    coverImage {
                        extraLarge
                        large
                        medium
                        color
                    }
                    bannerImage
                    genres
                    synonyms
                    averageScore
                    meanScore
                    popularity
                    favourites
                    source
                    isAdult
                    countryOfOrigin
                    studios {
                        nodes {
                            id
                            name
                        }
                    }
                    trailer {
                        id
                        site
                        thumbnail
                    }
                }
            }
        "#
    }

    /// Variables for get by ID query
    pub fn get_by_id_variables(anilist_id: i32) -> Value {
        json!({
            "id": anilist_id,
            "type": "ANIME"
        })
    }

    /// Trending anime query (if needed for future features)
    pub fn trending_anime() -> &'static str {
        r#"
            query ($perPage: Int, $page: Int, $type: MediaType) {
                Page(page: $page, perPage: $perPage) {
                    pageInfo {
                        total
                        currentPage
                        lastPage
                        hasNextPage
                        perPage
                    }
                    media(type: $type, sort: [TRENDING_DESC, POPULARITY_DESC]) {
                        id
                        idMal
                        title {
                            romaji
                            english
                            native
                            userPreferred
                        }
                        description
                        averageScore
                        popularity
                        coverImage {
                            large
                            medium
                        }
                        genres
                        format
                        status
                        episodes
                    }
                }
            }
        "#
    }

    /// Variables for trending query
    pub fn trending_variables(limit: usize, page: i32) -> Value {
        json!({
            "perPage": limit.min(50),
            "page": page.max(1),
            "type": "ANIME"
        })
    }

    /// Seasonal anime query (if needed for future features)
    pub fn seasonal_anime() -> &'static str {
        r#"
            query ($seasonYear: Int, $season: MediaSeason, $page: Int, $type: MediaType) {
                Page(page: $page, perPage: 50) {
                    pageInfo {
                        total
                        currentPage
                        lastPage
                        hasNextPage
                        perPage
                    }
                    media(seasonYear: $seasonYear, season: $season, type: $type, sort: [POPULARITY_DESC]) {
                        id
                        idMal
                        title {
                            romaji
                            english
                            native
                            userPreferred
                        }
                        description
                        averageScore
                        popularity
                        coverImage {
                            large
                            medium
                        }
                        genres
                        format
                        status
                        episodes
                        season
                        seasonYear
                    }
                }
            }
        "#
    }

    /// Variables for seasonal query
    pub fn seasonal_variables(year: i32, season: &str, page: i32) -> Value {
        json!({
            "seasonYear": year,
            "season": season.to_uppercase(),
            "page": page.max(1),
            "type": "ANIME"
        })
    }
}
