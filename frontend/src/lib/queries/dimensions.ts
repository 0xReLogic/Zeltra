'use client'

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from '@/lib/api/client'

export interface DimensionValue {
  id: string
  code: string
  name: string
  description?: string
  is_active?: boolean
}

export interface DimensionType {
  id: string
  code: string
  name: string
  values: DimensionValue[]
}

export function useDimensions() {
  return useQuery({
    queryKey: ['dimensions'],
    queryFn: () => apiClient<DimensionType[]>('/dimensions'),
  })
}

export function useCreateDimensionValue() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (data: { typeId: string, code: string, name: string }) =>
      apiClient(`/dimensions/${data.typeId}/values`, {
        method: 'POST',
        body: JSON.stringify(data)
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['dimensions'] })
    }
  })
}

export function useCreateDimensionType() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (data: { code: string, name: string }) =>
      apiClient('/dimension-types', { // Note: using /dimension-types as per PROMPT_FRONTEND
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
    mutationFn: (data: { typeId: string, id: string, name?: string, description?: string }) =>
      apiClient(`/dimensions/${data.typeId}/values/${data.id}`, {
        method: 'PATCH',
        body: JSON.stringify(data)
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['dimensions'] })
    }
  })
}

export function useToggleDimensionValueActive() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (data: { typeId: string, id: string, isActive: boolean }) =>
      apiClient(`/dimensions/${data.typeId}/values/${data.id}/status`, {
        method: 'PATCH',
        body: JSON.stringify({ is_active: data.isActive })
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['dimensions'] })
    }
  })
}
