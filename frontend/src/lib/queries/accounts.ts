import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from '../api/client'
import type { 
  AccountResponse,
  CreateAccountRequest, 
  UpdateAccountRequest,
  GetAccountsResponse 
} from '@/types/accounts'
import type { AccountLedgerResponse } from '@/types/ledger'

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
      apiClient<AccountResponse>('/accounts', {
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
    queryFn: () => apiClient<AccountResponse>(`/accounts/${id}`),
    enabled: !!id,
  })
}

export function useUpdateAccount() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ id, ...data }: { id: string } & Partial<UpdateAccountRequest>) =>
      apiClient<AccountResponse>(`/accounts/${id}`, {
        method: 'PUT',
        body: JSON.stringify(data),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['accounts'] })
    },
  })
}

export function useAccountLedger(id: string, params?: { page?: number; limit?: number; from?: string; to?: string }) {
  return useQuery({
    queryKey: ['account-ledger', id, params],
    queryFn: () => {
      const queryParams = new URLSearchParams()
      if (params?.page !== undefined) queryParams.set('page', params.page.toString())
      if (params?.limit !== undefined) queryParams.set('limit', params.limit.toString())
      if (params?.from) queryParams.set('from', params.from)
      if (params?.to) queryParams.set('to', params.to)

      return apiClient<AccountLedgerResponse>(`/accounts/${id}/ledger?${queryParams.toString()}`)
    },
    enabled: !!id,
  })
}

export function useDeleteAccount() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (id: string) =>
      apiClient<void>(`/accounts/${id}`, {
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
      apiClient<AccountResponse>(`/accounts/${id}/status`, {
        method: 'PATCH',
        body: JSON.stringify({ is_active: isActive }),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['accounts'] })
    },
  })
}
