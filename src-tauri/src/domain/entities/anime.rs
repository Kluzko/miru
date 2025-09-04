use super::genre::Genre;
use crate::domain::value_objects::{AnimeStatus, AnimeTier, AnimeType, QualityMetrics};
use chrono::{DateTime, Datelike, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Anime {
    pub id: Uuid,
    pub mal_id: Option<i32>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiredDates {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}

impl Anime {
    pub fn new(title: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            mal_id: None,
            title,
            title_english: None,
            title_japanese: None,
            score: None,
            scored_by: None,
            rank: None,
            popularity: None,
            members: None,
            favorites: None,
            synopsis: None,
            episodes: None,
            status: AnimeStatus::Unknown,
            aired: AiredDates {
                from: None,
                to: None,
            },
            anime_type: AnimeType::Unknown,
            rating: None,
            genres: Vec::new(),
            studios: Vec::new(),
            source: None,
            duration: None,
            image_url: None,
            mal_url: None,
            composite_score: 0.0,
            tier: AnimeTier::default(),
            quality_metrics: QualityMetrics::default(),
        }
    }

    pub fn is_airing(&self) -> bool {
        matches!(self.status, AnimeStatus::Airing)
    }

    pub fn is_finished(&self) -> bool {
        matches!(self.status, AnimeStatus::Finished)
    }

    pub fn has_aired(&self) -> bool {
        !matches!(self.status, AnimeStatus::NotYetAired)
    }

    pub fn days_since_aired(&self) -> Option<i64> {
        self.aired.from.map(|date| (Utc::now() - date).num_days())
    }

    pub fn is_recent(&self, days_threshold: i64) -> bool {
        self.days_since_aired()
            .map(|days| days >= 0 && days <= days_threshold)
            .unwrap_or(false)
    }

    pub fn get_year(&self) -> Option<i32> {
        self.aired.from.map(|date| date.year())
    }

    pub fn is_highly_rated(&self) -> bool {
        self.composite_score >= 7.5
    }

    pub fn is_popular(&self) -> bool {
        self.members.unwrap_or(0) > 100_000
    }

    pub fn update_scores(&mut self, calculator: &crate::domain::services::ScoreCalculator) {
        self.composite_score = calculator.calculate_composite_score(self);
        self.quality_metrics = calculator.calculate_quality_metrics(self);
        self.tier = calculator.determine_tier(self.composite_score);
    }
}
