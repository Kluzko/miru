use miru_lib::modules::provider::{
    infrastructure::adapters::{ProviderAdapter, TmdbAdapter},
    AnimeProvider,
};

const TEST_API_KEY: &str = "14a032abcf1763ff7568aaf97994df89";

#[test]
fn test_adapter_creation() {
    let adapter = TmdbAdapter::new(TEST_API_KEY.to_string());
    assert_eq!(adapter.get_provider_type(), AnimeProvider::TMDB);
}

#[test]
fn test_can_make_request() {
    let adapter = TmdbAdapter::new(TEST_API_KEY.to_string());
    let _can_request = adapter.can_make_request_now();
}

#[test]
fn test_multiple_adapters() {
    let adapter1 = TmdbAdapter::new(TEST_API_KEY.to_string());
    let adapter2 = TmdbAdapter::new(TEST_API_KEY.to_string());

    assert_eq!(adapter1.get_provider_type(), adapter2.get_provider_type());
}

#[test]
fn test_adapter_consistency() {
    let adapter1 = TmdbAdapter::new(TEST_API_KEY.to_string());
    let adapter2 = TmdbAdapter::new(TEST_API_KEY.to_string());

    assert_eq!(adapter1.get_provider_type(), adapter2.get_provider_type());
}

#[tokio::test]
async fn test_async_operations() {
    let adapter = TmdbAdapter::new(TEST_API_KEY.to_string());
    assert_eq!(adapter.get_provider_type(), AnimeProvider::TMDB);

    for _ in 0..5 {
        let _can_request = adapter.can_make_request_now();
    }
}

#[test]
fn test_build_image_url() {
    let adapter = TmdbAdapter::new(TEST_API_KEY.to_string());

    let url = adapter.build_image_url("/test.jpg", "original");
    assert_eq!(url, "https://image.tmdb.org/t/p/original/test.jpg");

    let url_w500 = adapter.build_image_url("/test.jpg", "w500");
    assert_eq!(url_w500, "https://image.tmdb.org/t/p/w500/test.jpg");
}

#[test]
fn test_different_api_keys() {
    let key1 = "key1".to_string();
    let key2 = "key2".to_string();

    let adapter1 = TmdbAdapter::new(key1);
    let adapter2 = TmdbAdapter::new(key2);

    assert_eq!(adapter1.get_provider_type(), adapter2.get_provider_type());
}
