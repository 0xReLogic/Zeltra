'use client'

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from '@/lib/api/client'
import type {
  ExchangeRate,
  GetExchangeRatesResponse,
  CreateExchangeRateRequest,
  BulkImportRequest,
  BulkImportResponse,
  FetchRatesRequest,
  FetchRatesResponse,
  GetCurrenciesResponse,
  Currency,
} from '@/types/exchange-rates'

// Re-export types for backward compatibility
export type { ExchangeRate, CreateExchangeRateRequest, FetchRatesRequest, Currency }

// Query keys for cache management
const EXCHANGE_RATE_KEYS = {
  all: ['exchange-rates'] as const,
  list: (filters?: ExchangeRateFilters) => [...EXCHANGE_RATE_KEYS.all, 'list', filters] as const,
  currencies: () => ['currencies'] as const,
}

interface ExchangeRateFilters {
  from_currency?: string | null
  to_currency?: string | null
  start_date?: string | null
  end_date?: string | null
  page?: number
  per_page?: number
}

/**
 * GET /organizations/{org_id}/exchange-rates/list
 * List exchange rates with optional filters
 */
export function useExchangeRates(filters?: ExchangeRateFilters) {
  return useQuery({
    queryKey: EXCHANGE_RATE_KEYS.list(filters),
    queryFn: () => {
      const params = new URLSearchParams()
      // All params are nullable - send empty string for null
      if (filters?.from_currency) params.set('from', filters.from_currency)
      if (filters?.to_currency) params.set('to', filters.to_currency)
      if (filters?.start_date) params.set('start_date', filters.start_date)
      if (filters?.end_date) params.set('end_date', filters.end_date)
      if (filters?.page) params.set('page', String(filters.page))
      if (filters?.per_page) params.set('per_page', String(filters.per_page))
      const queryString = params.toString()
      return apiClient<GetExchangeRatesResponse>(`/exchange-rates/list${queryString ? `?${queryString}` : ''}`)
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
    mutationFn: (data: FetchRatesRequest) =>
      apiClient<FetchRatesResponse>('/exchange-rates/fetch', {
        method: 'POST',
        body: JSON.stringify(data),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: EXCHANGE_RATE_KEYS.all })
    },
  })
}


/**
 * GET /currencies
 * List all available currencies
 */
export function useCurrencies() {
  return useQuery({
    queryKey: EXCHANGE_RATE_KEYS.currencies(),
    queryFn: () => 
      apiClient<GetCurrenciesResponse>('/currencies', { skipOrgPrefix: true }),
    staleTime: 1000 * 60 * 60, // Cache for 1 hour - currencies rarely change
  })
}
