use super::genre::Genre;
use crate::domain::value_objects::{AnimeStatus, AnimeTier, AnimeType, QualityMetrics};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Anime {
    pub id: Uuid,
    pub mal_id: i32,
    pub title: String,
    pub title_english: Option<String>,
    pub title_japanese: Option<String>,
    pub score: Option<f32>,
    pub scored_by: Option<i32>,
    pub rank: Option<i32>,
    pub popularity: Option<i32>,
    pub members: Option<i32>,
    pub favorites: Option<i32>,
    pub synopsis: Option<String>,
    pub episodes: Option<i32>,
    pub status: AnimeStatus,
    pub aired: AiredDates,
    pub anime_type: AnimeType,
    pub rating: Option<String>,
    pub genres: Vec<Genre>,
    pub studios: Vec<String>,
    pub source: Option<String>,
    pub duration: Option<String>,
    pub image_url: Option<String>,
    pub mal_url: Option<String>,
    pub composite_score: f32,
    pub tier: AnimeTier,
    pub quality_metrics: QualityMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AiredDates {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}

impl Anime {
    pub fn update_scores(&mut self, calculator: &crate::domain::services::ScoreCalculator) {
        self.composite_score = calculator.calculate_composite_score(self);
        self.quality_metrics = calculator.calculate_quality_metrics(self);
        self.tier = calculator.determine_tier(self.composite_score);
    }
}
