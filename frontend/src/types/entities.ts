/**
 * Entity types for multi-entity accounting
 * 
 * Entities represent legal or operational units (companies, subsidiaries, branches, divisions)
 * within an organization. This enables multi-entity accounting similar to NetSuite/Sage Intacct.
 */

import type { components } from './api.generated'

/**
 * Entity response from API
 */
export type Entity = components['schemas']['EntityResponse']

/**
 * Request body for creating an entity
 */
export type CreateEntityRequest = components['schemas']['CreateEntityRequest']

/**
 * Request body for updating an entity
 */
export type UpdateEntityRequest = components['schemas']['UpdateEntityRequest']

/**
 * Entity type enum
 */
export type EntityType = 'main' | 'subsidiary' | 'branch' | 'division'

/**
 * Helper type for entity selector options
 */
export interface EntityOption {
  value: string
  label: string
  type: EntityType
  currency: string
}
