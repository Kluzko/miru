/// Shared application layer patterns
///
/// This module contains application-level abstractions used across
/// multiple bounded contexts.
pub mod pagination;
pub mod use_case;

pub use pagination::*;
pub use use_case::UseCase;
