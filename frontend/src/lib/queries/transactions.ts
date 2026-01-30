import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from '../api/client'
import type {
  GetTransactionsResponse,
  GetPendingTransactionsResponse,
  CreateTransactionRequest,
  UpdateTransactionRequest,
  Transaction,
  RejectRequest,
  VoidRequest,
  BulkApproveRequest,
  BulkApproveResponse,
  PayInvoiceRequest,
} from '@/types/transactions'

// Query keys for cache management
const TRANSACTION_KEYS = {
  all: ['transactions'] as const,
  lists: () => [...TRANSACTION_KEYS.all, 'list'] as const,
  list: (filters: TransactionFilters) => [...TRANSACTION_KEYS.lists(), filters] as const,
  details: () => [...TRANSACTION_KEYS.all, 'detail'] as const,
  detail: (id: string) => [...TRANSACTION_KEYS.details(), id] as const,
  pending: () => [...TRANSACTION_KEYS.all, 'pending'] as const,
}

interface TransactionFilters {
  page?: number
  limit?: number
  status?: string
  start_date?: string
  end_date?: string
  account_id?: string
  dimension_value_id?: string
  entity_id?: string  // NEW: Filter by entity
}

/**
 * GET /organizations/{org_id}/transactions
 * List transactions with optional filters
 */
export function useTransactions(filters: TransactionFilters = {}) {
  const { page = 0, limit = 50, status, start_date, end_date, account_id, dimension_value_id, entity_id } = filters

  return useQuery({
    queryKey: TRANSACTION_KEYS.list(filters),
    queryFn: () => {
      const params = new URLSearchParams()
      if (page !== undefined) params.set('page', page.toString())
      if (limit !== undefined) params.set('limit', limit.toString())
      if (status) params.set('status', status)
      if (start_date) params.set('start_date', start_date)
      if (end_date) params.set('end_date', end_date)
      if (account_id) params.set('account_id', account_id)
      if (dimension_value_id && dimension_value_id !== 'all') {
        params.set('dimension_value_id', dimension_value_id)
      }
      if (entity_id) params.set('entity_id', entity_id)  // NEW: Add entity filter
      // Backend returns array directly, not paginated object
      return apiClient<GetTransactionsResponse>(`/transactions?${params.toString()}`)
    },
  })
}

/**
 * GET /organizations/{org_id}/transactions/{id}
 * Get single transaction with entries
 */
export function useTransaction(id: string) {
  return useQuery({
    queryKey: TRANSACTION_KEYS.detail(id),
    queryFn: () => apiClient<Transaction>(`/transactions/${id}`),
    enabled: !!id,
  })
}

/**
 * GET /organizations/{org_id}/transactions/pending
 * Get pending transactions for approval queue (includes can_approve flag)
 */
// Backend /transactions/pending returns { data: PendingTransaction[] }
export function usePendingTransactions() {
  return useQuery({
    queryKey: TRANSACTION_KEYS.pending(),
    queryFn: async () => {
      // Use dedicated pending endpoint that returns can_approve
      const response = await apiClient<GetPendingTransactionsResponse>('/transactions/pending')
      return response
    },
  })
}

/**
 * POST /organizations/{org_id}/transactions
 * Create a new transaction
 * 
 * Includes Idempotency-Key header to prevent duplicate transactions on network retries.
 */
/**
 * Generate a UUID v4 with fallback for environments without crypto.randomUUID
 */
