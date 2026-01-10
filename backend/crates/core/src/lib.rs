//! Core business logic for Zeltra.
//!
//! This crate contains pure business logic with ZERO web or database dependencies.
//! All domain types, validation rules, and calculations live here.
//!
//! # Modules
//!
//! - `auth` - Authentication and password hashing
//! - `ledger` - Double-entry bookkeeping logic
//! - `currency` - Multi-currency handling and exchange rates
//! - `fiscal` - Fiscal year and period management
//! - `budget` - Budget tracking and variance analysis
//! - `simulation` - What-if scenario projections
//! - `dimension` - Dimensional reporting and filtering
//! - `workflow` - Transaction workflow and approval management
//! - `reports` - Financial report generation
//! - `dashboard` - Dashboard metrics and activity types

pub mod auth;
pub mod budget;
pub mod currency;
pub mod dashboard;
pub mod dimension;
pub mod fiscal;
pub mod ledger;
pub mod reports;
pub mod simulation;
pub mod workflow;
