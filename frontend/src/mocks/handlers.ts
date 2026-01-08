import { http, HttpResponse } from 'msw'

export const handlers = [
  // Auth
  http.post('/api/v1/auth/login', () => {
    return HttpResponse.json({
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
    })
  }),

  // Accounts
  http.get('/api/v1/accounts', () => {
    return HttpResponse.json({
      data: [
        { id: 'acc_001', code: '1100', name: 'Cash', account_type: 'asset', balance: '150000.0000' },
        { id: 'acc_002', code: '1200', name: 'Bank BCA', account_type: 'asset', balance: '500000.0000' },
        { id: 'acc_003', code: '5100', name: 'Marketing Expense', account_type: 'expense', balance: '25000.0000' },
        { id: 'acc_004', code: '5200', name: 'Office Supplies', account_type: 'expense', balance: '8000.0000' },
      ]
    })
  }),

  // Transactions
  http.get('/api/v1/transactions', () => {
    return HttpResponse.json({
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
        }
      ],
      pagination: { page: 1, limit: 50, total: 3 }
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
]
