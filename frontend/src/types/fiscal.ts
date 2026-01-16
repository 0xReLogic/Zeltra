/**
 * Fiscal types - re-exported from OpenAPI generated types
 */

import type { components } from './api.generated'

// Re-export OpenAPI types
export type FiscalYearResponse = components['schemas']['FiscalYearResponse']
export type FiscalPeriodResponse = components['schemas']['FiscalPeriodResponse']
export type CreateFiscalYearRequest = components['schemas']['CreateFiscalYearRequest']
export type UpdatePeriodStatusRequest = components['schemas']['UpdatePeriodStatusRequest']

// Type aliases for backward compatibility
export type FiscalYear = FiscalYearResponse
export type FiscalPeriod = FiscalPeriodResponse

// Response types
export type GetFiscalYearsResponse = FiscalYearResponse[]

// Period status enum - must match backend: "open", "soft_close", "closed"
export type PeriodStatus = 'Open' | 'SoftClose' | 'Closed'

// Backend expects lowercase with underscore
export type PeriodStatusBackend = 'open' | 'soft_close' | 'closed'

// Convert display status to backend format
export function toBackendStatus(status: PeriodStatus): PeriodStatusBackend {
  const map: Record<PeriodStatus, PeriodStatusBackend> = {
    'Open': 'open',
    'SoftClose': 'soft_close',
    'Closed': 'closed',
  }
  return map[status]
}
