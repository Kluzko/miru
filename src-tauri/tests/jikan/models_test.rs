use serde_json;

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq)]
struct TestAnime {
    mal_id: u32,
    title: String,
    episodes: Option<u32>,
    score: Option<f32>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct TestResponse<T> {
    data: T,
}

#[test]
fn test_anime_deserialization() {
    let json = r#"{"mal_id": 1, "title": "Test", "episodes": 26, "score": 8.75}"#;
    let anime: TestAnime = serde_json::from_str(json).unwrap();
    assert_eq!(anime.mal_id, 1);
    assert_eq!(anime.title, "Test");
}

#[test]
fn test_optional_fields() {
    let json = r#"{"mal_id": 2, "title": "Test", "episodes": null, "score": null}"#;
    let anime: TestAnime = serde_json::from_str(json).unwrap();
    assert!(anime.episodes.is_none());
    assert!(anime.score.is_none());
}

#[test]
fn test_response_wrapper() {
    let json = r#"{"data": {"mal_id": 3, "title": "Wrapped", "episodes": 12, "score": 7.5}}"#;
    let response: TestResponse<TestAnime> = serde_json::from_str(json).unwrap();
    assert_eq!(response.data.mal_id, 3);
}

#[test]
fn test_array_response() {
    let json = r#"{"data": [{"mal_id": 4, "title": "A1", "episodes": 24, "score": 8.0}]}"#;
    let response: TestResponse<Vec<TestAnime>> = serde_json::from_str(json).unwrap();
    assert_eq!(response.data.len(), 1);
}

#[test]
fn test_serialization() {
    let anime = TestAnime {
        mal_id: 6,
        title: "Serialized".to_string(),
        episodes: Some(13),
        score: Some(8.5),
    };
    let json = serde_json::to_string(&anime).unwrap();
    let deserialized: TestAnime = serde_json::from_str(&json).unwrap();
    assert_eq!(anime.mal_id, deserialized.mal_id);
}

#[test]
fn test_invalid_json() {
    let invalid_json = r#"{"mal_id": "invalid", "title": "Test"}"#;
    let result = serde_json::from_str::<TestAnime>(invalid_json);
    assert!(result.is_err());
}

#[test]
fn test_missing_fields() {
    let json = r#"{"episodes": 26, "score": 8.75}"#;
    let result = serde_json::from_str::<TestAnime>(json);
    assert!(result.is_err());
}
