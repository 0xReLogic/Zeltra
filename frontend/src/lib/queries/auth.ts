import { useMutation } from '@tanstack/react-query'
import { apiClient } from '../api/client'
import { useAuthStore } from '../stores/authStore'
import { type LoginRequest, type RegisterRequest, type AuthResponse, type RegisterResponse, type VerifyEmailRequest, type VerifyEmailResponse, type ResendVerificationRequest, type ResendVerificationResponse } from '@/types/auth'
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
