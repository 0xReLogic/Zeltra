//! Transaction workflow management for Zeltra.
//!
//! This module implements the transaction lifecycle state machine,
//! approval rules engine, and void/reversal operations.
//!
//! # Modules
//!
//! - `types` - Workflow domain types (TransactionStatus, WorkflowAction)
//! - `error` - Workflow-specific error types
//! - `service` - State transition logic
//! - `approval` - Approval rules engine
//! - `reversal` - Void and reversing entry creation

pub mod approval;
pub mod error;
pub mod reversal;
pub mod service;
pub mod types;

pub use approval::{ApprovalEngine, ApprovalRule, UserRole};
pub use error::WorkflowError;
pub use reversal::{OriginalEntry, ReversalInput, ReversalOutput, ReversalService};
pub use service::WorkflowService;
pub use types::{TransactionStatus, WorkflowAction};
