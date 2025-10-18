/// Pagination support for queries
///
/// Standard pagination model used across all bounded contexts
use serde::{Deserialize, Serialize};
use specta::Type;

/// Pagination parameters for queries
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PaginationParams {
    pub page: u32,
    pub page_size: u32,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 20,
        }
    }
}

impl PaginationParams {
    pub fn new(page: u32, page_size: u32) -> Self {
        Self { page, page_size }
    }

    /// Calculate offset for database queries
    pub fn offset(&self) -> i64 {
        ((self.page - 1) * self.page_size) as i64
    }

    /// Get limit for database queries
    pub fn limit(&self) -> i64 {
        self.page_size as i64
    }
}

/// Paginated result wrapper
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

impl<T> PaginatedResult<T> {
    pub fn new(items: Vec<T>, total_count: u64, params: &PaginationParams) -> Self {
        let total_pages = ((total_count as f64) / (params.page_size as f64)).ceil() as u32;

        Self {
            items,
            total_count,
            page: params.page,
            page_size: params.page_size,
            total_pages,
        }
    }
}
