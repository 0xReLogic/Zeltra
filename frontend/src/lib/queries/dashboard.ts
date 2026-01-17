import { useQuery } from '@tanstack/react-query'
import { apiClient } from '../api/client'
import type { RecentActivityResponse } from '@/types/api-helpers'

export interface DashboardMetrics {
  cash_position: {
    balance: string
    currency: string
    change_percent: number
  }
  burn_rate: {
    daily: string
    monthly: string
  }
  runway_days: number
  pending_approvals: {
    count: number
    total_amount: string
  }
}

export function useDashboardMetrics() {
  return useQuery({
    queryKey: ['dashboard', 'metrics'],
    queryFn: () => apiClient<DashboardMetrics>('/dashboard/metrics'),
  })
}

// Raw response from backend (Decimal as string)
export interface CashFlowDataPointRaw {
  month: string
  period_name: string
  inflow: string
  outflow: string
  net: string
}

// Parsed for chart consumption
export interface CashFlowDataPoint {
  month: string
  period_name: string
  inflow: number
  outflow: number
  net: number
}

export interface CashFlowResponse {
  data: CashFlowDataPointRaw[]
}

export function useCashFlowData() {
  return useQuery({
    queryKey: ['dashboard', 'cash-flow'],
    queryFn: async () => {
      const response = await apiClient<CashFlowResponse>('/dashboard/cash-flow')
      // Parse string decimals to numbers for chart rendering
      return (response.data || []).map(point => ({
        month: point.month,
        period_name: point.period_name,
        inflow: parseFloat(point.inflow) || 0,
        outflow: parseFloat(point.outflow) || 0,
        net: parseFloat(point.net) || 0,
      }))
    },
  })
}

// TODO: Move types to lib/api/types.ts once generated
// REMOVED: Custom ActivityResponse interface - now using generated RecentActivityResponse type

export function useRecentActivity() {
    return useQuery({
        queryKey: ['dashboard', 'recent-activity'],
        queryFn: () => apiClient<RecentActivityResponse>('/dashboard/recent-activity'),
        refetchInterval: 30000 // Real-time feed, refresh every 30s
    })
}

// Budget vs Actual types
export interface BudgetSummary {
    total_budgeted: string
    total_actual: string
    variance: string
    variance_percent: number
}

export interface BudgetLineItem {
    account_id: string
    account_code: string
    account_name: string
    budgeted: string
    actual: string
    variance: string
    variance_percent: number
}

export interface BudgetVsActualResponse {
    budget_id: string | null
    budget_name: string | null
    summary: BudgetSummary
    line_items: BudgetLineItem[]
}

export function useBudgetVsActual(budgetId?: string) {
    return useQuery({
        queryKey: ['dashboard', 'budget-vs-actual', budgetId],
        queryFn: () => apiClient<BudgetVsActualResponse>(
            `/dashboard/budget-vs-actual${budgetId ? `?budget_id=${budgetId}` : ''}`
        ),
    })
}
