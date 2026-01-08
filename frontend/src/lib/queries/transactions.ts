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

export function useTransaction(id: string) {
  return useQuery({
    queryKey: ['transactions', id],
    queryFn: () => apiClient<Transaction>(`/transactions/${id}`),
    enabled: !!id,
  })
}

export function usePendingTransactions() {
  return useQuery({
    queryKey: ['transactions', 'pending'],
    queryFn: async () => {
      const res = await apiClient<GetTransactionsResponse>('/transactions')
      console.log('raw txns:', res.data)
      const pending = res.data.filter(t => t.status === 'pending')
      console.log('pending txns:', pending)
      return {
        ...res,
        data: pending
      }
    }
  })
}

export function useApproveTransaction() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (id: string) => apiClient<Transaction>(`/transactions/${id}/approve`, { method: 'POST' }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['transactions'] })
    }
  })
}

export function useRejectTransaction() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (id: string) => apiClient<Transaction>(`/transactions/${id}/reject`, { method: 'POST' }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['transactions'] })
    }
  })
}
