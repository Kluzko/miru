/// Shared infrastructure concerns
///
/// This module contains infrastructure implementations that are shared across
/// multiple bounded contexts (modules).
pub mod database;
pub mod database_state;

// Re-exports for convenience
pub use database::Database;
pub use database_state::DatabaseState;