function generateUUID(): string {
  // Use crypto.randomUUID if available (modern browsers)
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID()
  }
  
  // Fallback: generate UUID v4 using crypto.getRandomValues
  if (typeof crypto !== 'undefined' && typeof crypto.getRandomValues === 'function') {
    const bytes = new Uint8Array(16)
    crypto.getRandomValues(bytes)
    // Set version (4) and variant (RFC4122)
    bytes[6] = (bytes[6] & 0x0f) | 0x40
    bytes[8] = (bytes[8] & 0x3f) | 0x80
    
    const hex = Array.from(bytes, b => b.toString(16).padStart(2, '0')).join('')
    return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`
  }
  
  // Last resort fallback using Math.random (less secure but works everywhere)
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
    const r = Math.random() * 16 | 0
    const v = c === 'x' ? r : (r & 0x3 | 0x8)
    return v.toString(16)
  })
}

export function useCreateTransaction() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: CreateTransactionRequest) => {
      // Generate unique idempotency key for this request
      const idempotencyKey = generateUUID()
      
      return apiClient<Transaction>('/transactions', {
        method: 'POST',
        body: JSON.stringify(data),
        headers: {
          'Idempotency-Key': idempotencyKey,
        },
      })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: TRANSACTION_KEYS.all })
    },
  })
}

/**
 * PATCH /organizations/{org_id}/transactions/{id}
 * Update a draft transaction
 */
export function useUpdateTransaction() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ id, data }: { id: string; data: UpdateTransactionRequest }) =>
      apiClient<Transaction>(`/transactions/${id}`, {
        method: 'PATCH',
        body: JSON.stringify(data),
      }),
    onSuccess: (_, { id }) => {
      queryClient.invalidateQueries({ queryKey: TRANSACTION_KEYS.detail(id) })
      queryClient.invalidateQueries({ queryKey: TRANSACTION_KEYS.lists() })
    },
  })
}

/**
 * DELETE /organizations/{org_id}/transactions/{id}
 * Delete a draft transaction
 */
export function useDeleteTransaction() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (id: string) =>
      apiClient<void>(`/transactions/${id}`, {
        method: 'DELETE',
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: TRANSACTION_KEYS.all })
    },
  })
}

// ============================================================================
// Transaction Workflow Mutations
// ============================================================================

/**
 * POST /organizations/{org_id}/transactions/{id}/submit
 * Submit a draft transaction for approval (draft → pending)
 */
export function useSubmitTransaction() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (id: string) =>
      apiClient<Transaction>(`/transactions/${id}/submit`, {
        method: 'POST',
        body: JSON.stringify({}),
      }),
    onSuccess: (_, id) => {
      queryClient.invalidateQueries({ queryKey: TRANSACTION_KEYS.detail(id) })
      queryClient.invalidateQueries({ queryKey: TRANSACTION_KEYS.lists() })
      queryClient.invalidateQueries({ queryKey: TRANSACTION_KEYS.pending() })
    },
  })
}

/**
 * POST /organizations/{org_id}/transactions/{id}/approve
 * Approve a pending transaction (pending → approved)
 */
export function useApproveTransaction() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (id: string) =>
      apiClient<Transaction>(`/transactions/${id}/approve`, {
        method: 'POST',
        body: JSON.stringify({}),
      }),
    onSuccess: (_, id) => {
      queryClient.invalidateQueries({ queryKey: TRANSACTION_KEYS.detail(id) })
      queryClient.invalidateQueries({ queryKey: TRANSACTION_KEYS.lists() })
      queryClient.invalidateQueries({ queryKey: TRANSACTION_KEYS.pending() })
    },
  })
}

/**
 * POST /organizations/{org_id}/transactions/{id}/reject
 * Reject a pending transaction (pending → draft)
 */
export function usePayInvoice() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: PayInvoiceRequest) =>
      apiClient<Transaction>('/transactions/pay-invoice', {
        method: 'POST',
        body: JSON.stringify(data),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: TRANSACTION_KEYS.all })
    },
  })
}

export function useRejectTransaction() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ id, reason }: { id: string; reason: string }) =>
      apiClient<Transaction>(`/transactions/${id}/reject`, {
        method: 'POST',
        body: JSON.stringify({ reason } as RejectRequest),
      }),
    onSuccess: (_, { id }) => {
      queryClient.invalidateQueries({ queryKey: TRANSACTION_KEYS.detail(id) })
      queryClient.invalidateQueries({ queryKey: TRANSACTION_KEYS.lists() })
      queryClient.invalidateQueries({ queryKey: TRANSACTION_KEYS.pending() })
    },
  })
}

/**
 * POST /organizations/{org_id}/transactions/{id}/post
 * Post an approved transaction (approved → posted)
 */
export function usePostTransaction() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (id: string) =>
      apiClient<Transaction>(`/transactions/${id}/post`, {
        method: 'POST',
        body: JSON.stringify({}),
      }),
    onSuccess: (_, id) => {
      queryClient.invalidateQueries({ queryKey: TRANSACTION_KEYS.detail(id) })
      queryClient.invalidateQueries({ queryKey: TRANSACTION_KEYS.lists() })
    },
  })
}

/**
 * POST /organizations/{org_id}/transactions/{id}/void
 * Void a posted transaction (posted → voided)
 */
export function useVoidTransaction() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ id, reason }: { id: string; reason: string }) =>
      apiClient<Transaction>(`/transactions/${id}/void`, {
        method: 'POST',
        body: JSON.stringify({ reason } as VoidRequest),
      }),
    onSuccess: (_, { id }) => {
      queryClient.invalidateQueries({ queryKey: TRANSACTION_KEYS.detail(id) })
      queryClient.invalidateQueries({ queryKey: TRANSACTION_KEYS.lists() })
    },
  })
}

/**
 * POST /organizations/{org_id}/transactions/bulk-approve
 * Bulk approve multiple pending transactions
 */
export function useBulkApprove() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (transactionIds: string[]) =>
      apiClient<BulkApproveResponse>('/transactions/bulk-approve', {
        method: 'POST',
        body: JSON.stringify({ transaction_ids: transactionIds } as BulkApproveRequest),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: TRANSACTION_KEYS.all })
    },
  })
}
