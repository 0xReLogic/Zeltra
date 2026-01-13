export type AccountType = 'asset' | 'liability' | 'equity' | 'revenue' | 'expense'

export interface Account {
  id: string
  code: string
  name: string
  type: AccountType
  account_type?: AccountType // alias for backward compatibility
  subtype: string | null
  currency: string
  balance: string
  is_active: boolean
  allow_direct_posting: boolean
  parent_id: string | null
  description: string | null
}

// Backend returns { accounts: [...] } wrapper
export interface GetAccountsResponse {
  accounts: Account[]
}

export type CreateAccountRequest = {
  code: string
  name: string
  type: AccountType
  currency: string
  subtype?: string
  parent_id?: string
  description?: string
}
