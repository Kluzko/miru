use crate::domain::traits::scoreable::Scoreable;
use crate::domain::value_objects::{AnimeTier, QualityMetrics};

#[derive(Debug, Clone)]
pub struct ScoreCalculator {
    base_weights: ScoreWeights,
    max_momentum_weight: f32,
    context: NormalizationContext,
}

#[derive(Debug, Clone)]
struct ScoreWeights {
    bayesian_score: f32,
    popularity: f32,
    audience_reach: f32,
    favorites_intensity: f32,
    engagement: f32,
}

#[derive(Debug, Clone)]
struct NormalizationContext {
    max_members: f32,
    global_rating_mean: f32,
    vote_prior: f32,
    max_favorites_per_day: f32,
    recency_half_life_days: f32,
    vote_reliability_threshold: f32,
}

impl Default for ScoreCalculator {
    fn default() -> Self {
        Self {
            base_weights: ScoreWeights {
                bayesian_score: 0.45,
                popularity: 0.15,
                audience_reach: 0.15,
                favorites_intensity: 0.15,
                engagement: 0.1,
            },
            max_momentum_weight: 0.2,
            context: NormalizationContext {
                max_members: 5_000_000.0,
                global_rating_mean: 7.1,
                vote_prior: 10_000.0,
                max_favorites_per_day: 5_000.0,
                recency_half_life_days: 90.0,
                vote_reliability_threshold: 25_000.0,
            },
        }
    }
}

impl ScoreCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn calculate_composite_score(&self, anime: &dyn Scoreable) -> f32 {
        let mut parts = Vec::new();

        if let Some(score) = self.bayesian_score(anime) {
            parts.push((score, self.base_weights.bayesian_score));
        }

        if let Some(score) = self.popularity_score(anime) {
            parts.push((score, self.base_weights.popularity));
        }

        if let Some(score) = self.audience_reach_score(anime) {
            parts.push((score, self.base_weights.audience_reach));
        }

        if let Some(score) = self.favorites_intensity(anime) {
            parts.push((score, self.base_weights.favorites_intensity));
        }

        if let Some(score) = self.engagement_score(anime) {
            parts.push((score, self.base_weights.engagement));
        }

        if let Some(momentum) = self.momentum_score(anime) {
            let momentum_weight = self.momentum_weight(anime);
            if momentum_weight > 0.0 {
                parts.push((momentum, momentum_weight));
            }
        }

        if parts.is_empty() {
            return 0.0;
        }

        let weighted_sum: f32 = parts.iter().map(|(v, w)| v * w).sum();
        let weight_sum: f32 = parts.iter().map(|(_, w)| w).sum();

