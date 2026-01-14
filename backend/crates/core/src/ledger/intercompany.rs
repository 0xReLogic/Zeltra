//! Intercompany Elimination Hub logic for Zeltra Sentinel.
//!
//! Handles matching and elimination of cross-entity transactions.

use super::types::{CreateTransactionInput, EntryType, LedgerEntryInput, TransactionType};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

/// Match result for two intercompany entries.
#[derive(Debug, Clone)]
pub struct IntercompanyMatch {
    /// Source entry ID.
    pub source_entry_id: Uuid,
    /// Target entry ID.
    pub target_entry_id: Uuid,
    /// Whether the amounts match.
    pub amount_match: bool,
    /// Absolute difference in days between dates.
    pub date_diff_days: i64,
}

/// Engine for intercompany matching and elimination.
pub struct IntercompanyEngine;

impl IntercompanyEngine {
    /// Checks if two entries match for intercompany elimination.
    ///
    /// Criteria:
    /// - Opposite signs (One is Debit in Source, another is Credit in Target - relative to intercompany accounts)
    /// - Same source amount and currency
    /// - Date within a specified day tolerance
    pub fn is_match(
        source_amount: Decimal,
        target_amount: Decimal,
        source_date: NaiveDate,
        target_date: NaiveDate,
        day_tolerance: i64,
    ) -> bool {
        source_amount == target_amount
            && (source_date - target_date).num_days().abs() <= day_tolerance
    }
}

/// Input for generating an elimination transaction.
#[derive(Debug, Clone)]
pub struct EliminationTransactionInput {
    /// Consolidation organization ID.
    pub consolidation_org_id: Uuid,
    /// Source account ID.
    pub source_account_id: Uuid,
    /// Target account ID.
    pub target_account_id: Uuid,
    /// Currency ID.
    pub currency: String,
    /// Amount to eliminate.
    pub amount: Decimal,
    /// Date of elimination.
    pub date: NaiveDate,
    /// Reference string (source transaction reference).
    pub reference: String,
}

/// Input for generating a mirror transaction.
#[derive(Debug, Clone)]
pub struct MirrorTransactionInput {
    /// Target organization ID.
    pub target_org_id: Uuid,
    /// Target account ID.
    pub target_account_id: Uuid,
    /// Balancing account ID in target org.
    pub balancing_account_id: Uuid,
    /// Source currency ID.
    pub source_currency: String,
    /// Amount to mirror.
    pub amount: Decimal,
    /// Date of transaction.
    pub date: NaiveDate,
    /// Entry type in source transaction.
    pub source_entry_type: EntryType,
    /// Reference string.
    pub reference: String,
}

impl IntercompanyEngine {
    /// Generates an elimination entry for a matched pair of transactions.
    pub fn generate_elimination_transaction(
        input: &EliminationTransactionInput,
    ) -> CreateTransactionInput {
        CreateTransactionInput {
            organization_id: input.consolidation_org_id,
            transaction_type: TransactionType::Adjustment,
            transaction_date: input.date,
            description: format!(
                "Intercompany Elimination: {ref_str}",
                ref_str = input.reference
            ),
            reference_number: Some(format!("ELIM-{ref_str}", ref_str = input.reference)),
            memo: Some("Eliminating balance between mapping accounts".to_string()),
            entries: vec![
                LedgerEntryInput {
                    account_id: input.source_account_id,
                    source_currency: input.currency.clone(),
                    source_amount: input.amount,
                    entry_type: EntryType::Credit,
                    memo: Some("Intercompany Elimination (Source)".to_string()),
                    functional_amount: None,
                    compliance_metadata: None,
                    dimensions: vec![],
                },
                LedgerEntryInput {
                    account_id: input.target_account_id,
                    source_currency: input.currency.clone(),
                    source_amount: input.amount,
                    entry_type: EntryType::Debit,
                    memo: Some("Intercompany Elimination (Target)".to_string()),
                    functional_amount: None,
                    compliance_metadata: None,
                    dimensions: vec![],
                },
            ],
            created_by: Uuid::nil(),
        }
    }

    /// Generates a mirror transaction for a source entry.
    pub fn generate_mirror_transaction(input: &MirrorTransactionInput) -> CreateTransactionInput {
        let target_entry_type = match input.source_entry_type {
            EntryType::Debit => EntryType::Credit,
            EntryType::Credit => EntryType::Debit,
        };

        CreateTransactionInput {
            organization_id: input.target_org_id,
            transaction_type: TransactionType::Transfer,
            transaction_date: input.date,
            description: format!("Intercompany Mirror: {ref_str}", ref_str = input.reference),
            reference_number: Some(format!("MIR-{ref_str}", ref_str = input.reference)),
            memo: Some("Auto-mirrored from source transaction".to_string()),
            entries: vec![
                LedgerEntryInput {
                    account_id: input.target_account_id,
                    source_currency: input.source_currency.clone(),
                    source_amount: input.amount,
                    entry_type: target_entry_type,
                    memo: Some("Intercompany Mirroring".to_string()),
                    functional_amount: None,
                    compliance_metadata: None,
                    dimensions: vec![],
                },
                LedgerEntryInput {
                    account_id: input.balancing_account_id,
                    source_currency: input.source_currency.clone(),
                    source_amount: input.amount,
                    entry_type: input.source_entry_type,
                    memo: Some("Intercompany Mirroring (Offset)".to_string()),
                    functional_amount: None,
                    compliance_metadata: None,
                    dimensions: vec![],
                },
            ],
            created_by: Uuid::nil(),
        }
    }
}
