import { useQuery } from '@tanstack/react-query'
import { apiClient } from '../api/client'

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

export interface CashFlowDataPoint {
  month: string
  inflow: number
  outflow: number
}

// TODO: Add mock endpoint for this if not exists
export function useCashFlowData() {
  return useQuery({
    queryKey: ['dashboard', 'cash-flow'],
    queryFn: () => apiClient<CashFlowDataPoint[]>('/dashboard/cash-flow'),
  })
}

// TODO: Move types to lib/api/types.ts once generated
interface ActivityResponse {
    activities: {
        id: string
        type: string
        action: string
        entity_type: string
        entity_id: string
        description: string
        amount?: string
        currency?: string
        user: { id: string, full_name: string }
        timestamp: string
    }[]
    pagination: {
        limit: number
        has_more: boolean
        next_cursor: string | null
    }
}

export function useRecentActivity() {
    return useQuery({
        queryKey: ['dashboard', 'recent-activity'],
        queryFn: () => apiClient<ActivityResponse>('/dashboard/recent-activity'),
        refetchInterval: 30000 // Real-time feed, refresh every 30s
    })
}
