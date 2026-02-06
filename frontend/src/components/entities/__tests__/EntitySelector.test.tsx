/**
 * EntitySelector Component Tests
 * 
 * Tests for the EntitySelector component including:
 * - Component rendering with entities list
 * - Auto-select when only one entity exists
 * - localStorage persistence
 * - localStorage restoration
 * - Entity selection updates context
 * 
 * Requirements: 7.1, 7.2, 7.3, 7.4, 15.1, 15.2, 15.3
 */

import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import { userEvent } from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { EntitySelector } from '../EntitySelector'
import { useEntities } from '@/lib/queries/entities'
import { useAuthStore } from '@/lib/stores/authStore'

// Mock the queries and stores
vi.mock('@/lib/queries/entities')
vi.mock('@/lib/stores/authStore')

const mockEntities = [
  {
    id: 'entity-1',
    name: 'Main Company',
    entity_type: 'main',
    base_currency: 'USD',
    organization_id: 'org-1',
    is_active: true,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  },
  {
    id: 'entity-2',
    name: 'Subsidiary A',
    entity_type: 'subsidiary',
    base_currency: 'EUR',
    organization_id: 'org-1',
    is_active: true,
    created_at: '2024-01-02T00:00:00Z',
    updated_at: '2024-01-02T00:00:00Z',
  },
]

describe('EntitySelector', () => {
  let queryClient: QueryClient
  let mockSetCurrentEntityId: ReturnType<typeof vi.fn>

  beforeEach(() => {
    queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false },
      },
    })
    
    // Clear localStorage
    localStorage.clear()
    
    // Setup mocks
    mockSetCurrentEntityId = vi.fn()
    vi.mocked(useAuthStore).mockReturnValue({
      currentEntityId: null,
      setCurrentEntityId: mockSetCurrentEntityId,
    } as any)
  })

  const renderComponent = () => {
    return render(
      <QueryClientProvider client={queryClient}>
        <EntitySelector />
      </QueryClientProvider>
    )
  }

  it('should render with entities list', async () => {
    vi.mocked(useEntities).mockReturnValue({
      data: mockEntities,
      isLoading: false,
      error: null,
    } as any)

    renderComponent()

    await waitFor(() => {
      expect(screen.getByText('Select entity')).toBeInTheDocument()
    })
  })

  it('should auto-select when only one entity exists', async () => {
    const singleEntity = [mockEntities[0]]
    
    vi.mocked(useEntities).mockReturnValue({
      data: singleEntity,
      isLoading: false,
      error: null,
    } as any)

    renderComponent()

    await waitFor(() => {
      expect(mockSetCurrentEntityId).toHaveBeenCalledWith('entity-1')
      expect(localStorage.getItem('zeltra:currentEntityId')).toBe('entity-1')
    })
  })

  it('should persist selection to localStorage', async () => {
    vi.mocked(useEntities).mockReturnValue({
      data: mockEntities,
      isLoading: false,
      error: null,
    } as any)

    vi.mocked(useAuthStore).mockReturnValue({
      currentEntityId: 'entity-2',
      setCurrentEntityId: mockSetCurrentEntityId,
    } as any)

    renderComponent()

    await waitFor(() => {
      expect(localStorage.getItem('zeltra:currentEntityId')).toBe('entity-2')
    })
  })

  it('should restore selection from localStorage on mount', async () => {
    localStorage.setItem('zeltra:currentEntityId', 'entity-2')
    
    vi.mocked(useEntities).mockReturnValue({
      data: mockEntities,
      isLoading: false,
      error: null,
    } as any)

    vi.mocked(useAuthStore).mockReturnValue({
      currentEntityId: null,
      setCurrentEntityId: mockSetCurrentEntityId,
    } as any)

    renderComponent()

    await waitFor(() => {
      expect(mockSetCurrentEntityId).toHaveBeenCalledWith('entity-2')
    })
  })

  it('should update context when entity is selected', async () => {
    const user = userEvent.setup()
    
    vi.mocked(useEntities).mockReturnValue({
      data: mockEntities,
      isLoading: false,
      error: null,
    } as any)

    vi.mocked(useAuthStore).mockReturnValue({
      currentEntityId: 'entity-1',
      setCurrentEntityId: mockSetCurrentEntityId,
    } as any)

    renderComponent()

    // Click the select trigger
    const trigger = await screen.findByRole('combobox')
    await user.click(trigger)

    // Select the second entity
    const option = await screen.findByText('Subsidiary A')
    await user.click(option)

    await waitFor(() => {
      expect(mockSetCurrentEntityId).toHaveBeenCalledWith('entity-2')
    })
  })

  it('should show loading state', () => {
    vi.mocked(useEntities).mockReturnValue({
      data: undefined,
      isLoading: true,
      error: null,
    } as any)

    renderComponent()

    expect(screen.getByRole('status')).toBeInTheDocument()
  })

  it('should show error state', () => {
    vi.mocked(useEntities).mockReturnValue({
      data: undefined,
      isLoading: false,
      error: new Error('Failed to load'),
    } as any)

    renderComponent()

    expect(screen.getByText('Failed to load entities')).toBeInTheDocument()
  })

  it('should show no entities state', () => {
    vi.mocked(useEntities).mockReturnValue({
      data: [],
      isLoading: false,
      error: null,
    } as any)

    renderComponent()

    expect(screen.getByText('No entities available')).toBeInTheDocument()
  })
})
