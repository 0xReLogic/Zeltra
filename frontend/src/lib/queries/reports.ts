import { useQuery } from '@tanstack/react-query'
import { apiClient } from '../api/client'

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
  startDate: string
  endDate: string
  dimension: string
  values?: string[]
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
                start_date: params.startDate,
                end_date: params.endDate,
                dimension: params.dimension
            })
            // Handle array of values logic if needed, for mock simplistic passing
            return apiClient<DimensionalReportData>(`/reports/dimensional?${searchParams.toString()}`)
        }
    })
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
