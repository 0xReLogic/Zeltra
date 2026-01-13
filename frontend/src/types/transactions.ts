export type TransactionStatus = 'draft' | 'pending' | 'approved' | 'posted' | 'voided'
export type TransactionType = 'expense' | 'revenue' | 'transfer' | 'journal'

export interface TransactionEntry {
  account_code: string
  account_name: string
  debit: string
  credit: string
  dimensions?: string[] // Array of dimension value IDs
}

// List item from GET /transactions (no entries)
export interface TransactionListItem {
  id: string
  reference_number: string | null
  type: string // 'expense' | 'revenue' | 'transfer' | 'journal'
  transaction_date: string
  description: string
  status: string // 'draft' | 'pending' | 'approved' | 'posted' | 'voided'
  created_at: string
}

// Full transaction detail from GET /transactions/{id}
export interface Transaction {
  id: string
  reference_number: string | null
  transaction_type: TransactionType
  transaction_date: string
  description: string
  status: TransactionStatus
  entries: TransactionEntry[]
  created_at: string
  created_by: string
  fiscal_period_id: string
}

// Backend returns array directly, no pagination wrapper
export type GetTransactionsResponse = TransactionListItem[]

export interface CreateTransactionRequest {
  reference_number: string
  transaction_type: TransactionType
  transaction_date: string
  description: string
  entries: {
    account_code: string
    debit: string
    credit: string
    dimensions?: string[]
  }[]
}
