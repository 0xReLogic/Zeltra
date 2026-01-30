'use client'

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from '@/lib/api/client'
import type {
  FiscalYear,
  FiscalPeriod,
  GetFiscalYearsResponse,
  CreateFiscalYearRequest,
  PeriodStatus,
} from '@/types/fiscal'
import { toBackendStatus } from '@/types/fiscal'

// Re-export types for backward compatibility
export type { FiscalYear, FiscalPeriod, CreateFiscalYearRequest }

// Query keys for cache management
const FISCAL_KEYS = {
  all: ['fiscal'] as const,
  years: () => [...FISCAL_KEYS.all, 'years'] as const,
  periods: () => [...FISCAL_KEYS.all, 'periods'] as const,
  periodsByYear: (yearId: string) => [...FISCAL_KEYS.periods(), yearId] as const,
}

/**
 * GET /organizations/{org_id}/fiscal-years
 * List all fiscal years with their periods
 */
export function useFiscalYears(entity_id?: string) {
  return useQuery({
    queryKey: [...FISCAL_KEYS.years(), entity_id],
    queryFn: () => {
      const params = new URLSearchParams()
      if (entity_id) params.set('entity_id', entity_id)  // NEW: Add entity filter
      return apiClient<GetFiscalYearsResponse>(
        `/fiscal-years${params.toString() ? `?${params.toString()}` : ''}`
      )
    },
  })
}

/**
 * Get fiscal periods extracted from fiscal years response.
 * Backend doesn't have a separate /fiscal-periods endpoint,
 * so we extract periods from the /fiscal-years response.
 */
export function useFiscalPeriods(fiscalYearId?: string, entity_id?: string) {
  return useQuery({
    queryKey: [...FISCAL_KEYS.periodsByYear(fiscalYearId || 'all'), entity_id],
    queryFn: async () => {
      // Fetch fiscal years which include nested periods
      const params = new URLSearchParams()
      if (entity_id) params.set('entity_id', entity_id)  // NEW: Add entity filter
      const fiscalYears = await apiClient<GetFiscalYearsResponse>(
        `/fiscal-years${params.toString() ? `?${params.toString()}` : ''}`
      )
      
      // Extract all periods from fiscal years
      let allPeriods: FiscalPeriod[] = []
      
      if (Array.isArray(fiscalYears)) {
        for (const year of fiscalYears) {
          if (year.periods && Array.isArray(year.periods)) {
            // Filter by fiscal year if specified
            if (fiscalYearId) {
              if (year.id === fiscalYearId) {
                allPeriods = [...allPeriods, ...year.periods]
              }
            } else {
              allPeriods = [...allPeriods, ...year.periods]
            }
          }
        }
      }
      
      return allPeriods
    },
  })
}

/**
 * POST /organizations/{org_id}/fiscal-years
 * Create a new fiscal year
 */
export function useCreateFiscalYear() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: CreateFiscalYearRequest) =>
      apiClient<FiscalYear>('/fiscal-years', {
        method: 'POST',
        body: JSON.stringify(data),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: FISCAL_KEYS.all })
    },
  })
}

/**
 * PATCH /organizations/{org_id}/fiscal-periods/{id}/status
 * Update fiscal period status
 */
export function useUpdatePeriodStatus() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ id, status }: { id: string; status: PeriodStatus }) =>
      apiClient<FiscalPeriod>(`/fiscal-periods/${id}/status`, {
        method: 'PATCH',
        body: JSON.stringify({ status: toBackendStatus(status) }),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: FISCAL_KEYS.all })
    },
  })
}
