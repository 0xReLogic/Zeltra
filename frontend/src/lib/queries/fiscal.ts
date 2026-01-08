'use client'

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from '@/lib/api/client'

export interface FiscalPeriod {
  id: string
  name: string
  status: 'open' | 'closed' | 'locked'
  start_date: string
  end_date: string
}

export interface FiscalYear {
  id: string
  name: string
  status: 'open' | 'closed'
  start_date: string
  end_date: string
  periods: FiscalPeriod[]
}

export function useFiscalYears() {
  return useQuery({
    queryKey: ['fiscal-years'],
    queryFn: () => apiClient<{ data: FiscalYear[] }>('/fiscal-years'),
  })
}

export interface CreateFiscalYearRequest {
  name: string
  start_date: string
  end_date: string
}

export function useCreateFiscalYear() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (data: CreateFiscalYearRequest) =>
      apiClient<FiscalYear>('/fiscal-years', {
        method: 'POST',
        body: JSON.stringify(data)
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['fiscal-years'] })
    }
  })
}

export function useUpdatePeriodStatus() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ id, status }: { id: string, status: 'open' | 'closed' | 'locked' }) =>
      apiClient(`/fiscal-periods/${id}/status`, {
        method: 'PATCH',
        body: JSON.stringify({ status })
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['fiscal-years'] })
    }
  })
}
