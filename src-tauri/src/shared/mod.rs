// Shared Kernel - Domain Driven Design
// Following Clean Architecture + Hexagonal Architecture patterns

pub mod application;      // Shared application layer patterns
pub mod domain;          // Shared domain concepts (value objects, events)
pub mod errors;          // Shared error types
pub mod infrastructure;  // Shared infrastructure (database, logging)
pub mod utils;           // Shared utilities

// Re-exports for convenience (backward compatibility)
pub use infrastructure::database::Database;
pub use infrastructure::database_state::{DatabaseHealthMonitor, DatabaseState};
