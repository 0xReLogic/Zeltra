#![allow(clippy::needless_for_each)]
//! API route definitions.

use axum::{Router, middleware};

use crate::{AppState, middleware::auth::auth_middleware};

pub mod accounts;
pub mod approval_rules;
pub mod attachments;
pub mod auth;
pub mod budgets;
pub mod currencies;
pub mod dashboard;
pub mod dimensions;
pub mod entities;
pub mod exchange_rates;
pub mod fiscal;
pub mod forensic;
pub mod health;
pub mod organizations;
pub mod reports;
pub mod sentinel;
pub mod simulation;
pub mod transactions;

use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        health::health_check,
        auth::login,
        auth::register,
        auth::refresh,
        auth::logout,
        auth::verify_email,
        auth::resend_verification,
        auth::switch_organization,
        organizations::create_organization,
        organizations::get_organization,
        organizations::update_organization,
        organizations::list_users,
        organizations::add_user,
        organizations::remove_user,
        organizations::update_member,
        entities::list_entities,
        entities::create_entity,
        entities::get_entity,
        entities::update_entity,
        entities::delete_entity,
        accounts::list_accounts,
        accounts::create_account,
        accounts::get_account,
        accounts::update_account,
        accounts::delete_account,
        accounts::toggle_account_status,
        accounts::get_account_balance,
        transactions::list_transactions,
        transactions::create_transaction,
        transactions::get_transaction,
        transactions::update_transaction,
        transactions::delete_transaction,
        transactions::submit_transaction,
        transactions::approve_transaction,
        transactions::reject_transaction,
        transactions::post_transaction,
        transactions::void_transaction,
        transactions::pay_invoice,
        transactions::get_pending_transactions,
        transactions::bulk_approve_transactions,
        budgets::list_budgets,
        budgets::create_budget,
        budgets::get_budget,
        budgets::update_budget,
        budgets::list_budget_lines,
        budgets::create_budget_lines,
        budgets::lock_budget,
        budgets::get_budget_vs_actual,
        currencies::list_currencies,
        fiscal::list_fiscal_years,
        fiscal::create_fiscal_year,
        fiscal::update_period_status,
        dimensions::list_dimension_types,
        dimensions::create_dimension_type,
        dimensions::list_dimension_values,
        dimensions::create_dimension_value,
        dimensions::update_dimension_value,
        dimensions::toggle_dimension_value_status,
        exchange_rates::get_exchange_rate,
        exchange_rates::create_exchange_rate,
        exchange_rates::fetch_exchange_rates,
        exchange_rates::bulk_import_rates,
        exchange_rates::list_exchange_rates,
        attachments::request_upload,
        attachments::confirm_upload,
        attachments::list_attachments,
        attachments::get_attachment,
        attachments::delete_attachment,
        approval_rules::list_approval_rules,
        approval_rules::create_approval_rule,
        approval_rules::get_approval_rule,
        approval_rules::update_approval_rule,
        approval_rules::delete_approval_rule,
        reports::get_trial_balance,
        reports::get_balance_sheet,
        reports::get_income_statement,
        reports::get_dimensional_report,
        reports::get_account_ledger,
        dashboard::get_dashboard_metrics,
        dashboard::get_recent_activity,
        dashboard::get_cash_flow,
        dashboard::get_dashboard_budget_vs_actual,
        simulation::run_simulation,
        sentinel::list_revaluation_logs,
        sentinel::list_accrual_schedules,
        sentinel::create_accrual_schedule,
        sentinel::get_accrual_schedule,
        sentinel::list_intercompany_mappings,
        sentinel::create_intercompany_mapping,
        forensic::get_benford,
        forensic::get_health_score,
    ),
    components(
        schemas(
            crate::error::ApiError,
            zeltra_shared::types::pagination::PageRequest,
            zeltra_shared::auth::LoginRequest, zeltra_shared::auth::LoginResponse,
            zeltra_shared::auth::RegisterRequest, zeltra_shared::auth::RegisterResponse,
            zeltra_shared::auth::RefreshRequest, zeltra_shared::auth::RefreshResponse,
            zeltra_shared::auth::LogoutRequest,
            zeltra_shared::auth::VerifyEmailRequest, zeltra_shared::auth::VerifyEmailResponse,
            zeltra_shared::auth::ResendVerificationRequest, zeltra_shared::auth::ResendVerificationResponse,
            zeltra_shared::auth::SwitchOrganizationRequest, zeltra_shared::auth::SwitchOrganizationResponse,
            zeltra_shared::auth::UserInfo, zeltra_shared::auth::UserOrganization,
            organizations::OrganizationResponse,
            organizations::OrgUserResponse,
            organizations::MembershipResponse,
            zeltra_shared::auth::CreateOrganizationRequest,
            zeltra_shared::auth::UpdateOrganizationRequest,
            zeltra_shared::auth::AddUserRequest,
            zeltra_shared::auth::UpdateMemberRequest,
            entities::CreateEntityRequest,
            entities::UpdateEntityRequest,
            entities::EntityResponse,
            entities::GetEntitiesResponse,
            accounts::CreateAccountRequest,
            accounts::UpdateAccountRequest,
            accounts::ToggleStatusRequest,
            accounts::AccountResponse,
            transactions::CreateTransactionRequest,
            transactions::CreateEntryRequest,
            transactions::UpdateTransactionRequest,
            transactions::CreateTransactionRequest,
            transactions::CreateEntryRequest,
            transactions::UpdateTransactionRequest,
            transactions::TransactionResponse,
            transactions::EntryResponse,
            transactions::TransactionListItem,
            transactions::PaginatedTransactionsResponse,
            transactions::PaginationMeta,
            transactions::ApproveRequest,
            transactions::RejectRequest,
            transactions::VoidRequest,
            transactions::PayInvoiceRequest,
            transactions::BulkApproveRequest,
            transactions::VoidResponse,
            transactions::BulkApproveResponse,
            transactions::BulkApproveItemResponse,
            transactions::PendingTransactionResponse,
            budgets::CreateBudgetRequest,
            budgets::UpdateBudgetRequest,
            budgets::CreateBudgetLinesRequest,
            budgets::BudgetLineInput,
            budgets::BudgetResponse,
            currencies::CurrencyResponse,
            fiscal::CreateFiscalYearRequest,
            fiscal::UpdatePeriodStatusRequest,
            fiscal::FiscalPeriodResponse,
            fiscal::FiscalYearResponse,
            dimensions::CreateDimensionTypeRequest,
            dimensions::CreateDimensionValueRequest,
            dimensions::UpdateDimensionValueRequest,
            dimensions::ToggleDimensionValueStatusRequest,
            dimensions::DimensionTypeResponse,
            dimensions::DimensionValueResponse,
            exchange_rates::CreateExchangeRateRequest,
            exchange_rates::ExchangeRateResponse,
            exchange_rates::FetchRatesRequest,
            exchange_rates::FetchRatesResponse,
            exchange_rates::FetchedRateItem,
            exchange_rates::BulkImportRequest,
            exchange_rates::BulkRateItem,
            exchange_rates::ExchangeRateListItem,
            exchange_rates::BulkImportResponse,
            exchange_rates::BulkImportError,
            attachments::RequestUploadRequest,
            attachments::RequestUploadResponse,
            attachments::ConfirmUploadRequest,
            attachments::AttachmentResponse,
            approval_rules::CreateApprovalRuleRequest,
            approval_rules::UpdateApprovalRuleRequest,
            approval_rules::ApprovalRuleResponse,
            reports::TrialBalanceResponse,
            reports::AccountBalanceResponse,
            reports::TrialBalanceTotals,
            reports::BalanceSheetResponse,
            reports::BalanceSheetSectionResponse,
            reports::IncomeStatementResponse,
            reports::IncomeStatementSectionResponse,
            reports::DimensionalReportResponse,
            reports::DimensionalReportRowResponse,
            reports::DimensionValueResponse,
            reports::AccountLedgerResponse,
            reports::LedgerEntryResponse,
            reports::PaginationResponse,
            dashboard::DashboardMetricsResponse,
            dashboard::PeriodInfo,
            dashboard::CashPositionResponse,
            dashboard::BurnRateResponse,
            dashboard::PendingApprovalsResponse,
            dashboard::RecentActivityResponse,
            dashboard::ActivityItemResponse,
            dashboard::DashboardUserInfo,
            dashboard::PaginationInfo,
            dashboard::CashFlowResponse,
            dashboard::CashFlowDataPoint,
            dashboard::BudgetVsActualResponse,
            dashboard::BudgetSummary,
            dashboard::BudgetLineItemResponse,
            simulation::RunSimulationRequest,
            simulation::SimulationResponse,
            simulation::AccountProjectionResponse,
            simulation::AnnualSummaryResponse,
            simulation::MonthlySummaryResponse,
            sentinel::CreateAccrualScheduleRequest,
            sentinel::AccrualScheduleResponse,
            sentinel::RevaluationLogResponse,
            sentinel::IntercompanyMappingResponse,
            sentinel::CreateIntercompanyMappingRequest,
            organizations::TierLimitsResponse,
            forensic::BenfordResponse,
            forensic::HealthScoreResponse,
        )
    ),
    tags(
        (name = "Auth", description = "Authentication endpoints"),
        (name = "Health", description = "Health check endpoints"),
        (name = "Organizations", description = "Organization management endpoints"),
        (name = "Entities", description = "Entity management endpoints"),
        (name = "Accounts", description = "Account management endpoints"),
        (name = "Transactions", description = "Transaction management endpoints"),
        (name = "Budgets", description = "Budget management endpoints"),
        (name = "Currencies", description = "Currency listing endpoints"),
        (name = "Fiscal", description = "Fiscal year and period management endpoints"),
        (name = "Dimensions", description = "Dimension management endpoints"),
        (name = "Exchange Rates", description = "Exchange rate management endpoints"),
        (name = "Attachments", description = "Attachment management endpoints"),
        (name = "Approval Rules", description = "Approval rule management endpoints"),
        (name = "Reports", description = "Financial report generation endpoints"),
        (name = "Dashboard", description = "Dashboard and analytics endpoints"),
        (name = "Simulation", description = "Scenario planning and projection endpoints"),
        (name = "Sentinel", description = "Sentinel Intelligence - Revaluation, Accruals, Intercompany"),
        (name = "Forensic", description = "AI-driven forensic analysis (Benford, Z-Score)")
    ),
    modifiers(&SecurityAddon)
)]
/// OpenAPI documentation aggregator for the Zeltra API.
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearerAuth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

/// Creates the API router with all routes.
pub fn api_routes() -> Router<AppState> {
    Router::new().merge(health::routes()).merge(auth::routes())
}

/// Creates the API router with protected routes that need state for middleware.
#[allow(clippy::needless_pass_by_value)]
pub fn api_routes_with_state(state: AppState) -> Router<AppState> {
    // Protected routes that require authentication
    let protected_routes = Router::new()
        .merge(organizations::routes())
        .merge(entities::routes())
        .merge(fiscal::routes())
        .merge(accounts::routes())
        .merge(dimensions::routes())
        .merge(exchange_rates::routes())
        .merge(currencies::routes())
        .merge(transactions::routes())
        .merge(approval_rules::routes())
        .merge(budgets::routes())
        .merge(reports::routes())
        .merge(simulation::routes())
        .merge(dashboard::routes())
        .merge(attachments::routes())
        .merge(sentinel::routes())
        .merge(forensic::routes())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::check_subscription_status,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // Combine public and protected routes
    Router::new()
        .merge(health::routes())
        .merge(auth::routes())
        .merge(protected_routes)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
}
