//! Core business logic for Zeltra.
//!
//! This crate contains pure business logic with ZERO web or database dependencies.
//! All domain types, validation rules, and calculations live here.
//!
//! # Modules
//!
//! - `ledger` - Double-entry bookkeeping logic
//! - `currency` - Multi-currency handling and exchange rates
//! - `fiscal` - Fiscal year and period management
//! - `budget` - Budget tracking and variance analysis
//! - `simulation` - What-if scenario projections
//! - `dimension` - Dimensional reporting and filtering

pub mod budget;
pub mod currency;
pub mod dimension;
pub mod fiscal;
pub mod ledger;
pub mod simulation;
