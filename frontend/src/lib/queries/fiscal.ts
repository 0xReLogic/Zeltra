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
export function useFiscalYears() {
  return useQuery({
    queryKey: FISCAL_KEYS.years(),
    queryFn: () => apiClient<GetFiscalYearsResponse>('/fiscal-years'),
  })
}

/**
 * GET /organizations/{org_id}/fiscal-periods
 * List fiscal periods, optionally filtered by year
 */
export function useFiscalPeriods(fiscalYearId?: string) {
  return useQuery({
    queryKey: FISCAL_KEYS.periodsByYear(fiscalYearId || 'all'),
    queryFn: () => {
      const params = new URLSearchParams()
      if (fiscalYearId) params.set('fiscal_year_id', fiscalYearId)
      const queryString = params.toString()
      return apiClient<FiscalPeriod[]>(`/fiscal-periods${queryString ? `?${queryString}` : ''}`)
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
        body: JSON.stringify({ status }),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: FISCAL_KEYS.all })
    },
  })
}
