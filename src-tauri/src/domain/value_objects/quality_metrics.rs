use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct QualityMetrics {
    pub popularity_score: f32,
    pub engagement_score: f32,
    pub consistency_score: f32,
    pub audience_reach_score: f32,
}

impl QualityMetrics {
    pub fn new(
        popularity_score: f32,
        engagement_score: f32,
        consistency_score: f32,
        audience_reach_score: f32,
    ) -> Self {
        Self {
            popularity_score: Self::clamp_score(popularity_score),
            engagement_score: Self::clamp_score(engagement_score),
            consistency_score: Self::clamp_score(consistency_score),
            audience_reach_score: Self::clamp_score(audience_reach_score),
        }
    }

    pub fn average_score(&self) -> f32 {
        (self.popularity_score
            + self.engagement_score
            + self.consistency_score
            + self.audience_reach_score)
            / 4.0
    }

    pub fn is_high_quality(&self) -> bool {
        self.average_score() >= 7.0
    }

    pub fn strongest_metric(&self) -> (&str, f32) {
        let metrics = [
            ("popularity", self.popularity_score),
            ("engagement", self.engagement_score),
            ("consistency", self.consistency_score),
            ("audience_reach", self.audience_reach_score),
        ];

        metrics
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap()
    }

    pub fn weakest_metric(&self) -> (&str, f32) {
        let metrics = [
            ("popularity", self.popularity_score),
            ("engagement", self.engagement_score),
            ("consistency", self.consistency_score),
            ("audience_reach", self.audience_reach_score),
        ];

        metrics
            .into_iter()
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap()
    }

    fn clamp_score(score: f32) -> f32 {
        score.max(0.0).min(10.0)
    }
}

impl Default for QualityMetrics {
    fn default() -> Self {
        Self {
            popularity_score: 0.0,
            engagement_score: 0.0,
            consistency_score: 0.0,
            audience_reach_score: 0.0,
        }
    }
}
