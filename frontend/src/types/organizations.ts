export interface Organization {
  id: string
  name: string
  slug: string
  base_currency: string
  timezone: string
  created_at: string
  subscription_tier: string
  subscription_status: string
  trial_ends_at?: string | null
}

// All 6 roles as defined in backend OpenAPI spec
export type UserRole = 'owner' | 'admin' | 'approver' | 'accountant' | 'viewer' | 'submitter'

export interface OrganizationUser {
  id: string
  full_name: string
  email: string
  role: UserRole
  status: 'active' | 'invited' | 'disabled'
  joined_at: string | null
  approval_limit?: string
}

export interface CreateOrganizationRequest {
  name: string
  slug: string
  base_currency: string
  timezone?: string
}

export interface UpdateOrganizationRequest {
  base_currency?: string
  timezone?: string
}

export interface InviteUserRequest {
  email: string
  role: UserRole
}

export interface UpdateUserRoleRequest {
  role: UserRole
  approval_limit?: number
}
