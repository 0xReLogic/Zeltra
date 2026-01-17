//! Pagination types for list endpoints.

use serde::{Deserialize, Deserializer, Serialize};

/// Deserialize u32 from string or number
fn deserialize_u32_from_string<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrU32 {
        String(String),
        Number(u32),
    }
    
    match StringOrU32::deserialize(deserializer)? {
        StringOrU32::String(s) => s.parse::<u32>().map_err(D::Error::custom),
        StringOrU32::Number(n) => Ok(n),
    }
}

/// Request parameters for paginated queries.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
pub struct PageRequest {
    /// Page number (1-indexed).
    #[serde(default = "default_page", deserialize_with = "deserialize_u32_from_string")]
    #[param(default = 1, example = 1)]
    #[schema(default = 1, example = 1)]
    pub page: u32,
    /// Number of items per page.
    #[serde(default = "default_per_page", deserialize_with = "deserialize_u32_from_string")]
    #[param(default = 20, example = 10)]
    #[schema(default = 20, example = 10)]
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
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PageResponse<T> {
    /// The items in the current page.
    pub data: Vec<T>,
    /// Pagination metadata.
    pub meta: PageMeta,
}

/// Pagination metadata.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PageMeta {
    /// Current page number.
    #[schema(example = 1)]
    pub page: u32,
    /// Items per page.
    #[schema(example = 20)]
    pub per_page: u32,
    /// Total number of items across all pages.
    #[schema(example = 100)]
    pub total: u64,
    /// Total number of pages.
    #[schema(example = 5)]
    pub total_pages: u32,
}

impl<T> PageResponse<T> {
    /// Creates a new paginated response.
    #[must_use]
    pub fn new(data: Vec<T>, page: u32, per_page: u32, total: u64) -> Self {
        let total_pages = if total == 0 || per_page == 0 {
            1
        } else {
            let per_page_u64 = u64::from(per_page);
            let pages = total.div_ceil(per_page_u64);
            // Safe truncation: we clamp to u32::MAX first
            #[allow(clippy::cast_possible_truncation)]
            let result = pages.min(u64::from(u32::MAX)) as u32;
            result
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
