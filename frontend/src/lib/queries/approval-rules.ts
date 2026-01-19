import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from '../api/client'
import type { components } from '@/types/api.generated'

// Type definitions from OpenAPI
type ApprovalRuleResponse = components['schemas']['ApprovalRuleResponse']
type CreateApprovalRuleRequest = components['schemas']['CreateApprovalRuleRequest']
type UpdateApprovalRuleRequest = components['schemas']['UpdateApprovalRuleRequest']
type PaginatedApprovalRulesResponse = components['schemas']['PaginatedApprovalRulesResponse']

// Query keys for cache management
export const APPROVAL_RULE_KEYS = {
  all: ['approval-rules'] as const,
  lists: () => [...APPROVAL_RULE_KEYS.all, 'list'] as const,
  list: (filters: ApprovalRuleFilters) => [...APPROVAL_RULE_KEYS.lists(), filters] as const,
  details: () => [...APPROVAL_RULE_KEYS.all, 'detail'] as const,
  detail: (id: string) => [...APPROVAL_RULE_KEYS.details(), id] as const,
}

interface ApprovalRuleFilters {
  page?: number
  per_page?: number
  is_active?: boolean
  transaction_type?: string
  required_role?: string
  sort_by?: string
  sort_order?: 'asc' | 'desc'
}

/**
 * GET /organizations/{org_id}/approval-rules
 * List approval rules with pagination and filters
 */
export function useApprovalRules(filters: ApprovalRuleFilters = {}) {
  const { page = 1, per_page = 20, is_active, transaction_type, required_role, sort_by, sort_order } = filters

  return useQuery({
    queryKey: APPROVAL_RULE_KEYS.list(filters),
    queryFn: () => {
      const params = new URLSearchParams()
      params.set('page', page.toString())
      params.set('per_page', per_page.toString())
      if (is_active !== undefined) params.set('is_active', is_active.toString())
      if (transaction_type) params.set('transaction_type', transaction_type)
      if (required_role) params.set('required_role', required_role)
      if (sort_by) params.set('sort_by', sort_by)
      if (sort_order) params.set('sort_order', sort_order)
      
      return apiClient<PaginatedApprovalRulesResponse>(`/approval-rules?${params.toString()}`)
    },
  })
}

/**
 * GET /organizations/{org_id}/approval-rules/{rule_id}
 * Get single approval rule
 */
export function useApprovalRule(id: string) {
  return useQuery({
    queryKey: APPROVAL_RULE_KEYS.detail(id),
    queryFn: () => apiClient<ApprovalRuleResponse>(`/approval-rules/${id}`),
    enabled: !!id,
  })
}

/**
 * POST /organizations/{org_id}/approval-rules
 * Create a new approval rule
 */
export function useCreateApprovalRule() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: CreateApprovalRuleRequest) =>
      apiClient<ApprovalRuleResponse>('/approval-rules', {
        method: 'POST',
        body: JSON.stringify(data),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: APPROVAL_RULE_KEYS.all })
    },
  })
}

/**
 * PATCH /organizations/{org_id}/approval-rules/{rule_id}
 * Update an approval rule
 */
export function useUpdateApprovalRule() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ id, data }: { id: string; data: UpdateApprovalRuleRequest }) =>
      apiClient<ApprovalRuleResponse>(`/approval-rules/${id}`, {
        method: 'PATCH',
        body: JSON.stringify(data),
      }),
    onSuccess: (_, { id }) => {
      queryClient.invalidateQueries({ queryKey: APPROVAL_RULE_KEYS.detail(id) })
      queryClient.invalidateQueries({ queryKey: APPROVAL_RULE_KEYS.lists() })
    },
  })
}

/**
 * PATCH /organizations/{org_id}/approval-rules/{rule_id}
 * Toggle active status with optimistic update
 */
export function useToggleApprovalRuleStatus() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ id, is_active }: { id: string; is_active: boolean }) =>
      apiClient<ApprovalRuleResponse>(`/approval-rules/${id}`, {
        method: 'PATCH',
        body: JSON.stringify({ is_active }),
      }),
    // Optimistic update
    onMutate: async ({ id, is_active }) => {
      // Cancel outgoing refetches
      await queryClient.cancelQueries({ queryKey: APPROVAL_RULE_KEYS.lists() })
      await queryClient.cancelQueries({ queryKey: APPROVAL_RULE_KEYS.detail(id) })

      // Snapshot previous values
      const previousLists = queryClient.getQueriesData({ queryKey: APPROVAL_RULE_KEYS.lists() })
      const previousDetail = queryClient.getQueryData(APPROVAL_RULE_KEYS.detail(id))

      // Optimistically update list cache
      queryClient.setQueriesData<PaginatedApprovalRulesResponse>(
        { queryKey: APPROVAL_RULE_KEYS.lists() },
        (old) => {
          if (!old) return old
          return {
            ...old,
            data: old.data.map((rule) =>
              rule.id === id ? { ...rule, is_active } : rule
            ),
          }
        }
      )

      // Optimistically update detail cache
      queryClient.setQueryData<ApprovalRuleResponse>(
        APPROVAL_RULE_KEYS.detail(id),
        (old) => {
          if (!old) return old
          return { ...old, is_active }
        }
      )

      // Return context with previous values for rollback
      return { previousLists, previousDetail }
    },
    // Rollback on error
    onError: (err, { id }, context) => {
      if (context?.previousLists) {
        context.previousLists.forEach(([queryKey, data]) => {
          queryClient.setQueryData(queryKey, data)
        })
      }
      if (context?.previousDetail) {
        queryClient.setQueryData(APPROVAL_RULE_KEYS.detail(id), context.previousDetail)
      }
    },
    // Refetch on success or error
    onSettled: (_, __, { id }) => {
      queryClient.invalidateQueries({ queryKey: APPROVAL_RULE_KEYS.detail(id) })
      queryClient.invalidateQueries({ queryKey: APPROVAL_RULE_KEYS.lists() })
    },
  })
}

/**
 * DELETE /organizations/{org_id}/approval-rules/{rule_id}
 * Delete an approval rule with optimistic update
 */
export function useDeleteApprovalRule() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (id: string) =>
      apiClient<void>(`/approval-rules/${id}`, {
        method: 'DELETE',
      }),
    // Optimistic update
    onMutate: async (id) => {
      // Cancel outgoing refetches
      await queryClient.cancelQueries({ queryKey: APPROVAL_RULE_KEYS.lists() })

      // Snapshot previous values
      const previousLists = queryClient.getQueriesData({ queryKey: APPROVAL_RULE_KEYS.lists() })

      // Optimistically remove from list cache
      queryClient.setQueriesData<PaginatedApprovalRulesResponse>(
        { queryKey: APPROVAL_RULE_KEYS.lists() },
        (old) => {
          if (!old) return old
          return {
            ...old,
            data: old.data.filter((rule) => rule.id !== id),
            meta: {
              ...old.meta,
              total: old.meta.total - 1,
            },
          }
        }
      )

      // Return context with previous values for rollback
      return { previousLists }
    },
    // Rollback on error
    onError: (err, id, context) => {
      if (context?.previousLists) {
        context.previousLists.forEach(([queryKey, data]) => {
          queryClient.setQueryData(queryKey, data)
        })
      }
    },
    // Refetch on success or error
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: APPROVAL_RULE_KEYS.all })
    },
  })
}