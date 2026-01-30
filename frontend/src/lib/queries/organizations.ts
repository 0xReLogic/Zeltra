/**
 * Organization queries
 * 
 * Note: In the entities model, users have ONE organization (workspace)
 * and can create multiple entities (companies) within that workspace.
 * Multi-organization switching has been replaced with entity selection.
 */

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from '@/lib/api/client'
import { Organization, OrganizationUser, CreateOrganizationRequest, UpdateOrganizationRequest, InviteUserRequest, UpdateUserRoleRequest } from '@/types/organizations'
import { useAuthStore } from '@/lib/stores/authStore'

/**
 * Create organization (for initial onboarding)
 * Note: Users now have only one organization
 */
export function useCreateOrganization() {
  const queryClient = useQueryClient()
  const setOrg = useAuthStore((state) => state.setOrg)
  const addOrganization = useAuthStore((state) => state.addOrganization)

  return useMutation({
    mutationFn: (data: CreateOrganizationRequest) =>
      apiClient<Organization>('/organizations', {
        method: 'POST',
        body: JSON.stringify(data),
      }),
    onSuccess: (newOrg) => {
      // Add the new organization to user's organizations array
      addOrganization({
        id: newOrg.id,
        name: newOrg.name,
        slug: newOrg.slug,
        role: 'owner', // Creator is always owner
      })
      
      // Switch to the new organization
      setOrg(newOrg.id)
      
      queryClient.invalidateQueries({ queryKey: ['organization'] })
    },
  })
}

/**
 * Get user's organization
 * Note: Users now have only one organization
 */
export function useOrganization() {
  const currentOrgId = useAuthStore((state) => state.currentOrgId)

  return useQuery({
    queryKey: ['organization', currentOrgId],
    queryFn: () => apiClient<Organization>(`/organizations/${currentOrgId}`),
    enabled: !!currentOrgId,
    staleTime: 5 * 60 * 1000, // 5 minutes
  })
}

export function useUpdateOrganization() {
  const queryClient = useQueryClient()
  const currentOrgId = useAuthStore((state) => state.currentOrgId)

  return useMutation({
    mutationFn: (data: UpdateOrganizationRequest) =>
      apiClient<Organization>(`/organizations/${currentOrgId}`, {
        method: 'PATCH',
        body: JSON.stringify(data),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['organization', currentOrgId] })
    },
  })
}

export function useOrganizationUsers() {
  const currentOrgId = useAuthStore((state) => state.currentOrgId)

  return useQuery({
    queryKey: ['organization-users', currentOrgId],
    queryFn: () => apiClient<{ data: OrganizationUser[] }>(`/organizations/${currentOrgId}/users`),
    enabled: !!currentOrgId,
  })
}

export function useInviteUser() {
  const queryClient = useQueryClient()
  const currentOrgId = useAuthStore((state) => state.currentOrgId)

  return useMutation({
    mutationFn: (data: InviteUserRequest) =>
      apiClient<OrganizationUser>(`/organizations/${currentOrgId}/users`, {
        method: 'POST',
        body: JSON.stringify(data),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['organization-users', currentOrgId] })
    },
  })
}

export function useUpdateUserRole() {
  const queryClient = useQueryClient()
  const currentOrgId = useAuthStore((state) => state.currentOrgId)

  return useMutation({
    mutationFn: ({ userId, data }: { userId: string; data: UpdateUserRoleRequest }) =>
      apiClient<OrganizationUser>(`/organizations/${currentOrgId}/users/${userId}`, {
        method: 'PATCH',
        body: JSON.stringify(data),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['organization-users', currentOrgId] })
    },
  })
}

export function useRemoveUser() {
  const queryClient = useQueryClient()
  const currentOrgId = useAuthStore((state) => state.currentOrgId)

  return useMutation({
    mutationFn: (userId: string) =>
      apiClient<{ success: true }>(`/organizations/${currentOrgId}/users/${userId}`, {
        method: 'DELETE',
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['organization-users', currentOrgId] })
    },
  })
}
