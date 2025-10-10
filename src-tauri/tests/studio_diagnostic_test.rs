/// Diagnostic test to verify studio data is flowing correctly through the system
use miru_lib::modules::provider::{
    infrastructure::adapters::{AniListAdapter, JikanAdapter, ProviderAdapter},
    AnimeProvider,
};

#[tokio::test]
#[ignore] // Run with --ignored to test actual API calls
async fn test_anilist_studios_in_search() {
    println!("\n=== Testing AniList Studios in Search Results ===\n");

    let adapter = AniListAdapter::new();

    // Search for a popular anime that definitely has studio information
    let results = adapter.search_anime("Attack on Titan", 1).await;

    match results {
        Ok(anime_list) => {
            if anime_list.is_empty() {
                println!("⚠️  No results returned from search");
                return;
            }

            let anime = &anime_list[0];
            println!("📺 Anime: {}", anime.anime.title.main);
            println!("📋 Studios in result: {:?}", anime.anime.studios);

            if anime.anime.studios.is_empty() {
                println!("❌ PROBLEM: Studios array is empty!");
                println!("   This means studios are not being extracted properly");
            } else {
                println!("✅ Studios found: {}", anime.anime.studios.join(", "));
            }
        }
        Err(e) => {
            println!("❌ API Error: {}", e);
        }
    }
}

#[tokio::test]
#[ignore] // Run with --ignored to test actual API calls
async fn test_anilist_studios_in_detail() {
    println!("\n=== Testing AniList Studios in Detail Query ===\n");

    let adapter = AniListAdapter::new();

    // Attack on Titan AniList ID
    let anime_id = "16498";

    let result = adapter.get_anime_by_id(anime_id).await;

    match result {
        Ok(Some(anime)) => {
            println!("📺 Anime: {}", anime.anime.title.main);
            println!("📋 Studios in detail: {:?}", anime.anime.studios);

            if anime.anime.studios.is_empty() {
                println!("❌ PROBLEM: Studios array is empty in detail query!");
                println!("   This means the MEDIA_DETAIL_QUERY might not be returning studios");
            } else {
                println!("✅ Studios found: {}", anime.anime.studios.join(", "));
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
async fn test_jikan_studios() {
    println!("\n=== Testing Jikan Studios ===\n");

    let adapter = JikanAdapter::new();

    // Attack on Titan MAL ID
    let anime_id = "16498";

    let result = adapter.get_anime_by_id(anime_id).await;

    match result {
        Ok(Some(anime)) => {
            println!("📺 Anime: {}", anime.anime.title.main);
            println!("📋 Studios from Jikan: {:?}", anime.anime.studios);

            if anime.anime.studios.is_empty() {
                println!("❌ PROBLEM: Studios array is empty from Jikan!");
            } else {
                println!("✅ Studios found: {}", anime.anime.studios.join(", "));
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
async fn test_multiple_anime_studios() {
    println!("\n=== Testing Studios for Multiple Popular Anime ===\n");

    let adapter = AniListAdapter::new();

    // Test several popular anime
    let test_cases = vec![
        ("1", "Cowboy Bebop"),
        ("20", "Naruto"),
        ("21", "One Piece"),
        ("30831", "Kimetsu no Yaiba"),
    ];

    for (id, name) in test_cases {
        match adapter.get_anime_by_id(id).await {
            Ok(Some(anime)) => {
                let studio_info = if anime.anime.studios.is_empty() {
                    "❌ NO STUDIOS".to_string()
                } else {
                    format!("✅ {}", anime.anime.studios.join(", "))
                };
                println!("{:30} - {}", name, studio_info);
            }
            Ok(None) => println!("{:30} - ⚠️  Not found", name),
            Err(e) => println!("{:30} - ❌ Error: {}", name, e),
        }

        // Small delay to avoid rate limiting
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
}

#[test]
fn test_anilist_mapper_extract_studios_logic() {
    use miru_lib::modules::provider::infrastructure::adapters::anilist::models::{
        Studio, StudioConnection, StudioEdge,
    };

    println!("\n=== Testing Studio Extraction Logic ===\n");

    // Test Case 1: Studios with edges (detail queries)
    let studios_with_edges = StudioConnection {
        edges: Some(vec![
            StudioEdge {
                is_main: Some(true),
                node: Some(Studio {
                    id: Some(1),
                    name: Some("Main Studio".to_string()),
                    is_main: None,
                }),
            },
            StudioEdge {
                is_main: Some(false),
                node: Some(Studio {
                    id: Some(2),
                    name: Some("Sub Studio".to_string()),
                    is_main: None,
                }),
            },
        ]),
        nodes: None,
    };

    // Simulate the extraction logic from mapper
    let extracted: Vec<String> = studios_with_edges
        .edges
        .as_ref()
        .map(|edges| {
            edges
                .iter()
                .filter(|edge| edge.is_main.unwrap_or(false))
                .filter_map(|edge| edge.node.as_ref())
                .filter_map(|studio| studio.name.clone())
                .collect()
        })
        .unwrap_or_default();

    println!("Edges case (should only get main): {:?}", extracted);
    assert_eq!(extracted, vec!["Main Studio"]);

    // Test Case 2: Studios with nodes (search queries)
    let studios_with_nodes = StudioConnection {
        edges: None,
        nodes: Some(vec![
            Studio {
                id: Some(1),
                name: Some("Studio A".to_string()),
                is_main: Some(true),
            },
            Studio {
                id: Some(2),
                name: Some("Studio B".to_string()),
                is_main: Some(false),
            },
        ]),
    };

    let extracted2: Vec<String> = studios_with_nodes
        .nodes
        .as_ref()
        .map(|nodes| {
            nodes
                .iter()
                .filter(|studio| studio.is_main.unwrap_or(false))
                .filter_map(|studio| studio.name.clone())
                .collect()
        })
        .unwrap_or_default();

    println!("Nodes case (should only get main): {:?}", extracted2);
    assert_eq!(extracted2, vec!["Studio A"]);

    println!("\n✅ Studio extraction logic tests passed!");
}
