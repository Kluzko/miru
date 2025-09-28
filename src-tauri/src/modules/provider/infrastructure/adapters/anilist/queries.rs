//! AniList GraphQL queries
//!
//! Contains all GraphQL query templates for the AniList API.
//! Organized by functionality to match the Jikan adapter capabilities.

/// Media (anime) detail query - equivalent to get_anime_full
pub const MEDIA_DETAIL_QUERY: &str = r#"
query ($id: Int, $idMal: Int) {
  Media(id: $id, idMal: $idMal, type: ANIME) {
    id
    idMal
    title {
      romaji
      english
      native
      userPreferred
    }
    description(asHtml: false)
    format
    status
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
    episodes
    duration
    source
    genres
    synonyms
    coverImage {
      extraLarge
      large
      medium
      color
    }
    bannerImage
    averageScore
    meanScore
    popularity
    favourites
    studios {
      nodes {
        id
        name
        isMain
      }
    }
    tags {
      id
      name
      description
      category
      rank
      isGeneralSpoiler
      isMediaSpoiler
      isAdult
    }
    trailer {
      id
      site
      thumbnail
    }
    isAdult
    nextAiringEpisode {
      airingAt
      timeUntilAiring
      episode
    }
    externalLinks {
      id
      url
      site
      type
      language
    }
    streamingEpisodes {
      title
      thumbnail
      url
      site
    }
    siteUrl
  }
}
"#;

/// Basic anime search query - equivalent to search_anime_basic
pub const ANIME_SEARCH_QUERY: &str = r#"
query ($search: String, $page: Int, $perPage: Int) {
  Page(page: $page, perPage: $perPage) {
    pageInfo {
      total
      currentPage
      lastPage
      hasNextPage
      perPage
    }
    media(search: $search, type: ANIME, sort: SEARCH_MATCH) {
      id
      idMal
      title {
        romaji
        english
        native
        userPreferred
      }
      description(asHtml: false)
      format
      status
      episodes
      duration
      source
      genres
      synonyms
      averageScore
      popularity
      favourites
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
      coverImage {
        extraLarge
        large
        medium
        color
      }
      bannerImage
      trailer {
        id
        site
        thumbnail
      }
      studios {
        nodes {
          id
          name
          isMain
        }
      }
      isAdult
    }
  }
}
"#;

/// Advanced search with filters - equivalent to search_anime_advanced
pub const ANIME_SEARCH_ADVANCED_QUERY: &str = r#"
query ($search: String, $page: Int, $perPage: Int, $genre: [String], $status: MediaStatus, $format: MediaFormat, $seasonYear: Int, $season: MediaSeason, $sort: [MediaSort]) {
  Page(page: $page, perPage: $perPage) {
    pageInfo {
      total
      currentPage
      lastPage
      hasNextPage
      perPage
    }
    media(search: $search, type: ANIME, genre_in: $genre, status: $status, format: $format, seasonYear: $seasonYear, season: $season, sort: $sort) {
      id
      idMal
      title {
        romaji
        english
        native
        userPreferred
      }
      description(asHtml: false)
      format
      status
      episodes
      duration
      genres
      averageScore
      popularity
      season
      seasonYear
      startDate {
        year
        month
        day
      }
      coverImage {
        large
        medium
      }
      studios {
        nodes {
          name
        }
      }
      isAdult
    }
  }
}
"#;

/// Character list query - equivalent to get_anime_characters
pub const ANIME_CHARACTERS_QUERY: &str = r#"
query ($id: Int, $page: Int, $perPage: Int) {
  Media(id: $id, type: ANIME) {
    characters(page: $page, perPage: $perPage, sort: [ROLE, RELEVANCE, ID]) {
      pageInfo {
        total
        currentPage
        lastPage
        hasNextPage
        perPage
      }
      edges {
        id
        role
        name
        voiceActors(language: JAPANESE, sort: [RELEVANCE, ID]) {
          id
          name {
            first
            middle
            last
            full
            native
          }
          language
          image {
            large
            medium
          }
        }
        node {
          id
          name {
            first
            middle
            last
            full
            native
          }
          image {
            large
            medium
          }
          description(asHtml: false)
          gender
          dateOfBirth {
            year
            month
            day
          }
          age
          siteUrl
        }
      }
    }
  }
}
"#;

/// Staff list query - equivalent to get_anime_staff
pub const ANIME_STAFF_QUERY: &str = r#"
query ($id: Int, $page: Int, $perPage: Int) {
  Media(id: $id, type: ANIME) {
    staff(page: $page, perPage: $perPage, sort: [RELEVANCE, ID]) {
      pageInfo {
        total
        currentPage
        lastPage
        hasNextPage
        perPage
      }
      edges {
        id
        role
        node {
          id
          name {
            first
            middle
            last
            full
            native
          }
          language
          image {
            large
            medium
          }
          description(asHtml: false)
          primaryOccupations
          gender
          dateOfBirth {
            year
            month
            day
          }
          age
          siteUrl
        }
      }
    }
  }
}
"#;

