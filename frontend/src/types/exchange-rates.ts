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

// Type aliases for backward compatibility
export type ExchangeRate = ExchangeRateResponse

// Response types
export type GetExchangeRatesResponse = ExchangeRateResponse[]
