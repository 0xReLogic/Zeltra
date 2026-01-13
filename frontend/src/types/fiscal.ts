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

// Period status enum
export type PeriodStatus = 'Open' | 'SoftClose' | 'Closed'
