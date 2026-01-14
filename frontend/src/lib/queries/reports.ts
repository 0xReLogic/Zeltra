import { useQuery } from '@tanstack/react-query';
import { apiClient } from '../api/client';
import type { DimensionalReportResponse } from '@/types/dimensional-report';

export interface TrialBalanceItem {
  code: string
  name: string
  debit: string
  credit: string
  net_balance: string
  type: string
}

export interface TrialBalanceResponse {
  data: TrialBalanceItem[]
  total_debit: string
  total_credit: string
}

export interface ReportData {
  data: unknown
  summary?: unknown
}

export interface DimensionalReportData {
  dimension: string
  data: Array<{
      id: string
      name: string
      revenue: string
      expense: string
      net_profit: string
      breakdown: Array<{ account: string, amount: string }>
  }>
  summary: {
      global_revenue: string
      global_expense: string
      global_net: string
  }
}

export interface DimensionalReportParams {
  startDate: string;
  endDate: string;
  dimensionTypeId?: string;
  dimensionValueId?: string;
}

export function useTrialBalance() {
  return useQuery({
    queryKey: ['reports', 'trial-balance'],
    queryFn: () => apiClient<TrialBalanceResponse>('/reports/trial-balance'),
  })
}

export function useDimensionalReport(params: DimensionalReportParams) {
  return useQuery({
    queryKey: ['reports', 'dimensional', params],
    queryFn: () => {
      const searchParams = new URLSearchParams({
        from: params.startDate,
        to: params.endDate,
      });
      
      if (params.dimensionTypeId && params.dimensionTypeId !== 'all') {
        // Pass the dimension type CODE to backend
        searchParams.set('group_by', params.dimensionTypeId);
      } else {
        // Default to empty if no dimension selected
        searchParams.set('group_by', '');
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

export interface IncomeStatementItem {
  code: string
  name: string
  amount: string
}

export interface IncomeStatementResponse {
  data: {
    revenues: IncomeStatementItem[]
    expenses: IncomeStatementItem[]
    total_revenue: string
    total_expenses: string
    net_income: string
  }
}

export function useIncomeStatement() {
  return useQuery({
    queryKey: ['reports', 'income-statement'],
    queryFn: () => apiClient<IncomeStatementResponse>('/reports/income-statement'),
  })
}

export interface BalanceSheetItem {
  code: string
  name: string
  amount: string
}

export interface BalanceSheetResponse {
  data: {
    assets: BalanceSheetItem[]
    liabilities: BalanceSheetItem[]
    equity: BalanceSheetItem[]
    total_assets: string
    total_liabilities: string
    total_equity: string
  }
}

export function useBalanceSheet() {
  return useQuery({
    queryKey: ['reports', 'balance-sheet'],
    queryFn: () => apiClient<BalanceSheetResponse>('/reports/balance-sheet'),
  })
}
