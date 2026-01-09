# Requirements Document

## Introduction

This document defines the requirements for the Transaction Workflow feature in Zeltra - a B2B Expense & Budgeting Engine. The Transaction Workflow manages the complete lifecycle of financial transactions from draft creation through posting, including approval processes and void operations with reversing entries. This feature ensures immutable audit trails and enforces proper authorization controls.

## Glossary

- **Transaction**: A financial record containing one or more ledger entries that must balance (debits = credits)
- **Transaction_Status**: The current state of a transaction in the workflow (draft, pending, approved, posted, voided)
- **Ledger_Entry**: A single line item within a transaction representing a debit or credit to an account
- **Reversing_Entry**: A new transaction that exactly reverses all debits and credits of an original posted transaction
- **Approval_Rule**: A configurable rule that determines which role can approve transactions based on amount thresholds and transaction types
- **Approval_Limit**: The maximum transaction amount a user with approver role can approve
- **Workflow_Service**: The core service responsible for managing transaction status transitions
- **Approval_Engine**: The component that evaluates approval rules and determines required approvers
- **Immutable_Ledger**: The principle that posted transactions cannot be modified, only voided via reversing entries
- **Audit_Trail**: The complete history of who performed what action and when on a transaction

## Requirements

### Requirement 1: Transaction Status Transitions

**User Story:** As an accountant, I want to move transactions through a defined workflow, so that proper review and approval processes are followed before posting to the ledger.

#### Acceptance Criteria

1. WHEN a user submits a draft transaction, THE Workflow_Service SHALL change the status from draft to pending and record submitted_at and submitted_by
2. WHEN an authorized approver approves a pending transaction, THE Workflow_Service SHALL change the status from pending to approved and record approved_at, approved_by, and optional approval_notes
3. WHEN an authorized approver rejects a pending transaction, THE Workflow_Service SHALL change the status from pending to draft and record the rejection_reason
4. WHEN an authorized user posts an approved transaction, THE Workflow_Service SHALL change the status from approved to posted and record posted_at and posted_by
5. WHEN a user attempts an invalid status transition, THE Workflow_Service SHALL reject the operation and return a descriptive error
6. THE Workflow_Service SHALL enforce the following valid transitions: draft→pending, pending→approved, pending→draft (reject), approved→posted, posted→voided

### Requirement 2: Void Transaction with Reversing Entry

**User Story:** As an accountant, I want to void posted transactions by creating reversing entries, so that I can correct errors while maintaining a complete audit trail.

#### Acceptance Criteria

1. WHEN a user voids a posted transaction, THE Workflow_Service SHALL create a new reversing transaction with all debits and credits swapped
2. WHEN creating a reversing transaction, THE Workflow_Service SHALL link the original transaction to the reversing transaction via reversed_by_transaction_id
3. WHEN creating a reversing transaction, THE Workflow_Service SHALL link the reversing transaction to the original via reverses_transaction_id
4. WHEN a reversing transaction is created, THE Workflow_Service SHALL set its transaction_type to reversal
5. WHEN a transaction is voided, THE Workflow_Service SHALL record voided_at, voided_by, and void_reason
6. WHEN a reversing transaction is created, THE Workflow_Service SHALL automatically post it (status = posted) in the same database transaction
7. WHEN a void operation completes, THE Workflow_Service SHALL ensure account balances are correctly adjusted by the reversing entries

### Requirement 3: Approval Rules Engine

**User Story:** As an organization admin, I want to configure approval rules based on amount thresholds and transaction types, so that appropriate authorization levels are enforced.

#### Acceptance Criteria

