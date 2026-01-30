/**
 * Entity queries for multi-entity accounting
 * 
 * These hooks manage entity data using TanStack Query v5 patterns.
 * Entities represent legal or operational units within an organization.
 */

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useAuthStore } from '@/lib/stores/authStore'
import { api } from '@/lib/api'
import type { Entity, CreateEntityRequest, UpdateEntityRequest } from '@/types/entities'

/**
 * Query key factory for entities
 * Follows TanStack Query v5 best practices for key management
 */
export const entityKeys = {
  all: ['entities'] as const,
  lists: () => [...entityKeys.all, 'list'] as const,
  list: (orgId: string) => [...entityKeys.lists(), orgId] as const,
  details: () => [...entityKeys.all, 'detail'] as const,
  detail: (entityId: string) => [...entityKeys.details(), entityId] as const,
}

/**
 * List all entities for the current organization
 * 
 * @returns Query result with entities array
 */
export function useEntities() {
  const currentOrgId = useAuthStore((state) => state.currentOrgId)

  return useQuery({
    queryKey: entityKeys.list(currentOrgId || ''),
    queryFn: async () => {
      if (!currentOrgId) {
        throw new Error('No organization selected')
      }
      const response = await api.get<{ entities: Entity[] }>(
        `/organizations/${currentOrgId}/entities`
      )
      return response.entities
    },
    enabled: !!currentOrgId,
    staleTime: 5 * 60 * 1000, // 5 minutes
  })
}

/**
 * Get a single entity by ID
 * 
 * @param entityId - Entity UUID
 * @returns Query result with entity data
 */
export function useEntity(entityId: string) {
  const currentOrgId = useAuthStore((state) => state.currentOrgId)

  return useQuery({
    queryKey: entityKeys.detail(entityId),
    queryFn: async () => {
      if (!currentOrgId) {
        throw new Error('No organization selected')
      }
      const response = await api.get<Entity>(
        `/organizations/${currentOrgId}/entities/${entityId}`
      )
      return response
    },
    enabled: !!currentOrgId && !!entityId,
    staleTime: 5 * 60 * 1000, // 5 minutes
  })
}

/**
 * Create a new entity
 * 
 * @returns Mutation for creating entities
 */
export function useCreateEntity() {
  const currentOrgId = useAuthStore((state) => state.currentOrgId)
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (data: CreateEntityRequest) => {
      if (!currentOrgId) {
        throw new Error('No organization selected')
      }
      const response = await api.post<Entity>(
        `/organizations/${currentOrgId}/entities`,
        data
      )
      return response
    },
    onSuccess: () => {
      // Invalidate entities list to refetch
      if (currentOrgId) {
        queryClient.invalidateQueries({ queryKey: entityKeys.list(currentOrgId) })
      }
    },
  })
}

/**
 * Update an existing entity
 * 
 * @param entityId - Entity UUID to update
 * @returns Mutation for updating entities
 */
export function useUpdateEntity(entityId: string) {
  const currentOrgId = useAuthStore((state) => state.currentOrgId)
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (data: UpdateEntityRequest) => {
      if (!currentOrgId) {
        throw new Error('No organization selected')
      }
      const response = await api.patch<Entity>(
        `/organizations/${currentOrgId}/entities/${entityId}`,
        data
      )
      return response
    },
    onSuccess: () => {
      // Invalidate both list and detail queries
      if (currentOrgId) {
        queryClient.invalidateQueries({ queryKey: entityKeys.list(currentOrgId) })
        queryClient.invalidateQueries({ queryKey: entityKeys.detail(entityId) })
      }
    },
  })
}

/**
 * Delete an entity (soft delete)
 * 
 * @param entityId - Entity UUID to delete
 * @returns Mutation for deleting entities
 */
export function useDeleteEntity(entityId: string) {
  const currentOrgId = useAuthStore((state) => state.currentOrgId)
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async () => {
      if (!currentOrgId) {
        throw new Error('No organization selected')
      }
      await api.delete(`/organizations/${currentOrgId}/entities/${entityId}`)
    },
    onSuccess: () => {
      // Invalidate entities list to refetch
      if (currentOrgId) {
        queryClient.invalidateQueries({ queryKey: entityKeys.list(currentOrgId) })
      }
    },
  })
}
