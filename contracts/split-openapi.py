#!/usr/bin/env python3
"""
Split OpenAPI spec into manageable chunks for auditing.
"""

import yaml
import json
from pathlib import Path

# Read full OpenAPI spec
with open('openapi.yaml', 'r') as f:
    spec = yaml.safe_load(f)

# Create output directory
output_dir = Path('openapi-split')
output_dir.mkdir(exist_ok=True)

# Schema groupings by domain
schema_groups = {
    '01-auth-org': [
        'AddUserRequest', 'CreateOrganizationRequest', 'LoginRequest', 'LoginResponse',
        'LogoutRequest', 'ResendVerificationRequest', 'ResendVerificationResponse', 
        'VerifyEmailRequest', 'VerifyEmailResponse',
        'OrganizationResponse', 'OrganizationUserResponse', 'OrgUserResponse', 'MembershipResponse',
        'RefreshRequest', 'RefreshResponse', 'RegisterRequest', 'RegisterResponse', 
        'TierLimitsResponse', 'UpdateOrganizationRequest', 'UpdateUserRoleRequest', 'UpdateMemberRequest',
        'UserInfo', 'UserResponse', 'UserOrganization'
    ],
    '02-transactions': [
        'CreateTransactionRequest', 'CreateEntryRequest', 'TransactionResponse', 'TransactionListItem',
        'UpdateTransactionRequest', 'ApproveRequest', 'RejectRequest', 'VoidRequest', 'VoidResponse',
        'BulkApproveRequest', 'BulkApproveResponse', 'BulkApproveItemResponse',
        'PayInvoiceRequest', 'EntryResponse',
        # Pagination schemas for transactions
        'PaginatedTransactionsResponse', 'PaginationMeta', 'PendingTransactionResponse'
    ],
    '03-accounts-ledger': [
        'AccountResponse', 'CreateAccountRequest', 'UpdateAccountRequest', 'ToggleAccountStatusRequest',
        'AccountLedgerResponse', 'AccountBalanceResponse', 'GetAccountsResponse'
    ],
    '04-budgets': [
        'BudgetResponse', 'CreateBudgetRequest', 'UpdateBudgetRequest', 'CreateBudgetLinesRequest', 
        'BudgetLineInput', 'LockBudgetRequest',
        'BudgetVsActualResponse', 'BudgetLineItemResponse', 'BudgetSummary'
    ],
    '05-reports': [
        'TrialBalanceResponse', 'TrialBalanceTotals', 'BalanceSheetResponse', 'BalanceSheetSectionResponse',
        'IncomeStatementResponse', 'IncomeStatementSectionResponse', 'DimensionalReportResponse',
        'DimensionalReportItem', 'DimensionalReportRowResponse',
        # LedgerEntryResponse is used in account ledger reports
        'LedgerEntryResponse'
    ],
    '06-sentinel': [
        'AccrualScheduleResponse', 'CreateAccrualScheduleRequest', 'RevaluationLogResponse',
        'IntercompanyMappingResponse', 'CreateIntercompanyMappingRequest', 'ComplianceMetadata'
    ],
    '07-forensic': [
        'ReconciliationResponse', 'ReconciliationAccount', 'BenfordResponse', 'BenfordRecord',
        'HealthScoreResponse', 'AltmanDetails', 'BeneishDetails'
    ],
    '08-master-data': [
        'FiscalYearResponse', 'FiscalPeriodResponse', 'CreateFiscalYearRequest', 'PeriodInfo',
        'UpdatePeriodStatusRequest', 'DimensionTypeResponse', 'DimensionValueResponse',
        'CreateDimensionTypeRequest', 'CreateDimensionValueRequest', 'ToggleDimensionStatusRequest',
        'ToggleDimensionValueStatusRequest', 'UpdateDimensionValueRequest',
        'ExchangeRateResponse', 'ExchangeRateListItem', 'CreateExchangeRateRequest', 
        'BulkImportRequest', 'BulkImportResponse', 'BulkRateItem', 'BulkImportError', 
        'FetchRatesRequest', 'FetchRatesResponse', 'FetchedRateItem', 'CurrencyResponse'
    ],
    '09-dashboard': [
        'DashboardMetricsResponse', 'CashPositionResponse', 'BurnRateResponse',
        'CashFlowResponse', 'CashFlowDataPoint', 'ActivityItemResponse', 'RecentActivityResponse',
        # PendingApprovalsResponse is a dashboard widget schema
        'PendingApprovalsResponse',
        # DashboardUserInfo is used in dashboard metrics
        'DashboardUserInfo'
    ],
    '10-simulation-attachments': [
        'SimulationRequest', 'RunSimulationRequest', 'SimulationResponse', 'MonthlySummaryResponse',
        'AccountProjectionResponse', 'AnnualSummaryResponse',
        'AttachmentResponse', 'RequestUploadRequest', 'RequestUploadResponse', 'ConfirmUploadRequest'
    ],
    '11-common': [
        'ApiError', 'PaginationResponse', 'PaginationInfo',
        'PageMeta', 'PageRequest', 'PageResponse_ExchangeRateListItem',
        'SuccessResponse', 'HealthResponse', 'ToggleStatusRequest'
    ],
    '12-approval-rules': [
        'ApprovalRuleResponse', 'CreateApprovalRuleRequest', 'UpdateApprovalRuleRequest'
    ]
}

# Split schemas
schemas = spec.get('components', {}).get('schemas', {})

