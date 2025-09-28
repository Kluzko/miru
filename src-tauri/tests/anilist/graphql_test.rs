//! GraphQL queries tests for AniList

use miru_lib::modules::provider::infrastructure::adapters::anilist::queries::*;

#[test]
fn test_media_detail_query_structure() {
    // Test that the query contains expected fields
    assert!(MEDIA_DETAIL_QUERY.contains("query"));
    assert!(MEDIA_DETAIL_QUERY.contains("Media"));
    assert!(MEDIA_DETAIL_QUERY.contains("$id"));
    assert!(MEDIA_DETAIL_QUERY.contains("id"));
    assert!(MEDIA_DETAIL_QUERY.contains("title"));
    assert!(MEDIA_DETAIL_QUERY.contains("description"));
    assert!(MEDIA_DETAIL_QUERY.contains("episodes"));
    assert!(MEDIA_DETAIL_QUERY.contains("averageScore"));
}

#[test]
fn test_anime_search_query_structure() {
    assert!(ANIME_SEARCH_QUERY.contains("query"));
    assert!(ANIME_SEARCH_QUERY.contains("Page"));
    assert!(ANIME_SEARCH_QUERY.contains("$search"));
    assert!(ANIME_SEARCH_QUERY.contains("$perPage"));
    assert!(ANIME_SEARCH_QUERY.contains("search"));
    assert!(ANIME_SEARCH_QUERY.contains("perPage"));
    assert!(ANIME_SEARCH_QUERY.contains("media"));
    assert!(ANIME_SEARCH_QUERY.contains("pageInfo"));
}

#[test]
fn test_anime_search_advanced_query_structure() {
    assert!(ANIME_SEARCH_ADVANCED_QUERY.contains("query"));
    assert!(ANIME_SEARCH_ADVANCED_QUERY.contains("Page"));
    assert!(ANIME_SEARCH_ADVANCED_QUERY.contains("$genre"));
    assert!(ANIME_SEARCH_ADVANCED_QUERY.contains("$year"));
    assert!(ANIME_SEARCH_ADVANCED_QUERY.contains("$format"));
    assert!(ANIME_SEARCH_ADVANCED_QUERY.contains("$status"));
    assert!(ANIME_SEARCH_ADVANCED_QUERY.contains("genre_in"));
}

#[test]
fn test_seasonal_anime_query_structure() {
    assert!(SEASONAL_ANIME_QUERY.contains("query"));
    assert!(SEASONAL_ANIME_QUERY.contains("Page"));
    assert!(SEASONAL_ANIME_QUERY.contains("$season"));
    assert!(SEASONAL_ANIME_QUERY.contains("$seasonYear"));
    assert!(SEASONAL_ANIME_QUERY.contains("season"));
    assert!(SEASONAL_ANIME_QUERY.contains("seasonYear"));
}

#[test]
fn test_anime_characters_query_structure() {
    assert!(ANIME_CHARACTERS_QUERY.contains("query"));
    assert!(ANIME_CHARACTERS_QUERY.contains("Media"));
    assert!(ANIME_CHARACTERS_QUERY.contains("$id"));
    assert!(ANIME_CHARACTERS_QUERY.contains("characters"));
    assert!(ANIME_CHARACTERS_QUERY.contains("nodes"));
    assert!(ANIME_CHARACTERS_QUERY.contains("name"));
    assert!(ANIME_CHARACTERS_QUERY.contains("image"));
}

#[test]
fn test_anime_staff_query_structure() {
    assert!(ANIME_STAFF_QUERY.contains("query"));
    assert!(ANIME_STAFF_QUERY.contains("Media"));
    assert!(ANIME_STAFF_QUERY.contains("$id"));
    assert!(ANIME_STAFF_QUERY.contains("staff"));
    assert!(ANIME_STAFF_QUERY.contains("nodes"));
    assert!(ANIME_STAFF_QUERY.contains("name"));
    assert!(ANIME_STAFF_QUERY.contains("image"));
}

