use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use uuid::Uuid;

/// User's comprehensive rating for an anime
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UserRating {
    pub id: Uuid,
    pub anime_id: Uuid,
    pub user_id: String,

    /// Core rating fields
    pub overall_feeling: OverallFeeling,
    pub recommendation_preference: Option<RecommendationPreference>,

    /// Extensible rating criteria stored as JSON
    /// This allows users to add custom rating criteria
    pub rating_criteria: HashMap<String, RatingCriterion>,

    /// Free-form personal notes
    pub personal_notes: Option<String>,

    /// Tags that emotional impact this anime had
    pub emotional_tags: Vec<String>,

    /// Aspects that stood out (positive or negative)
    pub standout_aspects: Vec<String>,

    /// Whether user would rewatch
    pub would_rewatch: Option<bool>,

    /// Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Overall feeling about the anime (required field)
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
pub enum OverallFeeling {
    Loved,    // 5 stars equivalent
    Enjoyed,  // 4 stars
    Liked,    // 3 stars
    Okay,     // 2 stars
    Disliked, // 1 star
    Hated,    // 0 stars
}

impl OverallFeeling {
    /// Get numeric score (0-10 scale)
    pub fn to_score(&self) -> f32 {
        match self {
            Self::Loved => 10.0,
            Self::Enjoyed => 8.0,
            Self::Liked => 6.0,
            Self::Okay => 4.0,
            Self::Disliked => 2.0,
            Self::Hated => 0.0,
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Loved => "Loved it",
            Self::Enjoyed => "Enjoyed",
            Self::Liked => "Liked",
            Self::Okay => "Okay",
            Self::Disliked => "Disliked",
            Self::Hated => "Hated",
        }
    }

    /// Get color class for UI
    pub fn color_class(&self) -> &'static str {
        match self {
            Self::Loved => "text-emerald-600",
            Self::Enjoyed => "text-green-600",
            Self::Liked => "text-blue-600",
            Self::Okay => "text-yellow-600",
            Self::Disliked => "text-orange-600",
            Self::Hated => "text-red-600",
        }
    }

    /// Get emoji representation
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Loved => "😍",
            Self::Enjoyed => "😊",
            Self::Liked => "👍",
            Self::Okay => "😐",
            Self::Disliked => "👎",
            Self::Hated => "😤",
        }
    }
}

/// How this rating should affect recommendations
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
pub enum RecommendationPreference {
    /// Show me more like this
    Recommend,
    /// No preference either way
    Neutral,
    /// Avoid showing me similar anime
    Avoid,
}

impl RecommendationPreference {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Recommend => "Show similar",
            Self::Neutral => "No preference",
            Self::Avoid => "Avoid similar",
        }
    }

    /// Get weight for recommendation algorithm
    pub fn recommendation_weight(&self) -> f32 {
        match self {
            Self::Recommend => 1.0,
            Self::Neutral => 0.0,
            Self::Avoid => -1.0,
        }
    }
}

