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
