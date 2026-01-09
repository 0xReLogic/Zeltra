# Design Document: Transaction Workflow

## Overview

The Transaction Workflow feature implements a state machine for managing financial transaction lifecycles in Zeltra. It enforces proper authorization, maintains immutable audit trails, and supports void operations through reversing entries. The design follows accounting best practices where posted transactions are never modified—only reversed.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         API Layer (Axum)                        │
│  POST /transactions/:id/submit  POST /transactions/:id/approve  │
│  POST /transactions/:id/reject  POST /transactions/:id/post     │
│  POST /transactions/:id/void    GET /transactions/pending       │
│  POST /transactions/bulk-approve                                │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Core Layer (Business Logic)                │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ WorkflowService │  │ ApprovalEngine  │  │ ReversalService │ │
│  │                 │  │                 │  │                 │ │
│  │ - submit()      │  │ - get_required  │  │ - create_       │ │
│  │ - approve()     │  │   _approval()   │  │   reversing_    │ │
│  │ - reject()      │  │ - can_approve() │  │   transaction() │ │
│  │ - post()        │  │ - match_rules() │  │                 │ │
│  │ - void()        │  │                 │  │                 │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      DB Layer (SeaORM)                          │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ TransactionRepo │  │ ApprovalRuleRepo│  │ LedgerEntryRepo │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      PostgreSQL                                 │
│  transactions, ledger_entries, approval_rules                   │
│  + Triggers: prevent_posted_modification, check_transaction_    │
│              balance                                            │
└─────────────────────────────────────────────────────────────────┘
```

## Components and Interfaces

### State Machine

```
                    ┌──────────┐
                    │  draft   │◄────────────────┐
                    └────┬─────┘                 │
                         │ submit()              │ reject()
                         ▼                       │
                    ┌──────────┐                 │
                    │ pending  │─────────────────┘
                    └────┬─────┘
                         │ approve()
                         ▼
                    ┌──────────┐
                    │ approved │
                    └────┬─────┘
                         │ post()
                         ▼
                    ┌──────────┐     void()     ┌──────────┐
                    │  posted  │───────────────►│  voided  │
                    └──────────┘                └──────────┘
```

### WorkflowService (core/src/workflow/service.rs)

```rust
use rust_decimal::Decimal;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::workflow::error::WorkflowError;
use crate::workflow::types::{TransactionStatus, WorkflowAction};

pub struct WorkflowService;

impl WorkflowService {
    /// Submit a draft transaction for approval
    pub fn submit(
        current_status: TransactionStatus,
        submitted_by: Uuid,
    ) -> Result<WorkflowAction, WorkflowError> {
        match current_status {
            TransactionStatus::Draft => Ok(WorkflowAction::Submit {
                new_status: TransactionStatus::Pending,
                submitted_by,
                submitted_at: Utc::now(),
            }),
            _ => Err(WorkflowError::InvalidTransition {
                from: current_status,
                to: TransactionStatus::Pending,
            }),
        }
    }

    /// Approve a pending transaction
    pub fn approve(
        current_status: TransactionStatus,
        approved_by: Uuid,
        approval_notes: Option<String>,
    ) -> Result<WorkflowAction, WorkflowError> {
        match current_status {
            TransactionStatus::Pending => Ok(WorkflowAction::Approve {
                new_status: TransactionStatus::Approved,
                approved_by,
                approved_at: Utc::now(),
                approval_notes,
            }),
            _ => Err(WorkflowError::InvalidTransition {
                from: current_status,
                to: TransactionStatus::Approved,
            }),
        }
    }

    /// Reject a pending transaction back to draft
    pub fn reject(
        current_status: TransactionStatus,
        rejection_reason: String,
    ) -> Result<WorkflowAction, WorkflowError> {
        match current_status {
            TransactionStatus::Pending => Ok(WorkflowAction::Reject {
                new_status: TransactionStatus::Draft,
                rejection_reason,
            }),
            _ => Err(WorkflowError::InvalidTransition {
                from: current_status,
                to: TransactionStatus::Draft,
            }),
        }
    }

    /// Post an approved transaction to the ledger
    pub fn post(
        current_status: TransactionStatus,
        posted_by: Uuid,
    ) -> Result<WorkflowAction, WorkflowError> {
        match current_status {
            TransactionStatus::Approved => Ok(WorkflowAction::Post {
                new_status: TransactionStatus::Posted,
                posted_by,
                posted_at: Utc::now(),
            }),
            _ => Err(WorkflowError::InvalidTransition {
                from: current_status,
                to: TransactionStatus::Posted,
            }),
        }
    }

