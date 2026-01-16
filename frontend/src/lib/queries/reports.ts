import { useQuery } from '@tanstack/react-query';
import { apiClient } from '../api/client';
import type { 
  TrialBalanceResponse,
  BalanceSheetResponse,
  IncomeStatementResponse,
  DimensionalReportResponse,
} from '@/types/api-helpers';

// Query keys for cache management
const REPORT_KEYS = {
  all: ['reports'] as const,
  trialBalance: () => [...REPORT_KEYS.all, 'trial-balance'] as const,
  balanceSheet: () => [...REPORT_KEYS.all, 'balance-sheet'] as const,
  incomeStatement: () => [...REPORT_KEYS.all, 'income-statement'] as const,
  dimensional: (params: DimensionalReportParams) => [...REPORT_KEYS.all, 'dimensional', params] as const,
}

export interface DimensionalReportParams {
  startDate: string;
  endDate: string;
  dimensionTypeId?: string;
  dimensionValueId?: string;
}

/**
 * GET /organizations/{org_id}/reports/trial-balance
 * Trial balance report
 * Returns { report_type, as_of, currency, accounts: AccountBalanceResponse[], totals: TrialBalanceTotals }
 */
export function useTrialBalance() {
  return useQuery({
    queryKey: REPORT_KEYS.trialBalance(),
    queryFn: () => apiClient<TrialBalanceResponse>('/reports/trial-balance'),
  })
}

/**
 * GET /organizations/{org_id}/reports/balance-sheet
 * Balance sheet report
 * Returns { report_type, as_of, currency, assets, liabilities, equity, total_assets, total_liabilities_and_equity, is_balanced }
 */
export function useBalanceSheet() {
  return useQuery({
    queryKey: REPORT_KEYS.balanceSheet(),
    queryFn: () => apiClient<BalanceSheetResponse>('/reports/balance-sheet'),
  })
}

/**
 * GET /organizations/{org_id}/reports/income-statement
 * Income statement report
 * Returns { report_type, period_start, period_end, currency, revenue, cost_of_goods_sold, gross_profit, operating_expenses, operating_income, other_income_expenses, net_income }
 */
export function useIncomeStatement() {
  return useQuery({
    queryKey: REPORT_KEYS.incomeStatement(),
    queryFn: () => apiClient<IncomeStatementResponse>('/reports/income-statement'),
  })
}

/**
 * GET /organizations/{org_id}/reports/dimensional
 * Dimensional report
 * Returns { report_type, period_start, period_end, currency, group_by, rows, grand_total }
 */
export function useDimensionalReport(params: DimensionalReportParams) {
  return useQuery({
    queryKey: REPORT_KEYS.dimensional(params),
    queryFn: () => {
      const searchParams = new URLSearchParams({
        from: params.startDate,
        to: params.endDate,
      });
      
      if (params.dimensionTypeId && params.dimensionTypeId !== 'all') {
        searchParams.set('group_by', params.dimensionTypeId);
      } else {
        // group_by is required, use a default
        searchParams.set('group_by', 'department');
      }
      
      if (params.dimensionValueId) {
        searchParams.set('dimensions', params.dimensionValueId);
      }
      
      return apiClient<DimensionalReportResponse>(
        `/reports/dimensional?${searchParams.toString()}`
      );
    },
    enabled: !!params.startDate && !!params.endDate && !!params.dimensionTypeId,
  });
}
