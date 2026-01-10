import { http, HttpResponse } from 'msw'

// Keep track of transactions in memory for persistence test
const MOCK_TRANSACTIONS = [
  {
    id: 'txn_001',
    reference_number: 'TXN-2026-0001',
    transaction_type: 'expense' as const,
    transaction_date: '2026-01-15',
    description: 'Office supplies purchase',
    status: 'posted' as const,
    entries: [
      { account_code: '5200', account_name: 'Office Supplies', debit: '150.0000', credit: '0.0000', dimensions: ['val_eng'] },
      { account_code: '1100', account_name: 'Cash', debit: '0.0000', credit: '150.0000' },
    ]
  },
  {
    id: 'txn_002',
    reference_number: 'TXN-2026-0002',
    transaction_type: 'revenue' as const,
    transaction_date: '2026-01-16',
    description: 'Project Alpha Payment',
    status: 'approved' as const,
    entries: [
      { account_code: '1200', account_name: 'Bank BCA', debit: '5000.0000', credit: '0.0000' },
      { account_code: '4100', account_name: 'Service Revenue', debit: '0.0000', credit: '5000.0000', dimensions: ['val_p1'] },
    ]
  },
  {
    id: 'txn_003',
    reference_number: 'TXN-2026-0003',
    transaction_type: 'journal' as const,
    transaction_date: '2026-01-17',
    description: 'Accrued Rent Expense',
    status: 'pending' as const,
    entries: [
      { account_code: '5300', account_name: 'Rent Expense', debit: '2500.0000', credit: '0.0000', dimensions: ['val_ops'] },
      { account_code: '2100', account_name: 'Accrued Expenses', debit: '0.0000', credit: '2500.0000' },
    ]
  },
  {
    id: 'txn_004_test',
    reference_number: 'TXN-TEST-PENDING',
    transaction_type: 'expense' as const,
    transaction_date: '2026-01-20',
    description: 'Test Pending for Approval',
    status: 'pending' as const,
    entries: [
      { account_code: '5200', account_name: 'Office Supplies', debit: '500.0000', credit: '0.0000', dimensions: ['val_eng'] },
      { account_code: '1100', account_name: 'Cash', debit: '0.0000', credit: '500.0000' },
    ]
  }
]

// Keep track of statuses in memory for testing
const budgetStatuses: Record<string, 'open' | 'locked'> = {
    'bdg_002': 'locked' 
}

