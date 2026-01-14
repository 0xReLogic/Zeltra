/**
 * Simulation Types
 * Re-exports from OpenAPI generated types for simulation feature
 */

import type { components } from './api.generated'

// Request/Response types
export type RunSimulationRequest = components['schemas']['RunSimulationRequest']
export type SimulationResponse = components['schemas']['SimulationResponse']

// Related types
export type AccountProjectionResponse = components['schemas']['AccountProjectionResponse']
export type AnnualSummaryResponse = components['schemas']['AnnualSummaryResponse']
export type MonthlySummaryResponse = components['schemas']['MonthlySummaryResponse']

// Legacy type aliases for backward compatibility with existing page
export type SimulationRequest = RunSimulationRequest
export type SimulationResult = SimulationResponse
