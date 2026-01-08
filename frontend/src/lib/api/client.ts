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
  },
  '/accounts': {
     data: [
        { id: 'acc_001', code: '1100', name: 'Cash', account_type: 'asset', balance: '150000.0000' },
        { id: 'acc_002', code: '1200', name: 'Bank BCA', account_type: 'asset', balance: '500000.0000' },
        { id: 'acc_003', code: '5100', name: 'Marketing Expense', account_type: 'expense', balance: '25000.0000' },
        { id: 'acc_004', code: '5200', name: 'Office Supplies', account_type: 'expense', balance: '8000.0000' },
     ]
  },
  '/transactions': {
      data: [
        {
          id: 'txn_001',
          reference_number: 'TXN-2026-0001',
          transaction_type: 'expense',
          transaction_date: '2026-01-15',
          description: 'Office supplies purchase',
          status: 'posted',
          entries: [
            { account_code: '5200', account_name: 'Office Supplies', debit: '150.0000', credit: '0.0000' },
            { account_code: '1100', account_name: 'Cash', debit: '0.0000', credit: '150.0000' },
          ]
        },
        {
          id: 'txn_002',
          reference_number: 'TXN-2026-0002',
          transaction_type: 'revenue',
          transaction_date: '2026-01-16',
          description: 'Project Alpha Payment',
          status: 'approved',
          entries: [
            { account_code: '1200', account_name: 'Bank BCA', debit: '5000.0000', credit: '0.0000' },
            { account_code: '4100', account_name: 'Service Revenue', debit: '0.0000', credit: '5000.0000' },
          ]
        }
      ],
      pagination: { page: 1, limit: 50, total: 2 }
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
      // Short timeout for fast fallback
       signal: AbortSignal.timeout(3000) 
    })
    
    if (!res.ok) {
       // If 404, it might be that the BE is down or not found, try mock fallback
       if (res.status === 404 && process.env.NEXT_PUBLIC_API_MOCK === 'true') {
           throw new Error('Fallback to mock')
       }
      const error = await res.json().catch(() => ({}))
      throw new Error(error.error?.message || `API Error: ${res.status} ${res.statusText}`)
    }
    return res.json()
  } catch (error) {
    // Fallback Mock Logic
    if (process.env.NEXT_PUBLIC_API_MOCK === 'true') {
      // Clean endpoint for matching (remove query params)
      const cleanEndpoint = endpoint.split('?')[0]
      console.warn(`[Mock Fallback] API request failed (${error}), using internal mock for: ${cleanEndpoint}`)
      
      // 1. Try exact match
      let mockResponse = MOCK_DATA[cleanEndpoint]

      // 2. Try matching dynamic routes (simple heiristic)
      if (!mockResponse) {
          // Check for /accounts/:id
          if (cleanEndpoint.match(/^\/accounts\/[^/]+$/)) {
              // Return a single account mock
              mockResponse = MOCK_DATA['/accounts'].data[0]
          }
          // Check for /accounts/:id/ledger
           else if (cleanEndpoint.match(/^\/accounts\/[^/]+\/ledger$/)) {
              mockResponse = MOCK_DATA['/transactions'] // Reuse transactions mock for ledger
          }
      }

      if (mockResponse) {
        // Simulate network delay
        await new Promise(resolve => setTimeout(resolve, 300))
        return mockResponse as T
      }
    }
    throw error
  }
}