#[test]
fn test_anime_schedule_query_structure() {
    assert!(ANIME_SCHEDULE_QUERY.contains("query"));
    assert!(ANIME_SCHEDULE_QUERY.contains("Page"));
    assert!(ANIME_SCHEDULE_QUERY.contains("airingSchedules"));
    assert!(ANIME_SCHEDULE_QUERY.contains("$airingAt_greater"));
    assert!(ANIME_SCHEDULE_QUERY.contains("$airingAt_lesser"));
    assert!(ANIME_SCHEDULE_QUERY.contains("episode"));
    assert!(ANIME_SCHEDULE_QUERY.contains("airingAt"));
}

#[test]
fn test_anime_relations_query_structure() {
    assert!(ANIME_RELATIONS_QUERY.contains("query"));
    assert!(ANIME_RELATIONS_QUERY.contains("Media"));
    assert!(ANIME_RELATIONS_QUERY.contains("$id"));
    assert!(ANIME_RELATIONS_QUERY.contains("relations"));
    assert!(ANIME_RELATIONS_QUERY.contains("nodes"));
    assert!(ANIME_RELATIONS_QUERY.contains("relationType"));
}

#[test]
fn test_anime_recommendations_query_structure() {
    assert!(ANIME_RECOMMENDATIONS_QUERY.contains("query"));
    assert!(ANIME_RECOMMENDATIONS_QUERY.contains("Media"));
    assert!(ANIME_RECOMMENDATIONS_QUERY.contains("$id"));
    assert!(ANIME_RECOMMENDATIONS_QUERY.contains("recommendations"));
    assert!(ANIME_RECOMMENDATIONS_QUERY.contains("nodes"));
    assert!(ANIME_RECOMMENDATIONS_QUERY.contains("rating"));
    assert!(ANIME_RECOMMENDATIONS_QUERY.contains("mediaRecommendation"));
}

#[test]
fn test_anime_statistics_query_structure() {
    assert!(ANIME_STATISTICS_QUERY.contains("query"));
    assert!(ANIME_STATISTICS_QUERY.contains("Media"));
    assert!(ANIME_STATISTICS_QUERY.contains("$id"));
    assert!(ANIME_STATISTICS_QUERY.contains("stats"));
    assert!(ANIME_STATISTICS_QUERY.contains("scoreDistribution"));
    assert!(ANIME_STATISTICS_QUERY.contains("statusDistribution"));
}

#[test]
fn test_all_queries_are_valid_graphql() {
    let queries = [
        MEDIA_DETAIL_QUERY,
        ANIME_SEARCH_QUERY,
        ANIME_SEARCH_ADVANCED_QUERY,
        SEASONAL_ANIME_QUERY,
        ANIME_CHARACTERS_QUERY,
        ANIME_STAFF_QUERY,
        ANIME_SCHEDULE_QUERY,
        ANIME_RELATIONS_QUERY,
        ANIME_RECOMMENDATIONS_QUERY,
        ANIME_STATISTICS_QUERY,
    ];

    for query in queries.iter() {
        // Basic GraphQL syntax validation
        assert!(query.contains("query"));
        assert!(query.contains("{"));
        assert!(query.contains("}"));
        // Ensure proper bracket balancing (basic check)
        let open_count = query.matches("{").count();
        let close_count = query.matches("}").count();
        assert_eq!(open_count, close_count, "Unbalanced brackets in query");
    }
}

#[test]
fn test_query_variables() {
    // Test that queries properly define their variables
    assert!(MEDIA_DETAIL_QUERY.contains("$id: Int"));
    assert!(ANIME_SEARCH_QUERY.contains("$search: String"));
    assert!(ANIME_SEARCH_QUERY.contains("$perPage: Int"));
    assert!(SEASONAL_ANIME_QUERY.contains("$season: MediaSeason"));
    assert!(SEASONAL_ANIME_QUERY.contains("$seasonYear: Int"));
    assert!(ANIME_CHARACTERS_QUERY.contains("$perPage: Int"));
    assert!(ANIME_STAFF_QUERY.contains("$perPage: Int"));
}

#[test]
fn test_query_complexity() {
    // Ensure queries aren't too simple (have reasonable field selection)
    for query in [
        MEDIA_DETAIL_QUERY,
        ANIME_SEARCH_QUERY,
        ANIME_CHARACTERS_QUERY,
        ANIME_STAFF_QUERY,
    ]
    .iter()
    {
        // Should have multiple fields selected
        let field_count = query.matches("\n").count();
        assert!(field_count > 10, "Query seems too simple: {}", query);
    }
}