    /// Void a posted transaction (requires reversing entry)
    pub fn void(
        current_status: TransactionStatus,
        voided_by: Uuid,
        void_reason: String,
    ) -> Result<WorkflowAction, WorkflowError> {
        match current_status {
            TransactionStatus::Posted => Ok(WorkflowAction::Void {
                new_status: TransactionStatus::Voided,
                voided_by,
                voided_at: Utc::now(),
                void_reason,
            }),
            _ => Err(WorkflowError::InvalidTransition {
                from: current_status,
                to: TransactionStatus::Voided,
            }),
        }
    }

    /// Check if a status transition is valid
    pub fn is_valid_transition(from: TransactionStatus, to: TransactionStatus) -> bool {
        matches!(
            (from, to),
            (TransactionStatus::Draft, TransactionStatus::Pending)
                | (TransactionStatus::Pending, TransactionStatus::Approved)
                | (TransactionStatus::Pending, TransactionStatus::Draft)
                | (TransactionStatus::Approved, TransactionStatus::Posted)
                | (TransactionStatus::Posted, TransactionStatus::Voided)
        )
    }
}
```

### WorkflowTypes (core/src/workflow/types.rs)

```rust
use rust_decimal::Decimal;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    Draft,
    Pending,
    Approved,
    Posted,
    Voided,
}

impl TransactionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Posted => "posted",
            Self::Voided => "voided",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "draft" => Some(Self::Draft),
            "pending" => Some(Self::Pending),
            "approved" => Some(Self::Approved),
            "posted" => Some(Self::Posted),
            "voided" => Some(Self::Voided),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum WorkflowAction {
    Submit {
        new_status: TransactionStatus,
        submitted_by: Uuid,
        submitted_at: DateTime<Utc>,
    },
    Approve {
        new_status: TransactionStatus,
        approved_by: Uuid,
        approved_at: DateTime<Utc>,
        approval_notes: Option<String>,
    },
    Reject {
        new_status: TransactionStatus,
        rejection_reason: String,
    },
    Post {
        new_status: TransactionStatus,
        posted_by: Uuid,
        posted_at: DateTime<Utc>,
    },
    Void {
        new_status: TransactionStatus,
        voided_by: Uuid,
        voided_at: DateTime<Utc>,
        void_reason: String,
    },
}
```

### WorkflowError (core/src/workflow/error.rs)

```rust
use thiserror::Error;
use uuid::Uuid;
use rust_decimal::Decimal;

use crate::workflow::types::TransactionStatus;

#[derive(Debug, Error)]
pub enum WorkflowError {
    #[error("Invalid status transition from {from:?} to {to:?}")]
    InvalidTransition {
        from: TransactionStatus,
        to: TransactionStatus,
    },

    #[error("Cannot modify posted transaction")]
    CannotModifyPosted,

    #[error("Cannot modify voided transaction")]
    CannotModifyVoided,

    #[error("User {user_id} is not authorized to approve this transaction")]
    NotAuthorizedToApprove { user_id: Uuid },

    #[error("Transaction amount {amount} exceeds user approval limit {limit}")]
    ExceedsApprovalLimit { amount: Decimal, limit: Decimal },

    #[error("No approval rule found for transaction type {transaction_type} with amount {amount}")]
    NoApprovalRuleFound {
        transaction_type: String,
        amount: Decimal,
    },

    #[error("User role {user_role} does not meet required role {required_role}")]
    InsufficientRole {
        user_role: String,
        required_role: String,
    },

    #[error("Transaction {0} not found")]
    TransactionNotFound(Uuid),

    #[error("Void reason is required")]
    VoidReasonRequired,

    #[error("Rejection reason is required")]
    RejectionReasonRequired,

    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
}
```

### ApprovalEngine (core/src/workflow/approval.rs)

```rust
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::workflow::error::WorkflowError;

