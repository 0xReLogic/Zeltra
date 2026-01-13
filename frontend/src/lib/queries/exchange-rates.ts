'use client'

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from '@/lib/api/client'
import type {
  ExchangeRate,
  GetExchangeRatesResponse,
  CreateExchangeRateRequest,
  BulkImportRequest,
  BulkImportResponse,
} from '@/types/exchange-rates'

// Re-export types for backward compatibility
export type { ExchangeRate, CreateExchangeRateRequest }

// Query keys for cache management
const EXCHANGE_RATE_KEYS = {
  all: ['exchange-rates'] as const,
  list: (filters?: ExchangeRateFilters) => [...EXCHANGE_RATE_KEYS.all, 'list', filters] as const,
}

interface ExchangeRateFilters {
  from_currency?: string
  to_currency?: string
  effective_date?: string
}

/**
 * GET /organizations/{org_id}/exchange-rates
 * List exchange rates with optional filters
 */
export function useExchangeRates(filters?: ExchangeRateFilters) {
  return useQuery({
    queryKey: EXCHANGE_RATE_KEYS.list(filters),
    queryFn: () => {
      const params = new URLSearchParams()
      if (filters?.from_currency) params.set('from_currency', filters.from_currency)
      if (filters?.to_currency) params.set('to_currency', filters.to_currency)
      if (filters?.effective_date) params.set('effective_date', filters.effective_date)
      const queryString = params.toString()
      return apiClient<GetExchangeRatesResponse>(`/exchange-rates${queryString ? `?${queryString}` : ''}`)
    },
  })
}

/**
 * POST /organizations/{org_id}/exchange-rates
 * Create a new exchange rate
 */
export function useCreateExchangeRate() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: CreateExchangeRateRequest) =>
      apiClient<ExchangeRate>('/exchange-rates', {
        method: 'POST',
        body: JSON.stringify(data),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: EXCHANGE_RATE_KEYS.all })
    },
  })
}

/**
 * POST /organizations/{org_id}/exchange-rates/bulk
 * Bulk import exchange rates
 */
export function useBulkImportRates() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: BulkImportRequest) =>
      apiClient<BulkImportResponse>('/exchange-rates/bulk', {
        method: 'POST',
        body: JSON.stringify(data),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: EXCHANGE_RATE_KEYS.all })
    },
  })
}

// Alias for backward compatibility
export const useBulkImportExchangeRates = useBulkImportRates

/**
 * POST /organizations/{org_id}/exchange-rates/fetch
 * Fetch live rates from external API
 */
export function useFetchLiveRates() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: () =>
      apiClient('/exchange-rates/fetch', {
        method: 'POST',
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: EXCHANGE_RATE_KEYS.all })
    },
  })
}
