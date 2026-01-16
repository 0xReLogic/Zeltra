/**
 * Dimension types - re-exported from OpenAPI generated types
 */

import type { components } from './api.generated'

// Re-export OpenAPI types
export type DimensionTypeResponse = components['schemas']['DimensionTypeResponse']
export type DimensionValueResponse = components['schemas']['DimensionValueResponse'] & { id: string; description?: string }
export type CreateDimensionTypeRequest = components['schemas']['CreateDimensionTypeRequest']
export type CreateDimensionValueRequest = components['schemas']['CreateDimensionValueRequest']
export type UpdateDimensionValueRequest = components['schemas']['UpdateDimensionValueRequest']

// Type aliases for backward compatibility
export type DimensionType = DimensionTypeResponse
export type DimensionValue = DimensionValueResponse

// Response types
export type GetDimensionTypesResponse = { dimension_types: DimensionTypeResponse[] }
export type GetDimensionValuesResponse = { dimension_values: DimensionValueResponse[] }
