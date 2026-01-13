export interface Account {
  id: string
  code: string
  name: string
  account_type: 'asset' | 'liability' | 'equity' | 'revenue' | 'expense'
  balance: string // money type usually string from backend
  is_active?: boolean
}

// Backend returns array directly, no wrapper
export type GetAccountsResponse = Account[]

export type CreateAccountRequest = {
  code: string
  name: string
  account_type: Account['account_type']
}
