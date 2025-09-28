use serde_json;

use miru_lib::modules::provider::infrastructure::adapters::anilist::models::*;

#[test]
fn test_media_deserialization() {
    let json = r#"{
        "id": 1,
        "idMal": 1,
        "title": {
            "romaji": "Test Anime",
            "english": "Test Anime English",
            "native": "テストアニメ"
        },
        "description": "Test description",
        "type": "ANIME",
        "format": "TV",
        "status": "FINISHED",
        "episodes": 26,
        "averageScore": 87
    }"#;

    let media: Media = serde_json::from_str(json).unwrap();
    assert_eq!(media.id, Some(1));
    assert_eq!(media.id_mal, Some(1));
    assert_eq!(media.episodes, Some(26));
    assert_eq!(media.average_score, Some(87));
}

#[test]
fn test_optional_fields() {
    let json = r#"{
        "id": 2,
        "title": {
            "romaji": "Test Anime 2"
        },
        "type": "ANIME",
        "episodes": null,
        "averageScore": null
    }"#;

    let media: Media = serde_json::from_str(json).unwrap();
    assert_eq!(media.id, Some(2));
    assert!(media.episodes.is_none());
    assert!(media.average_score.is_none());
}

#[test]
fn test_search_response() {
    let json = r#"{
        "Page": {
            "media": [{
                "id": 3,
                "title": {
                    "romaji": "Search Result"
                },
                "type": "ANIME"
            }],
            "pageInfo": {
                "total": 1,
                "perPage": 10,
                "currentPage": 1,
                "lastPage": 1,
                "hasNextPage": false
            }
        }
    }"#;

    let response: AniListSearchResponse = serde_json::from_str(json).unwrap();
    assert_eq!(response.page.media.len(), 1);
    assert_eq!(response.page.media[0].id, Some(3));

    let page_info = response.page.page_info.unwrap();
    assert_eq!(page_info.total, Some(1));
    assert_eq!(page_info.has_next_page, Some(false));
}

#[test]
fn test_media_response() {
    let json = r#"{
        "Media": {
            "id": 4,
            "title": {
                "romaji": "Single Media"
            },
            "type": "ANIME"
        }
    }"#;

    let response: AniListMediaResponse = serde_json::from_str(json).unwrap();
    assert!(response.media.is_some());
    assert_eq!(response.media.unwrap().id, Some(4));
}

#[test]
fn test_character_response() {
    let json = r#"{
        "Media": {
            "characters": {
                "nodes": [{
                    "id": 5,
                    "name": {
                        "full": "Test Character",
                        "native": "テストキャラクター"
                    },
                    "image": {
                        "large": "https://example.com/large.jpg",
                        "medium": "https://example.com/medium.jpg"
                    }
                }]
            }
        }
    }"#;

    let response: AniListCharactersResponse = serde_json::from_str(json).unwrap();
    let characters = &response.media.unwrap().characters.nodes;
    assert_eq!(characters.len(), 1);
    assert_eq!(characters[0].id, Some(5));
    assert_eq!(
        characters[0].name.as_ref().unwrap().full,
        Some("Test Character".to_string())
    );
}

#[test]
fn test_staff_response() {
    let json = r#"{
        "Media": {
            "staff": {
                "nodes": [{
                    "id": 6,
                    "name": {
                        "full": "Test Staff",
                        "native": "テストスタッフ"
                    },
                    "image": {
                        "large": "https://example.com/staff_large.jpg",
                        "medium": "https://example.com/staff_medium.jpg"
                    }
                }]
            }
        }
    }"#;

    let response: AniListStaffResponse = serde_json::from_str(json).unwrap();
    let staff = &response.media.unwrap().staff.nodes;
    assert_eq!(staff.len(), 1);
    assert_eq!(staff[0].id, Some(6));
    assert_eq!(
        staff[0].name.as_ref().unwrap().full,
        Some("Test Staff".to_string())
    );
}

#[test]
fn test_fuzzy_date() {
    let json = r#"{
        "year": 2023,
        "month": 4,
        "day": 15
    }"#;

    let date: FuzzyDate = serde_json::from_str(json).unwrap();
    assert_eq!(date.year, Some(2023));
    assert_eq!(date.month, Some(4));
    assert_eq!(date.day, Some(15));
}

#[test]
fn test_media_enums() {
    let media_type: MediaType = serde_json::from_str("\"ANIME\"").unwrap();
    assert_eq!(media_type, MediaType::Anime);

    let format: MediaFormat = serde_json::from_str("\"TV\"").unwrap();
    assert_eq!(format, MediaFormat::Tv);

    let status: MediaStatus = serde_json::from_str("\"FINISHED\"").unwrap();
    assert_eq!(status, MediaStatus::Finished);

    let season: MediaSeason = serde_json::from_str("\"FALL\"").unwrap();
    assert_eq!(season, MediaSeason::Fall);
}

#[test]
fn test_serialization() {
    let media = Media {
        id: Some(7),
        id_mal: Some(7),
        title: Some(MediaTitle {
            romaji: Some("Test Serialization".to_string()),
            english: None,
            native: None,
        }),
        episodes: Some(12),
        average_score: Some(85),
        ..Default::default()
    };

    let json = serde_json::to_string(&media).unwrap();
    let deserialized: Media = serde_json::from_str(&json).unwrap();
    assert_eq!(media.id, deserialized.id);
    assert_eq!(media.episodes, deserialized.episodes);
}

#[test]
fn test_invalid_json() {
    let invalid_json = r#"{"id": "invalid", "type": "ANIME"}"#;
    let result = serde_json::from_str::<Media>(invalid_json);
    assert!(result.is_err());
}

#[test]
fn test_empty_response() {
    let json = r#"{
        "Page": {
            "media": [],
            "pageInfo": {
                "total": 0,
                "hasNextPage": false
            }
        }
    }"#;

    let response: AniListSearchResponse = serde_json::from_str(json).unwrap();
    assert_eq!(response.page.media.len(), 0);
}
