// Organization Types - imported from OpenAPI generated types
import type { components } from './api.generated'

export type TierLimitsResponse = components['schemas']['TierLimitsResponse']
export type OrganizationResponse = components['schemas']['OrganizationResponse']
export type CreateOrganizationRequest = components['schemas']['CreateOrganizationRequest']
export type UpdateOrganizationRequest = components['schemas']['UpdateOrganizationRequest']
export type AddUserRequest = components['schemas']['AddUserRequest']
export type UpdateMemberRequest = components['schemas']['UpdateMemberRequest']
export type OrgUserResponse = components['schemas']['OrgUserResponse']
export type MembershipResponse = components['schemas']['MembershipResponse']

// Alias for backward compatibility
export type Organization = OrganizationResponse
export type InviteUserRequest = AddUserRequest
export type UpdateUserRoleRequest = UpdateMemberRequest

// Extended OrganizationUser with frontend-specific fields
// Note: Backend OrgUserResponse doesn't have status/joined_at yet
export interface OrganizationUser extends OrgUserResponse {
  status?: 'active' | 'invited' | 'disabled'
  joined_at?: string | null
}

// All 6 roles as defined in backend OpenAPI spec
export type UserRole = 'owner' | 'admin' | 'approver' | 'accountant' | 'viewer' | 'submitter'
