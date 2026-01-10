//! Budget tracking and variance analysis.

pub mod error;
pub mod service;
pub mod types;
pub mod variance;

#[cfg(test)]
mod tests;

pub use error::BudgetError;
pub use service::BudgetService;
pub use types::{
    Budget, BudgetLine, BudgetLineWithActual, BudgetSummary, BudgetType, BudgetVsActualReport,
    BudgetVsActualSummary, CreateBudgetInput, CreateBudgetLineInput, DimensionInfo, VarianceResult,
    VarianceStatus,
};
pub use variance::{BudgetVariance, VarianceType};
