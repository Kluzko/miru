//! User anime data management methods

use chrono::Utc;

use super::{UserAnimeData, WatchingStatus};

// Management methods for UserAnimeData
impl UserAnimeData {
    /// Start watching (set status and start date)
    pub fn start_watching(&mut self) {
        if self.start_date.is_none() {
            self.start_date = Some(Utc::now());
        }
        self.status = WatchingStatus::Watching;
        self.updated_at = Utc::now();
    }

    /// Mark as completed
    pub fn complete(&mut self) {
        self.status = WatchingStatus::Completed;
        self.finish_date = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Update episodes watched
    pub fn update_progress(&mut self, episodes_watched: i32) {
        self.episodes_watched = episodes_watched.max(0);

        // Auto-start if not already started
        if self.status == WatchingStatus::PlanToWatch && episodes_watched > 0 {
            self.start_watching();
        }

        self.updated_at = Utc::now();
    }

    /// Add personal tag
    pub fn add_tag(&mut self, tag: String) {
        let tag = tag.trim().to_lowercase();
        if !tag.is_empty() && !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = Utc::now();
        }
    }

    /// Remove personal tag
    pub fn remove_tag(&mut self, tag: &str) {
        let tag = tag.trim().to_lowercase();
        self.tags.retain(|t| t != &tag);
        self.updated_at = Utc::now();
    }

    /// Set personal rating
    pub fn set_rating(&mut self, rating: Option<f32>) {
        if let Some(r) = rating {
            self.personal_rating = Some(r.clamp(0.0, 10.0));
        } else {
            self.personal_rating = None;
        }
        self.updated_at = Utc::now();
    }

    /// Toggle favorite status
    pub fn toggle_favorite(&mut self) {
        self.is_favorite = !self.is_favorite;
        self.updated_at = Utc::now();
    }

    /// Get completion percentage (if total episodes known)
    pub fn completion_percentage(&self, total_episodes: Option<i32>) -> Option<f32> {
        total_episodes.map(|total| {
            if total > 0 {
                (self.episodes_watched as f32 / total as f32 * 100.0).min(100.0)
            } else {
                0.0
            }
        })
    }

    /// Get time spent watching (rough estimate)
    pub fn estimated_watch_time_minutes(
        &self,
        episode_duration_minutes: Option<i32>,
    ) -> Option<i32> {
        episode_duration_minutes.map(|duration| self.episodes_watched * duration)
    }
}
