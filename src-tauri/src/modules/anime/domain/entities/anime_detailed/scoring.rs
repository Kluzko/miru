use chrono::Utc;

use super::AnimeDetailed;

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
