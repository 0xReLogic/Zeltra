//! What-if scenario projections.

pub mod cache;
pub mod engine;
pub mod error;
pub mod scenario;
pub mod types;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod benchmark;

pub use cache::SimulationCache;
pub use engine::SimulationEngine;
pub use error::SimulationError;
pub use scenario::{Scenario, ScenarioResult};
pub use types::{
    AccountProjection, AnnualSummary, HistoricalAccountData, SimulationParams, SimulationResult,
};
