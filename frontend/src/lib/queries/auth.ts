import { useMutation } from '@tanstack/react-query'
import { apiClient } from '../api/client'
import { useAuthStore } from '../stores/authStore'
import { type LoginRequest, type RegisterRequest, type AuthResponse } from '@/types/auth'
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
      setAuth(data.user, data.access_token, data.refresh_token)
      toast.success('Login successful')
      router.push('/dashboard')
    },
    onError: (error) => {
      toast.error(error.message || 'Failed to login')
    },
  })
}

export function useRegister() {
  const setAuth = useAuthStore((state) => state.setAuth)
  const router = useRouter()

  return useMutation({
    mutationFn: (data: RegisterRequest) =>
      apiClient<AuthResponse>('/auth/register', {
        method: 'POST',
        body: JSON.stringify(data),
      }),
    onSuccess: (data) => {
      setAuth(data.user, data.access_token, data.refresh_token)
      toast.success('Registration successful')
      router.push('/dashboard')
    },
    onError: (error) => {
      toast.error(error.message || 'Failed to register')
    },
  })
}

export function useLogout() {
  const logout = useAuthStore((state) => state.logout)
  const router = useRouter()

  return useMutation({
    mutationFn: () => apiClient('/auth/logout', { method: 'POST' }),
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
