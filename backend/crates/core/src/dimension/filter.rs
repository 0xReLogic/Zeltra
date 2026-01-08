//! Dimensional filtering for reports.

use serde::{Deserialize, Serialize};
use zeltra_shared::types::DimensionValueId;

/// Filter for dimensional queries.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DimensionFilter {
    /// Filter by specific dimension values.
    pub dimension_values: Vec<DimensionValueId>,
    /// Include transactions without dimension values.
    pub include_untagged: bool,
}

impl DimensionFilter {
    /// Creates a new empty filter.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a dimension value to the filter.
    #[must_use]
    pub fn with_value(mut self, value_id: DimensionValueId) -> Self {
        self.dimension_values.push(value_id);
        self
    }

    /// Sets whether to include untagged transactions.
    #[must_use]
    pub const fn include_untagged(mut self, include: bool) -> Self {
        self.include_untagged = include;
        self
    }

    /// Returns true if the filter is empty (matches everything).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.dimension_values.is_empty() && !self.include_untagged
    }
}
