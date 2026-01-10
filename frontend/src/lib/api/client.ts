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
      { id: 'acc_001', code: '1100', name: 'Cash', account_type: 'asset', balance: '150000.0000', is_active: true },
      { id: 'acc_002', code: '1200', name: 'Bank BCA', account_type: 'asset', balance: '500000.0000', is_active: true },
      { id: 'acc_003', code: '5100', name: 'Marketing Expense', account_type: 'expense', balance: '25000.0000', is_active: true },
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
      },
      {
        id: 'txn_003',
        reference_number: 'TXN-2026-0003',
        transaction_type: 'journal',
        transaction_date: '2026-01-17',
        description: 'Accrued Rent Expense',
        status: 'pending',
        entries: [
          { account_code: '5300', account_name: 'Rent Expense', debit: '2500.0000', credit: '0.0000' },
          { account_code: '2100', account_name: 'Accrued Expenses', debit: '0.0000', credit: '2500.0000' },
        ]
      },
      {
        id: 'txn_004_test',
        reference_number: 'TXN-TEST-PENDING',
        transaction_type: 'expense',
        transaction_date: '2026-01-20',
        description: 'Test Pending for Approval',
        status: 'pending',
        entries: [
          { account_code: '5200', account_name: 'Office Supplies', debit: '500.0000', credit: '0.0000' },
          { account_code: '1100', account_name: 'Cash', debit: '0.0000', credit: '500.0000' },
        ]
      }
    ],
    pagination: { page: 1, limit: 50, total: 3 }
  },
  '/budgets': {
    data: [
      { id: 'bdg_001', department: 'Engineering', budget_limit: '50000.0000', actual_spent: '35000.0000', period: '2026-01' },
      { id: 'bdg_002', department: 'Marketing', budget_limit: '25000.0000', actual_spent: '28000.0000', period: '2026-01' },
      { id: 'bdg_003', department: 'Operations', budget_limit: '15000.0000', actual_spent: '12000.0000', period: '2026-01' },
      { id: 'bdg_004', department: 'HR', budget_limit: '10000.0000', actual_spent: '5000.0000', period: '2026-01' },
    ]
  },
  '/reports/trial-balance': {
    data: [
      { code: '1100', name: 'Cash', debit: '150000.0000', credit: '0.0000', net_balance: '150000.0000', type: 'asset' },
      { code: '1200', name: 'Bank BCA', debit: '500000.0000', credit: '0.0000', net_balance: '500000.0000', type: 'asset' },
      { code: '2100', name: 'Accounts Payable', debit: '0.0000', credit: '25000.0000', net_balance: '-25000.0000', type: 'liability' },
      { code: '3100', name: 'Capital Stock', debit: '0.0000', credit: '600000.0000', net_balance: '-600000.0000', type: 'equity' },
      { code: '4100', name: 'Service Revenue', debit: '0.0000', credit: '50000.0000', net_balance: '-50000.0000', type: 'revenue' },
      { code: '5100', name: 'Marketing Expense', debit: '15000.0000', credit: '0.0000', net_balance: '15000.0000', type: 'expense' },
      { code: '5200', name: 'Office Supplies', debit: '5000.0000', credit: '0.0000', net_balance: '5000.0000', type: 'expense' },
      { code: '5300', name: 'Rent Expense', debit: '5000.0000', credit: '0.0000', net_balance: '5000.0000', type: 'expense' },
    ],
    total_debit: '675000.0000',
    total_credit: '675000.0000'
  },
  '/reports/income-statement': {
    data: {
      revenues: [
        { code: '4100', name: 'Service Revenue', amount: '50000.0000' }
      ],
      expenses: [
        { code: '5100', name: 'Marketing Expense', amount: '15000.0000' },
        { code: '5200', name: 'Office Supplies', amount: '5000.0000' },
        { code: '5300', name: 'Rent Expense', amount: '5000.0000' }
      ],
      total_revenue: '50000.0000',
      total_expenses: '25000.0000',
      net_income: '25000.0000'
    }
  },
  '/reports/balance-sheet': {
    data: {
      assets: [
        { code: '1100', name: 'Cash', amount: '150000.0000' },
        { code: '1200', name: 'Bank BCA', amount: '500000.0000' }
      ],
      liabilities: [
        { code: '2100', name: 'Accounts Payable', amount: '25000.0000' }
      ],
      equity: [
        { code: '3100', name: 'Capital Stock', amount: '600000.0000' },
        { code: '3200', name: 'Retained Earnings', amount: '25000.0000' }
      ],
      total_assets: '650000.0000',
      total_liabilities: '25000.0000',
      total_equity: '625000.0000'
    }
  },
  '/fiscal-years': {
    data: [
      {
        id: 'fy_2026',
        name: 'FY 2026',
        status: 'open',
        start_date: '2026-01-01',
        end_date: '2026-12-31',
        periods: [
          { id: 'fp_2026_01', name: 'January 2026', status: 'open', start_date: '2026-01-01', end_date: '2026-01-31' },
          { id: 'fp_2026_02', name: 'February 2026', status: 'closed', start_date: '2026-02-01', end_date: '2026-02-28' },
          { id: 'fp_2026_03', name: 'March 2026', status: 'locked', start_date: '2026-03-01', end_date: '2026-03-31' },
        ]
      },
      {
        id: 'fy_2025',
        name: 'FY 2025',
        status: 'closed',
        start_date: '2025-01-01',
        end_date: '2025-12-31',
        periods: []
      }
    ]
  },
  '/dimensions': [
    {
      id: 'dim_dept',
      code: 'DEPT',
      name: 'Department',
      values: [
        { id: 'val_eng', code: 'ENG', name: 'Engineering', description: 'Product and Tech team', is_active: true },
        { id: 'val_mkt', code: 'MKT', name: 'Marketing', description: 'Growth and Brand', is_active: true },
        { id: 'val_ops', code: 'OPS', name: 'Operations', description: 'General operations', is_active: true },
      ]
    },
    {
      id: 'dim_proj',
      code: 'PROJ',
      name: 'Project',
      values: [
        { id: 'val_p1', code: 'P001', name: 'Website Redesign', is_active: true },
        { id: 'val_p2', code: 'P002', name: 'Q1 Campaign', is_active: true },
      ]
    }
  ],
  '/dimension-types': { // Mock for POST /dimension-types
    id: 'dim_new', code: 'NEW', name: 'New Dimension', values: []
  },
  '/exchange-rates': {
    data: [
      { id: 'er_1', from_currency: 'USD', to_currency: 'IDR', rate: '15500.00', date: '2026-01-01' },
      { id: 'er_2', from_currency: 'SGD', to_currency: 'IDR', rate: '11500.00', date: '2026-01-01' },
    ]
  },
  '/reports/dimensional': {
    dimension: 'DEPT',
    data: [
      {
        id: 'val_eng',
        name: 'Engineering',
        revenue: '0.0000',
        expense: '45000.0000',
        net_profit: '-45000.0000',
        breakdown: [
          { account: 'Salaries', amount: '30000.0000' },
          { account: 'Server Costs', amount: '15000.0000' }
        ]
      },
      {
        id: 'val_mkt',
        name: 'Marketing',
        revenue: '0.0000',
        expense: '15000.0000',
        net_profit: '-15000.0000',
        breakdown: [
          { account: 'Ads', amount: '12000.0000' },
          { account: 'Events', amount: '3000.0000' }
        ]
      },
      {
        id: 'val_sales',
        name: 'Sales',
        revenue: '120000.0000',
        expense: '10000.0000',
        net_profit: '110000.0000',
        breakdown: [
          { account: 'Commissions', amount: '8000.0000' },
          { account: 'Travel', amount: '2000.0000' }
        ]
      }
    ],
    summary: {
      global_revenue: '120000.0000',
      global_expense: '70000.0000',
      global_net: '50000.0000'
    }
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

      // Special handling for reports/dimensional to respect 'dimension' param
      if (cleanEndpoint === '/reports/dimensional') {
        const url = new URL(`http://localhost${endpoint}`)
        const dimParam = url.searchParams.get('dimension')
        if (dimParam === 'PROJ') {
          mockResponse = {
            dimension: 'PROJ',
            data: [
              {
                id: 'val_p1',
                name: 'Website Redesign',
                revenue: '0.0000',
                expense: '25000.0000',
                net_profit: '-25000.0000',
                breakdown: [{ account: 'Dev Agency', amount: '20000.0000' }, { account: 'Assets', amount: '5000.0000' }]
              },
              {
                id: 'val_p2',
                name: 'Q1 Campaign',
                revenue: '80000.0000',
                expense: '30000.0000',
                net_profit: '50000.0000',
                breakdown: [{ account: 'Ads', amount: '25000.0000' }, { account: 'Creative', amount: '5000.0000' }]
              }
            ],
            summary: { global_revenue: '80000.0000', global_expense: '55000.0000', global_net: '25000.0000' }
          }
        }
      }

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
        // Check for /transactions/:id
        else if (cleanEndpoint.match(/^\/transactions\/[^\/]+$/)) {
          mockResponse = MOCK_DATA['/transactions'].data[0]
        }
        // Check for /budgets/:id
        else if (cleanEndpoint.match(/^\/budgets\/[^\/]+$/)) {
          // Return a detailed budget mock
          mockResponse = {
            id: cleanEndpoint.split('/').pop(),
            department: 'Engineering',
            period: '2026-01',
            budget_limit: '50000.0000',
            actual_spent: '35000.0000',
            lines: [
              { id: 'bl_1', account_name: 'Server Cost', limit: '30000.0000', actual: '25000.0000' },
              { id: 'bl_2', account_name: 'Software Licenses', limit: '20000.0000', actual: '10000.0000' },
            ]
          }
        }
        // Check for /budgets/:id/lines (POST)
        else if (cleanEndpoint.match(/^\/budgets\/[^\/]+\/lines$/)) {
          mockResponse = {
            id: `bl_${Date.now()}`,
            account_name: 'New Budget Line',
            limit: '1000.0000',
            actual: '0.0000'
          }
        }
        // Check for /accounts/:id/status (PATCH)
        else if (cleanEndpoint.match(/^\/accounts\/[^/]+\/status$/)) {
          mockResponse = { success: true, is_active: true }
        }
        // Check for /dimensions/:typeId/values/:id (PATCH) or /status
        else if (cleanEndpoint.match(/^\/dimensions\/[^/]+\/values\/[^/]+(\/status)?$/)) {
          mockResponse = { success: true }
        }
        // Check for /dimension-types (POST)
        else if (cleanEndpoint === '/dimension-types') {
          mockResponse = { id: `dim_${Date.now()}`, code: 'NEW', name: 'New Dimension', values: [] }
        }
        // Check for /exchange-rates/bulk (POST)
        else if (cleanEndpoint === '/exchange-rates/bulk') {
          mockResponse = { success: true, count: 5 }
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
