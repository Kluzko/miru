//! User anime list statistics

use serde::{Deserialize, Serialize};
use specta::Type;

use super::{UserAnimeData, WatchingStatus};

/// User's anime list statistics
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UserAnimeStats {
    pub total_anime: i32,
    pub completed: i32,
    pub watching: i32,
    pub plan_to_watch: i32,
    pub on_hold: i32,
    pub dropped: i32,
    pub rewatching: i32,
    pub total_episodes: i32,
    pub total_watch_time_minutes: i32,
    pub mean_score: Option<f32>,
    pub favorites_count: i32,
}

impl UserAnimeStats {
    pub fn calculate_from_data(user_data: &[UserAnimeData]) -> Self {
        let total_anime = user_data.len() as i32;
        let completed = user_data
            .iter()
            .filter(|d| d.status == WatchingStatus::Completed)
            .count() as i32;
        let watching = user_data
            .iter()
            .filter(|d| d.status == WatchingStatus::Watching)
            .count() as i32;
        let plan_to_watch = user_data
            .iter()
            .filter(|d| d.status == WatchingStatus::PlanToWatch)
            .count() as i32;
        let on_hold = user_data
            .iter()
            .filter(|d| d.status == WatchingStatus::OnHold)
            .count() as i32;
        let dropped = user_data
            .iter()
            .filter(|d| d.status == WatchingStatus::Dropped)
            .count() as i32;
        let rewatching = user_data
            .iter()
            .filter(|d| d.status == WatchingStatus::Rewatching)
            .count() as i32;

        let total_episodes = user_data.iter().map(|d| d.episodes_watched).sum();
        let favorites_count = user_data.iter().filter(|d| d.is_favorite).count() as i32;

        // Calculate mean score from rated anime
        let rated_anime: Vec<f32> = user_data.iter().filter_map(|d| d.personal_rating).collect();

        let mean_score = if !rated_anime.is_empty() {
            Some(rated_anime.iter().sum::<f32>() / rated_anime.len() as f32)
        } else {
            None
        };

        Self {
            total_anime,
            completed,
            watching,
            plan_to_watch,
            on_hold,
            dropped,
            rewatching,
            total_episodes,
            total_watch_time_minutes: total_episodes * 24, // Estimate 24 min per episode
            mean_score,
            favorites_count,
        }
    }
}
