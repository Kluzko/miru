/// Diagnostic test to verify age rating data is flowing correctly
use miru_lib::modules::provider::infrastructure::adapters::{
    AniListAdapter, JikanAdapter, ProviderAdapter,
};

#[tokio::test]
#[ignore] // Run with --ignored to test actual API calls
async fn test_danmachi_age_rating_from_jikan() {
    println!("\n=== Testing DanMachi Age Rating from Jikan ===\n");

    let adapter = JikanAdapter::new();

    // DanMachi MAL ID
    let anime_id = "28121";

    let result = adapter.get_anime_by_id(anime_id).await;

    match result {
        Ok(Some(anime)) => {
            println!("📺 Anime: {}", anime.anime.title.main);
            println!(
                "📋 Age restriction from Jikan: {:?}",
                anime.anime.age_restriction
            );

            if anime.anime.age_restriction.is_none() {
                println!("❌ PROBLEM: Age restriction is None from Jikan!");
                println!("   This means Jikan mapper might not be extracting it");
            } else {
                println!(
                    "✅ Age restriction found: {:?}",
                    anime.anime.age_restriction
                );
            }
        }
        Ok(None) => {
            println!("⚠️  Anime not found");
        }
        Err(e) => {
            println!("❌ API Error: {}", e);
        }
    }
}

#[tokio::test]
#[ignore] // Run with --ignored to test actual API calls
async fn test_danmachi_age_rating_from_anilist() {
    println!("\n=== Testing DanMachi Age Rating from AniList ===\n");

    let adapter = AniListAdapter::new();

    // DanMachi AniList ID
    let anime_id = "20920";

    let result = adapter.get_anime_by_id(anime_id).await;

    match result {
        Ok(Some(anime)) => {
            println!("📺 Anime: {}", anime.anime.title.main);
            println!(
                "📋 Age restriction from AniList: {:?}",
                anime.anime.age_restriction
            );

            if anime.anime.age_restriction.is_none() {
                println!("⚠️  Age restriction is None from AniList (expected - AniList doesn't provide this)");
            } else {
                println!(
                    "✅ Age restriction found: {:?}",
                    anime.anime.age_restriction
                );
            }
        }
        Ok(None) => {
            println!("⚠️  Anime not found");
        }
        Err(e) => {
            println!("❌ API Error: {}", e);
        }
    }
}

#[tokio::test]
#[ignore] // Run with --ignored to test actual API calls
async fn test_multiple_anime_age_ratings() {
    println!("\n=== Testing Age Ratings for Multiple Anime ===\n");

    let jikan = JikanAdapter::new();

    // Test several anime with different ratings
    let test_cases = vec![
        ("28121", "DanMachi"),
        ("16498", "Attack on Titan"),
        ("1", "Cowboy Bebop"),
        ("11061", "Hunter x Hunter"),
    ];

    for (id, name) in test_cases {
        match jikan.get_anime_by_id(id).await {
            Ok(Some(anime)) => {
                let rating_info = if anime.anime.age_restriction.is_some() {
                    format!("✅ {:?}", anime.anime.age_restriction.unwrap())
                } else {
                    "❌ NO RATING".to_string()
                };
                println!("{:30} - {}", name, rating_info);
            }
            Ok(None) => println!("{:30} - ⚠️  Not found", name),
            Err(e) => println!("{:30} - ❌ Error: {}", name, e),
        }

        // Small delay to avoid rate limiting
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    }
}
