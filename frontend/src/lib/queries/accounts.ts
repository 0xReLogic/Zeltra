import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from '../api/client'
import type { Account, CreateAccountRequest, GetAccountsResponse } from '@/types/accounts'

export function useAccounts(type?: string) {
  return useQuery({
    queryKey: ['accounts', { type }],
    queryFn: () => apiClient<GetAccountsResponse>(
      `/accounts${type ? `?type=${type}` : ''}`
    ),
  })
}

export function useCreateAccount() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: CreateAccountRequest) =>
      apiClient<Account>('/accounts', {
        method: 'POST',
        body: JSON.stringify(data),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['accounts'] })
    },
  })
}

export function useAccount(id: string) {
  return useQuery({
    queryKey: ['accounts', id],
    queryFn: () => apiClient<Account>(`/accounts/${id}`),
    enabled: !!id,
  })
}

export type UpdateAccountRequest = Partial<CreateAccountRequest> & { id: string }

export function useUpdateAccount() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ id, ...data }: UpdateAccountRequest) =>
      apiClient<Account>(`/accounts/${id}`, {
        method: 'PUT',
        body: JSON.stringify(data),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['accounts'] })
    },
  })
}

// ...existing code...
export interface LedgerEntry {
  id: string
  transaction_date: string
  reference_number: string
  description: string
  debit: string
  credit: string
  running_balance: string
}

export interface GetLedgerResponse {
  data: LedgerEntry[]
  pagination: {
    page: number
    limit: number
    total: number
  }
}

export function useAccountLedger(id: string, params?: { page?: number; limit?: number; from?: string; to?: string }) {
  return useQuery({
    queryKey: ['account-ledger', id, params],
    queryFn: () => {
      const queryParams = new URLSearchParams()
      if (params?.page) queryParams.set('page', params.page.toString())
      if (params?.limit) queryParams.set('limit', params.limit.toString())
      if (params?.from) queryParams.set('from', params.from)
      if (params?.to) queryParams.set('to', params.to)

      return apiClient<GetLedgerResponse>(`/accounts/${id}/ledger?${queryParams.toString()}`)
    },
    enabled: !!id,
  })
}

export function useDeleteAccount() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (id: string) =>
      apiClient<{ success: boolean }>(`/accounts/${id}`, {
        method: 'DELETE',
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['accounts'] })
    },
  })
}

export function useToggleAccountActive() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ id, isActive }: { id: string; isActive: boolean }) =>
      apiClient<{ success: boolean; is_active: boolean }>(`/accounts/${id}/status`, {
        method: 'PATCH',
        body: JSON.stringify({ is_active: isActive }),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['accounts'] })
    },
  })
}
