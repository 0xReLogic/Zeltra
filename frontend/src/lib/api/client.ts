import { QueryClient } from '@tanstack/react-query'
import { useAuthStore } from '../stores/authStore'

const API_BASE = process.env.NEXT_PUBLIC_API_URL || ''

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 30 * 1000,
      gcTime: 5 * 60 * 1000,
      retry: 1,
    },
  },
})

// Custom error classes for better error handling
export class ApiError extends Error {
  constructor(
    message: string,
    public status: number,
    public code?: string,
    public details?: Record<string, string[]>
  ) {
    super(message)
    this.name = 'ApiError'
  }
}

export class PermissionDeniedError extends ApiError {
  constructor(message: string = 'Permission denied') {
    super(message, 403, 'PERMISSION_DENIED')
    this.name = 'PermissionDeniedError'
  }
}

export class UnauthorizedError extends ApiError {
  constructor(message: string = 'Unauthorized') {
    super(message, 401, 'UNAUTHORIZED')
    this.name = 'UnauthorizedError'
  }
}

// Token refresh function
async function refreshAccessToken(): Promise<boolean> {
  const state = useAuthStore.getState()
  const { refreshToken, isRefreshing, setTokens, setRefreshing, logout } = state
  
  if (!refreshToken || isRefreshing) return false
  
  setRefreshing(true)
  
  try {
    const baseUrl = API_BASE || '/api/v1'
    const res = await fetch(`${baseUrl}/auth/refresh`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ refresh_token: refreshToken }),
      signal: AbortSignal.timeout(10000)
    })
    
    if (!res.ok) {
      logout()
      return false
    }
    
    const data = await res.json()
    setTokens(data.access_token, data.refresh_token, data.expires_in)
    return true
  } catch {
    logout()
    return false
  } finally {
    setRefreshing(false)
  }
}

interface ApiClientOptions extends RequestInit {
  skipAuth?: boolean
}

export async function apiClient<T>(
  endpoint: string,
  options?: ApiClientOptions
): Promise<T> {
  const baseUrl = API_BASE || '/api/v1'
  
  // Get auth state
  let token: string | null = null
  let orgId: string | null = null
  
  if (typeof window !== 'undefined' && !options?.skipAuth) {
    const state = useAuthStore.getState()
    token = state.accessToken
    orgId = state.currentOrgId
  }
  
  const makeRequest = async (authToken: string | null): Promise<Response> => {
    return fetch(`${baseUrl}${endpoint}`, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...(authToken && { Authorization: `Bearer ${authToken}` }),
        ...(orgId && { 'X-Organization-ID': orgId }),
        ...options?.headers,
      },
      signal: AbortSignal.timeout(30000)
    })
  }
  
  try {
    let res = await makeRequest(token)
    
    // Handle 401 - attempt token refresh
    if (res.status === 401 && token && !options?.skipAuth) {
      const refreshed = await refreshAccessToken()
      if (refreshed) {
        const newToken = useAuthStore.getState().accessToken
        res = await makeRequest(newToken)
      } else {
        // Redirect to login
        if (typeof window !== 'undefined') {
          window.location.href = '/login'
        }
        throw new UnauthorizedError('Session expired. Please login again.')
      }
    }
    
    // Handle error responses
    if (!res.ok) {
      const errorBody = await res.json().catch(() => ({}))
      const message = errorBody.error?.message || `API Error: ${res.status} ${res.statusText}`
      const code = errorBody.error?.code
      const details = errorBody.error?.details
      
      if (res.status === 403) {
        throw new PermissionDeniedError(message)
      }
      
      if (res.status === 401) {
        throw new UnauthorizedError(message)
      }
      
      throw new ApiError(message, res.status, code, details)
    }
    
    return res.json()
  } catch (error) {
    // Handle network errors
    if (error instanceof TypeError && error.message.includes('fetch')) {
      throw new ApiError(
        'Unable to connect to server. Please check your internet connection.',
        0,
        'NETWORK_ERROR'
      )
    }
    
    // Handle timeout
    if (error instanceof DOMException && error.name === 'AbortError') {
      throw new ApiError(
        'Request timed out. Please try again.',
        0,
        'TIMEOUT'
      )
    }
    
    // Re-throw ApiError instances
    if (error instanceof ApiError) {
      throw error
    }
    
    // Handle unknown errors
    throw new ApiError(
      error instanceof Error ? error.message : 'An unexpected error occurred',
      0,
      'UNKNOWN'
    )
  }
}