        (weighted_sum / weight_sum * 100.0).round() / 100.0
    }

    pub fn calculate_quality_metrics(&self, anime: &dyn Scoreable) -> QualityMetrics {
        QualityMetrics {
            popularity_score: self.popularity_score(anime).unwrap_or(0.0),
            engagement_score: self.engagement_score(anime).unwrap_or(0.0),
            consistency_score: anime.score().unwrap_or(0.0),
            audience_reach_score: self.audience_reach_score(anime).unwrap_or(0.0),
        }
    }

    pub fn determine_tier(&self, score: f32) -> AnimeTier {
        AnimeTier::from_score(score)
    }

    fn bayesian_score(&self, anime: &dyn Scoreable) -> Option<f32> {
        let r = anime.score()?;
        let v = anime.scored_by()? as f32;
        if v <= 0.0 {
            return None;
        }

        let c = self.context.global_rating_mean;
        let m = self.context.vote_prior;
        let b = (v / (v + m)) * r + (m / (v + m)) * c;
        Some(self.clamp(b, 0.0, 10.0))
    }

    fn popularity_score(&self, anime: &dyn Scoreable) -> Option<f32> {
        // Use internal calculation based on intrinsic factors
        // No longer dependent on external API popularity rankings

        let recency_score = self.recency_score(anime).unwrap_or(0.5);
        let quality_score = anime.score().unwrap_or(5.0) / 10.0;
        let engagement_score = self.engagement_score(anime).unwrap_or(0.3);

        // Internal popularity based on recency, quality, and engagement
        let internal_popularity =
            (recency_score * 0.3 + quality_score * 0.5 + engagement_score * 0.2) * 10.0;
        Some(self.clamp(internal_popularity, 0.0, 10.0))
    }

    fn audience_reach_score(&self, anime: &dyn Scoreable) -> Option<f32> {
        // Use favorites as proxy for audience reach since we removed members
        let favorites = anime.favorites()? as f32;
        if favorites <= 0.0 {
            return None;
        }
        // Scale favorites to represent audience reach
        Some(self.normalize_log(favorites, self.context.max_members * 0.1) * 10.0)
    }

    fn favorites_intensity(&self, anime: &dyn Scoreable) -> Option<f32> {
        let f = anime.favorites()? as f32;
        // Use favorites relative to expected baseline instead of members ratio
        let baseline_favorites = 100.0; // Expected favorites for average anime
        let intensity_ratio = f / baseline_favorites;

        // Apply logarithmic scaling to handle wide range of favorites
        Some(self.clamp(self.normalize_log(intensity_ratio, 100.0) * 10.0, 0.0, 10.0))
    }

    fn engagement_score(&self, anime: &dyn Scoreable) -> Option<f32> {
        // Use favorites directly as engagement metric (scored_by maps to favorites now)
        let engagement = anime.scored_by()? as f32; // This maps to favorites in our unified system
        if engagement <= 0.0 {
            return None;
        }

        // Normalize engagement relative to expected baseline
        let baseline_engagement = 1000.0; // Expected engagement for average anime
        let engagement_ratio = engagement / baseline_engagement;

        Some(self.clamp(self.normalize_log(engagement_ratio, 10.0) * 10.0, 0.0, 10.0))
    }

    /// Calculate recency score based on air date
    fn recency_score(&self, anime: &dyn Scoreable) -> Option<f32> {
        let from = anime.aired_from()?;
        let days_since_aired = (chrono::Utc::now() - from).num_days() as f32;

        // Newer anime get higher scores, with decay over time
        let years_since_aired = days_since_aired / 365.25;
        let recency_score = (-years_since_aired / 5.0).exp(); // Decay over 5 years

        Some(self.clamp(recency_score, 0.0, 1.0))
    }

    fn momentum_score(&self, anime: &dyn Scoreable) -> Option<f32> {
        let from = anime.aired_from()?;
        let days = (chrono::Utc::now() - from).num_days() as f32;

        if days < 0.0 || days > 365.0 {
            return None;
        }

        let days = days.max(1.0);
        let f = anime.favorites().unwrap_or(0) as f32;
        let score = anime.score().unwrap_or(5.0);

        // Calculate momentum based on favorites per day and quality
        let favorites_per_day = f / days;
        let quality_factor = score / 10.0;

        let fr = self.normalize_log(favorites_per_day, self.context.max_favorites_per_day);

        // Weight favorites momentum higher since we don't have members/votes
        let momentum = 0.7 * fr + 0.3 * quality_factor;
        Some(self.clamp(momentum * 10.0, 0.0, 10.0))
    }

    fn momentum_weight(&self, anime: &dyn Scoreable) -> f32 {
        let from = match anime.aired_from() {
            Some(date) => date,
            None => return 0.0,
        };

        let days = (chrono::Utc::now() - from).num_days() as f32;
        if days > 365.0 || days < 0.0 {
            return 0.0;
        }

        let freshness = self.half_life_decay(days, self.context.recency_half_life_days);
        let engagement = anime.scored_by().unwrap_or(0) as f32; // Maps to favorites in unified system
        let reliability = self.clamp(
            engagement / (self.context.vote_reliability_threshold * 0.1),
            0.0,
            1.0,
        );

        self.max_momentum_weight * freshness * (1.0 - reliability)
    }

    fn normalize_log(&self, x: f32, xmax: f32) -> f32 {
        if x <= 0.0 {
            return 0.0;
        }
        let denom = (1.0 + xmax.max(1.0)).ln();
        if denom > 0.0 {
            (1.0 + x).ln() / denom
        } else {
            0.0
        }
    }

    fn half_life_decay(&self, days: f32, half_life: f32) -> f32 {
        if half_life <= 0.0 {
            return 0.0;
        }
        0.5_f32.powf(days / half_life)
    }

    fn clamp(&self, x: f32, min: f32, max: f32) -> f32 {
        x.max(min).min(max)
    }
}