/// Seasonal anime query - equivalent to get_season_now/get_season
pub const SEASONAL_ANIME_QUERY: &str = r#"
query ($page: Int, $perPage: Int, $season: MediaSeason, $seasonYear: Int, $sort: [MediaSort]) {
  Page(page: $page, perPage: $perPage) {
    pageInfo {
      total
      currentPage
      lastPage
      hasNextPage
      perPage
    }
    media(type: ANIME, season: $season, seasonYear: $seasonYear, sort: $sort) {
      id
      idMal
      title {
        romaji
        english
        native
        userPreferred
      }
      description(asHtml: false)
      format
      status
      episodes
      duration
      genres
      averageScore
      popularity
      season
      seasonYear
      startDate {
        year
        month
        day
      }
      coverImage {
        large
        medium
      }
      studios {
        nodes {
          name
          isMain
        }
      }
      nextAiringEpisode {
        airingAt
        timeUntilAiring
        episode
      }
      isAdult
    }
  }
}
"#;

/// Trending anime query - equivalent to get_season_now with trending sort
pub const TRENDING_ANIME_QUERY: &str = r#"
query ($page: Int, $perPage: Int) {
  Page(page: $page, perPage: $perPage) {
    pageInfo {
      total
      currentPage
      lastPage
      hasNextPage
      perPage
    }
    media(type: ANIME, sort: TRENDING_DESC) {
      id
      idMal
      title {
        romaji
        english
        native
        userPreferred
      }
      description(asHtml: false)
      format
      status
      episodes
      duration
      genres
      averageScore
      popularity
      trending
      season
      seasonYear
      startDate {
        year
        month
        day
      }
      coverImage {
        large
        medium
      }
      studios {
        nodes {
          name
          isMain
        }
      }
      nextAiringEpisode {
        airingAt
        timeUntilAiring
        episode
      }
      isAdult
    }
  }
}
"#;

/// Airing schedule query - equivalent to get_schedules
pub const AIRING_SCHEDULE_QUERY: &str = r#"
query ($page: Int, $perPage: Int, $airingAt_greater: Int, $airingAt_lesser: Int) {
  Page(page: $page, perPage: $perPage) {
    pageInfo {
      total
      currentPage
      lastPage
      hasNextPage
      perPage
    }
    airingSchedules(airingAt_greater: $airingAt_greater, airingAt_lesser: $airingAt_lesser, sort: TIME) {
      id
      airingAt
      timeUntilAiring
      episode
      media {
        id
        idMal
        title {
          romaji
          english
          native
          userPreferred
        }
        episodes
        duration
        genres
        averageScore
        popularity
        coverImage {
          large
          medium
        }
        studios {
          nodes {
            name
            isMain
          }
        }
        isAdult
      }
    }
  }
}
"#;

/// Media relations query - equivalent to get_anime_relations
pub const ANIME_RELATIONS_QUERY: &str = r#"
query ($id: Int) {
  Media(id: $id, type: ANIME) {
    relations {
      pageInfo {
        total
        currentPage
        lastPage
        hasNextPage
        perPage
      }
      edges {
        id
        relationType
        node {
          id
          idMal
          title {
            romaji
            english
            native
            userPreferred
          }
          type
          format
          status
          episodes
          duration
          genres
          averageScore
          popularity
          startDate {
            year
          }
          coverImage {
            large
            medium
          }
          isAdult
        }
      }
    }
  }
}
"#;

/// Media recommendations query - equivalent to get_anime_recommendations
pub const ANIME_RECOMMENDATIONS_QUERY: &str = r#"
query ($id: Int, $page: Int, $perPage: Int) {
  Media(id: $id, type: ANIME) {
    recommendations(page: $page, perPage: $perPage, sort: [RATING_DESC, ID]) {
      pageInfo {
        total
        currentPage
        lastPage
        hasNextPage
        perPage
      }
      nodes {
        id
        rating
        userRating
        mediaRecommendation {
          id
          idMal
          title {
            romaji
            english
            native
            userPreferred
          }
          format
          status
          episodes
          duration
          genres
          averageScore
          popularity
          startDate {
            year
          }
          coverImage {
            large
            medium
          }
          isAdult
        }
      }
    }
  }
}
"#;

/// Media statistics query - equivalent to get_anime_statistics
pub const ANIME_STATISTICS_QUERY: &str = r#"
query ($id: Int) {
  Media(id: $id, type: ANIME) {
    id
    idMal
    stats {
      scoreDistribution {
        score
        amount
      }
      statusDistribution {
        status
        amount
      }
    }
    rankings {
      id
      rank
      type
      format
      year
      season
      allTime
      context
    }
    favourites
    popularity
    trending
  }
}
"#;
