/**
 * Sentinel Intelligence API Queries
 * 
 * React Query hooks for Accruals, Revaluation, and Intercompany features.
 */

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from '@/lib/api/client'
import { useAuthStore } from '@/lib/stores/authStore'
import type {
  AccrualScheduleResponse,
  CreateAccrualScheduleRequest,
  RevaluationLogResponse,
  IntercompanyMappingResponse,
  CreateIntercompanyMappingRequest,
} from '@/types/api-helpers'

// ============================================================================
// Accrual Schedules
// ============================================================================

/**
 * Fetch all accrual schedules for the current organization.
 */
export function useAccrualSchedules() {
  const currentOrgId = useAuthStore((state) => state.currentOrgId)

  return useQuery({
    queryKey: ['accrual-schedules', currentOrgId],
    queryFn: () =>
      apiClient<AccrualScheduleResponse[]>(
        `/organizations/${currentOrgId}/accrual-schedules`
      ),
    enabled: !!currentOrgId,
  })
}

/**
 * Fetch a single accrual schedule by ID.
 */
export function useAccrualSchedule(scheduleId: string) {
  const currentOrgId = useAuthStore((state) => state.currentOrgId)

  return useQuery({
    queryKey: ['accrual-schedules', currentOrgId, scheduleId],
    queryFn: () =>
      apiClient<AccrualScheduleResponse>(
        `/organizations/${currentOrgId}/accrual-schedules/${scheduleId}`
      ),
    enabled: !!currentOrgId && !!scheduleId,
  })
}

/**
 * Create a new accrual schedule.
 */
export function useCreateAccrualSchedule() {
  const queryClient = useQueryClient()
  const currentOrgId = useAuthStore((state) => state.currentOrgId)

  return useMutation({
    mutationFn: (data: CreateAccrualScheduleRequest) =>
      apiClient<AccrualScheduleResponse>(
        `/organizations/${currentOrgId}/accrual-schedules`,
        {
          method: 'POST',
          body: JSON.stringify(data),
        }
      ),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['accrual-schedules', currentOrgId] })
    },
  })
}

// ============================================================================
// Revaluation Logs
// ============================================================================

/**
 * Fetch revaluation logs for the current organization.
 * @param params Optional filters for date range
 */
export function useRevaluationLogs(params?: { from?: string; to?: string }) {
  const currentOrgId = useAuthStore((state) => state.currentOrgId)

  return useQuery({
    queryKey: ['revaluation-logs', currentOrgId, params],
    queryFn: () => {
      const queryParams = new URLSearchParams()
      if (params?.from) queryParams.set('from', params.from)
      if (params?.to) queryParams.set('to', params.to)
      
      const queryString = queryParams.toString()
      const url = `/organizations/${currentOrgId}/revaluation-logs${queryString ? `?${queryString}` : ''}`
      
      return apiClient<RevaluationLogResponse[]>(url)
    },
    enabled: !!currentOrgId,
  })
}

// ============================================================================
// Intercompany Mappings
// ============================================================================

/**
 * Fetch all intercompany mappings for the current organization.
 */
export function useIntercompanyMappings() {
  const currentOrgId = useAuthStore((state) => state.currentOrgId)

  return useQuery({
    queryKey: ['intercompany-mappings', currentOrgId],
    queryFn: () =>
      apiClient<IntercompanyMappingResponse[]>(
        `/organizations/${currentOrgId}/intercompany/mappings`
      ),
    enabled: !!currentOrgId,
  })
}

/**
 * Create a new intercompany mapping.
 */
export function useCreateIntercompanyMapping() {
  const queryClient = useQueryClient()
  const currentOrgId = useAuthStore((state) => state.currentOrgId)

  return useMutation({
    mutationFn: (data: CreateIntercompanyMappingRequest) =>
      apiClient<IntercompanyMappingResponse>(
        `/organizations/${currentOrgId}/intercompany/connect`,
        {
          method: 'POST',
          body: JSON.stringify(data),
        }
      ),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['intercompany-mappings', currentOrgId] })
    },
  })
}
