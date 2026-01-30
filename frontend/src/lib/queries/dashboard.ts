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

export function useDashboardMetrics(entity_id?: string, consolidated?: boolean) {
  return useQuery({
    queryKey: ['dashboard', 'metrics', entity_id, consolidated],
    queryFn: () => {
      const params = new URLSearchParams()
      if (entity_id) params.set('entity_id', entity_id)  // NEW: Add entity filter
      if (consolidated) params.set('consolidated', 'true')  // NEW: Add consolidated mode
      return apiClient<DashboardMetrics>(
        `/dashboard/metrics${params.toString() ? `?${params.toString()}` : ''}`
      )
    },
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

export function useCashFlowData(entity_id?: string, consolidated?: boolean) {
  return useQuery({
    queryKey: ['dashboard', 'cash-flow', entity_id, consolidated],
    queryFn: async () => {
      const params = new URLSearchParams()
      if (entity_id) params.set('entity_id', entity_id)  // NEW: Add entity filter
      if (consolidated) params.set('consolidated', 'true')  // NEW: Add consolidated mode
      const response = await apiClient<CashFlowResponse>(
        `/dashboard/cash-flow${params.toString() ? `?${params.toString()}` : ''}`
      )
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

export function useRecentActivity(entity_id?: string) {
    return useQuery({
        queryKey: ['dashboard', 'recent-activity', entity_id],
        queryFn: () => {
            const params = new URLSearchParams()
            if (entity_id) params.set('entity_id', entity_id)  // NEW: Add entity filter
            return apiClient<RecentActivityResponse>(
                `/dashboard/recent-activity${params.toString() ? `?${params.toString()}` : ''}`
            )
        },
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

export function useBudgetVsActual(budgetId?: string, entity_id?: string) {
    return useQuery({
        queryKey: ['dashboard', 'budget-vs-actual', budgetId, entity_id],
        queryFn: () => {
            const params = new URLSearchParams()
            if (budgetId) params.set('budget_id', budgetId)
            if (entity_id) params.set('entity_id', entity_id)  // NEW: Add entity filter
            return apiClient<BudgetVsActualResponse>(
                `/dashboard/budget-vs-actual${params.toString() ? `?${params.toString()}` : ''}`
            )
        },
    })
}
