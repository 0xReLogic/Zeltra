/**
 * Account types - re-exported from generated OpenAPI types
 */
import type { 
  AccountResponse as GeneratedAccountResponse,
  CreateAccountRequest as GeneratedCreateAccountRequest,
  UpdateAccountRequest as GeneratedUpdateAccountRequest,
  GetAccountsResponse as GeneratedGetAccountsResponse,
} from './api-helpers'

// Account type enum for convenience
export type AccountType = 'asset' | 'liability' | 'equity' | 'revenue' | 'expense'

// Re-export generated types with aliases for backward compatibility
export type Account = GeneratedAccountResponse
export type AccountResponse = GeneratedAccountResponse
export type CreateAccountRequest = GeneratedCreateAccountRequest
export type UpdateAccountRequest = GeneratedUpdateAccountRequest
export type GetAccountsResponse = GeneratedGetAccountsResponse
