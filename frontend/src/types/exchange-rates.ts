/**
 * Exchange Rate types - re-exported from OpenAPI generated types
 */

import type { components } from './api.generated'

// Re-export OpenAPI types
export type ExchangeRateResponse = components['schemas']['ExchangeRateResponse']
export type CreateExchangeRateRequest = components['schemas']['CreateExchangeRateRequest']
export type BulkImportRequest = components['schemas']['BulkImportRequest']
export type BulkImportResponse = components['schemas']['BulkImportResponse']
export type BulkRateItem = components['schemas']['BulkRateItem']
export type ExchangeRateListItem = components['schemas']['ExchangeRateListItem']
export type PageResponseExchangeRateListItem = components['schemas']['PageResponse_ExchangeRateListItem']
export type FetchRatesRequest = components['schemas']['FetchRatesRequest']
export type FetchRatesResponse = components['schemas']['FetchRatesResponse']
export type CurrencyResponse = components['schemas']['CurrencyResponse']

// Type aliases for backward compatibility
export type ExchangeRate = ExchangeRateResponse
export type Currency = CurrencyResponse

// Response types - paginated response
export type GetExchangeRatesResponse = PageResponseExchangeRateListItem

// Currencies response
export interface GetCurrenciesResponse {
  currencies: CurrencyResponse[]
}
