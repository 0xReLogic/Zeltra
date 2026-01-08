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

export function useTrialBalance() {
  return useQuery({
    queryKey: ['reports', 'trial-balance'],
    queryFn: () => apiClient<TrialBalanceResponse>('/reports/trial-balance'),
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
