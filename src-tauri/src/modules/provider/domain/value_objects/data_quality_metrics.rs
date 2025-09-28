use serde::{Deserialize, Serialize};
use specta::Type;

/// Quality metrics for anime data assessment
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct QualityMetrics {
    /// Overall quality score (0.0 to 1.0)
    pub overall_score: f32,
    /// Completeness score (0.0 to 1.0)
    pub completeness: f32,
    /// Consistency score (0.0 to 1.0)
    pub consistency: f32,
    /// Accuracy score based on cross-provider validation
    pub accuracy: f32,
    /// Fields that are missing or incomplete
    pub missing_fields: Vec<String>,
}

impl QualityMetrics {
    pub fn new() -> Self {
        Self {
            overall_score: 0.0,
            completeness: 0.0,
            consistency: 0.0,
            accuracy: 0.0,
            missing_fields: Vec::new(),
        }
    }

    /// Calculate overall score based on individual metrics
    pub fn calculate_overall_score(&mut self) {
        self.overall_score =
            (self.completeness * 0.4 + self.consistency * 0.3 + self.accuracy * 0.3)
                .min(1.0)
                .max(0.0);
    }

    /// Check if quality meets minimum standards
    pub fn meets_standards(&self, threshold: f32) -> bool {
        self.overall_score >= threshold && self.completeness >= 0.6
    }

    /// Get human-readable quality assessment
    pub fn quality_grade(&self) -> QualityGrade {
        match self.overall_score {
            s if s >= 0.9 => QualityGrade::Excellent,
            s if s >= 0.8 => QualityGrade::Good,
            s if s >= 0.7 => QualityGrade::Fair,
            s if s >= 0.6 => QualityGrade::Poor,
            _ => QualityGrade::VeryPoor,
        }
    }
}

/// Quality grade enumeration
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum QualityGrade {
    Excellent,
    Good,
    Fair,
    Poor,
    VeryPoor,
}

impl std::fmt::Display for QualityGrade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QualityGrade::Excellent => write!(f, "Excellent"),
            QualityGrade::Good => write!(f, "Good"),
            QualityGrade::Fair => write!(f, "Fair"),
            QualityGrade::Poor => write!(f, "Poor"),
            QualityGrade::VeryPoor => write!(f, "Very Poor"),
        }
    }
}

impl Default for QualityMetrics {
    fn default() -> Self {
        Self::new()
    }
}
