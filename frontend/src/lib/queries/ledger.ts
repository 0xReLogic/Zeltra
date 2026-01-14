import { useQuery } from '@tanstack/react-query';
import { apiClient } from '../api/client';
import type { AccountLedgerResponse } from '@/types/ledger';

interface LedgerQueryParams {
  accountId: string;
  startDate?: string;
  endDate?: string;
  page?: number;
  limit?: number;
}

export function useLedger({ accountId, startDate, endDate, page = 1, limit = 50 }: LedgerQueryParams) {
  return useQuery({
    queryKey: ['ledger', accountId, startDate, endDate, page, limit],
    queryFn: () => {
      const params = new URLSearchParams();
      if (startDate) params.append('start_date', startDate);
      if (endDate) params.append('end_date', endDate);
      params.append('page', page.toString());
      params.append('limit', limit.toString());
      
      return apiClient<AccountLedgerResponse>(
        `/accounts/${accountId}/ledger?${params.toString()}`
      );
    },
    enabled: !!accountId,
  });
}
