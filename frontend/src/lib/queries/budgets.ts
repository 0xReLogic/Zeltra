import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from '../api/client'

export interface BudgetLine {
  id: string
  account_name: string
  limit: string
  actual: string
}

export interface Budget {
  id: string
  department: string
  budget_limit: string
  actual_spent: string
  period: string
  lines?: BudgetLine[]
}

export interface GetBudgetsResponse {
  data: Budget[]
}

export interface CreateBudgetRequest {
  department: string
  budget_limit: string
  period: string
}

export interface AddBudgetLineRequest {
  account_name: string
  limit: string
}

export function useBudgets() {
  return useQuery({
    queryKey: ['budgets'],
    queryFn: () => apiClient<GetBudgetsResponse>('/budgets'),
  })
}

export function useBudget(id: string) {
  return useQuery({
    queryKey: ['budgets', id],
    queryFn: () => apiClient<Budget>(`/budgets/${id}`),
    enabled: !!id
  })
}

export function useCreateBudget() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (data: CreateBudgetRequest) => 
      apiClient<Budget>('/budgets', {
        method: 'POST',
        body: JSON.stringify(data)
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['budgets'] })
    }
  })
}

export function useAddBudgetLine() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ budgetId, data }: { budgetId: string, data: AddBudgetLineRequest }) =>
      apiClient<BudgetLine>(`/budgets/${budgetId}/lines`, {
        method: 'POST',
        body: JSON.stringify(data)
      }),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ['budgets', variables.budgetId] })
    }
  })
}
