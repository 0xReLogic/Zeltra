export type TransactionStatus = 'draft' | 'pending' | 'approved' | 'posted' | 'voided'
export type TransactionType = 'expense' | 'revenue' | 'transfer' | 'journal'

export interface TransactionEntry {
  account_code: string
  account_name: string
  debit: string
  credit: string
}

export interface Transaction {
  id: string
  reference_number: string
  transaction_type: TransactionType
  transaction_date: string
  description: string
  status: TransactionStatus
  entries: TransactionEntry[]
}

export interface GetTransactionsResponse {
  data: Transaction[]
  pagination: {
    page: number
    limit: number
    total: number
  }
}

export interface CreateTransactionRequest {
  reference_number: string
  transaction_type: TransactionType
  transaction_date: string
  description: string
  entries: {
    account_code: string // We will use code or ID, let's assume code for now
    debit: string
    credit: string
  }[]
}
