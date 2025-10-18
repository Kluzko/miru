pub mod create_anime;
pub mod discover_relations;
pub mod search_anime;
pub mod update_anime_score;

pub use create_anime::{CreateAnimeCommand, CreateAnimeHandler, CreateAnimeResult};
pub use discover_relations::{
    DiscoverRelationsCommand, DiscoverRelationsHandler, DiscoverRelationsResult,
};
pub use search_anime::{SearchAnimeHandler, SearchAnimeQuery, SearchAnimeResult};
pub use update_anime_score::{
    UpdateAnimeScoreCommand, UpdateAnimeScoreHandler, UpdateAnimeScoreResult,
};
