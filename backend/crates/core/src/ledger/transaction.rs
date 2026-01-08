//! Transaction aggregate.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use zeltra_shared::types::{FiscalPeriodId, OrganizationId, TransactionId, UserId};

use super::entry::LedgerEntry;

/// Transaction status in the approval workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionStatus {
    /// Transaction is being drafted.
    Draft,
    /// Transaction has been submitted for approval.
    Pending,
    /// Transaction has been approved.
    Approved,
    /// Transaction has been rejected.
    Rejected,
    /// Transaction has been posted to the ledger.
    Posted,
    /// Transaction has been voided.
    Voided,
}

/// A financial transaction consisting of balanced ledger entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Unique identifier.
    pub id: TransactionId,
    /// Organization this transaction belongs to.
    pub organization_id: OrganizationId,
    /// Fiscal period this transaction is recorded in.
    pub fiscal_period_id: FiscalPeriodId,
    /// Transaction date.
    pub transaction_date: NaiveDate,
    /// Transaction reference number.
    pub reference: String,
    /// Transaction description.
    pub description: String,
    /// Currency code (ISO 4217).
    pub currency: String,
    /// Total amount (sum of debits or credits).
    pub total_amount: Decimal,
    /// Current status.
    pub status: TransactionStatus,
    /// User who created the transaction.
    pub created_by: UserId,
    /// When the transaction was created.
    pub created_at: DateTime<Utc>,
    /// When the transaction was last updated.
    pub updated_at: DateTime<Utc>,
    /// Ledger entries (populated when needed).
    #[serde(default)]
    pub entries: Vec<LedgerEntry>,
}

impl Transaction {
    /// Returns true if the transaction can be edited.
    #[must_use]
    pub fn is_editable(&self) -> bool {
        matches!(self.status, TransactionStatus::Draft | TransactionStatus::Rejected)
    }

    /// Returns true if the transaction can be submitted for approval.
    #[must_use]
    pub fn can_submit(&self) -> bool {
        matches!(self.status, TransactionStatus::Draft | TransactionStatus::Rejected)
    }

    /// Returns true if the transaction can be approved.
    #[must_use]
    pub fn can_approve(&self) -> bool {
        self.status == TransactionStatus::Pending
    }

    /// Returns true if the transaction can be posted.
    #[must_use]
    pub fn can_post(&self) -> bool {
        self.status == TransactionStatus::Approved
    }

    /// Returns true if the transaction can be voided.
    #[must_use]
    pub fn can_void(&self) -> bool {
        self.status == TransactionStatus::Posted
    }
}
