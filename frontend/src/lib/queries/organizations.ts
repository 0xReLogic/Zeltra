import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from '@/lib/api/client'
import { Organization, OrganizationUser, UpdateOrganizationRequest, InviteUserRequest, UpdateUserRoleRequest } from '@/types/organizations'
import { useAuthStore } from '@/lib/stores/authStore'

export function useOrganization() {
  const currentOrgId = useAuthStore((state) => state.currentOrgId)

  return useQuery({
    queryKey: ['organization', currentOrgId],
    queryFn: () => apiClient<Organization>(`/organizations/${currentOrgId}`),
    enabled: !!currentOrgId,
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
