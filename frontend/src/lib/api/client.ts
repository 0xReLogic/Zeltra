import { QueryClient } from '@tanstack/react-query'

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

import { useAuthStore } from '../stores/authStore'

// Fallback Mock Data (in case MSW SW fails on non-secure context)
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const MOCK_DATA: Record<string, any> = {
  '/auth/login': {
    user: { 
      id: 'usr_001', 
      email: 'demo@zeltra.io', 
      full_name: 'Demo User',
      organizations: [
        { id: 'org_001', name: 'Acme Corp', slug: 'acme', role: 'owner' }
      ]
    },
    access_token: 'mock_access_token_xxx',
    refresh_token: 'mock_refresh_token_xxx',
    expires_in: 3600
  },
  '/auth/register': {
    user: { 
      id: 'usr_002', 
      email: 'new@zeltra.io', 
      full_name: 'New User',
      organizations: []
    },
    access_token: 'mock_access_token_yyy',
    refresh_token: 'mock_refresh_token_yyy',
    expires_in: 3600
  }
}

export async function apiClient<T>(
  endpoint: string,
  options?: RequestInit
): Promise<T> {
  // Client-side only
  let token = null
  let orgId = null
  
  // Access store directly (works on client)
  if (typeof window !== 'undefined') {
    const state = useAuthStore.getState()
    token = state.accessToken
    orgId = state.currentOrgId
  }
  
  // Use relative URL by default to avoid 'localhost' issues on remote devices
  const baseUrl = API_BASE || '/api/v1'
  
  try {
    const res = await fetch(`${baseUrl}${endpoint}`, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...(token && { Authorization: `Bearer ${token}` }),
        ...(orgId && { 'X-Organization-ID': orgId }),
        ...options?.headers,
      },
    })
    
    if (!res.ok) {
      const error = await res.json().catch(() => ({}))
      throw new Error(error.error?.message || `API Error: ${res.status} ${res.statusText}`)
    }
    return res.json()
  } catch (error) {
    // Fallback Mock Logic
    if (process.env.NEXT_PUBLIC_API_MOCK === 'true') {
      console.warn(`[Mock Fallback] API request failed, using internal mock for: ${endpoint}`)
      const mockResponse = MOCK_DATA[endpoint]
      if (mockResponse) {
        // Simulate network delay
        await new Promise(resolve => setTimeout(resolve, 500))
        return mockResponse as T
      }
    }
    throw error
  }
}
