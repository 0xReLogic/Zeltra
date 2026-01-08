export interface Account {
  id: string
  code: string
  name: string
  account_type: 'asset' | 'liability' | 'equity' | 'revenue' | 'expense'
  balance: string // money type usually string from backend
}

export interface GetAccountsResponse {
  data: Account[]
}

export type CreateAccountRequest = {
  code: string
  name: string
  account_type: Account['account_type']
}