/// Flexible rating criterion that can be extended by users
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RatingCriterion {
    /// Type of rating criterion
    pub criterion_type: RatingCriterionType,
    /// The value/score for this criterion
    pub value: RatingValue,
    /// Optional weight for this criterion (default 1.0)
    pub weight: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum RatingCriterionType {
    /// Built-in criteria
    StoryQuality,
    AnimationQuality,
    MusicQuality,
    CharacterDevelopment,
    Pacing,
    WorldBuilding,
    EmotionalImpact,
    /// Custom user-defined criterion
    Custom(String),
}

impl RatingCriterionType {
    pub fn display_name(&self) -> String {
        match self {
            Self::StoryQuality => "Story Quality".to_string(),
            Self::AnimationQuality => "Animation Quality".to_string(),
            Self::MusicQuality => "Music Quality".to_string(),
            Self::CharacterDevelopment => "Character Development".to_string(),
            Self::Pacing => "Pacing".to_string(),
            Self::WorldBuilding => "World Building".to_string(),
            Self::EmotionalImpact => "Emotional Impact".to_string(),
            Self::Custom(name) => name.clone(),
        }
    }

    /// Get default criteria that can be suggested to users
    pub fn get_default_criteria() -> Vec<Self> {
        vec![
            Self::StoryQuality,
            Self::AnimationQuality,
            Self::MusicQuality,
            Self::CharacterDevelopment,
            Self::Pacing,
            Self::WorldBuilding,
            Self::EmotionalImpact,
        ]
    }
}

/// Flexible value type for rating criteria
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum RatingValue {
    /// Numeric score (0-100)
    Numeric(f32),
    /// Boolean value (yes/no, good/bad)
    Boolean(bool),
    /// Text value for qualitative ratings
    Text(String),
    /// Multiple choice from predefined options
    Choice(String),
    /// Multiple selections
    MultipleChoice(Vec<String>),
}

impl RatingValue {
    /// Convert to numeric score for calculations (0-10 scale)
    pub fn to_numeric_score(&self) -> Option<f32> {
        match self {
            Self::Numeric(score) => Some((*score / 100.0) * 10.0), // Convert 0-100 to 0-10
            Self::Boolean(true) => Some(10.0),
            Self::Boolean(false) => Some(0.0),
            Self::Choice(choice) => {
                // Could implement mapping for common choices
                match choice.to_lowercase().as_str() {
                    "excellent" | "outstanding" => Some(10.0),
                    "great" | "very good" => Some(8.0),
                    "good" => Some(6.0),
                    "average" | "okay" => Some(5.0),
                    "poor" | "bad" => Some(2.0),
                    "terrible" | "awful" => Some(0.0),
                    _ => None,
                }
            }
            _ => None, // Text and MultipleChoice need context to convert
        }
    }
}

impl UserRating {
    /// Create new user rating with minimal data
    pub fn new(anime_id: Uuid, user_id: String, overall_feeling: OverallFeeling) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            anime_id,
            user_id,
            overall_feeling,
            recommendation_preference: None,
            rating_criteria: HashMap::new(),
            personal_notes: None,
            emotional_tags: Vec::new(),
            standout_aspects: Vec::new(),
            would_rewatch: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Add or update a rating criterion
    pub fn add_criterion(&mut self, criterion_type: RatingCriterionType, value: RatingValue) {
        let key = match &criterion_type {
            RatingCriterionType::Custom(name) => {
                format!("custom_{}", name.to_lowercase().replace(' ', "_"))
            }
            _ => format!("{:?}", criterion_type).to_lowercase(),
        };

        self.rating_criteria.insert(
            key,
            RatingCriterion {
                criterion_type,
                value,
                weight: None,
            },
        );
        self.updated_at = Utc::now();
    }

    /// Add emotional tag
    pub fn add_emotional_tag(&mut self, tag: String) {
        let tag = tag.trim().to_lowercase();
        if !tag.is_empty() && !self.emotional_tags.contains(&tag) {
            self.emotional_tags.push(tag);
            self.updated_at = Utc::now();
        }
    }

    /// Add standout aspect
    pub fn add_standout_aspect(&mut self, aspect: String) {
        let aspect = aspect.trim().to_string();
        if !aspect.is_empty() && !self.standout_aspects.contains(&aspect) {
            self.standout_aspects.push(aspect);
            self.updated_at = Utc::now();
        }
    }

    /// Calculate composite score from all criteria
    pub fn calculate_composite_score(&self) -> f32 {
        let mut total_score = 0.0;
        let mut total_weight = 0.0;

        // Base score from overall feeling
        total_score += self.overall_feeling.to_score();
        total_weight += 1.0;

        // Add scores from numeric criteria
        for criterion in self.rating_criteria.values() {
            if let Some(score) = criterion.value.to_numeric_score() {
                let weight = criterion.weight.unwrap_or(1.0);
                total_score += score * weight;
                total_weight += weight;
            }
        }

        if total_weight > 0.0 {
            total_score / total_weight
        } else {
            self.overall_feeling.to_score()
        }
    }

    /// Get criteria of specific type
    pub fn get_criterion(&self, criterion_type: &RatingCriterionType) -> Option<&RatingCriterion> {
        let key = match criterion_type {
            RatingCriterionType::Custom(name) => {
                format!("custom_{}", name.to_lowercase().replace(' ', "_"))
            }
            _ => format!("{:?}", criterion_type).to_lowercase(),
        };
        self.rating_criteria.get(&key)
    }

    /// Check if rating is positive (for recommendations)
    pub fn is_positive_rating(&self) -> bool {
        matches!(
            self.overall_feeling,
            OverallFeeling::Loved | OverallFeeling::Enjoyed | OverallFeeling::Liked
        )
    }

    /// Get recommendation impact (how much this should influence recommendations)
    pub fn get_recommendation_impact(&self) -> f32 {
        let base_impact = self.overall_feeling.to_score() / 10.0; // 0.0 to 1.0

        match self.recommendation_preference {
            Some(RecommendationPreference::Recommend) => base_impact * 1.5,
            Some(RecommendationPreference::Avoid) => base_impact * -1.0,
            Some(RecommendationPreference::Neutral) | None => base_impact,
        }
    }

    /// Update notes
    pub fn set_notes(&mut self, notes: Option<String>) {
        self.personal_notes = notes;
        self.updated_at = Utc::now();
    }

    /// Set rewatch preference
    pub fn set_rewatch_preference(&mut self, would_rewatch: Option<bool>) {
        self.would_rewatch = would_rewatch;
        self.updated_at = Utc::now();
    }
}