#[derive(Debug, Clone)]
pub struct ApprovalRule {
    pub id: Uuid,
    pub name: String,
    pub min_amount: Option<Decimal>,
    pub max_amount: Option<Decimal>,
    pub transaction_types: Vec<String>,
    pub required_role: String,
    pub priority: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UserRole {
    Viewer = 0,
    Submitter = 1,
    Approver = 2,
    Accountant = 3,
    Admin = 4,
    Owner = 5,
}

impl UserRole {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "viewer" => Some(Self::Viewer),
            "submitter" => Some(Self::Submitter),
            "approver" => Some(Self::Approver),
            "accountant" => Some(Self::Accountant),
            "admin" => Some(Self::Admin),
            "owner" => Some(Self::Owner),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Viewer => "viewer",
            Self::Submitter => "submitter",
            Self::Approver => "approver",
            Self::Accountant => "accountant",
            Self::Admin => "admin",
            Self::Owner => "owner",
        }
    }
}

pub struct ApprovalEngine;

impl ApprovalEngine {
    /// Determine required approver role for a transaction
    pub fn get_required_approval(
        rules: &[ApprovalRule],
        transaction_type: &str,
        total_amount: Decimal,
    ) -> Option<String> {
        let mut applicable: Vec<_> = rules
            .iter()
            .filter(|r| r.transaction_types.contains(&transaction_type.to_string()))
            .filter(|r| {
                let above_min = r.min_amount.map_or(true, |min| total_amount >= min);
                let below_max = r.max_amount.map_or(true, |max| total_amount <= max);
                above_min && below_max
            })
            .collect();

        // Sort by priority (lower = higher priority)
        applicable.sort_by_key(|r| r.priority);
        applicable.first().map(|r| r.required_role.clone())
    }

    /// Check if user can approve a transaction
    pub fn can_approve(
        user_role: &str,
        user_approval_limit: Option<Decimal>,
        required_role: &str,
        transaction_amount: Decimal,
    ) -> Result<(), WorkflowError> {
        let user_role_enum = UserRole::from_str(user_role).ok_or_else(|| {
            WorkflowError::InsufficientRole {
                user_role: user_role.to_string(),
                required_role: required_role.to_string(),
            }
        })?;

        let required_role_enum = UserRole::from_str(required_role).ok_or_else(|| {
            WorkflowError::InsufficientRole {
                user_role: user_role.to_string(),
                required_role: required_role.to_string(),
            }
        })?;

        // Check role hierarchy
        if user_role_enum < required_role_enum {
            return Err(WorkflowError::InsufficientRole {
                user_role: user_role.to_string(),
                required_role: required_role.to_string(),
            });
        }

        // Check approval limit (only for approver role, admin/owner have unlimited)
        if user_role_enum == UserRole::Approver {
            if let Some(limit) = user_approval_limit {
                if transaction_amount > limit {
                    return Err(WorkflowError::ExceedsApprovalLimit {
                        amount: transaction_amount,
                        limit,
                    });
                }
            }
        }

        Ok(())
    }
}
```

### ReversalService (core/src/workflow/reversal.rs)

```rust
use rust_decimal::Decimal;
use uuid::Uuid;
use chrono::{NaiveDate, DateTime, Utc};

use crate::ledger::types::{LedgerEntryInput, EntryType, TransactionType};

#[derive(Debug)]
pub struct ReversalInput {
    pub original_transaction_id: Uuid,
    pub original_entries: Vec<OriginalEntry>,
    pub transaction_date: NaiveDate,
    pub fiscal_period_id: Uuid,
    pub voided_by: Uuid,
    pub void_reason: String,
}

#[derive(Debug, Clone)]
pub struct OriginalEntry {
    pub account_id: Uuid,
    pub source_currency: String,
    pub source_amount: Decimal,
    pub exchange_rate: Decimal,
    pub functional_amount: Decimal,
    pub debit: Decimal,
    pub credit: Decimal,
    pub memo: Option<String>,
    pub dimensions: Vec<Uuid>,
}

#[derive(Debug)]
pub struct ReversalOutput {
    pub reversing_transaction_id: Uuid,
    pub reversing_entries: Vec<LedgerEntryInput>,
    pub description: String,
}

pub struct ReversalService;

impl ReversalService {
    /// Create reversing entries by swapping debits and credits
    pub fn create_reversing_entries(input: &ReversalInput) -> ReversalOutput {
        let reversing_entries: Vec<LedgerEntryInput> = input
            .original_entries
            .iter()
            .map(|entry| {
                // Swap debit and credit
                let entry_type = if entry.debit > Decimal::ZERO {
                    EntryType::Credit
                } else {
                    EntryType::Debit
                };

                LedgerEntryInput {
                    account_id: entry.account_id,
                    source_currency: entry.source_currency.clone(),
                    source_amount: entry.source_amount,
                    entry_type,
                    memo: Some(format!(
                        "Reversal: {}",
                        entry.memo.clone().unwrap_or_default()
                    )),
                    dimensions: entry.dimensions.clone(),
                }
            })
            .collect();

        ReversalOutput {
            reversing_transaction_id: Uuid::new_v4(),
            reversing_entries,
            description: format!(
                "Reversal of transaction {}. Reason: {}",
                input.original_transaction_id, input.void_reason
            ),
        }
    }

