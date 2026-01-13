'use client'

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from '@/lib/api/client'
import type {
  DimensionType,
  DimensionValue,
  GetDimensionTypesResponse,
  GetDimensionValuesResponse,
  CreateDimensionTypeRequest,
  CreateDimensionValueRequest,
  UpdateDimensionValueRequest,
} from '@/types/dimensions'

// Query keys for cache management
const DIMENSION_KEYS = {
  all: ['dimensions'] as const,
  types: () => [...DIMENSION_KEYS.all, 'types'] as const,
  values: () => [...DIMENSION_KEYS.all, 'values'] as const,
  valuesByType: (typeId: string) => [...DIMENSION_KEYS.values(), typeId] as const,
}

/**
 * GET /organizations/{org_id}/dimension-types
 * List all dimension types with their values
 */
export function useDimensions() {
  return useQuery({
    queryKey: DIMENSION_KEYS.types(),
    queryFn: () => apiClient<GetDimensionTypesResponse>('/dimension-types'),
  })
}

// Alias for backward compatibility
export const useDimensionTypes = useDimensions

/**
 * GET /organizations/{org_id}/dimension-values
 * List dimension values, optionally filtered by type
 */
export function useDimensionValues(dimensionTypeId?: string) {
  return useQuery({
    queryKey: DIMENSION_KEYS.valuesByType(dimensionTypeId || 'all'),
    queryFn: () => {
      const params = new URLSearchParams()
      if (dimensionTypeId) params.set('dimension_type_id', dimensionTypeId)
      const queryString = params.toString()
      return apiClient<GetDimensionValuesResponse>(`/dimension-values${queryString ? `?${queryString}` : ''}`)
    },
  })
}

/**
 * POST /organizations/{org_id}/dimension-types
 * Create a new dimension type
 */
export function useCreateDimensionType() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: CreateDimensionTypeRequest) =>
      apiClient<DimensionType>('/dimension-types', {
        method: 'POST',
        body: JSON.stringify(data),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: DIMENSION_KEYS.all })
    },
  })
}

/**
 * POST /organizations/{org_id}/dimension-values
 * Create a new dimension value
 */
export function useCreateDimensionValue() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: CreateDimensionValueRequest) =>
      apiClient<DimensionValue>('/dimension-values', {
        method: 'POST',
        body: JSON.stringify(data),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: DIMENSION_KEYS.all })
    },
  })
}

/**
 * PATCH /organizations/{org_id}/dimension-values/{id}
 * Update a dimension value
 */
export function useUpdateDimensionValue() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ id, data }: { id: string; data: UpdateDimensionValueRequest }) =>
      apiClient<DimensionValue>(`/dimension-values/${id}`, {
        method: 'PATCH',
        body: JSON.stringify(data),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: DIMENSION_KEYS.all })
    },
  })
}

// Alias for backward compatibility
export const useEditDimensionValue = () => {
  const mutation = useUpdateDimensionValue()
  return {
    ...mutation,
    mutate: (data: { id: string; name?: string; description?: string; code?: string }) => {
      mutation.mutate({ id: data.id, data: { name: data.name, description: data.description, code: data.code } })
    },
    mutateAsync: (data: { id: string; name?: string; description?: string; code?: string }) => {
      return mutation.mutateAsync({ id: data.id, data: { name: data.name, description: data.description, code: data.code } })
    },
  }
}

/**
 * PATCH /organizations/{org_id}/dimension-values/{id}/status
 * Toggle dimension value active status
 */
export function useToggleDimensionValueStatus() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ id, is_active }: { id: string; is_active: boolean }) =>
      apiClient<{ id: string; is_active: boolean }>(`/dimension-values/${id}/status`, {
        method: 'PATCH',
        body: JSON.stringify({ is_active }),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: DIMENSION_KEYS.all })
    },
  })
}

// Alias for backward compatibility
export const useToggleDimensionValueActive = () => {
  const mutation = useToggleDimensionValueStatus()
  return {
    ...mutation,
    mutate: (data: { id: string; isActive: boolean }) => {
      mutation.mutate({ id: data.id, is_active: data.isActive })
    },
    mutateAsync: (data: { id: string; isActive: boolean }) => {
      return mutation.mutateAsync({ id: data.id, is_active: data.isActive })
    },
  }
}
