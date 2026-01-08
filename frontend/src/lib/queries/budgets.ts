import { useQuery } from '@tanstack/react-query'
import { apiClient } from '../api/client'

export interface Budget {
  id: string
  department: string
  budget_limit: string
  actual_spent: string
  period: string
}

export interface GetBudgetsResponse {
  data: Budget[]
}

export function useBudgets() {
  return useQuery({
    queryKey: ['budgets'],
    queryFn: () => apiClient<GetBudgetsResponse>('/budgets'),
  })
}
