/// Anime Relations - Child Entity
///
/// Relations are child entities within the Anime aggregate.
/// They cannot exist independently of the parent Anime.
use crate::modules::anime::domain::value_objects::AnimeRelationType;
use uuid::Uuid;

/// A relation from one anime to another
///
/// This is a child entity that only exists within the AnimeAggregate boundary.
#[derive(Debug, Clone)]
pub struct AnimeRelation {
    /// ID of the related anime
    pub related_anime_id: Uuid,

    /// Type of relation (sequel, prequel, side story, etc.)
    pub relation_type: AnimeRelationType,

    /// Optional title of the related anime (for display)
    pub related_title: Option<String>,
}

impl AnimeRelation {
    pub fn new(
        related_anime_id: Uuid,
        relation_type: AnimeRelationType,
        related_title: Option<String>,
    ) -> Self {
        Self {
            related_anime_id,
            relation_type,
            related_title,
        }
    }
}

// Note: In the future, the AnimeAggregate will manage a collection of these relations
// and ensure bidirectional consistency is maintained through business logic.
