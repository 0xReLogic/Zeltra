/**
 * Type helper utilities for extracting types from OpenAPI generated types.
 * Use these helpers to get request/response types from api.generated.ts
 */

import type { components, operations } from './api.generated'

// ============================================================================
// Schema Type Helpers
// ============================================================================

/**
 * Extract a schema type by name from components.schemas
 * @example type Account = Schema<'AccountResponse'>
 */
export type Schema<T extends keyof components['schemas']> = components['schemas'][T]

// ============================================================================
// Operation Type Helpers
// ============================================================================

/**
 * Extract request body type from an operation
 * @example type LoginRequest = RequestBody<'login'>
 */
export type RequestBody<T extends keyof operations> =
  operations[T] extends { requestBody: { content: { 'application/json': infer R } } }
    ? R
    : never

/**
 * Extract successful response type from an operation (200 or 201)
 * @example type LoginResponse = ResponseBody<'login'>
 */
export type ResponseBody<T extends keyof operations> =
  operations[T] extends { responses: { 200: { content: { 'application/json': infer R } } } }
    ? R
    : operations[T] extends { responses: { 201: { content: { 'application/json': infer R } } } }
      ? R
      : never

/**
 * Extract response type for a specific status code
 * @example type NotFoundError = ResponseForStatus<'get_account', 404>
 */
export type ResponseForStatus<
  T extends keyof operations,
  S extends number
> = operations[T] extends { responses: { [K in S]: { content: { 'application/json': infer R } } } }
  ? R
  : never

// ============================================================================
// Common Schema Types (Re-exports for convenience)
// ============================================================================

// Auth
export type LoginRequest = Schema<'LoginRequest'>
export type LoginResponse = Schema<'LoginResponse'>
export type RegisterRequest = Schema<'RegisterRequest'>
export type RefreshRequest = Schema<'RefreshRequest'>
export type LogoutRequest = Schema<'LogoutRequest'>

// Accounts
export type AccountResponse = Schema<'AccountResponse'>
export type AccountBalanceResponse = Schema<'AccountBalanceResponse'>
export type AccountLedgerResponse = Schema<'AccountLedgerResponse'>
export type CreateAccountRequest = Schema<'CreateAccountRequest'>
export type UpdateAccountRequest = Schema<'UpdateAccountRequest'>

// Transactions
export type TransactionResponse = Schema<'TransactionResponse'>
export type TransactionListItem = Schema<'TransactionListItem'>
export type CreateTransactionRequest = Schema<'CreateTransactionRequest'>
export type UpdateTransactionRequest = Schema<'UpdateTransactionRequest'>
export type CreateEntryRequest = Schema<'CreateEntryRequest'>
export type EntryResponse = Schema<'EntryResponse'>
export type LedgerEntryResponse = Schema<'LedgerEntryResponse'>
export type PayInvoiceRequest = Schema<'PayInvoiceRequest'>
export type PendingTransactionResponse = Schema<'PendingTransactionResponse'>
export type PaginatedTransactionsResponse = Schema<'PaginatedTransactionsResponse'>
export type PaginationMeta = Schema<'PaginationMeta'>

// Transaction Workflow
export type ApproveRequest = Schema<'ApproveRequest'>
export type RejectRequest = Schema<'RejectRequest'>
export type VoidRequest = Schema<'VoidRequest'>
export type VoidResponse = Schema<'VoidResponse'>
export type BulkApproveRequest = Schema<'BulkApproveRequest'>
export type BulkApproveResponse = Schema<'BulkApproveResponse'>
export type BulkApproveItemResponse = Schema<'BulkApproveItemResponse'>

// Budgets
export type BudgetResponse = Schema<'BudgetResponse'>
export type BudgetSummary = Schema<'BudgetSummary'>
export type BudgetLineItemResponse = Schema<'BudgetLineItemResponse'>
export type BudgetLineInput = Schema<'BudgetLineInput'>
export type CreateBudgetRequest = Schema<'CreateBudgetRequest'>
export type CreateBudgetLinesRequest = Schema<'CreateBudgetLinesRequest'>
export type UpdateBudgetRequest = Schema<'UpdateBudgetRequest'>
export type BudgetVsActualResponse = Schema<'BudgetVsActualResponse'>

