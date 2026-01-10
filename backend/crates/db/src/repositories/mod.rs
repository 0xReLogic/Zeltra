//! Repository abstractions for data access.
//!
//! Repositories provide a clean interface for database operations,
//! hiding the `SeaORM` implementation details from the rest of the application.

pub mod account;
pub mod approval_rule;
pub mod budget;
pub mod dashboard;
pub mod dimension;
pub mod email_verification;
pub mod exchange_rate;
pub mod fiscal;
pub mod organization;
pub mod report;
pub mod session;
pub mod simulation;
pub mod subscription;
pub mod transaction;
pub mod user;
pub mod workflow;

#[cfg(test)]
mod workflow_integration_tests;

#[cfg(test)]
mod budget_integration_tests;

#[cfg(test)]
mod report_integration_tests;

#[cfg(test)]
mod simulation_integration_tests;

#[cfg(test)]
mod dashboard_integration_tests;

pub use account::{
    AccountError, AccountFilter, AccountRepository, AccountWithBalance, CreateAccountInput,
    UpdateAccountInput,
};
pub use approval_rule::{
    ApprovalRuleError, ApprovalRuleRepository, CreateApprovalRuleInput, UpdateApprovalRuleInput,
};
pub use budget::{
    ActualAmountResult, BudgetError, BudgetLineWithActual, BudgetLineWithDimensions,
    BudgetRepository, BudgetVsActualSummary, BudgetWithSummary, CreateBudgetInput,
    CreateBudgetLineInput, DimensionValueInfo, UpdateBudgetInput, UpdateBudgetLineInput,
    calculate_actual_by_account_type, is_debit_normal_account,
};
pub use dashboard::{
    ActivityEvent, ActivityPagination, BudgetStatus, BurnRate, CashPosition, CurrencyExposure,
    DashboardError, DashboardRepository, DepartmentExpense, PendingApprovals,
};
pub use dimension::{
    CreateDimensionTypeInput, CreateDimensionValueInput, DimensionError, DimensionRepository,
    DimensionTypeFilter, DimensionValueFilter, UpdateDimensionTypeInput, UpdateDimensionValueInput,
};
pub use email_verification::EmailVerificationRepository;
pub use exchange_rate::{
    CreateExchangeRateInput, ExchangeRateError, ExchangeRateLookup, ExchangeRateRepository,
    RateLookupMethod,
};
pub use fiscal::{CreateFiscalYearInput, FiscalError, FiscalRepository, FiscalYearWithPeriods};
pub use organization::{OrganizationError, OrganizationRepository};
pub use report::{
    AccountBalance, AccountLedgerEntry, DimensionInfo, DimensionalReportRow, ReportError,
    ReportRepository, calculate_balance, is_debit_normal,
};
pub use session::SessionRepository;
pub use simulation::{HistoricalAccountData, SimulationRepoError, SimulationRepository};
pub use subscription::{Feature, LimitCheckResult, ResourceLimit, SubscriptionRepository};
pub use transaction::{
    CreateLedgerEntryInput, CreateTransactionInput, LedgerEntryWithDimensions, TransactionError,
    TransactionFilter, TransactionRepository, TransactionWithEntries,
};
pub use user::UserRepository;
pub use workflow::{
    BulkApproveItemResult, BulkApproveResult, PendingTransaction, VoidResult, WorkflowRepository,
};
