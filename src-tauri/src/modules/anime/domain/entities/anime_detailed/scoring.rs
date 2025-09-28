use chrono::Utc;

use super::AnimeDetailed;
use crate::modules::anime::domain::traits::Scoreable;

// Scoring and quality methods for AnimeDetailed
impl AnimeDetailed {
    /// Update scores using internal calculator
    pub fn update_scores(
        &mut self,
        calculator: &crate::modules::anime::domain::services::score_calculator::ScoreCalculator,
    ) {
        self.composite_score = calculator.calculate_composite_score(self);
        self.quality_metrics = calculator.calculate_quality_metrics(self);
        self.tier = calculator.determine_tier(self.composite_score);
        self.updated_at = Utc::now();
    }
}

// Implement Scoreable trait for AnimeDetailed
impl Scoreable for AnimeDetailed {
    fn score(&self) -> Option<f32> {
        self.score
    }

    fn scored_by(&self) -> Option<i32> {
        // Use favorites as a proxy for scored_by since that's what we have
        self.favorites.map(|f| f as i32)
    }

    fn popularity(&self) -> Option<i32> {
        // We don't have a direct popularity field, but can use favorites as proxy
        self.favorites.map(|f| f as i32)
    }

    fn members(&self) -> Option<i32> {
        // We don't have members field, use favorites as proxy
        self.favorites.map(|f| f as i32)
    }

    fn favorites(&self) -> Option<i32> {
        self.favorites.map(|f| f as i32)
    }

    fn aired_from(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.aired.from
    }
}
