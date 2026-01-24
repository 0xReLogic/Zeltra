import { useMutation } from '@tanstack/react-query'
import { apiClient } from '../api/client'
import { useAuthStore } from '../stores/authStore'
import { type LoginRequest, type RegisterRequest, type AuthResponse, type RegisterResponse, type VerifyEmailRequest, type VerifyEmailResponse, type ResendVerificationRequest, type ResendVerificationResponse, type SwitchOrganizationRequest, type SwitchOrganizationResponse } from '@/types/auth'
import { toast } from 'sonner'
import { useRouter } from 'next/navigation'

export function useLogin() {
  const setAuth = useAuthStore((state) => state.setAuth)
  const router = useRouter()

  return useMutation({
    mutationFn: (data: LoginRequest) => 
      apiClient<AuthResponse>('/auth/login', {
        method: 'POST',
        body: JSON.stringify(data),
      }),
    onSuccess: (data) => {
      setAuth(data.user, data.access_token, data.refresh_token, data.expires_in)
      toast.success('Login successful')
      
      // Check if user has organizations
      if (data.user.organizations.length === 0) {
        // Redirect to create organization page
        router.push('/onboarding/create-organization')
      } else {
        router.push('/dashboard')
      }
    },
    // Error handling is done by apiClient - no duplicate toast needed
  })
}

export function useRegister() {
  const router = useRouter()

  return useMutation({
    mutationFn: (data: RegisterRequest) =>
      apiClient<RegisterResponse>('/auth/register', {
        method: 'POST',
        body: JSON.stringify(data),
      }),
    onSuccess: (data) => {
      toast.success(data.message || 'Registration successful! Please check your email to verify your account.')
      router.push('/login')
    },
    // Error handling is done by apiClient - no duplicate toast needed
  })
}

export function useLogout() {
  const logout = useAuthStore((state) => state.logout)
  const refreshToken = useAuthStore((state) => state.refreshToken)
  const router = useRouter()

  return useMutation({
    mutationFn: () => apiClient('/auth/logout', { 
      method: 'POST',
      body: JSON.stringify({ refresh_token: refreshToken || '' }),
    }),
    onSuccess: () => {
      logout()
      router.push('/login')
      toast.success('Logged out')
    },
    // Logout locally even if API fails
    onError: () => {
      logout()
      router.push('/login')
    }
  })
}

export function useVerifyEmail() {
  return useMutation({
    mutationFn: (data: VerifyEmailRequest) =>
      apiClient<VerifyEmailResponse>('/auth/verify-email', {
        method: 'POST',
        body: JSON.stringify(data),
      }),
    onSuccess: (data) => {
      toast.success(data.message || 'Email verified successfully')
    },
    // Error handling is done by apiClient - no duplicate toast needed
  })
}

export function useResendVerification() {
  return useMutation({
    mutationFn: (data: ResendVerificationRequest) =>
      apiClient<ResendVerificationResponse>('/auth/resend-verification', {
        method: 'POST',
        body: JSON.stringify(data),
      }),
    onSuccess: (data) => {
      toast.success(data.message || 'Verification email sent')
    },
    // Error handling is done by apiClient - no duplicate toast needed
  })
}

interface RefreshResponse {
  access_token: string
  refresh_token: string
  expires_in: number
}

export function useRefresh() {
  const setTokens = useAuthStore((state) => state.setTokens)
  const refreshToken = useAuthStore((state) => state.refreshToken)
  
  return useMutation({
    mutationFn: () => 
      apiClient<RefreshResponse>('/auth/refresh', {
        method: 'POST',
        body: JSON.stringify({ refresh_token: refreshToken || '' }),
      }),
    onSuccess: (data) => {
      setTokens(data.access_token, data.refresh_token, data.expires_in)
    },
  })
}

export function useSwitchOrganization() {
  const setAuth = useAuthStore((state) => state.setAuth)
  const user = useAuthStore((state) => state.user)
  const router = useRouter()

  return useMutation({
    mutationFn: (data: SwitchOrganizationRequest) =>
      apiClient<SwitchOrganizationResponse>('/auth/switch-organization', {
        method: 'POST',
        body: JSON.stringify(data),
        skipOrgPrefix: true,
      }),
    onSuccess: (data) => {
      // Update auth store with new tokens and organization
      if (user) {
        // Update user's current organization in the organizations array
        const updatedUser = {
          ...user,
          organizations: user.organizations.map(org => 
            org.id === data.organization.id ? data.organization : org
          )
        }
        setAuth(updatedUser, data.access_token, data.refresh_token, data.expires_in)
      }
      toast.success(`Switched to ${data.organization.name}`)
      // Reload to refresh all data with new organization context
      window.location.reload()
    },
  })
}