    /// Validate that reversing entries will balance
    pub fn validate_reversal(original_entries: &[OriginalEntry]) -> bool {
        let total_debit: Decimal = original_entries.iter().map(|e| e.debit).sum();
        let total_credit: Decimal = original_entries.iter().map(|e| e.credit).sum();
        
        // Original must be balanced for reversal to be balanced
        total_debit == total_credit
    }
}
```

## Data Models

### Database Tables (existing, from DATABASE_SCHEMA.md)

The workflow uses existing tables:
- `transactions` - status, audit fields (submitted_at, approved_at, etc.)
- `ledger_entries` - linked to transactions
- `approval_rules` - configurable approval rules

### API Request/Response Models

```rust
// api/src/routes/workflow.rs

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;

#[derive(Debug, Deserialize)]
pub struct SubmitRequest {
    // No body needed, transaction_id from path
}

#[derive(Debug, Deserialize)]
pub struct ApproveRequest {
    pub approval_notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RejectRequest {
    pub rejection_reason: String,
}

#[derive(Debug, Deserialize)]
pub struct VoidRequest {
    pub void_reason: String,
}

#[derive(Debug, Deserialize)]
pub struct BulkApproveRequest {
    pub transaction_ids: Vec<Uuid>,
    pub approval_notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BulkApproveResponse {
    pub results: Vec<BulkApproveResult>,
}

#[derive(Debug, Serialize)]
pub struct BulkApproveResult {
    pub transaction_id: Uuid,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateApprovalRuleRequest {
    pub name: String,
    pub min_amount: Option<Decimal>,
    pub max_amount: Option<Decimal>,
    pub transaction_types: Vec<String>,
    pub required_role: String,
    pub priority: i16,
}

#[derive(Debug, Serialize)]
pub struct ApprovalRuleResponse {
    pub id: Uuid,
    pub name: String,
    pub min_amount: Option<Decimal>,
    pub max_amount: Option<Decimal>,
    pub transaction_types: Vec<String>,
    pub required_role: String,
    pub priority: i16,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub struct PendingTransactionResponse {
    pub id: Uuid,
    pub reference_number: Option<String>,
    pub transaction_type: String,
    pub transaction_date: String,
    pub description: String,
    pub total_amount: Decimal,
    pub submitted_by: Uuid,
    pub submitted_at: String,
    pub can_approve: bool,
}
```



## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Valid State Transitions

*For any* transaction in a given status, calling the corresponding workflow action SHALL result in the correct new status and audit fields being set. Specifically:
- draft + submit → pending (with submitted_at, submitted_by)
- pending + approve → approved (with approved_at, approved_by)
- pending + reject → draft (with rejection_reason in approval_notes)
- approved + post → posted (with posted_at, posted_by)
- posted + void → voided (with voided_at, voided_by, void_reason)

**Validates: Requirements 1.1, 1.2, 1.3, 1.4, 1.6**

### Property 2: Invalid State Transitions Rejected

*For any* transaction status and *for any* workflow action that is not a valid transition from that status, the Workflow_Service SHALL return an InvalidTransition error and the transaction status SHALL remain unchanged.

**Validates: Requirements 1.5, 1.6**

### Property 3: Void Creates Balanced Reversing Entry

*For any* posted transaction with balanced entries (total debits = total credits), voiding it SHALL create a reversing transaction where:
- Each original debit becomes a credit of the same amount
- Each original credit becomes a debit of the same amount
- The reversing transaction is also balanced

**Validates: Requirements 2.1, 2.7**

### Property 4: Void Creates Bidirectional Links

*For any* voided transaction, the original transaction's `reversed_by_transaction_id` SHALL equal the reversing transaction's `id`, AND the reversing transaction's `reverses_transaction_id` SHALL equal the original transaction's `id`.

**Validates: Requirements 2.2, 2.3**

### Property 5: Reversing Transaction Properties

*For any* reversing transaction created by a void operation:
- Its `transaction_type` SHALL be "reversal"
- Its `status` SHALL be "posted"
- Its `description` SHALL reference the original transaction

**Validates: Requirements 2.4, 2.6**

### Property 6: Posted/Voided Transactions Are Immutable

*For any* transaction with status "posted" or "voided", *any* attempt to update fields (except status change to voided for posted) or delete SHALL be rejected with the appropriate error (CannotModifyPosted or CannotModifyVoided).

**Validates: Requirements 4.1, 4.2, 4.3, 4.4**

### Property 7: Draft/Pending Transactions Are Mutable

*For any* transaction with status "draft" or "pending", updates to allowed fields SHALL succeed.

**Validates: Requirements 4.5**

### Property 8: Approval Rule Priority Ordering

*For any* set of approval rules where multiple rules match a transaction (by type and amount range), the rule with the lowest `priority` value SHALL be selected.

**Validates: Requirements 3.2, 3.3**

### Property 9: Role Hierarchy Enforcement

*For any* user role and required role, approval SHALL be allowed if and only if the user's role is greater than or equal to the required role in the hierarchy: viewer(0) < submitter(1) < approver(2) < accountant(3) < admin(4) < owner(5).

**Validates: Requirements 3.4, 3.6**

### Property 10: Approval Limit Enforcement

*For any* user with role "approver" and a defined approval_limit, attempting to approve a transaction with total_amount > approval_limit SHALL be rejected with ExceedsApprovalLimit error.

**Validates: Requirements 3.5**

### Property 11: Bulk Approval Partial Success

*For any* bulk approval request containing N transactions where M transactions fail validation, the response SHALL contain N results with exactly M failures and (N-M) successes, and the successful transactions SHALL be approved.

**Validates: Requirements 5.2, 5.3, 5.4**

## Error Handling

### Error Types and HTTP Status Codes

| Error | HTTP Status | Description |
|-------|-------------|-------------|
| InvalidTransition | 400 | Attempted invalid status transition |
| CannotModifyPosted | 400 | Attempted to modify posted transaction |
| CannotModifyVoided | 400 | Attempted to modify voided transaction |
| NotAuthorizedToApprove | 403 | User lacks approval permission |
| ExceedsApprovalLimit | 403 | Transaction exceeds user's limit |
| InsufficientRole | 403 | User role below required level |
| TransactionNotFound | 404 | Transaction ID not found |
| VoidReasonRequired | 400 | Void attempted without reason |
| RejectionReasonRequired | 400 | Reject attempted without reason |

### Error Response Format

```json
{
  "error": {
    "code": "INVALID_TRANSITION",
    "message": "Invalid status transition from draft to posted",
    "details": {
      "from_status": "draft",
      "to_status": "posted",
      "valid_transitions": ["pending"]
    }
  }
}
```

## Testing Strategy

### Property-Based Testing

Use `proptest` crate for property-based testing with minimum 100 iterations per property.

```rust
// Example property test structure
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// Feature: transaction-workflow, Property 1: Valid State Transitions
    #[test]
    fn prop_valid_state_transitions(
        status in prop::sample::select(vec![
            TransactionStatus::Draft,
            TransactionStatus::Pending,
            TransactionStatus::Approved,
            TransactionStatus::Posted,
        ]),
        user_id in any::<u128>().prop_map(|n| Uuid::from_u128(n)),
    ) {
        // Test implementation
    }
}
```

### Unit Tests

Unit tests for specific examples and edge cases:
- Empty rejection reason
- Empty void reason
- Boundary amounts for approval limits
- Role hierarchy edge cases (viewer trying to approve)

### Integration Tests

Integration tests for API endpoints:
- Full workflow: draft → pending → approved → posted
- Void flow with reversing entry verification
- Bulk approval with mixed results
- Approval queue filtering

### Test File Structure

```
backend/crates/core/src/workflow/
├── mod.rs
├── service.rs
├── service_props.rs      # Property tests for WorkflowService
├── types.rs
├── error.rs
├── approval.rs
├── approval_props.rs     # Property tests for ApprovalEngine
├── reversal.rs
└── reversal_props.rs     # Property tests for ReversalService

backend/crates/api/src/routes/
├── workflow.rs
└── workflow_test.rs      # Integration tests for API endpoints

backend/crates/db/tests/
└── workflow_test.rs      # Database integration tests
```
