import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from '../api/client'
import type { GetTransactionsResponse, CreateTransactionRequest, Transaction } from '@/types/transactions'

export function useTransactions(page = 1, limit = 50) {
  return useQuery({
    queryKey: ['transactions', { page, limit }],
    queryFn: () => apiClient<GetTransactionsResponse>(
      `/transactions?page=${page}&limit=${limit}`
    ),
  })
}

export function useCreateTransaction() {
  const queryClient = useQueryClient()
  
  return useMutation({
    mutationFn: (data: CreateTransactionRequest) =>
      apiClient<Transaction>('/transactions', {
        method: 'POST',
        body: JSON.stringify(data),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['transactions'] })
    },
  })
}
