use chrono::{DateTime, Utc};

/// Trait for entities that can be scored by the ScoreCalculator
pub trait Scoreable {
    fn score(&self) -> Option<f32>;
    fn scored_by(&self) -> Option<i32>;
    fn popularity(&self) -> Option<i32>;
    fn members(&self) -> Option<i32>;
    fn favorites(&self) -> Option<i32>;
    fn aired_from(&self) -> Option<DateTime<Utc>>;
}