export const handlers = [
  // Auth
  http.post('/api/v1/auth/login', async ({ request }) => {
    const body = await request.json() as { email?: string; password?: string }

    if (body.email === 'wrong@example.com' || body.password === 'wrongpass') {
        return HttpResponse.json({ error: { message: 'Invalid credentials' } }, { status: 401 })
    }

    return HttpResponse.json({
      user: { 
        id: `usr_${Date.now()}`, 
        email: body.email || 'demo@zeltra.io', 
        full_name: 'Demo User',
        organizations: [
          { id: 'org_001', name: 'Acme Corp', slug: 'acme', role: 'owner' }
        ]
      },
      access_token: 'mock_access_token_xxx',
      refresh_token: 'mock_refresh_token_xxx',
      expires_in: 3600
    })
  }),

  http.post('/api/v1/auth/logout', () => {
    return HttpResponse.json({ success: true })
  }),

  http.post('/api/v1/auth/verify-email', async ({ request }) => {
      const body = await request.json() as { token: string }
      if (body.token === 'invalid_token') {
          return HttpResponse.json({ error: { message: 'Invalid or expired token' } }, { status: 400 })
      }
      return HttpResponse.json({ message: 'Email verified successfully', verified: true })
  }),

  http.post('/api/v1/auth/resend-verification', () => {
      return HttpResponse.json({ message: 'Verification email sent' })
  }),

  // Accounts
  http.get('/api/v1/accounts', () => {
    return HttpResponse.json({
      data: [
        { id: 'acc_001', code: '1100', name: 'Cash', account_type: 'asset', balance: '150000.0000' },
        { id: 'acc_002', code: '1200', name: 'Bank BCA', account_type: 'asset', balance: '500000.0000' },
        { id: 'acc_003', code: '5100', name: 'Marketing Expense', account_type: 'expense', balance: '25000.0000' },
        { id: 'acc_004', code: '5200', name: 'Office Supplies', account_type: 'expense', balance: '8000.0000' },
        { id: 'acc_005', code: '2100', name: 'Accounts Payable', account_type: 'liability', balance: '-25000.0000' },
      ]
    })
  }),

  http.post('/api/v1/accounts', async ({ request }) => {
    const body = await request.json() as Record<string, unknown>
    return HttpResponse.json({
        id: `acc_${Date.now()}`,
        balance: '0.0000',
        ...body
    })
  }),

  http.put('/api/v1/accounts/:id', async ({ request, params }) => {
    const body = await request.json() as Record<string, unknown>
    return HttpResponse.json({
        id: params.id,
        balance: '0.0000', 
        ...body
    })
  }),

  http.delete('/api/v1/accounts/:id', () => {
    return HttpResponse.json({ success: true })
  }),

  // Account Ledger
  http.get('/api/v1/accounts/:id/ledger', () => {
    return HttpResponse.json({
        data: [
            { id: 'le_001', transaction_date: '2026-01-01', reference_number: 'TXN-001', description: 'Opening Balance', debit: '100000.0000', credit: '0.0000', running_balance: '100000.0000' },
            { id: 'le_002', transaction_date: '2026-01-05', reference_number: 'TXN-005', description: 'Office Supplies', debit: '0.0000', credit: '500.0000', running_balance: '99500.0000' },
            { id: 'le_003', transaction_date: '2026-01-10', reference_number: 'TXN-012', description: 'Client Payment', debit: '50000.0000', credit: '0.0000', running_balance: '149500.0000' },
            { id: 'le_004', transaction_date: '2026-01-15', reference_number: 'TXN-015', description: 'Server Cost', debit: '0.0000', credit: '200.0000', running_balance: '149300.0000' },
        ],
        pagination: { page: 1, limit: 50, total: 4 }
    })
  }),

  // Transactions
  http.get('/api/v1/transactions', ({ request }) => {
    const url = new URL(request.url)
    const dimension = url.searchParams.get('dimension')
    
    let filtered = MOCK_TRANSACTIONS
    if (dimension && dimension !== 'all') {
        filtered = MOCK_TRANSACTIONS.filter(t => 
            t.entries.some(e => e.dimensions?.includes(dimension))
        )
    }

    return HttpResponse.json({
      data: filtered,
      pagination: { page: 1, limit: 50, total: filtered.length }
    })
  }),

  http.post('/api/v1/transactions', async ({ request }) => {
    const body = await request.json() as Record<string, unknown> & { entries: Record<string, unknown>[] }
    const newTxn = {
        ...body,
        id: `txn_${Date.now()}`,
        status: 'draft' as const,
        entries: body.entries.map((e) => ({
            ...e,
            account_name: 'Mock Account' 
        })),
        source_currency: body.source_currency || 'USD',
        exchange_rate: body.exchange_rate || '1.0'
    }
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    MOCK_TRANSACTIONS.unshift(newTxn as any)
    
    return HttpResponse.json(newTxn)
  }),

  // Bulk Approve
  http.post('/api/v1/organizations/:orgId/transactions/bulk-approve', async ({ request }) => {
       const body = await request.json() as { transaction_ids: string[] }
       const approvedIds = []
       const failedIds = []

       for (const id of body.transaction_ids) {
           const tx = MOCK_TRANSACTIONS.find(t => t.id === id)
           // eslint-disable-next-line @typescript-eslint/no-explicit-any
           if (tx) {
               (tx as any).status = 'approved';
               (tx as any).approved_at = new Date().toISOString()
               approvedIds.push({ id, status: 'approved' })
           } else {
               failedIds.push({ id, error: 'Not found' })
           }
       }

       return HttpResponse.json({
           approved: approvedIds,
           failed: failedIds
       })
  }),

  // Attachments
  http.post('/api/v1/transactions/:id/attachments', async () => {
      // Simulate upload delay
      await new Promise(resolve => setTimeout(resolve, 500))
      return HttpResponse.json({
          id: crypto.randomUUID(),
          file_name: 'invoice_scan.pdf',
          file_size: 1024 * 500, // 500KB
          content_type: 'application/pdf',
          uploaded_at: new Date().toISOString()
      })
  }),

  http.get('/api/v1/transactions/:id/attachments', () => {
      return HttpResponse.json([
          {
              id: 'att_1',
              file_name: 'receipt_001.jpg',
              file_size: 1024 * 250, 
              content_type: 'image/jpeg',
              uploaded_at: '2025-01-10T08:00:00Z'
          },
           {
              id: 'att_2',
              file_name: 'contract_signed.pdf',
              file_size: 1024 * 1024 * 2, 
              content_type: 'application/pdf',
              uploaded_at: '2025-01-09T14:30:00Z'
          }
      ])
  }),

  http.get('/api/v1/transactions/:id', ({ params }) => {
    const txn = MOCK_TRANSACTIONS.find(t => t.id === params.id)
    
    if (txn) {
        return HttpResponse.json(txn)
    }

    // Fallback if not found in list
    return HttpResponse.json({
        id: params.id,
        reference_number: 'TXN-UNKNOWN',
        transaction_type: 'journal',
        transaction_date: '2026-01-01',
        description: 'Transaction not found in mock list',
        status: 'draft',
        entries: []
    })
  }),

  // Transaction Actions
  http.post('/api/v1/transactions/:id/approve', ({ params }) => {
    return HttpResponse.json({
       id: params.id,
       status: 'posted'
    })
  }),

  http.post('/api/v1/transactions/:id/reject', ({ params }) => {
    return HttpResponse.json({
       id: params.id,
       status: 'voided'
    })
  }),

  // Dashboard
  http.get('/api/v1/dashboard/metrics', () => {
    return HttpResponse.json({
      cash_position: { balance: '150000.0000', currency: 'USD', change_percent: 5.2 },
      burn_rate: { daily: '2500.0000', monthly: '75000.0000' },
      runway_days: 60,
      pending_approvals: { count: 3, total_amount: '15000.0000' }
    })
  }),

  // Recent Activity
  http.get('/api/v1/dashboard/recent-activity', () => {
    return HttpResponse.json({
        activities: [
            {
                id: 'act_1',
                type: 'transaction_created',
                action: 'created',
                entity_type: 'transaction',
                entity_id: 'txn_001',
                description: 'Office supplies purchase',
                amount: '150.0000',
                currency: 'USD',
                user: { id: 'usr_001', full_name: 'Demo User' },
                timestamp: new Date(Date.now() - 1000 * 60 * 30).toISOString() // 30 mins ago
            },
            {
                id: 'act_2',
                type: 'budget_updated',
                action: 'updated',
                entity_type: 'budget',
                entity_id: 'bdg_001',
                description: 'Engineering Budget FY2026',
                user: { id: 'usr_002', full_name: 'Admin' },
                timestamp: new Date(Date.now() - 1000 * 60 * 60 * 2).toISOString() // 2 hours ago
            },
            {
                id: 'act_3',
                type: 'transaction_approved',
                action: 'approved',
                entity_type: 'transaction',
                entity_id: 'txn_002',
                description: 'Project Alpha Payment',
                amount: '5000.0000',
                currency: 'USD',
                user: { id: 'usr_003', full_name: 'Manager' },
                timestamp: new Date(Date.now() - 1000 * 60 * 60 * 5).toISOString() // 5 hours ago
            }
        ],
        pagination: { limit: 10, has_more: false, next_cursor: null }
    })
  }),

  http.get('/api/v1/dashboard/cash-flow', () => {
    return HttpResponse.json([
      { month: 'Jan', inflow: 45000, outflow: 32000 },
      { month: 'Feb', inflow: 52000, outflow: 35000 },
      { month: 'Mar', inflow: 48000, outflow: 40000 },
      { month: 'Apr', inflow: 61000, outflow: 38000 },
      { month: 'May', inflow: 55000, outflow: 42000 },
      { month: 'Jun', inflow: 67000, outflow: 45000 },
    ])
  }),

  // Budgets
  http.get('/api/v1/budgets', () => {
    return HttpResponse.json({
      data: [
          { id: 'bdg_001', department: 'Engineering', budget_limit: '50000.0000', actual_spent: '35000.0000', period: '2026-01' },
          { id: 'bdg_002', department: 'Marketing', budget_limit: '25000.0000', actual_spent: '28000.0000', period: '2026-01' },
          { id: 'bdg_003', department: 'Operations', budget_limit: '15000.0000', actual_spent: '12000.0000', period: '2026-01' },
          { id: 'bdg_004', department: 'HR', budget_limit: '10000.0000', actual_spent: '5000.0000', period: '2026-01' },
      ]
    })
  }),

  http.post('/api/v1/budgets', async ({ request }) => {
    const body = await request.json() as Record<string, unknown>
    return HttpResponse.json({
        id: `bdg_${Date.now()}`,
        actual_spent: '0.0000',
        ...body
    })
  }),

// Keep track of statuses in memory for testing
  http.get('/api/v1/budgets/:id', ({ params }) => {
     const id = params.id as string
     // Default to 'open' if not set in our memory store
     const currentStatus = budgetStatuses[id] || 'open'
     
     // Different data per budget
     const budgetData: Record<string, { department: string, budget_limit: string, actual_spent: string, lines: Array<{ id: string, account_name: string, limit: string, actual: string, dimension_value_id: string | null }> }> = {
       'bdg_001': {
         department: 'Engineering',
         budget_limit: '50000.0000',
         actual_spent: '35000.0000',
         lines: [
           { id: 'bl_1', account_name: 'Server Cost', limit: '30000.0000', actual: '25000.0000', dimension_value_id: null },
           { id: 'bl_2', account_name: 'Software Licenses', limit: '20000.0000', actual: '10000.0000', dimension_value_id: 'val_p1' },
         ]
       },
       'bdg_002': {
         department: 'Marketing',
         budget_limit: '25000.0000',
         actual_spent: '28000.0000',
         lines: [
           { id: 'bl_3', account_name: 'Advertising', limit: '15000.0000', actual: '18000.0000', dimension_value_id: null },
           { id: 'bl_4', account_name: 'Events', limit: '10000.0000', actual: '10000.0000', dimension_value_id: 'val_p2' },
         ]
       },
       'bdg_003': {
         department: 'Operations',
         budget_limit: '15000.0000',
         actual_spent: '12000.0000',
         lines: [
           { id: 'bl_5', account_name: 'Office Supplies', limit: '5000.0000', actual: '3000.0000', dimension_value_id: null },
           { id: 'bl_6', account_name: 'Utilities', limit: '10000.0000', actual: '9000.0000', dimension_value_id: null },
         ]
       },
       'bdg_004': {
         department: 'HR',
         budget_limit: '10000.0000',
         actual_spent: '5000.0000',
         lines: [
           { id: 'bl_7', account_name: 'Training', limit: '5000.0000', actual: '2500.0000', dimension_value_id: null },
           { id: 'bl_8', account_name: 'Recruitment', limit: '5000.0000', actual: '2500.0000', dimension_value_id: null },
         ]
       }
     }
     
     const budget = budgetData[id] || budgetData['bdg_001']
     
     return HttpResponse.json({
         id: id,
         department: budget.department,
         period: '2026-01',
         budget_limit: budget.budget_limit,
         actual_spent: budget.actual_spent,
         status: currentStatus,
         lines: budget.lines
     })
  }),

  http.post('/api/v1/budgets/:id/lines', async ({ request, params }) => {
      const body = await request.json() as Record<string, unknown>
      return HttpResponse.json({
          id: `bl_${Date.now()}`,
          budget_id: params.id,
          actual: '0.0000',
          dimension_value_id: (body.dimension_value_id as string) || null, 
          ...(body as Record<string, unknown>)
      })
  }),

  http.patch('/api/v1/budgets/:id/status', async ({ request, params }) => {
      const body = await request.json() as Record<string, unknown>
      const id = params.id as string
      
      // Update our memory store
      if (body.status === 'open' || body.status === 'locked') {
        budgetStatuses[id] = body.status as 'open' | 'locked'
      }

      return HttpResponse.json({
          id: id,
          status: body.status
      })
  }),

  // Reports
  http.get('/api/v1/reports/trial-balance', () => {
    return HttpResponse.json({
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
    })
  }),

  http.get('/api/v1/reports/income-statement', () => {
      return HttpResponse.json({
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
      })
  }),

  http.get('/api/v1/reports/balance-sheet', () => {
      return HttpResponse.json({
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
      })
  }),

   http.get('/api/v1/reports/dimensional', ({ request }) => {
      const url = new URL(request.url)
      const dimension = url.searchParams.get('dimension') || 'DEPT'
      
      let data: Record<string, unknown>[] = []
      let summary = { global_revenue: '0', global_expense: '0', global_net: '0' }

      if (dimension === 'PROJ') {
         data = [
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
         ]
         summary = { global_revenue: '80000.0000', global_expense: '55000.0000', global_net: '25000.0000' }
      } else {
         // Default DEPT
         data = [
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
         ]
         summary = { global_revenue: '120000.0000', global_expense: '70000.0000', global_net: '50000.0000' }
      }

      return HttpResponse.json({
          dimension: dimension,
          data: data,
          summary: summary
      })
  }),

  // Fiscal
  http.get('/api/v1/fiscal-years', () => {
      // ... existing GET handler logic ...
      return HttpResponse.json({
        data: [
          // ... existing mocks ...
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
      })
  }),

  http.post('/api/v1/fiscal-years', async ({ request }) => {
    const body = await request.json() as { start_date: string, include_adjustment: boolean, name: string, end_date: string }
    const startDate = new Date(body.start_date)
    const year = startDate.getFullYear()
    const numPeriods = body.include_adjustment ? 13 : 12
    
    // Generate monthly periods (+ optional adjustment period)
    const periods = Array.from({ length: numPeriods }, (_, i) => {
        const formatDate = (d: Date) => d.toISOString().split('T')[0]
        
        if (i < 12) {
            const monthStart = new Date(year, i, 1)
            const monthEnd = new Date(year, i + 1, 0)
            const monthName = monthStart.toLocaleString('default', { month: 'long' })
            return {
                id: `fp_${year}_${(i + 1).toString().padStart(2, '0')}`,
                name: `${monthName} ${year}`,
                status: 'open',
                start_date: formatDate(monthStart),
                end_date: formatDate(monthEnd)
            }
        } else {
            // Period 13: Adjustment period (same as Dec end date)
            return {
                id: `fp_${year}_13`,
                name: `Adjustment ${year}`,
                status: 'open',
                start_date: `${year}-12-31`,
                end_date: `${year}-12-31`
            }
        }
    })

    return HttpResponse.json({
        id: `fy_${year}`,
        name: body.name,
        status: 'open',
        start_date: body.start_date,
        end_date: body.end_date,
        periods: periods
    })
  }),

  http.patch('/api/v1/fiscal-periods/:id/status', async ({ request }) => {
     const body = await request.json() as { status: string }
     return HttpResponse.json({ success: true, status: body.status })
  }),

  // Dimensions
  http.get('/api/v1/dimensions', () => {
     return HttpResponse.json([
          {
              id: 'dim_dept',
              code: 'DEPT',
              name: 'Department',
              values: [
                  { id: 'val_eng', code: 'ENG', name: 'Engineering', description: 'Product and Tech team' },
                  { id: 'val_mkt', code: 'MKT', name: 'Marketing', description: 'Growth and Brand' },
                  { id: 'val_ops', code: 'OPS', name: 'Operations', description: 'General operations' },
              ]
          },
          {
              id: 'dim_proj',
              code: 'PROJ',
              name: 'Project',
              values: [
                  { id: 'val_p1', code: 'P001', name: 'Website Redesign' },
                  { id: 'val_p2', code: 'P002', name: 'Q1 Campaign' },
              ]
          }
     ]) 
  }),

  http.post('/api/v1/dimensions/:typeId/values', async ({ request }) => {
     const body = await request.json() as Record<string, unknown>
     return HttpResponse.json({ success: true, ...body })
  }),

  // Exchange Rates
  http.get('/api/v1/exchange-rates', () => {
      return HttpResponse.json({
        data: [
          { id: 'er_1', from_currency: 'USD', to_currency: 'IDR', rate: '15500.00', date: '2026-01-01' },
          { id: 'er_2', from_currency: 'SGD', to_currency: 'IDR', rate: '11500.00', date: '2026-01-01' },
          { id: 'er_3', from_currency: 'USD', to_currency: 'IDR', rate: '15550.00', date: '2026-01-02' },
        ]
      })
  }),

  http.post('/api/v1/exchange-rates', async ({ request }) => {
      const body = await request.json() as Record<string, unknown>
      return HttpResponse.json({ 
          id: `er_${Date.now()}`,
          ...body
      })
  }),

  // Simulation
  http.post('/api/v1/simulation/run', async ({ request }) => {
    const body = await request.json() as { 
        base_period_start: string, 
        revenue_growth_rate: string, 
        expense_growth_rate: string,
        projection_months: number 
    }
    
    const revenueGrowth = parseFloat(body.revenue_growth_rate || '0')
    const expenseGrowth = parseFloat(body.expense_growth_rate || '0')
    const months = body.projection_months || 12
    const startYear = new Date(body.base_period_start).getFullYear() + 1 // Project for next year relative to base

    // Mock baseline values (monthly average)
    const BASE_REVENUE = 100000
    const BASE_EXPENSE = 60000

    const projections = Array.from({ length: months }, (_, i) => {
        const monthIndex = i % 12
        const yearOffset = Math.floor(i / 12)
        const currentYear = startYear + yearOffset
        const monthNum = monthIndex + 1
        const monthName = new Date(currentYear, monthIndex).toLocaleString('default', { month: 'short' })
        
        // Compound growth per month for visual ramp-up (simplified)
        // Or just flat growth applied to baseline:
        const projectedRevenue = BASE_REVENUE * (1 + revenueGrowth)
        const projectedExpense = BASE_EXPENSE * (1 + expenseGrowth)
        const netIncome = projectedRevenue - projectedExpense

        return {
            period_name: `${currentYear}-${monthNum.toString().padStart(2, '0')}`,
            period_start: `${currentYear}-${monthNum.toString().padStart(2, '0')}-01`,
            period_end: `${currentYear}-${monthNum.toString().padStart(2, '0')}-28`, // simplified
            account_id: 'acc_sim_01',
            account_code: '0000',
            account_name: 'Aggregated',
            account_type: 'revenue',
            baseline_amount: BASE_REVENUE.toFixed(4),
            projected_amount: projectedRevenue.toFixed(4),
            change_percent: (revenueGrowth * 100).toFixed(2),
            revenue: projectedRevenue.toFixed(4), // Included for chart convenience
            expenses: projectedExpense.toFixed(4),
            net_income: netIncome.toFixed(4),
            month: monthName
        }
    })

    const totalRevenue = projections.reduce((sum, p) => sum + parseFloat(p.revenue), 0)
    const totalExpense = projections.reduce((sum, p) => sum + parseFloat(p.expenses), 0)
    const totalNet = totalRevenue - totalExpense

    return HttpResponse.json({
        simulation_id: `sim_${Date.now()}`,
        parameters_hash: 'hash_xyz',
        cached: false,
        projections: projections, // Keeping detailed array
        annual_summary: {
            total_projected_revenue: totalRevenue.toFixed(4),
            total_projected_expenses: totalExpense.toFixed(4),
            projected_net_income: totalNet.toFixed(4),
            net_profit_margin: ((totalNet / totalRevenue) * 100).toFixed(2)
        },
        monthly_summary: projections.map(p => ({
            month: p.month,
            period_name: p.period_name,
            revenue: p.revenue,
            expenses: p.expenses,
            net_income: p.net_income
        }))
    })
  }),

  // Organization Settings & Team
  http.get('/api/v1/organizations/:id', ({ params }) => {
    return HttpResponse.json({
        id: params.id,
        name: 'Acme Corp',
        slug: 'acme',
        base_currency: 'USD',
        timezone: 'Asia/Jakarta',
        created_at: '2026-01-01T00:00:00Z',
        subscription_tier: 'enterprise'
    })
  }),

  http.patch('/api/v1/organizations/:id', async ({ request, params }) => {
    const body = await request.json() as Record<string, unknown>
    return HttpResponse.json({
        id: params.id,
        name: 'Acme Corp',
        ...body
    })
  }),

  http.get('/api/v1/organizations/:id/users', () => {
    return HttpResponse.json({
        data: [
            { id: 'usr_001', full_name: 'Demo User', email: 'demo@zeltra.io', role: 'owner', status: 'active', joined_at: '2026-01-01' },
            { id: 'usr_002', full_name: 'Alice Finance', email: 'alice@zeltra.io', role: 'accountant', status: 'active', joined_at: '2026-01-02' },
            { id: 'usr_003', full_name: 'Bob Auditor', email: 'bob@zeltra.io', role: 'viewer', status: 'invited', joined_at: null },
        ]
    })
  }),

  http.post('/api/v1/organizations/:id/users', async ({ request }) => {
      const body = await request.json() as Record<string, unknown>
      return HttpResponse.json({
          id: `usr_${Date.now()}`,
          status: 'invited',
          ...body
      })
  }),

  http.patch('/api/v1/organizations/:id/users/:userId', async ({ request, params }) => {
      const body = await request.json() as Record<string, unknown>
      return HttpResponse.json({
          id: params.userId,
          status: 'active',
          ...body
      })
  }),

  http.delete('/api/v1/organizations/:id/users/:userId', () => {
      return HttpResponse.json({ success: true })
  })
]
