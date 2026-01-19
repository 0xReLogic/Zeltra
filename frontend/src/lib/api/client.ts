import { QueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'
import { useAuthStore } from '../stores/authStore'
import { useUpgradeStore } from '../stores/upgradeStore'

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

/**
 * Wait for Zustand store to finish hydrating from localStorage.
 * This prevents race conditions where API calls are made before
 * the auth token is loaded from storage.
 */
async function waitForHydration(): Promise<void> {
  // Skip on server-side
  if (typeof window === 'undefined') {
    return
  }
  
  // Safety check for persist middleware
  if (!useAuthStore.persist) {
    return
  }
  
  // If already hydrated, return immediately
  if (useAuthStore.persist.hasHydrated()) {
    return
  }
  
  // Wait for hydration to complete with timeout
  return new Promise((resolve) => {
    const timeout = setTimeout(() => {
      // Timeout after 5 seconds - proceed anyway
      resolve()
    }, 5000)
    
    const unsub = useAuthStore.persist.onFinishHydration(() => {
      clearTimeout(timeout)
      unsub()
      resolve()
    })
  })
}

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

/**
 * Mutex-based token refresh to prevent race conditions.
 * When multiple requests get 401 simultaneously, only ONE refresh happens.
 * Other requests wait for that refresh to complete.
 */
let refreshPromise: Promise<boolean> | null = null

async function refreshAccessToken(): Promise<boolean> {
  // If a refresh is already in progress, wait for it
  if (refreshPromise) {
    console.log('🔄 Refresh already in progress, waiting...')
    return refreshPromise
  }
  
  const state = useAuthStore.getState()
  const { refreshToken, setTokens, logout } = state
  
  if (!refreshToken) {
    console.log('❌ No refresh token available')
    return false
  }
  
  console.log('🔄 Starting token refresh...')
  
  // Create the refresh promise - all concurrent callers will share this
  refreshPromise = (async () => {
    try {
      const baseUrl = API_BASE || '/api/v1'
      const res = await fetch(`${baseUrl}/auth/refresh`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ refresh_token: refreshToken }),
        signal: AbortSignal.timeout(10000)
      })
      
      if (!res.ok) {
        const errorBody = await res.json().catch(() => ({}))
        console.error('❌ Token refresh failed:', res.status, errorBody)
        console.log('🚪 LOGOUT TRIGGERED: Token refresh failed in client.ts')
        logout()
        return false
      }
      
      const data = await res.json()
      // Backend /auth/refresh only returns { access_token, expires_in }
      // It does NOT return a new refresh_token, so we keep the existing one
      setTokens(data.access_token, refreshToken, data.expires_in)
      console.log('✅ Token refreshed successfully, expires in:', data.expires_in, 'seconds')
      return true
    } catch (error) {
      console.error('❌ Token refresh error:', error)
      console.log('🚪 LOGOUT TRIGGERED: Token refresh exception in client.ts')
      logout()
      return false
    } finally {
      // Clear the promise so future refreshes can happen
      refreshPromise = null
    }
  })()
  
  return refreshPromise
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
  
  // Get auth state - wait for hydration first to prevent race conditions
  let token: string | null = null
  let orgId: string | null = null
  
  if (typeof window !== 'undefined' && !options?.skipAuth) {
    // Wait for Zustand to hydrate from localStorage before reading state
    await waitForHydration()
    
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
        ...(['POST', 'PUT', 'PATCH'].includes(options?.method || 'GET') && { 'Content-Type': 'application/json' }),
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
      console.log('⚠️ Got 401 response, attempting token refresh...')
      const refreshed = await refreshAccessToken()
      if (refreshed) {
        const newToken = useAuthStore.getState().accessToken
        console.log('✅ Retrying request with new token')
        res = await makeRequest(newToken)
      } else {
        console.log('❌ Token refresh failed, redirecting to login')
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
        case 402:
          // Payment Required - Trigger Upgrade Modal
          useUpgradeStore.getState().openModal(message)
          // We don't throw an error that triggers a toast here, as the modal is the UI feedback
          // But we throw to stop execution flow
          throw new ApiError(message, 402, 'PAYMENT_REQUIRED')
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
    
    // Handle 204 No Content - no body to parse
    if (res.status === 204) {
      return undefined as T
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
