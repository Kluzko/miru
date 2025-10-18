use crate::shared::errors::AppResult;
/// Base trait for all use cases following CQRS pattern
///
/// This trait provides a standard interface for command/query handlers
/// following the Single Responsibility Principle.
///
/// # Example
///
/// ```rust
/// struct CreateAnimeCommand {
///     title: String,
///     provider: AnimeProvider,
/// }
///
/// struct CreateAnimeResult {
///     id: Uuid,
///     title: String,
/// }
///
/// struct CreateAnimeUseCase {
///     repository: Arc<dyn AnimeRepository>,
/// }
///
/// #[async_trait]
/// impl UseCase<CreateAnimeCommand, CreateAnimeResult> for CreateAnimeUseCase {
///     async fn execute(&self, command: CreateAnimeCommand) -> Result<CreateAnimeResult, AppError> {
///         // Use case logic here
///     }
/// }
/// ```
use async_trait::async_trait;

/// Base trait for use cases (command handlers)
#[async_trait]
pub trait UseCase<TCommand, TResult> {
    /// Execute the use case with the given command
    async fn execute(&self, command: TCommand) -> AppResult<TResult>;
}

/// Base trait for queries (query handlers)
#[async_trait]
pub trait Query<TQuery, TResult> {
    /// Execute the query
    async fn execute(&self, query: TQuery) -> AppResult<TResult>;
}
