'use client'

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from '@/lib/api/client'

export interface ExchangeRate {
  id: string
  from_currency: string
  to_currency: string
  rate: string
  date: string
}

export interface CreateExchangeRateRequest {
  from_currency: string
  to_currency: string
  rate: string
  date: string
}

export interface GetExchangeRatesResponse {
  data: ExchangeRate[]
}

export function useExchangeRates() {
  return useQuery({
    queryKey: ['exchange-rates'],
    queryFn: () => apiClient<GetExchangeRatesResponse>('/exchange-rates'),
  })
}

export function useCreateExchangeRate() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (data: CreateExchangeRateRequest) =>
      apiClient<ExchangeRate>('/exchange-rates', {
        method: 'POST',
        body: JSON.stringify(data),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['exchange-rates'] })
    },
  })
}

export function useBulkImportExchangeRates() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (data: { rates: CreateExchangeRateRequest[] }) =>
      apiClient('/exchange-rates/bulk', {
        method: 'POST',
        body: JSON.stringify(data),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['exchange-rates'] })
    },
  })
}
