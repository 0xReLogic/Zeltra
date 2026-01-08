//! Fiscal period types.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use zeltra_shared::types::{FiscalPeriodId, FiscalYearId, OrganizationId};

/// Fiscal year definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiscalYear {
    /// Unique identifier.
    pub id: FiscalYearId,
    /// Organization this fiscal year belongs to.
    pub organization_id: OrganizationId,
    /// Year name (e.g., "FY2026").
    pub name: String,
    /// Start date of the fiscal year.
    pub start_date: NaiveDate,
    /// End date of the fiscal year.
    pub end_date: NaiveDate,
    /// Whether this is the current active fiscal year.
    pub is_active: bool,
}

/// Status of a fiscal period.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FiscalPeriodStatus {
    /// Period is not yet open for transactions.
    Future,
    /// Period is open for transactions.
    Open,
    /// Period is closed, no new transactions allowed.
    Closed,
    /// Period is locked, no changes allowed.
    Locked,
}

/// A fiscal period within a fiscal year.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiscalPeriod {
    /// Unique identifier.
    pub id: FiscalPeriodId,
    /// Fiscal year this period belongs to.
    pub fiscal_year_id: FiscalYearId,
    /// Period number within the year (1-12 for monthly).
    pub period_number: i32,
    /// Period name (e.g., "January 2026").
    pub name: String,
    /// Start date of the period.
    pub start_date: NaiveDate,
    /// End date of the period.
    pub end_date: NaiveDate,
    /// Current status.
    pub status: FiscalPeriodStatus,
}

impl FiscalPeriod {
    /// Returns true if transactions can be posted to this period.
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.status == FiscalPeriodStatus::Open
    }

    /// Returns true if the given date falls within this period.
    #[must_use]
    pub fn contains_date(&self, date: NaiveDate) -> bool {
        date >= self.start_date && date <= self.end_date
    }
}