for group_name, schema_names in schema_groups.items():
    group_schemas = {}
    for name in schema_names:
        if name in schemas:
            group_schemas[name] = schemas[name]
    
    if group_schemas:
        output = {
            'components': {
                'schemas': group_schemas
            }
        }
        
        output_file = output_dir / f'{group_name}-schemas.yaml'
        with open(output_file, 'w') as f:
            yaml.dump(output, f, default_flow_style=False, sort_keys=False, allow_unicode=True)
        
        print(f'✅ Created {output_file} ({len(group_schemas)} schemas)')

# Split paths by domain
paths = spec.get('paths', {})

path_groups = {
    '13-auth-endpoints': ['/auth/login', '/auth/register', '/auth/refresh', '/auth/logout', '/auth/verify-email', '/auth/resend-verification'],
    '14-org-endpoints': [
        '/organizations', 
        '/organizations/{org_id}', 
        '/organizations/{org_id}/users',
        '/organizations/{org_id}/users/{user_id}'
    ],
    '15-transaction-endpoints': [
        '/organizations/{org_id}/transactions',
        '/organizations/{org_id}/transactions/{transaction_id}',
        '/organizations/{org_id}/transactions/{transaction_id}/submit',
        '/organizations/{org_id}/transactions/{transaction_id}/approve',
        '/organizations/{org_id}/transactions/{transaction_id}/reject',
        '/organizations/{org_id}/transactions/{transaction_id}/post',
        '/organizations/{org_id}/transactions/{transaction_id}/void',
        '/organizations/{org_id}/transactions/pending',
        '/organizations/{org_id}/transactions/bulk-approve',
        '/organizations/{org_id}/transactions/pay-invoice'
    ],
    '16-accounts-endpoints': [
        '/organizations/{org_id}/accounts',
        '/organizations/{org_id}/accounts/{account_id}',
        '/organizations/{org_id}/accounts/{account_id}/ledger',
        '/organizations/{org_id}/accounts/{account_id}/balance',
        '/organizations/{org_id}/accounts/{account_id}/status'
    ],
    '17-fiscal-endpoints': [
        '/organizations/{org_id}/fiscal-years',
        '/organizations/{org_id}/fiscal-periods',
        '/organizations/{org_id}/fiscal-periods/{period_id}/status'
    ],
    '18-dimensions-endpoints': [
        '/organizations/{org_id}/dimension-types',
        '/organizations/{org_id}/dimension-types/{type_id}/values',
        '/organizations/{org_id}/dimension-values',
        '/organizations/{org_id}/dimension-values/{value_id}',
        '/organizations/{org_id}/dimension-values/{value_id}/status'
    ],
    '19-exchange-rates-endpoints': [
        '/organizations/{org_id}/exchange-rates',
        '/organizations/{org_id}/exchange-rates/list',
        '/organizations/{org_id}/exchange-rates/bulk',
        '/organizations/{org_id}/exchange-rates/fetch',
        '/currencies'
    ],
    '20-budgets-endpoints': [
        '/organizations/{org_id}/budgets',
        '/organizations/{org_id}/budgets/{budget_id}',
        '/organizations/{org_id}/budgets/{budget_id}/lines',
        '/organizations/{org_id}/budgets/{budget_id}/lock',
        '/organizations/{org_id}/budgets/{budget_id}/vs-actual'
    ],
    '21-reports-endpoints': [
        '/organizations/{org_id}/reports/trial-balance',
        '/organizations/{org_id}/reports/balance-sheet',
        '/organizations/{org_id}/reports/income-statement',
        '/organizations/{org_id}/reports/dimensional'
    ],
    '22-sentinel-endpoints': [
        '/organizations/{org_id}/revaluation-logs',
        '/organizations/{org_id}/accrual-schedules',
        '/organizations/{org_id}/accrual-schedules/{schedule_id}',
        '/organizations/{org_id}/intercompany/connect',
        '/organizations/{org_id}/intercompany/mappings'
    ],
    '23-forensic-endpoints': [
        '/organizations/{org_id}/forensic/reconciliation',
        '/organizations/{org_id}/forensic/benford',
        '/organizations/{org_id}/forensic/health-score'
    ],
    '24-dashboard-endpoints': [
        '/organizations/{org_id}/dashboard/metrics',
        '/organizations/{org_id}/dashboard/cash-flow',
        '/organizations/{org_id}/dashboard/recent-activity',
        '/organizations/{org_id}/dashboard/budget-vs-actual'
    ],
    '25-simulation-endpoints': [
        '/organizations/{org_id}/simulation/run'
    ],
    '26-attachments-endpoints': [
        '/organizations/{org_id}/attachments/{attachment_id}',
        '/organizations/{org_id}/transactions/{transaction_id}/attachments',
        '/organizations/{org_id}/transactions/{transaction_id}/attachments/upload'
    ],
    '27-approval-rules-endpoints': [
        '/organizations/{org_id}/approval-rules',
        '/organizations/{org_id}/approval-rules/{rule_id}'
    ],
    '28-health-endpoints': [
        '/health'
    ],
    '99-other-endpoints': []  # Catch remaining
}

# Collect all defined paths
defined_paths = set()
for group_paths in path_groups.values():
    defined_paths.update(group_paths)

# Add undefined paths to "other"
for path in paths.keys():
    if path not in defined_paths:
        path_groups['99-other-endpoints'].append(path)

# Write path groups
for group_name, path_list in path_groups.items():
    group_paths = {}
    for path in path_list:
        if path in paths:
            group_paths[path] = paths[path]
    
    if group_paths:
        output = {
            'paths': group_paths
        }
        
        output_file = output_dir / f'{group_name}.yaml'
        with open(output_file, 'w') as f:
            yaml.dump(output, f, default_flow_style=False, sort_keys=False, allow_unicode=True)
        
        print(f'✅ Created {output_file} ({len(group_paths)} endpoints)')

print(f'\n🎉 OpenAPI split complete! Check {output_dir}/')
