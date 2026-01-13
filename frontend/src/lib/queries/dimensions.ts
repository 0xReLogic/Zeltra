'use client'

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from '@/lib/api/client'

export interface DimensionValue {
  id: string
  code: string
  name: string
  description?: string | null
  is_active?: boolean
  dimension_type_id?: string
}

export interface DimensionType {
  id: string
  code: string
  name: string
  description?: string | null
  is_required?: boolean
  is_active?: boolean
  sort_order?: number
  values: DimensionValue[]
}

// Backend returns array of DimensionTypeResponse with embedded values
export function useDimensions() {
  return useQuery({
    queryKey: ['dimensions'],
    queryFn: () => apiClient<DimensionType[]>('/dimension-types'),
  })
}

export function useCreateDimensionValue() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (data: { dimension_type_id: string, code: string, name: string, description?: string }) =>
      apiClient<DimensionValue>('/dimension-values', {
        method: 'POST',
        body: JSON.stringify({
          dimension_type_id: data.dimension_type_id,
          code: data.code,
          name: data.name,
          description: data.description
        })
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['dimensions'] })
    }
  })
}

export function useCreateDimensionType() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (data: { code: string, name: string, description?: string, is_required?: boolean }) =>
      apiClient<DimensionType>('/dimension-types', {
        method: 'POST',
        body: JSON.stringify(data)
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['dimensions'] })
    }
  })
}

export function useEditDimensionValue() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (data: { id: string, name?: string, description?: string, code?: string }) =>
      apiClient<DimensionValue>(`/dimension-values/${data.id}`, {
        method: 'PATCH',
        body: JSON.stringify({
          name: data.name,
          description: data.description,
          code: data.code
        })
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['dimensions'] })
    }
  })
}

export function useToggleDimensionValueActive() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (data: { id: string, isActive: boolean }) =>
      apiClient<{ id: string, is_active: boolean }>(`/dimension-values/${data.id}/status`, {
        method: 'PATCH',
        body: JSON.stringify({ is_active: data.isActive })
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['dimensions'] })
    }
  })
}
