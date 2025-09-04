use crate::domain::entities::Anime;
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
    total_titles: f32,
    max_members: f32,
    global_rating_mean: f32,
    vote_prior: f32,
    confidence_z: f32,
    max_members_per_day: f32,
    max_favorites_per_day: f32,
    max_votes_per_day: f32,
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
                total_titles: 20000.0,
                max_members: 5_000_000.0,
                global_rating_mean: 7.1,
                vote_prior: 10_000.0,
                confidence_z: 1.96,
                max_members_per_day: 50_000.0,
                max_favorites_per_day: 5_000.0,
                max_votes_per_day: 10_000.0,
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

    pub fn calculate_composite_score(&self, anime: &Anime) -> f32 {
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

    pub fn calculate_quality_metrics(&self, anime: &Anime) -> QualityMetrics {
        QualityMetrics {
            popularity_score: self.popularity_score(anime).unwrap_or(0.0),
            engagement_score: self.engagement_score(anime).unwrap_or(0.0),
            consistency_score: anime.score.unwrap_or(0.0),
            audience_reach_score: self.audience_reach_score(anime).unwrap_or(0.0),
        }
    }

    pub fn determine_tier(&self, score: f32) -> AnimeTier {
        AnimeTier::new(score)
    }

    fn bayesian_score(&self, anime: &Anime) -> Option<f32> {
        let r = anime.score?;
        let v = anime.scored_by? as f32;
        if v <= 0.0 {
            return None;
        }

        let c = self.context.global_rating_mean;
        let m = self.context.vote_prior;
        let b = (v / (v + m)) * r + (m / (v + m)) * c;
        Some(self.clamp(b, 0.0, 10.0))
    }

    fn popularity_score(&self, anime: &Anime) -> Option<f32> {
        if let Some(popularity) = anime.popularity {
            let rank = self.clamp(popularity as f32, 1.0, self.context.total_titles);
            let pct = 1.0 - (rank - 1.0) / (self.context.total_titles - 1.0);
            return Some(self.clamp(pct * 10.0, 0.0, 10.0));
        }

        if let Some(members) = anime.members {
            if members > 0 {
                return Some(self.normalize_log(members as f32, self.context.max_members) * 10.0);
            }
        }

        None
    }

    fn audience_reach_score(&self, anime: &Anime) -> Option<f32> {
        let members = anime.members? as f32;
        if members <= 0.0 {
            return None;
        }
        Some(self.normalize_log(members, self.context.max_members) * 10.0)
    }

    fn favorites_intensity(&self, anime: &Anime) -> Option<f32> {
        let f = anime.favorites? as f32;
        let n = anime.members? as f32;
        if n <= 0.0 {
            return None;
        }

        let lb = self.wilson_lower_bound(f, n, self.context.confidence_z);
        Some(self.clamp((lb / 0.05) * 10.0, 0.0, 10.0))
    }

    fn engagement_score(&self, anime: &Anime) -> Option<f32> {
        let v = anime.scored_by? as f32;
        let n = anime.members? as f32;
        if n <= 0.0 {
            return None;
        }

        let lb = self.wilson_lower_bound(v, n, self.context.confidence_z);
        Some(self.clamp(lb * 10.0, 0.0, 10.0))
    }

    fn momentum_score(&self, anime: &Anime) -> Option<f32> {
        let from = anime.aired.from?;
        let days = (chrono::Utc::now() - from).num_days() as f32;

        if days < 0.0 || days > 365.0 {
            return None;
        }

        let days = days.max(1.0);
        let m = anime.members.unwrap_or(0) as f32;
        let f = anime.favorites.unwrap_or(0) as f32;
        let v = anime.scored_by.unwrap_or(0) as f32;

        let members_per_day = m / days;
        let favorites_per_day = f / days;
        let votes_per_day = v / days;

        let mr = self.normalize_log(members_per_day, self.context.max_members_per_day);
        let fr = self.normalize_log(favorites_per_day, self.context.max_favorites_per_day);
        let vr = self.normalize_log(votes_per_day, self.context.max_votes_per_day);

        let score01 = 0.5 * mr + 0.3 * fr + 0.2 * vr;
        Some(self.clamp(score01 * 10.0, 0.0, 10.0))
    }

    fn momentum_weight(&self, anime: &Anime) -> f32 {
        let from = match anime.aired.from {
            Some(date) => date,
            None => return 0.0,
        };

        let days = (chrono::Utc::now() - from).num_days() as f32;
        if days > 365.0 || days < 0.0 {
            return 0.0;
        }

        let freshness = self.half_life_decay(days, self.context.recency_half_life_days);
        let votes = anime.scored_by.unwrap_or(0) as f32;
        let reliability = self.clamp(votes / self.context.vote_reliability_threshold, 0.0, 1.0);

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

    fn wilson_lower_bound(&self, successes: f32, n: f32, z: f32) -> f32 {
        if n <= 0.0 {
            return 0.0;
        }
        let p = self.clamp(successes / n, 0.0, 1.0);
        let z2 = z * z;
        let denom = 1.0 + z2 / n;
        let center = p + z2 / (2.0 * n);
        let margin = z * ((p * (1.0 - p) + z2 / (4.0 * n)) / n).sqrt();
        self.clamp((center - margin) / denom, 0.0, 1.0)
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
