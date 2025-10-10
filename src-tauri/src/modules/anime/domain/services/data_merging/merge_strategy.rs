use super::field_mergers::*;
use super::merge_context::MergeContext;
use crate::modules::provider::domain::entities::anime_data::AnimeData;
use crate::shared::errors::AppResult;

/// Strategy for merging anime data from multiple sources
///
/// Implements Strategy Pattern for flexible merge behavior
pub trait MergeStrategy: Send + Sync {
    /// Merge anime data using this strategy
    fn merge(&self, context: MergeContext) -> AppResult<AnimeData>;
}

/// Default merge strategy that uses field-specific mergers
///
/// This strategy delegates to specialized field mergers for each field type,
/// following Single Responsibility Principle
#[derive(Debug, Clone, Copy)]
pub struct DefaultMergeStrategy {
    title_merger: TitleMerger,
    metadata_merger: MetadataMerger,
    collection_merger: CollectionMerger,
    rating_merger: RatingMerger,
    media_merger: MediaMerger,
}

impl DefaultMergeStrategy {
    pub fn new() -> Self {
        Self {
            title_merger: TitleMerger,
            metadata_merger: MetadataMerger,
            collection_merger: CollectionMerger,
            rating_merger: RatingMerger,
            media_merger: MediaMerger,
        }
    }
}

impl Default for DefaultMergeStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl MergeStrategy for DefaultMergeStrategy {
    fn merge(&self, context: MergeContext) -> AppResult<AnimeData> {
        let mut merged = context.base.clone();

        // Merge each category using specialized mergers
        // Order matters: merge basic fields first, then derived fields

        // 1. Titles (foundational)
        self.title_merger.merge_into(&mut merged.anime, &context);

        // 2. Metadata (descriptions, source, duration)
        self.metadata_merger.merge_into(&mut merged.anime, &context);

        // 3. Collections (genres, studios)
        self.collection_merger
            .merge_into(&mut merged.anime, &context);

        // 4. Ratings and restrictions
        self.rating_merger.merge_into(&mut merged.anime, &context);

        // 5. Media (images, trailers)
        self.media_merger.merge_into(&mut merged.anime, &context);

        // Update metadata after merging
        merged = self.update_metadata(merged, &context);

        Ok(merged)
    }
}

impl DefaultMergeStrategy {
    /// Update source metadata after merging
    fn update_metadata(&self, mut merged: AnimeData, context: &MergeContext) -> AnimeData {
        // Collect all providers involved
        let mut all_providers = vec![context.base.source.primary_provider];
        all_providers.extend(context.sources.iter().map(|s| s.source.primary_provider));
        merged.source.providers_used = all_providers;

        // Calculate merged confidence
        let total_sources = 1 + context.sources.len();
        let avg_quality = (context.base.quality.score
            + context.sources.iter().map(|s| s.quality.score).sum::<f32>())
            / total_sources as f32;

        // Bonus for multiple sources (capped at 30%)
        let multi_source_bonus = (total_sources as f32).min(3.0) * 0.1;
        merged.source.confidence = (avg_quality + multi_source_bonus).min(1.0);

        // Recalculate quality
        use crate::modules::provider::domain::entities::anime_data::DataQuality;
        merged.quality = DataQuality::calculate(&merged.anime);

        merged
    }
}