1. WHEN creating an approval rule, THE Approval_Engine SHALL accept min_amount, max_amount, transaction_types array, required_role, and priority
2. WHEN evaluating a transaction for approval, THE Approval_Engine SHALL match rules by transaction_type and amount range
3. WHEN multiple rules match a transaction, THE Approval_Engine SHALL select the rule with the lowest priority value (highest priority)
4. WHEN checking if a user can approve, THE Approval_Engine SHALL verify the user's role meets or exceeds the required_role in the hierarchy
5. WHEN checking if a user can approve, THE Approval_Engine SHALL verify the transaction amount does not exceed the user's approval_limit
6. THE Approval_Engine SHALL use the role hierarchy: viewer < submitter < approver < accountant < admin < owner

### Requirement 4: Immutability Enforcement

**User Story:** As an auditor, I want posted and voided transactions to be immutable, so that the integrity of the financial records is guaranteed.

#### Acceptance Criteria

1. WHEN a user attempts to update a posted transaction (except to void it), THE Workflow_Service SHALL reject the operation with CannotModifyPosted error
2. WHEN a user attempts to delete a posted transaction, THE Workflow_Service SHALL reject the operation with CannotModifyPosted error
3. WHEN a user attempts to update a voided transaction, THE Workflow_Service SHALL reject the operation with CannotModifyVoided error
4. WHEN a user attempts to delete a voided transaction, THE Workflow_Service SHALL reject the operation with CannotModifyVoided error
5. THE Workflow_Service SHALL only allow modification of transactions in draft or pending status

### Requirement 5: Approval Queue and Bulk Operations

**User Story:** As an approver, I want to view pending transactions and approve multiple at once, so that I can efficiently process approval requests.

#### Acceptance Criteria

1. WHEN an approver requests the approval queue, THE Workflow_Service SHALL return all pending transactions the user is authorized to approve
2. WHEN an approver requests bulk approval, THE Workflow_Service SHALL validate each transaction individually before approving
3. WHEN bulk approving transactions, THE Workflow_Service SHALL return success/failure status for each transaction
4. IF any transaction in a bulk approval fails validation, THE Workflow_Service SHALL continue processing remaining transactions and report individual failures

### Requirement 6: Workflow API Endpoints

**User Story:** As a frontend developer, I want REST API endpoints for all workflow operations, so that I can build the approval and posting UI.

#### Acceptance Criteria

1. WHEN a client calls POST /transactions/:id/submit, THE API SHALL transition the transaction from draft to pending
2. WHEN a client calls POST /transactions/:id/approve, THE API SHALL transition the transaction from pending to approved
3. WHEN a client calls POST /transactions/:id/reject with rejection_reason, THE API SHALL transition the transaction from pending to draft
4. WHEN a client calls POST /transactions/:id/post, THE API SHALL transition the transaction from approved to posted
5. WHEN a client calls POST /transactions/:id/void with void_reason, THE API SHALL void the posted transaction and create a reversing entry
6. WHEN a client calls GET /transactions/pending, THE API SHALL return the approval queue for the authenticated user
7. WHEN a client calls POST /transactions/bulk-approve with transaction_ids array, THE API SHALL approve multiple transactions
8. WHEN a client calls POST /approval-rules, THE API SHALL create a new approval rule
9. WHEN a client calls GET /approval-rules, THE API SHALL return all approval rules for the organization

### Requirement 7: Audit Trail Completeness

**User Story:** As a compliance officer, I want complete audit trails for all workflow actions, so that I can trace who did what and when.

#### Acceptance Criteria

1. WHEN any workflow action is performed, THE Workflow_Service SHALL record the user_id and timestamp
2. WHEN a transaction is submitted, THE Workflow_Service SHALL store submitted_at and submitted_by
3. WHEN a transaction is approved, THE Workflow_Service SHALL store approved_at, approved_by, and approval_notes
4. WHEN a transaction is rejected, THE Workflow_Service SHALL store the rejection in approval_notes and reset submitted_at/submitted_by
5. WHEN a transaction is posted, THE Workflow_Service SHALL store posted_at and posted_by
6. WHEN a transaction is voided, THE Workflow_Service SHALL store voided_at, voided_by, and void_reason
