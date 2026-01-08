'use client'

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from '@/lib/api/client'

export interface DimensionValue {
  id: string
  code: string
  name: string
  description?: string
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
