import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from '../api/client'
import type { GetTransactionsResponse, CreateTransactionRequest, Transaction, TransactionListItem } from '@/types/transactions'

export function useTransactions(page = 1, limit = 50, dimension?: string) {
  return useQuery({
    queryKey: ['transactions', { page, limit, dimension }],
    queryFn: () => {
      const params = new URLSearchParams()
      params.set('page', page.toString())
      params.set('limit', limit.toString())
      if (dimension && dimension !== 'all') {
        params.set('dimension_value_id', dimension)
      }
      // Backend returns array directly, not paginated object
      return apiClient<GetTransactionsResponse>(`/transactions?${params.toString()}`)
    },
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
      // Backend returns array directly
      const transactions = await apiClient<GetTransactionsResponse>('/transactions')
      console.log('raw txns:', transactions)
      const pending = transactions.filter(t => t.status === 'pending')
      console.log('pending txns:', pending)
      return pending
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
