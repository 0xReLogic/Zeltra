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
  entity_id?: string;  // NEW: Entity filter
  consolidated?: boolean;  // NEW: Consolidated mode
}

/**
 * GET /organizations/{org_id}/reports/trial-balance
 * Trial balance report
 * Returns { report_type, as_of, currency, accounts: AccountBalanceResponse[], totals: TrialBalanceTotals }
 */
export function useTrialBalance(entity_id?: string, consolidated?: boolean) {
  return useQuery({
    queryKey: [...REPORT_KEYS.trialBalance(), entity_id, consolidated],
    queryFn: () => {
      const params = new URLSearchParams()
      if (entity_id) params.set('entity_id', entity_id)  // NEW: Add entity filter
      if (consolidated) params.set('consolidated', 'true')  // NEW: Add consolidated mode
      return apiClient<TrialBalanceResponse>(
        `/reports/trial-balance${params.toString() ? `?${params.toString()}` : ''}`
      )
    },
  })
}

/**
 * GET /organizations/{org_id}/reports/balance-sheet
 * Balance sheet report
 * Returns { report_type, as_of, currency, assets, liabilities, equity, total_assets, total_liabilities_and_equity, is_balanced }
 */
export function useBalanceSheet(entity_id?: string, consolidated?: boolean) {
  return useQuery({
    queryKey: [...REPORT_KEYS.balanceSheet(), entity_id, consolidated],
    queryFn: () => {
      const params = new URLSearchParams()
      if (entity_id) params.set('entity_id', entity_id)  // NEW: Add entity filter
      if (consolidated) params.set('consolidated', 'true')  // NEW: Add consolidated mode
      return apiClient<BalanceSheetResponse>(
        `/reports/balance-sheet${params.toString() ? `?${params.toString()}` : ''}`
      )
    },
  })
}

/**
 * GET /organizations/{org_id}/reports/income-statement
 * Income statement report
 * Returns { report_type, period_start, period_end, currency, revenue, cost_of_goods_sold, gross_profit, operating_expenses, operating_income, other_income_expenses, net_income }
 */
export function useIncomeStatement(entity_id?: string, consolidated?: boolean) {
  return useQuery({
    queryKey: [...REPORT_KEYS.incomeStatement(), entity_id, consolidated],
    queryFn: () => {
      const params = new URLSearchParams()
      if (entity_id) params.set('entity_id', entity_id)  // NEW: Add entity filter
      if (consolidated) params.set('consolidated', 'true')  // NEW: Add consolidated mode
      return apiClient<IncomeStatementResponse>(
        `/reports/income-statement${params.toString() ? `?${params.toString()}` : ''}`
      )
    },
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
      
      if (params.entity_id) {
        searchParams.set('entity_id', params.entity_id);  // NEW: Add entity filter
      }
      
      if (params.consolidated) {
        searchParams.set('consolidated', 'true');  // NEW: Add consolidated mode
      }
      
      return apiClient<DimensionalReportResponse>(
        `/reports/dimensional?${searchParams.toString()}`
      );
    },
    enabled: !!params.startDate && !!params.endDate && !!params.dimensionTypeId,
  });
}
