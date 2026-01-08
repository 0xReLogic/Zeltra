export interface Organization {
  id: string
  name: string
  slug: string
  base_currency: string
  timezone: string
  created_at: string
  subscription_tier: string
}

export interface OrganizationUser {
  id: string
  full_name: string
  email: string
  role: 'owner' | 'admin' | 'accountant' | 'approver' | 'viewer'
  status: 'active' | 'invited' | 'disabled'
  joined_at: string | null
  approval_limit?: string
}

export interface UpdateOrganizationRequest {
  base_currency?: string
  timezone?: string
}

export interface InviteUserRequest {
  email: string
  role: OrganizationUser['role']
}

export interface UpdateUserRoleRequest {
  role: OrganizationUser['role']
  approval_limit?: number
}
