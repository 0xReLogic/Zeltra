/**
 * Entity Queries Tests
 * 
 * Tests for entity query hooks including:
 * - useEntities hook fetches entities
 * - useCreateEntity mutation
 * - useUpdateEntity mutation
 * - useDeleteEntity mutation
 * - Query invalidation after mutations
 * 
 * Requirements: 2.1, 2.6, 2.7, 2.8
 */

import { describe, it, expect, beforeEach, vi } from 'vitest'
import { renderHook, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { type ReactNode, createElement } from 'react'
import {
  useEntities,
  useCreateEntity,
  useUpdateEntity,
  useDeleteEntity,
} from '../entities'
import { api } from '@/lib/api'
import { useAuthStore } from '@/lib/stores/authStore'

// Mock dependencies
vi.mock('@/lib/api')
vi.mock('@/lib/stores/authStore')

const mockOrgId = 'org-123'
const mockEntityId = 'entity-456'

const mockEntity = {
  id: mockEntityId,
  organization_id: mockOrgId,
  name: 'Test Entity',
  entity_type: 'main',
  base_currency: 'USD',
  is_active: true,
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
}

describe('Entity Queries', () => {
  let queryClient: QueryClient
  let wrapper: (props: { children: ReactNode }) => ReturnType<typeof createElement>

  beforeEach(() => {
    queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false },
        mutations: { retry: false },
      },
    })

    wrapper = ({ children }: { children: ReactNode }) =>
      createElement(QueryClientProvider, { client: queryClient }, children)

    // Mock auth store
    vi.mocked(useAuthStore).mockReturnValue({
      currentOrgId: mockOrgId,
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
    } as any)
  })

  describe('useEntities', () => {
    it('should fetch entities for current organization', async () => {
      const mockEntities = [mockEntity]
      vi.mocked(api.get).mockResolvedValue({ entities: mockEntities })

      const { result } = renderHook(() => useEntities(), { wrapper })

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true)
      })

      expect(result.current.data).toEqual(mockEntities)
      expect(api.get).toHaveBeenCalledWith(
        `/organizations/${mockOrgId}/entities`
      )
    })

    it('should not fetch when no organization is selected', () => {
      vi.mocked(useAuthStore).mockReturnValue({
        currentOrgId: null,
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
      } as any)

      const { result } = renderHook(() => useEntities(), { wrapper })

      expect(result.current.fetchStatus).toBe('idle')
      expect(api.get).not.toHaveBeenCalled()
    })
  })

  describe('useCreateEntity', () => {
    it('should create a new entity', async () => {
      vi.mocked(api.post).mockResolvedValue(mockEntity)

      const { result } = renderHook(() => useCreateEntity(), { wrapper })

      const createData = {
        name: 'New Entity',
        base_currency: 'EUR',
        entity_type: 'subsidiary' as const,
      }

      result.current.mutate(createData)

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true)
      })

      expect(api.post).toHaveBeenCalledWith(
        `/organizations/${mockOrgId}/entities`,
        createData
      )
    })

    it('should invalidate entities list after creation', async () => {
      vi.mocked(api.post).mockResolvedValue(mockEntity)

      const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries')

      const { result } = renderHook(() => useCreateEntity(), { wrapper })

      result.current.mutate({
        name: 'New Entity',
        base_currency: 'USD',
        entity_type: 'main' as const,
      })

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true)
      })

      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: ['entities', 'list', mockOrgId],
      })
    })
  })

  describe('useUpdateEntity', () => {
    it('should update an existing entity', async () => {
      const updatedEntity = { ...mockEntity, name: 'Updated Entity' }
      vi.mocked(api.patch).mockResolvedValue(updatedEntity)

      const { result } = renderHook(() => useUpdateEntity(mockEntityId), {
        wrapper,
      })

      const updateData = {
        name: 'Updated Entity',
      }

      result.current.mutate(updateData)

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true)
      })

      expect(api.patch).toHaveBeenCalledWith(
        `/organizations/${mockOrgId}/entities/${mockEntityId}`,
        updateData
      )
    })

    it('should invalidate both list and detail queries after update', async () => {
      vi.mocked(api.patch).mockResolvedValue(mockEntity)

      const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries')

      const { result } = renderHook(() => useUpdateEntity(mockEntityId), {
        wrapper,
      })

      result.current.mutate({ name: 'Updated' })

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true)
      })

      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: ['entities', 'list', mockOrgId],
      })
      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: ['entities', 'detail', mockEntityId],
      })
    })
  })

  describe('useDeleteEntity', () => {
    it('should delete an entity', async () => {
      vi.mocked(api.delete).mockResolvedValue(undefined)

      const { result } = renderHook(() => useDeleteEntity(mockEntityId), {
        wrapper,
      })

      result.current.mutate()

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true)
      })

      expect(api.delete).toHaveBeenCalledWith(
        `/organizations/${mockOrgId}/entities/${mockEntityId}`
      )
    })

    it('should invalidate entities list after deletion', async () => {
      vi.mocked(api.delete).mockResolvedValue(undefined)

      const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries')

      const { result } = renderHook(() => useDeleteEntity(mockEntityId), {
        wrapper,
      })

      result.current.mutate()

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true)
      })

      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: ['entities', 'list', mockOrgId],
      })
    })
  })
})
