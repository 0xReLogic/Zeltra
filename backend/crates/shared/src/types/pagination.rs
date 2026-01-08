//! Pagination types for list endpoints.

use serde::{Deserialize, Serialize};

/// Request parameters for paginated queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageRequest {
    /// Page number (1-indexed).
    #[serde(default = "default_page")]
    pub page: u32,
    /// Number of items per page.
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    20
}

impl Default for PageRequest {
    fn default() -> Self {
        Self {
            page: default_page(),
            per_page: default_per_page(),
        }
    }
}

impl PageRequest {
    /// Calculates the offset for database queries.
    #[must_use]
    pub fn offset(&self) -> u64 {
        u64::from((self.page.saturating_sub(1)) * self.per_page)
    }

    /// Returns the limit for database queries.
    #[must_use]
    pub fn limit(&self) -> u64 {
        u64::from(self.per_page)
    }
}

/// Response wrapper for paginated data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageResponse<T> {
    /// The items in the current page.
    pub data: Vec<T>,
    /// Pagination metadata.
    pub meta: PageMeta,
}

/// Pagination metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageMeta {
    /// Current page number.
    pub page: u32,
    /// Items per page.
    pub per_page: u32,
    /// Total number of items across all pages.
    pub total: u64,
    /// Total number of pages.
    pub total_pages: u32,
}

impl<T> PageResponse<T> {
    /// Creates a new paginated response.
    #[must_use]
    pub fn new(data: Vec<T>, page: u32, per_page: u32, total: u64) -> Self {
        let total_pages = if total == 0 {
            1
        } else {
            ((total as f64) / (per_page as f64)).ceil() as u32
        };

        Self {
            data,
            meta: PageMeta {
                page,
                per_page,
                total,
                total_pages,
            },
        }
    }
}
