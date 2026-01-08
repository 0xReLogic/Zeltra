import { useQuery } from '@tanstack/react-query'
import { apiClient } from '../api/client'
import { GetTransactionsResponse } from '@/types/transactions' // Reuse transaction types for ledger

export function useLedger(accountId: string) {
  return useQuery({
    queryKey: ['ledger', accountId],
    queryFn: () => apiClient<GetTransactionsResponse>(`/accounts/${accountId}/ledger`),
    enabled: !!accountId,
  })
}
