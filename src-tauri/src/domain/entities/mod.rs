// AnimeBasic entity removed - was not used anywhere
pub mod anime_detailed;
mod collection;
mod genre;

pub use anime_detailed::{AiredDates, AnimeDetailed};

pub use collection::{Collection, CollectionAnime};
pub use genre::Genre;
