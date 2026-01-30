/**
 * EntitySelector Component
 * 
 * Dropdown component for selecting the current entity context.
 * Features:
 * - Auto-selects if only one entity exists
 * - Persists selection to localStorage
 * - Restores selection from localStorage on mount
 * - Triggers data refresh when entity changes
 * 
 * Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 15.1, 15.2, 15.3, 15.4
 */

'use client'

import { useEffect } from 'react'
import { Building2 } from 'lucide-react'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { useEntities } from '@/lib/queries/entities'
import { useAuthStore } from '@/lib/stores/authStore'
import { Skeleton } from '@/components/ui/skeleton'
import type { Entity } from '@/types/entities'

const ENTITY_STORAGE_KEY = 'zeltra:currentEntityId'

export function EntitySelector() {
  const { data: entities, isLoading, error } = useEntities()
  const currentEntityId = useAuthStore((state) => state.currentEntityId)
  const setCurrentEntityId = useAuthStore((state) => state.setCurrentEntityId)

  // Auto-select if only one entity exists
  useEffect(() => {
    if (entities && entities.length === 1 && !currentEntityId) {
      const entityId = entities[0].id
      console.log('🏢 Auto-selecting single entity:', entityId)
      setCurrentEntityId(entityId)
      localStorage.setItem(ENTITY_STORAGE_KEY, entityId)
    }
  }, [entities, currentEntityId, setCurrentEntityId])

  // Restore selection from localStorage on mount
  useEffect(() => {
    const storedEntityId = localStorage.getItem(ENTITY_STORAGE_KEY)
    if (storedEntityId && entities?.some((entity: Entity) => entity.id === storedEntityId)) {
      if (currentEntityId !== storedEntityId) {
        console.log('🏢 Restoring entity from localStorage:', storedEntityId)
        setCurrentEntityId(storedEntityId)
      }
    }
  }, [entities, currentEntityId, setCurrentEntityId])

  // Persist selection to localStorage when it changes
  useEffect(() => {
    if (currentEntityId) {
      console.log('🏢 Persisting entity to localStorage:', currentEntityId)
      localStorage.setItem(ENTITY_STORAGE_KEY, currentEntityId)
    }
  }, [currentEntityId])

  // Handle entity selection change
  const handleEntityChange = (entityId: string) => {
    console.log('🏢 Entity changed:', entityId)
    setCurrentEntityId(entityId)
    // Note: Data refresh is handled by query invalidation in consuming components
  }

  // Loading state
  if (isLoading) {
    return (
      <div className="flex items-center gap-2 px-3 py-2">
        <Building2 className="h-4 w-4 text-muted-foreground" />
        <Skeleton className="h-4 w-32" />
      </div>
    )
  }

  // Error state
  if (error) {
    return (
      <div className="flex items-center gap-2 px-3 py-2 text-sm text-destructive">
        <Building2 className="h-4 w-4" />
        <span>Failed to load entities</span>
      </div>
    )
  }

  // No entities state
  if (!entities || entities.length === 0) {
    return (
      <div className="flex items-center gap-2 px-3 py-2 text-sm text-muted-foreground">
        <Building2 className="h-4 w-4" />
        <span>No entities available</span>
      </div>
    )
  }

  // Find current entity for display
  const currentEntity = entities.find((entity: Entity) => entity.id === currentEntityId)

  return (
    <div className="flex items-center gap-2">
      <Building2 className="h-4 w-4 text-muted-foreground" />
      <Select value={currentEntityId || undefined} onValueChange={handleEntityChange}>
        <SelectTrigger className="w-[200px]">
          <SelectValue placeholder="Select entity">
            {currentEntity?.name || 'Select entity'}
          </SelectValue>
        </SelectTrigger>
        <SelectContent>
          {entities.map((entity: Entity) => (
            <SelectItem key={entity.id} value={entity.id}>
              <div className="flex flex-col">
                <span className="font-medium">{entity.name}</span>
                <span className="text-xs text-muted-foreground">
                  {entity.entity_type} • {entity.base_currency}
                </span>
              </div>
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  )
}