// Dimensions
export type DimensionTypeResponse = Schema<'DimensionTypeResponse'>
export type DimensionValueResponse = Schema<'DimensionValueResponse'>
export type CreateDimensionTypeRequest = Schema<'CreateDimensionTypeRequest'>
export type CreateDimensionValueRequest = Schema<'CreateDimensionValueRequest'>
export type UpdateDimensionValueRequest = Schema<'UpdateDimensionValueRequest'>
export type ToggleDimensionValueStatusRequest = Schema<'ToggleDimensionValueStatusRequest'>

// Fiscal
export type FiscalYearResponse = Schema<'FiscalYearResponse'>
export type FiscalPeriodResponse = Schema<'FiscalPeriodResponse'>
export type CreateFiscalYearRequest = Schema<'CreateFiscalYearRequest'>
export type UpdatePeriodStatusRequest = Schema<'UpdatePeriodStatusRequest'>

// Exchange Rates
export type ExchangeRateResponse = Schema<'ExchangeRateResponse'>
export type CreateExchangeRateRequest = Schema<'CreateExchangeRateRequest'>
export type BulkImportRequest = Schema<'BulkImportRequest'>
export type BulkImportResponse = Schema<'BulkImportResponse'>
export type BulkRateItem = Schema<'BulkRateItem'>
export type FetchRatesRequest = Schema<'FetchRatesRequest'>
export type FetchRatesResponse = Schema<'FetchRatesResponse'>

// Simulation
export type RunSimulationRequest = Schema<'RunSimulationRequest'>
export type SimulationResponse = Schema<'SimulationResponse'>
export type AccountProjectionResponse = Schema<'AccountProjectionResponse'>

// Approval Rules
export type ApprovalRuleResponse = Schema<'ApprovalRuleResponse'>
export type CreateApprovalRuleRequest = Schema<'CreateApprovalRuleRequest'>
export type UpdateApprovalRuleRequest = Schema<'UpdateApprovalRuleRequest'>

// Attachments
export type AttachmentResponse = Schema<'AttachmentResponse'>
export type RequestUploadRequest = Schema<'RequestUploadRequest'>
export type RequestUploadResponse = Schema<'RequestUploadResponse'>
export type ConfirmUploadRequest = Schema<'ConfirmUploadRequest'>

// Organizations
export type OrganizationResponse = Schema<'OrganizationResponse'>
export type CreateOrganizationRequest = Schema<'CreateOrganizationRequest'>
export type UpdateOrganizationRequest = Schema<'UpdateOrganizationRequest'>
export type OrgUserResponse = Schema<'OrgUserResponse'>
export type AddUserRequest = Schema<'AddUserRequest'>
export type UpdateMemberRequest = Schema<'UpdateMemberRequest'>

// Reports
export type TrialBalanceResponse = Schema<'TrialBalanceResponse'>
export type IncomeStatementResponse = Schema<'IncomeStatementResponse'>
export type BalanceSheetResponse = Schema<'BalanceSheetResponse'>
export type DimensionalReportResponse = Schema<'DimensionalReportResponse'>

// Dashboard
export type DashboardMetricsResponse = Schema<'DashboardMetricsResponse'>
export type RecentActivityResponse = Schema<'RecentActivityResponse'>
export type ActivityItemResponse = Schema<'ActivityItemResponse'>
export type PendingApprovalsResponse = Schema<'PendingApprovalsResponse'>
export type CashFlowResponse = Schema<'CashFlowResponse'>
export type BurnRateResponse = Schema<'BurnRateResponse'>

// Currencies
export type CurrencyResponse = Schema<'CurrencyResponse'>

// Pagination
export type PaginationResponse = Schema<'PaginationResponse'>
export type PaginationInfo = Schema<'PaginationInfo'>

// Error
export type ApiError = Schema<'ApiError'>
