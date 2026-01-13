import { QueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'
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
  /** Set to true to skip org prefix for non-org-scoped endpoints like /auth/* */
  skipOrgPrefix?: boolean
}

/**
 * Helper to build org-scoped endpoint path
 * Endpoints that don't need org prefix: /auth/*, /organizations (root)
 */
export function orgScopedEndpoint(path: string, orgId: string | null): string {
  // Skip org prefix for auth endpoints and organization management
  const skipPrefixPatterns = [
    /^\/auth\//,
    /^\/organizations$/,
    /^\/organizations\/[^/]+$/, // GET/PATCH single org
  ]
  
  if (skipPrefixPatterns.some(pattern => pattern.test(path))) {
    return path
  }
  
  // If we have an orgId and path doesn't already have org prefix, add it
  if (orgId && !path.startsWith('/organizations/')) {
    return `/organizations/${orgId}${path}`
  }
  
  return path
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
  
  // Build the final endpoint path with org prefix if needed
  const finalEndpoint = options?.skipOrgPrefix 
    ? endpoint 
    : orgScopedEndpoint(endpoint, orgId)
  
  const makeRequest = async (authToken: string | null): Promise<Response> => {
    return fetch(`${baseUrl}${finalEndpoint}`, {
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
      
      // Show toast notification based on status code
      switch (res.status) {
        case 400:
          // Validation error - show the specific message
          toast.error(message)
          break
        case 401:
          toast.error('Session expired, please login again')
          throw new UnauthorizedError(message)
        case 403:
          toast.error('Permission denied')
          throw new PermissionDeniedError(message)
        case 404:
          toast.error('Resource not found')
          break
        case 409:
          // Conflict error - show the specific message
          toast.error(message)
          break
        case 422:
          // Validation error with details
          if (details) {
            const detailMessages = Object.entries(details)
              .map(([field, errors]) => `${field}: ${(errors as string[]).join(', ')}`)
              .join('\n')
            toast.error(detailMessages || message)
          } else {
            toast.error(message)
          }
          break
        default:
          if (res.status >= 500) {
            toast.error('Server error, please try again')
          } else {
            toast.error(message)
          }
      }
      
      throw new ApiError(message, res.status, code, details)
    }
    
    return res.json()
  } catch (error) {
    // Handle network errors
    if (error instanceof TypeError && error.message.includes('fetch')) {
      const networkError = new ApiError(
        'Unable to connect to server. Please check your internet connection.',
        0,
        'NETWORK_ERROR'
      )
      toast.error(networkError.message)
      throw networkError
    }
    
    // Handle timeout
    if (error instanceof DOMException && error.name === 'AbortError') {
      const timeoutError = new ApiError(
        'Request timed out. Please try again.',
        0,
        'TIMEOUT'
      )
      toast.error(timeoutError.message)
      throw timeoutError
    }
    
    // Re-throw ApiError instances (already handled with toast)
    if (error instanceof ApiError) {
      throw error
    }
    
    // Handle unknown errors
    const unknownError = new ApiError(
      error instanceof Error ? error.message : 'An unexpected error occurred',
      0,
      'UNKNOWN'
    )
    toast.error(unknownError.message)
    throw unknownError
  }
}
