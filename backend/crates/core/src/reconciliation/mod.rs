//! Reconciliation module for balance drift detection.
//!
//! Implements automated comparison of calculated ledger balances
//! against stored account balances to detect discrepancies.

mod service;

pub use service::{
    AccountDiscrepancy, ReconciliationReport, ReconciliationService, ReconciliationStatus,
};
