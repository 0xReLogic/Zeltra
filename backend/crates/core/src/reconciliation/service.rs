//! Reconciliation service for detecting balance drift.
//!
//! Compares calculated balances from ledger entries against stored account balances.
//! Based on industry best practices:
//! - Regular scheduling (configurable frequency)
//! - Tolerance thresholds for minor FX drift
//! - Full audit trail of discrepancies
//! - Risk-based approach (flag high-value accounts)

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Tolerance threshold for acceptable balance drift (in functional currency units).
/// Accounts within this threshold are considered "within tolerance" but still logged.
pub const DEFAULT_TOLERANCE: Decimal = Decimal::ONE;

/// Status of the reconciliation for a single account.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReconciliationStatus {
    /// Calculated balance matches stored balance exactly.
    Matched,
    /// Difference is within tolerance threshold.
    WithinTolerance,
    /// Difference exceeds tolerance - requires investigation.
    Discrepancy,
}

/// Details of a discrepancy for a single account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountDiscrepancy {
    /// Account ID.
    pub account_id: Uuid,
    /// Account code (for human readability).
    pub account_code: String,
    /// Account name.
    pub account_name: String,
    /// Balance stored in the account record.
    pub stored_balance: Decimal,
    /// Balance calculated from SUM of ledger entries.
    pub calculated_balance: Decimal,
    /// Difference (stored - calculated).
    pub difference: Decimal,
    /// Status of this account's reconciliation.
    pub status: ReconciliationStatus,
}

/// Full reconciliation report for an organization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationReport {
    /// Organization ID.
    pub organization_id: Uuid,
    /// Timestamp when reconciliation was run.
    pub run_at: chrono::DateTime<chrono::Utc>,
    /// Total accounts checked.
    pub total_accounts: usize,
    /// Accounts with exact match.
    pub matched_count: usize,
    /// Accounts within tolerance.
    pub within_tolerance_count: usize,
    /// Accounts with discrepancies requiring investigation.
    pub discrepancy_count: usize,
    /// Detailed list of all account results.
    pub accounts: Vec<AccountDiscrepancy>,
    /// Overall status: true if no discrepancies found.
    pub is_clean: bool,
}

/// Reconciliation service for running balance checks.
///
/// This service compares the stored `current_balance` on each account
/// against the calculated sum of all posted ledger entries.
#[derive(Debug, Clone)]
pub struct ReconciliationService {
    /// Tolerance threshold for acceptable drift.
    tolerance: Decimal,
}

impl Default for ReconciliationService {
    fn default() -> Self {
        Self::new()
    }
}

impl ReconciliationService {
    /// Creates a new reconciliation service with default tolerance.
    #[must_use]
    pub fn new() -> Self {
        Self {
            tolerance: DEFAULT_TOLERANCE,
        }
    }

    /// Creates a reconciliation service with custom tolerance.
    #[must_use]
    pub fn with_tolerance(tolerance: Decimal) -> Self {
        Self { tolerance }
    }

    /// Analyzes a list of account balance comparisons and produces a report.
    ///
    /// This is a pure function that takes pre-fetched data and returns a report.
    /// The actual database queries are performed by the repository layer.
    ///
    /// # Arguments
    ///
    /// * `organization_id` - The organization being reconciled.
    /// * `comparisons` - List of (account_id, code, name, stored_balance, calculated_balance).
    ///
    /// # Returns
    ///
    /// A complete reconciliation report.
    #[must_use]
    pub fn analyze(
        &self,
        organization_id: Uuid,
        comparisons: Vec<(Uuid, String, String, Decimal, Decimal)>,
    ) -> ReconciliationReport {
        let run_at = chrono::Utc::now();
        let total_accounts = comparisons.len();

        let mut matched_count = 0;
        let mut within_tolerance_count = 0;
        let mut discrepancy_count = 0;
        let mut accounts = Vec::with_capacity(total_accounts);

        for (account_id, account_code, account_name, stored_balance, calculated_balance) in
            comparisons
        {
            let difference = stored_balance - calculated_balance;
            let abs_difference = difference.abs();

            let status = if abs_difference.is_zero() {
                matched_count += 1;
                ReconciliationStatus::Matched
            } else if abs_difference <= self.tolerance {
                within_tolerance_count += 1;
                ReconciliationStatus::WithinTolerance
            } else {
                discrepancy_count += 1;
                ReconciliationStatus::Discrepancy
            };

            accounts.push(AccountDiscrepancy {
                account_id,
                account_code,
                account_name,
                stored_balance,
                calculated_balance,
                difference,
                status,
            });
        }

        ReconciliationReport {
            organization_id,
            run_at,
            total_accounts,
            matched_count,
            within_tolerance_count,
            discrepancy_count,
            accounts,
            is_clean: discrepancy_count == 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_exact_match() {
        let service = ReconciliationService::new();
        let org_id = Uuid::new_v4();
        let account_id = Uuid::new_v4();

        let comparisons = vec![(
            account_id,
            "1000".to_string(),
            "Cash".to_string(),
            dec!(1000.00),
            dec!(1000.00),
        )];

        let report = service.analyze(org_id, comparisons);

        assert!(report.is_clean);
        assert_eq!(report.matched_count, 1);
        assert_eq!(report.discrepancy_count, 0);
        assert_eq!(report.accounts[0].status, ReconciliationStatus::Matched);
    }

    #[test]
    fn test_within_tolerance() {
        let service = ReconciliationService::new();
        let org_id = Uuid::new_v4();
        let account_id = Uuid::new_v4();

        // Difference of 0.50 is within default tolerance of 1.00
        let comparisons = vec![(
            account_id,
            "1000".to_string(),
            "Cash".to_string(),
            dec!(1000.50),
            dec!(1000.00),
        )];

        let report = service.analyze(org_id, comparisons);

        assert!(report.is_clean);
        assert_eq!(report.within_tolerance_count, 1);
        assert_eq!(
            report.accounts[0].status,
            ReconciliationStatus::WithinTolerance
        );
    }

    #[test]
    fn test_discrepancy_detected() {
        let service = ReconciliationService::new();
        let org_id = Uuid::new_v4();
        let account_id = Uuid::new_v4();

        // Difference of 5.00 exceeds default tolerance of 1.00
        let comparisons = vec![(
            account_id,
            "1000".to_string(),
            "Cash".to_string(),
            dec!(1005.00),
            dec!(1000.00),
        )];

        let report = service.analyze(org_id, comparisons);

        assert!(!report.is_clean);
        assert_eq!(report.discrepancy_count, 1);
        assert_eq!(report.accounts[0].status, ReconciliationStatus::Discrepancy);
        assert_eq!(report.accounts[0].difference, dec!(5.00));
    }

    #[test]
    fn test_mixed_results() {
        let service = ReconciliationService::new();
        let org_id = Uuid::new_v4();

        let comparisons = vec![
            (
                Uuid::new_v4(),
                "1000".to_string(),
                "Cash".to_string(),
                dec!(1000.00),
                dec!(1000.00),
            ),
            (
                Uuid::new_v4(),
                "1100".to_string(),
                "Bank".to_string(),
                dec!(5000.50),
                dec!(5000.00),
            ),
            (
                Uuid::new_v4(),
                "2000".to_string(),
                "Payables".to_string(),
                dec!(3000.00),
                dec!(3010.00),
            ),
        ];

        let report = service.analyze(org_id, comparisons);

        assert!(!report.is_clean);
        assert_eq!(report.total_accounts, 3);
        assert_eq!(report.matched_count, 1);
        assert_eq!(report.within_tolerance_count, 1);
        assert_eq!(report.discrepancy_count, 1);
    }
}
