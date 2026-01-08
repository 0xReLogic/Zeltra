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
