//! Broadcast information with proper typing

use chrono::{NaiveTime, Weekday};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use specta::Type;

/// Broadcast information with proper typing
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BroadcastInfo {
    /// Day of the week the anime airs (e.g., "Monday", "Tuesday")
    pub day: Option<String>,
    /// Local time the anime airs (e.g., "23:30")
    pub time: Option<String>,
    /// Timezone for the broadcast time (e.g., "Asia/Tokyo", "America/New_York")
    pub timezone: Option<String>,
    /// Optional cached display string (derived data)
    pub string: Option<String>,
}

impl BroadcastInfo {
    /// Create new broadcast info from typed components
    pub fn new(day: Option<Weekday>, time: Option<NaiveTime>, timezone: Option<Tz>) -> Self {
        let day_str = day.map(|d| d.to_string());
        let time_str = time.map(|t| t.format("%H:%M").to_string());
        let tz_str = timezone.map(|tz| tz.name().to_string());
        let string = Self::generate_display_string(&day_str, &time_str, &tz_str);

        Self {
            day: day_str,
            time: time_str,
            timezone: tz_str,
            string,
        }
    }

    /// Create new broadcast info from strings
    pub fn from_strings(
        day: Option<String>,
        time: Option<String>,
        timezone: Option<String>,
    ) -> Self {
        let string = Self::generate_display_string(&day, &time, &timezone);

        Self {
            day,
            time,
            timezone,
            string,
        }
    }

    /// Generate display string from string components
    fn generate_display_string(
        day: &Option<String>,
        time: &Option<String>,
        timezone: &Option<String>,
    ) -> Option<String> {
        match (day, time, timezone) {
            (Some(d), Some(t), Some(tz)) => Some(format!("{} at {} {}", d, t, tz)),
            (Some(d), Some(t), None) => Some(format!("{} at {}", d, t)),
            (Some(d), None, _) => Some(d.clone()),
            (None, Some(t), Some(tz)) => Some(format!("{} {}", t, tz)),
            (None, Some(t), None) => Some(t.clone()),
            _ => None,
        }
    }

    /// Refresh the cached display string
    pub fn refresh_display_string(&mut self) {
        self.string = Self::generate_display_string(&self.day, &self.time, &self.timezone);
    }

    /// Parse from Jikan API string format
    pub fn from_jikan_string(broadcast_string: &str) -> Self {
        // Parse strings like "Saturdays at 23:30 (JST)" or "Unknown"
        let mut day = None;
        let mut time = None;
        let mut timezone = None;

        if broadcast_string.to_lowercase() != "unknown" {
            // Try to extract day
            let weekdays = [
                ("monday", "Monday"),
                ("tuesday", "Tuesday"),
                ("wednesday", "Wednesday"),
                ("thursday", "Thursday"),
                ("friday", "Friday"),
                ("saturday", "Saturday"),
                ("sunday", "Sunday"),
            ];

            for (day_name, proper_name) in &weekdays {
                let plural = format!("{}s", day_name);
                if broadcast_string.to_lowercase().contains(&plural) {
                    day = Some(proper_name.to_string());
                    break;
                }
            }

            // Try to extract time (simple regex-like parsing)
            if let Some(at_pos) = broadcast_string.find(" at ") {
                let time_part = &broadcast_string[at_pos + 4..];
                if let Some(space_pos) = time_part.find(' ') {
                    let time_str = &time_part[..space_pos];
                    // Validate time format
                    if NaiveTime::parse_from_str(time_str, "%H:%M").is_ok() {
                        time = Some(time_str.to_string());
                    }
                } else if NaiveTime::parse_from_str(time_part, "%H:%M").is_ok() {
                    time = Some(time_part.to_string());
                }
            }

            // Try to extract timezone (JST, etc.)
            if broadcast_string.contains("JST") {
                timezone = Some("Asia/Tokyo".to_string());
            }
        }

        Self {
            day,
            time,
            timezone,
            string: Some(broadcast_string.to_string()),
        }
    }

    /// Get parsed weekday if valid
    pub fn get_weekday(&self) -> Option<Weekday> {
        self.day
            .as_ref()
            .and_then(|d| match d.to_lowercase().as_str() {
                "monday" => Some(Weekday::Mon),
                "tuesday" => Some(Weekday::Tue),
                "wednesday" => Some(Weekday::Wed),
                "thursday" => Some(Weekday::Thu),
                "friday" => Some(Weekday::Fri),
                "saturday" => Some(Weekday::Sat),
                "sunday" => Some(Weekday::Sun),
                _ => None,
            })
    }

    /// Get parsed time if valid
    pub fn get_time(&self) -> Option<NaiveTime> {
        self.time
            .as_ref()
            .and_then(|t| NaiveTime::parse_from_str(t, "%H:%M").ok())
    }

    /// Get parsed timezone if valid
    pub fn get_timezone(&self) -> Option<Tz> {
        self.timezone.as_ref().and_then(|tz| tz.parse().ok())
    }
}

impl Default for BroadcastInfo {
    fn default() -> Self {
        Self {
            day: None,
            time: None,
            timezone: None,
            string: None,
        }
    }
}
