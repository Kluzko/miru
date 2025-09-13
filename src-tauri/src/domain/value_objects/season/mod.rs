//! Season and broadcast information types

// Serde and specta imports removed - not used in this module directly

// Sub-modules
mod broadcast_info;
mod season_enum;

// Re-export types
pub use broadcast_info::BroadcastInfo;
pub use season_enum::Season;
