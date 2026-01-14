'use client'

import { useMutation } from '@tanstack/react-query'
import { apiClient } from '@/lib/api/client'
import type { RunSimulationRequest, SimulationResponse } from '@/types/simulation'

// Query keys for cache management
const SIMULATION_KEYS = {
  all: ['simulation'] as const,
  run: () => [...SIMULATION_KEYS.all, 'run'] as const,
}

/**
 * POST /organizations/{org_id}/simulation/run
 * Run a financial simulation with projections
 * 
 * NOTE: Requires Enterprise tier subscription
 */
export function useRunSimulation() {
  return useMutation({
    mutationKey: SIMULATION_KEYS.run(),
    mutationFn: (params: RunSimulationRequest) =>
      apiClient<SimulationResponse>('/simulation/run', {
        method: 'POST',
        body: JSON.stringify(params),
      }),
  })
}

// Re-export types for convenience
export type { RunSimulationRequest, SimulationResponse }
