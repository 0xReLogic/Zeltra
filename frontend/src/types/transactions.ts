/**
 * Transaction types - re-exported from OpenAPI generated types
 * with backward compatibility aliases
 */

import { components } from './api.generated'

import type {
  TransactionResponse,
  TransactionListItem as ApiTransactionListItem,
  CreateTransactionRequest as ApiCreateTransactionRequest,
  UpdateTransactionRequest as ApiUpdateTransactionRequest,
  CreateEntryRequest as ApiCreateEntryRequest,
  EntryResponse,
  RejectRequest,
  VoidRequest,
  VoidResponse,
  BulkApproveRequest,
  BulkApproveResponse,
  PayInvoiceRequest as ApiPayInvoiceRequest,
  PendingTransactionResponse,
  PaginatedTransactionsResponse,
  PaginationMeta,
} from './api-helpers'

// Re-export OpenAPI types
export type {
  TransactionResponse,
  ApiCreateEntryRequest,
  EntryResponse,
  RejectRequest,
  VoidRequest,
  VoidResponse,
  BulkApproveRequest,
  BulkApproveResponse,
  PendingTransactionResponse,
  PaginatedTransactionsResponse,
  PaginationMeta,
}

// Use generated CreateEntryRequest directly
export type CreateEntryRequest = components['schemas']['CreateEntryRequest']

export type PayInvoiceRequest = ApiPayInvoiceRequest

// Transaction status enum (for type safety)
export type TransactionStatus = 'draft' | 'pending' | 'approved' | 'posted' | 'voided'

// Transaction type enum
export type TransactionType = 'expense' | 'revenue' | 'transfer' | 'journal'

// List item from GET /transactions (no entries)
export type TransactionListItem = ApiTransactionListItem

// Full transaction detail from GET /transactions/{id}
export type Transaction = TransactionResponse

// Create transaction request
export type CreateTransactionRequest = Omit<ApiCreateTransactionRequest, 'entries'> & {
    entries: CreateEntryRequest[]
}

// Update transaction request
export type UpdateTransactionRequest = ApiUpdateTransactionRequest

// Backend returns structured response with pagination
export type GetTransactionsResponse = PaginatedTransactionsResponse

// Backend /transactions/pending returns { data: PendingTransactionResponse[] }
export type GetPendingTransactionsResponse = {
  data: PendingTransactionResponse[]
}

// Legacy alias for backward compatibility
export interface TransactionEntry {
  account_id: string
  account_code: string
  account_name: string
  debit: string
  credit: string
  entry_type: string
  source_amount: string
  source_currency: string
  dimensions?: string[]
  memo?: string | null
}
