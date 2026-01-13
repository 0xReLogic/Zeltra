import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from '../api/client'
import type {
  Budget,
  BudgetWithLines,
  GetBudgetsResponse,
  CreateBudgetRequest,
  CreateBudgetLinesRequest,
  UpdateBudgetRequest,
  BudgetVsActualResponse,
  BudgetLineItemResponse,
} from '@/types/budgets'

// Query keys for cache management
const BUDGET_KEYS = {
  all: ['budgets'] as const,
  lists: () => [...BUDGET_KEYS.all, 'list'] as const,
  list: (filters?: BudgetFilters) => [...BUDGET_KEYS.lists(), filters] as const,
  details: () => [...BUDGET_KEYS.all, 'detail'] as const,
  detail: (id: string) => [...BUDGET_KEYS.details(), id] as const,
  lines: (id: string) => [...BUDGET_KEYS.detail(id), 'lines'] as const,
  vsActual: (id: string) => [...BUDGET_KEYS.detail(id), 'vs-actual'] as const,
}

interface BudgetFilters {
  fiscal_year_id?: string
  is_active?: boolean
}

/**
 * GET /organizations/{org_id}/budgets
 * List budgets with summary totals
 */
export function useBudgets(filters?: BudgetFilters) {
  return useQuery({
    queryKey: BUDGET_KEYS.list(filters),
    queryFn: () => {
      const params = new URLSearchParams()
      if (filters?.fiscal_year_id) params.set('fiscal_year_id', filters.fiscal_year_id)
      if (filters?.is_active !== undefined) params.set('is_active', String(filters.is_active))
      const queryString = params.toString()
      return apiClient<GetBudgetsResponse>(`/budgets${queryString ? `?${queryString}` : ''}`)
    },
  })
}

/**
 * GET /organizations/{org_id}/budgets/{id}
 * Get budget with all lines
 */
export function useBudget(id: string) {
  return useQuery({
    queryKey: BUDGET_KEYS.detail(id),
    queryFn: () => apiClient<BudgetWithLines>(`/budgets/${id}`),
    enabled: !!id,
  })
}

/**
 * GET /organizations/{org_id}/budgets/{id}/lines
 * Get budget lines
 */
export function useBudgetLines(budgetId: string) {
  return useQuery({
    queryKey: BUDGET_KEYS.lines(budgetId),
    queryFn: () => apiClient<BudgetLineItemResponse[]>(`/budgets/${budgetId}/lines`),
    enabled: !!budgetId,
  })
}

/**
 * POST /organizations/{org_id}/budgets
 * Create a new budget
 */
export function useCreateBudget() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: CreateBudgetRequest) =>
      apiClient<Budget>('/budgets', {
        method: 'POST',
        body: JSON.stringify(data),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: BUDGET_KEYS.all })
    },
  })
}

/**
 * PUT /organizations/{org_id}/budgets/{id}
 * Update budget
 */
export function useUpdateBudget() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ id, data }: { id: string; data: UpdateBudgetRequest }) =>
      apiClient<Budget>(`/budgets/${id}`, {
        method: 'PUT',
        body: JSON.stringify(data),
      }),
    onSuccess: (_, { id }) => {
      queryClient.invalidateQueries({ queryKey: BUDGET_KEYS.detail(id) })
      queryClient.invalidateQueries({ queryKey: BUDGET_KEYS.lists() })
    },
  })
}

/**
 * POST /organizations/{org_id}/budgets/{id}/lines
 * Create budget lines in bulk
 */
export function useCreateBudgetLines() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ budgetId, data }: { budgetId: string; data: CreateBudgetLinesRequest }) =>
      apiClient<BudgetLineItemResponse[]>(`/budgets/${budgetId}/lines`, {
        method: 'POST',
        body: JSON.stringify(data),
      }),
    onSuccess: (_, { budgetId }) => {
      queryClient.invalidateQueries({ queryKey: BUDGET_KEYS.detail(budgetId) })
      queryClient.invalidateQueries({ queryKey: BUDGET_KEYS.lines(budgetId) })
    },
  })
}

/**
 * POST /organizations/{org_id}/budgets/{id}/lock
 * Lock budget (prevents further modifications)
 */
export function useLockBudget() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (budgetId: string) =>
      apiClient<Budget>(`/budgets/${budgetId}/lock`, {
        method: 'POST',
      }),
    onSuccess: (_, budgetId) => {
      queryClient.invalidateQueries({ queryKey: BUDGET_KEYS.detail(budgetId) })
      queryClient.invalidateQueries({ queryKey: BUDGET_KEYS.lists() })
    },
  })
}

/**
 * GET /organizations/{org_id}/budgets/{id}/vs-actual
 * Get budget vs actual comparison
 */
export function useBudgetVsActual(budgetId: string) {
  return useQuery({
    queryKey: BUDGET_KEYS.vsActual(budgetId),
    queryFn: () => apiClient<BudgetVsActualResponse>(`/budgets/${budgetId}/vs-actual`),
    enabled: !!budgetId,
  })
}
