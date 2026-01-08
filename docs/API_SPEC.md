# API Specification

Enterprise-grade REST API with multi-currency support, dimensional filtering, and strict fiscal period management.

Base URL: `/api/v1`

## Authentication

All endpoints require authentication unless marked as `(Public)`.

### Headers

```
Authorization: Bearer <jwt_token>
X-Organization-ID: <organization_uuid>
Content-Type: application/json
```

### Error Response Format

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human readable message",
    "details": {},
    "request_id": "uuid"
  }
}
```

### Common Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `UNAUTHORIZED` | 401 | Invalid or expired token |
| `FORBIDDEN` | 403 | Insufficient permissions |
| `NOT_FOUND` | 404 | Resource not found |
| `VALIDATION_ERROR` | 400 | Invalid request body |
| `UNBALANCED_TRANSACTION` | 400 | Debit != Credit |
| `PERIOD_CLOSED` | 400 | Fiscal period is closed |
| `PERIOD_SOFT_CLOSED` | 400 | Only accountants can post |
| `NO_EXCHANGE_RATE` | 400 | Missing exchange rate |
| `INVALID_DIMENSION` | 400 | Dimension value not found |
| `CONCURRENT_MODIFICATION` | 409 | Optimistic lock failure |

---

## Auth

### POST /auth/register (Public)

```json
// Request
{
  "email": "user@example.com",
  "password": "SecureP@ss123",
  "full_name": "John Doe"
}

// Response 201
{
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@example.com",
    "full_name": "John Doe",
    "created_at": "2026-01-07T10:00:00Z"
  },
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
  "expires_in": 3600
}
```

### POST /auth/login (Public)

```json
// Request
{
  "email": "user@example.com",
  "password": "SecureP@ss123"
}

// Response 200
{
  "user": {
    "id": "uuid",
    "email": "user@example.com",
    "full_name": "John Doe",
    "organizations": [
      {
        "id": "org-uuid",
        "name": "Acme Corp",
        "slug": "acme-corp",
        "role": "owner"
      }
    ]
  },
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
  "expires_in": 3600
}
```

### POST /auth/refresh

```json
// Request
{
  "refresh_token": "eyJhbGciOiJIUzI1NiIs..."
}

// Response 200
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
  "expires_in": 3600
}
```

### POST /auth/logout

Response: `204 No Content`

### POST /auth/verify-email (Public)

Verify user's email address using token from verification email.

```json
// Request
{
  "token": "abc123xyz..."
}

// Response 200
{
  "message": "Email verified successfully",
  "verified": true
}

// Response 400 - Invalid/Expired Token
{
  "error": {
    "code": "INVALID_TOKEN",
    "message": "Invalid or expired verification token"
  }
}
```

### POST /auth/resend-verification (Public)

Resend verification email. Returns success even if email doesn't exist (security).

```json
// Request
{
  "email": "user@example.com"
}

// Response 200
{
  "message": "If an account exists with this email, a verification link has been sent."
}

// Response 400 - Already Verified
{
  "error": {
    "code": "ALREADY_VERIFIED",
    "message": "Email is already verified"
  }
}
```

---

## Organizations

### GET /organizations

```json
// Response 200
{
  "data": [
    {
      "id": "uuid",
      "name": "Acme Corp",
      "slug": "acme-corp",
      "base_currency": "USD",
      "timezone": "Asia/Jakarta",
      "role": "owner",
      "created_at": "2026-01-07T10:00:00Z"
    }
  ]
}
```

### POST /organizations

```json
// Request
{
  "name": "Acme Corp",
  "slug": "acme-corp",
  "base_currency": "USD",
  "timezone": "Asia/Jakarta"
}

// Response 201
{
  "id": "uuid",
  "name": "Acme Corp",
  "slug": "acme-corp",
  "base_currency": "USD",
  "timezone": "Asia/Jakarta",
  "created_at": "2026-01-07T10:00:00Z"
}
```

### GET /organizations/:id/users

```json
// Response 200
{
  "data": [
    {
      "user_id": "uuid",
      "email": "user@example.com",
      "full_name": "John Doe",
      "role": "owner",
      "approval_limit": null,
      "created_at": "2026-01-07T10:00:00Z"
    }
  ]
}
```

### POST /organizations/:id/users

```json
// Request
{
  "email": "newuser@example.com",
  "role": "accountant",
  "approval_limit": 10000000
}

// Response 201
{
  "user_id": "uuid",
  "role": "accountant",
  "approval_limit": "10000000.0000"
}
```

---

## Fiscal Periods

### GET /fiscal-years

```json
// Response 200
{
  "data": [
    {
      "id": "uuid",
      "name": "FY 2026",
      "start_date": "2026-01-01",
      "end_date": "2026-12-31",
      "status": "OPEN",
      "periods": [
        {
          "id": "uuid",
          "name": "January 2026",
          "period_number": 1,
          "start_date": "2026-01-01",
          "end_date": "2026-01-31",
          "status": "OPEN",
          "is_adjustment_period": false
        }
      ]
    }
  ]
}
```

### POST /fiscal-years

```json
// Request
{
  "name": "FY 2026",
  "start_date": "2026-01-01",
  "end_date": "2026-12-31",
  "period_frequency": "monthly"
}

// Response 201 - Creates fiscal year with 12 monthly periods
{
  "id": "uuid",
  "name": "FY 2026",
  "periods_created": 12
}
```

### PATCH /fiscal-periods/:id/status

```json
// Request
{
  "status": "SOFT_CLOSE"
}

// Response 200
{
  "id": "uuid",
  "name": "January 2026",
  "status": "SOFT_CLOSE",
  "closed_by": "user-uuid",
  "closed_at": "2026-02-05T10:00:00Z"
}
```

---

## Currencies & Exchange Rates

### GET /currencies

```json
// Response 200
{
  "data": [
    {
      "code": "USD",
      "name": "US Dollar",
      "symbol": "$",
      "decimal_places": 2
    },
    {
      "code": "IDR",
      "name": "Indonesian Rupiah",
      "symbol": "Rp",
      "decimal_places": 0
    }
  ]
}
```

### GET /exchange-rates

Query: `?from=USD&to=IDR&date=2026-01-07`

```json
// Response 200
{
  "from_currency": "USD",
  "to_currency": "IDR",
  "rate": "15850.0000000000",
  "effective_date": "2026-01-07",
  "source": "manual"
}
```

### POST /exchange-rates

```json
// Request
{
  "from_currency": "USD",
  "to_currency": "IDR",
  "rate": 15850,
  "effective_date": "2026-01-07",
  "source": "manual"
}

// Response 201
{
  "id": "uuid",
  "from_currency": "USD",
  "to_currency": "IDR",
  "rate": "15850.0000000000",
  "effective_date": "2026-01-07"
}
```

### POST /exchange-rates/bulk

```json
// Request - Import multiple rates
{
  "rates": [
    { "from": "USD", "to": "IDR", "rate": 15850, "date": "2026-01-07" },
    { "from": "EUR", "to": "IDR", "rate": 17200, "date": "2026-01-07" },
    { "from": "SGD", "to": "IDR", "rate": 11800, "date": "2026-01-07" }
  ]
}

// Response 201
{
  "imported": 3,
  "failed": 0
}
```

---

## Dimensions

### GET /dimension-types

```json
// Response 200
{
  "data": [
    {
      "id": "uuid",
      "code": "DEPARTMENT",
      "name": "Department",
      "is_required": true,
      "is_active": true,
      "sort_order": 1
    },
    {
      "id": "uuid",
      "code": "PROJECT",
      "name": "Project",
      "is_required": false,
      "is_active": true,
      "sort_order": 2
    }
  ]
}
```

### POST /dimension-types

```json
// Request
{
  "code": "COST_CENTER",
  "name": "Cost Center",
  "is_required": false
}

// Response 201
{
  "id": "uuid",
  "code": "COST_CENTER",
  "name": "Cost Center"
}
```

### GET /dimension-values

Query: `?type=DEPARTMENT&active=true`

```json
// Response 200
{
  "data": [
    {
      "id": "uuid",
      "dimension_type": "DEPARTMENT",
      "code": "ENG",
      "name": "Engineering",
      "parent_id": null,
      "is_active": true
    },
    {
      "id": "uuid",
      "dimension_type": "DEPARTMENT",
      "code": "ENG-BE",
      "name": "Backend Engineering",
      "parent_id": "parent-uuid",
      "is_active": true
    }
  ]
}
```

### POST /dimension-values

```json
// Request
{
  "dimension_type_id": "uuid",
  "code": "MKT",
  "name": "Marketing",
  "parent_id": null
}

// Response 201
{
  "id": "uuid",
  "code": "MKT",
  "name": "Marketing"
}
```

---

## Chart of Accounts

### GET /accounts

Query: `?type=expense&active=true&currency=USD`

```json
// Response 200
{
  "data": [
    {
      "id": "uuid",
      "code": "5100",
      "name": "Marketing Expense",
      "account_type": "expense",
      "account_subtype": "operating_expense",
      "currency": "USD",
      "parent_id": null,
      "is_active": true,
      "allow_direct_posting": true,
      "balance": "25000.0000"
    }
  ]
}
```

### POST /accounts

```json
// Request
{
  "code": "5100",
  "name": "Marketing Expense",
  "account_type": "expense",
  "account_subtype": "operating_expense",
  "currency": "USD",
  "parent_id": null,
  "description": "All marketing related expenses"
}

// Response 201
{
  "id": "uuid",
  "code": "5100",
  "name": "Marketing Expense",
  "account_type": "expense"
}
```

### GET /accounts/:id/balance

Query: `?as_of=2026-01-31`

```json
// Response 200
{
  "account_id": "uuid",
  "account_code": "5100",
  "balance": "25000.0000",
  "currency": "USD",
  "as_of": "2026-01-31T23:59:59Z"
}
```

### GET /accounts/:id/ledger

Query: `?from=2026-01-01&to=2026-01-31&page=1&limit=50`

```json
// Response 200
{
  "data": [
    {
      "id": "uuid",
      "transaction_id": "uuid",
      "transaction_date": "2026-01-15",
      "description": "Office supplies",
      "source_currency": "USD",
      "source_amount": "150.0000",
      "exchange_rate": "1.0000000000",
      "functional_amount": "150.0000",
      "debit": "150.0000",
      "credit": "0.0000",
      "running_balance": "25150.0000",
      "dimensions": [
        { "type": "DEPARTMENT", "code": "ENG", "name": "Engineering" }
      ]
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 50,
    "total": 125
  }
}
```


---

## Transactions

### GET /transactions

Query: `?status=posted&from=2026-01-01&to=2026-01-31&type=expense&dimension=uuid&page=1&limit=50`

```json
// Response 200
{
  "data": [
    {
      "id": "uuid",
      "reference_number": "TXN-2026-0001",
      "transaction_type": "expense",
      "transaction_date": "2026-01-15",
      "description": "Office supplies purchase",
      "status": "posted",
      "fiscal_period": {
        "id": "uuid",
        "name": "January 2026"
      },
      "entries": [
        {
          "id": "uuid",
          "account_id": "uuid",
          "account_code": "5200",
          "account_name": "Office Supplies",
          "source_currency": "USD",
          "source_amount": "150.0000",
          "exchange_rate": "1.0000000000",
          "functional_amount": "150.0000",
          "debit": "150.0000",
          "credit": "0.0000",
          "memo": "Printer paper and ink",
          "dimensions": [
            { "type": "DEPARTMENT", "code": "ENG", "name": "Engineering" },
            { "type": "PROJECT", "code": "P001", "name": "Project Alpha" }
          ]
        },
        {
          "id": "uuid",
          "account_id": "uuid",
          "account_code": "1100",
          "account_name": "Cash",
          "source_currency": "USD",
          "source_amount": "150.0000",
          "exchange_rate": "1.0000000000",
          "functional_amount": "150.0000",
          "debit": "0.0000",
          "credit": "150.0000",
          "memo": null,
          "dimensions": []
        }
      ],
      "attachments": [
        {
          "id": "uuid",
          "file_name": "receipt.pdf",
          "attachment_type": "receipt"
        }
      ],
      "created_by": {
        "id": "uuid",
        "full_name": "John Doe"
      },
      "approved_by": {
        "id": "uuid",
        "full_name": "Jane Smith"
      },
      "posted_at": "2026-01-15T14:05:00Z",
      "created_at": "2026-01-15T10:30:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 50,
    "total": 150,
    "total_pages": 3
  }
}
```

### POST /transactions

```json
// Request - Multi-currency transaction with dimensions
{
  "transaction_type": "expense",
  "transaction_date": "2026-01-15",
  "description": "International software subscription",
  "reference_number": "TXN-2026-0002",
  "memo": "Annual license renewal",
  "entries": [
    {
      "account_id": "expense-account-uuid",
      "source_currency": "EUR",
      "source_amount": "1000.00",
      "entry_type": "debit",
      "memo": "Software license",
      "dimensions": ["department-uuid", "project-uuid"]
    },
    {
      "account_id": "bank-account-uuid",
      "source_currency": "EUR",
      "source_amount": "1000.00",
      "entry_type": "credit",
      "dimensions": []
    }
  ]
}

// Response 201
{
  "id": "uuid",
  "reference_number": "TXN-2026-0002",
  "status": "draft",
  "entries": [
    {
      "id": "uuid",
      "account_code": "5300",
      "source_currency": "EUR",
      "source_amount": "1000.0000",
      "exchange_rate": "1.0850000000",
      "functional_currency": "USD",
      "functional_amount": "1085.0000",
      "debit": "1085.0000",
      "credit": "0.0000"
    },
    {
      "id": "uuid",
      "account_code": "1200",
      "source_currency": "EUR",
      "source_amount": "1000.0000",
      "exchange_rate": "1.0850000000",
      "functional_currency": "USD",
      "functional_amount": "1085.0000",
      "debit": "0.0000",
      "credit": "1085.0000"
    }
  ],
  "totals": {
    "functional_debit": "1085.0000",
    "functional_credit": "1085.0000",
    "is_balanced": true
  },
  "created_at": "2026-01-15T10:30:00Z"
}
```

### Error Response - Unbalanced

```json
// Response 400
{
  "error": {
    "code": "UNBALANCED_TRANSACTION",
    "message": "Transaction is not balanced",
    "details": {
      "functional_debit": "1085.0000",
      "functional_credit": "1000.0000",
      "difference": "85.0000"
    }
  }
}
```

### Error Response - Period Closed

```json
// Response 400
{
  "error": {
    "code": "PERIOD_CLOSED",
    "message": "Fiscal period is closed, no posting allowed",
    "details": {
      "period_id": "uuid",
      "period_name": "December 2025",
      "status": "CLOSED"
    }
  }
}
```

### POST /transactions/:id/submit

Submit draft for approval.

```json
// Response 200
{
  "id": "uuid",
  "status": "pending",
  "submitted_at": "2026-01-15T11:00:00Z",
  "required_approval_role": "approver"
}
```

### POST /transactions/:id/approve

```json
// Request
{
  "notes": "Approved - within budget"
}

// Response 200
{
  "id": "uuid",
  "status": "approved",
  "approved_by": "user-uuid",
  "approved_at": "2026-01-15T14:00:00Z",
  "approval_notes": "Approved - within budget"
}
```

### POST /transactions/:id/reject

```json
// Request
{
  "reason": "Missing receipt attachment"
}

// Response 200
{
  "id": "uuid",
  "status": "draft",
  "rejection_reason": "Missing receipt attachment"
}
```

### POST /transactions/:id/post

```json
// Response 200
{
  "id": "uuid",
  "status": "posted",
  "posted_by": "user-uuid",
  "posted_at": "2026-01-15T14:05:00Z",
  "account_balances": [
    {
      "account_id": "uuid",
      "account_code": "5300",
      "new_balance": "15085.0000"
    },
    {
      "account_id": "uuid",
      "account_code": "1200",
      "new_balance": "48915.0000"
    }
  ]
}
```

### POST /transactions/:id/void

```json
// Request
{
  "reason": "Duplicate entry - see TXN-2026-0003"
}

// Response 200
{
  "id": "uuid",
  "status": "voided",
  "voided_by": "user-uuid",
  "voided_at": "2026-01-16T09:00:00Z",
  "void_reason": "Duplicate entry - see TXN-2026-0003",
  "reversing_transaction_id": "new-uuid"
}
```

---

## Budgets

### GET /budgets

Query: `?fiscal_year_id=uuid&account_id=uuid`

```json
// Response 200
{
  "data": [
    {
      "id": "uuid",
      "name": "FY 2026 Operating Budget",
      "fiscal_year": "FY 2026",
      "budget_type": "annual",
      "currency": "USD",
      "is_locked": false,
      "total_budgeted": "1200000.0000",
      "total_actual": "450000.0000",
      "total_variance": "750000.0000"
    }
  ]
}
```

### POST /budgets

```json
// Request
{
  "name": "FY 2026 Operating Budget",
  "fiscal_year_id": "uuid",
  "budget_type": "annual",
  "currency": "USD"
}

// Response 201
{
  "id": "uuid",
  "name": "FY 2026 Operating Budget"
}
```

### POST /budgets/:id/lines

```json
// Request - Bulk create budget lines
{
  "lines": [
    {
      "account_id": "uuid",
      "fiscal_period_id": "uuid",
      "amount": 50000,
      "dimensions": ["department-uuid"]
    },
    {
      "account_id": "uuid",
      "fiscal_period_id": "uuid",
      "amount": 75000,
      "dimensions": ["department-uuid"]
    }
  ]
}

// Response 201
{
  "created": 2,
  "total_amount": "125000.0000"
}
```

### GET /budgets/:id/vs-actual

Query: `?period_id=uuid&dimension=uuid`

```json
// Response 200
{
  "budget_id": "uuid",
  "budget_name": "FY 2026 Operating Budget",
  "period": "January 2026",
  "lines": [
    {
      "account_id": "uuid",
      "account_code": "5100",
      "account_name": "Marketing",
      "budgeted": "50000.0000",
      "actual": "35000.0000",
      "variance": "15000.0000",
      "utilization_percent": 70.0,
      "status": "under_budget"
    },
    {
      "account_id": "uuid",
      "account_code": "5200",
      "account_name": "Office Supplies",
      "budgeted": "10000.0000",
      "actual": "12500.0000",
      "variance": "-2500.0000",
      "utilization_percent": 125.0,
      "status": "over_budget"
    }
  ],
  "summary": {
    "total_budgeted": "60000.0000",
    "total_actual": "47500.0000",
    "total_variance": "12500.0000",
    "overall_utilization": 79.17
  }
}
```

---

## Simulation

### POST /simulation/run

```json
// Request
{
  "base_period": {
    "start": "2025-01-01",
    "end": "2025-12-31"
  },
  "projection_months": 12,
  "parameters": {
    "revenue_growth_rate": 0.15,
    "expense_growth_rate": 0.05,
    "account_adjustments": {
      "account-uuid-1": -0.10,
      "account-uuid-2": 0.20
    },
    "dimension_adjustments": {
      "department-uuid": 0.08
    }
  },
  "dimension_filters": ["department-uuid-1", "department-uuid-2"]
}

// Response 200
{
  "simulation_id": "uuid",
  "parameters_hash": "sha256...",
  "projections": [
    {
      "period": "2026-01",
      "period_start": "2026-01-01",
      "period_end": "2026-01-31",
      "summary": {
        "projected_revenue": "120000.0000",
        "projected_expenses": "85000.0000",
        "projected_net_income": "35000.0000"
      },
      "accounts": [
        {
          "account_id": "uuid",
          "account_code": "4100",
          "account_name": "Sales Revenue",
          "account_type": "revenue",
          "baseline": "100000.0000",
          "projected": "115000.0000",
          "change_percent": 15.0
        },
        {
          "account_id": "uuid",
          "account_code": "5100",
          "account_name": "Marketing",
          "account_type": "expense",
          "baseline": "10000.0000",
          "projected": "9000.0000",
          "change_percent": -10.0
        }
      ]
    }
  ],
  "annual_summary": {
    "total_projected_revenue": "1440000.0000",
    "total_projected_expenses": "1020000.0000",
    "total_projected_net_income": "420000.0000",
    "revenue_growth": "15.00%",
    "expense_growth": "5.00%"
  },
  "cached": false,
  "computed_at": "2026-01-07T10:30:00Z"
}
```

---

## Reports

### GET /reports/trial-balance

Query: `?as_of=2026-01-31&dimension=uuid`

```json
// Response 200
{
  "report_type": "trial_balance",
  "as_of": "2026-01-31",
  "currency": "USD",
  "accounts": [
    {
      "account_id": "uuid",
      "code": "1100",
      "name": "Cash",
      "account_type": "asset",
      "debit": "150000.0000",
      "credit": "0.0000",
      "balance": "150000.0000"
    },
    {
      "account_id": "uuid",
      "code": "2100",
      "name": "Accounts Payable",
      "account_type": "liability",
      "debit": "0.0000",
      "credit": "50000.0000",
      "balance": "50000.0000"
    }
  ],
  "totals": {
    "total_debit": "500000.0000",
    "total_credit": "500000.0000",
    "is_balanced": true
  },
  "generated_at": "2026-01-31T23:59:59Z"
}
```

### GET /reports/balance-sheet

Query: `?as_of=2026-01-31`

```json
// Response 200
{
  "report_type": "balance_sheet",
  "as_of": "2026-01-31",
  "currency": "USD",
  "assets": {
    "current_assets": {
      "total": "200000.0000",
      "accounts": [
        { "code": "1100", "name": "Cash", "balance": "150000.0000" },
        { "code": "1200", "name": "Accounts Receivable", "balance": "50000.0000" }
      ]
    },
    "fixed_assets": {
      "total": "100000.0000",
      "accounts": []
    },
    "total_assets": "300000.0000"
  },
  "liabilities": {
    "current_liabilities": {
      "total": "50000.0000",
      "accounts": []
    },
    "long_term_liabilities": {
      "total": "0.0000",
      "accounts": []
    },
    "total_liabilities": "50000.0000"
  },
  "equity": {
    "total": "250000.0000",
    "accounts": [
      { "code": "3100", "name": "Owner's Equity", "balance": "200000.0000" },
      { "code": "3200", "name": "Retained Earnings", "balance": "50000.0000" }
    ]
  },
  "liabilities_and_equity": "300000.0000",
  "is_balanced": true
}
```

### GET /reports/income-statement

Query: `?from=2026-01-01&to=2026-01-31&dimension=uuid`

```json
// Response 200
{
  "report_type": "income_statement",
  "period": {
    "from": "2026-01-01",
    "to": "2026-01-31"
  },
  "currency": "USD",
  "revenue": {
    "total": "100000.0000",
    "accounts": [
      { "code": "4100", "name": "Sales Revenue", "amount": "95000.0000" },
      { "code": "4200", "name": "Service Revenue", "amount": "5000.0000" }
    ]
  },
  "cost_of_goods_sold": {
    "total": "40000.0000",
    "accounts": []
  },
  "gross_profit": "60000.0000",
  "operating_expenses": {
    "total": "35000.0000",
    "accounts": [
      { "code": "5100", "name": "Marketing", "amount": "15000.0000" },
      { "code": "5200", "name": "Salaries", "amount": "20000.0000" }
    ]
  },
  "operating_income": "25000.0000",
  "other_income_expense": {
    "total": "0.0000",
    "accounts": []
  },
  "net_income": "25000.0000"
}
```

### GET /reports/dimensional

Query: `?from=2026-01-01&to=2026-01-31&group_by=DEPARTMENT,PROJECT&account_type=expense`

```json
// Response 200
{
  "report_type": "dimensional",
  "period": {
    "from": "2026-01-01",
    "to": "2026-01-31"
  },
  "group_by": ["DEPARTMENT", "PROJECT"],
  "data": [
    {
      "dimensions": {
        "DEPARTMENT": { "code": "ENG", "name": "Engineering" },
        "PROJECT": { "code": "P001", "name": "Project Alpha" }
      },
      "accounts": [
        {
          "code": "5100",
          "name": "Marketing",
          "debit": "5000.0000",
          "credit": "0.0000",
          "balance": "5000.0000"
        }
      ],
      "total": "5000.0000"
    },
    {
      "dimensions": {
        "DEPARTMENT": { "code": "MKT", "name": "Marketing" },
        "PROJECT": { "code": "P002", "name": "Project Beta" }
      },
      "accounts": [],
      "total": "12000.0000"
    }
  ],
  "grand_total": "17000.0000"
}
```

---

## Attachments

### POST /attachments/upload

```
Content-Type: multipart/form-data

file: <binary>
transaction_id: uuid (optional)
attachment_type: receipt | invoice | contract | supporting_document | other
```

```json
// Response 201
{
  "id": "uuid",
  "file_name": "receipt-2026-01-15.pdf",
  "file_size": 245678,
  "mime_type": "application/pdf",
  "storage_provider": "cloudflare_r2",
  "attachment_type": "receipt",
  "download_url": "https://...",
  "uploaded_at": "2026-01-15T10:30:00Z"
}
```

### GET /attachments/:id

```json
// Response 200
{
  "id": "uuid",
  "file_name": "receipt-2026-01-15.pdf",
  "file_size": 245678,
  "mime_type": "application/pdf",
  "attachment_type": "receipt",
  "transaction_id": "uuid",
  "download_url": "https://...",
  "extracted_data": {
    "vendor": "Office Depot",
    "amount": "150.00",
    "date": "2026-01-15"
  },
  "uploaded_by": {
    "id": "uuid",
    "full_name": "John Doe"
  },
  "created_at": "2026-01-15T10:30:00Z"
}
```

### DELETE /attachments/:id

Response: `204 No Content`

---

## Dashboard

### GET /dashboard/metrics

Query: `?period_id=uuid`

```json
// Response 200
{
  "period": {
    "id": "uuid",
    "name": "January 2026"
  },
  "cash_position": {
    "balance": "150000.0000",
    "currency": "USD",
    "change_from_last_period": "15000.0000",
    "change_percent": 11.11
  },
  "burn_rate": {
    "daily": "2500.0000",
    "monthly": "75000.0000"
  },
  "runway_days": 60,
  "pending_approvals": {
    "count": 5,
    "total_amount": "25000.0000"
  },
  "budget_status": {
    "total_budgeted": "100000.0000",
    "total_spent": "75000.0000",
    "utilization_percent": 75.0,
    "days_remaining": 16,
    "projected_end_of_period": "95000.0000"
  },
  "top_expenses_by_department": [
    { "department": "Engineering", "amount": "35000.0000", "percent": 46.67 },
    { "department": "Marketing", "amount": "25000.0000", "percent": 33.33 },
    { "department": "Operations", "amount": "15000.0000", "percent": 20.00 }
  ],
  "currency_exposure": [
    { "currency": "USD", "balance": "150000.0000", "percent": 85.0 },
    { "currency": "EUR", "balance": "15000.0000", "functional_value": "16275.0000", "percent": 9.2 },
    { "currency": "IDR", "balance": "150000000", "functional_value": "9464.0000", "percent": 5.8 }
  ],
  "cash_flow_chart": {
    "labels": ["Week 1", "Week 2", "Week 3", "Week 4"],
    "inflow": ["25000.0000", "30000.0000", "28000.0000", "35000.0000"],
    "outflow": ["20000.0000", "22000.0000", "25000.0000", "18000.0000"]
  },
  "utilization_chart": {
    "labels": ["Engineering", "Marketing", "Operations", "HR"],
    "budgeted": ["50000.0000", "30000.0000", "15000.0000", "5000.0000"],
    "actual": ["35000.0000", "25000.0000", "12000.0000", "3000.0000"]
  }
}
```

### GET /dashboard/recent-activity

Query: `?limit=10&type=all|transaction|budget|approval`

Returns recent activity log for the organization.

```json
// Response 200
{
  "activities": [
    {
      "id": "uuid",
      "type": "transaction_posted",
      "action": "posted",
      "entity_type": "transaction",
      "entity_id": "uuid",
      "description": "Office supplies purchase",
      "amount": "150.0000",
      "currency": "USD",
      "user": {
        "id": "uuid",
        "full_name": "John Doe"
      },
      "metadata": {
        "reference_number": "TXN-2026-0001",
        "account_code": "5200"
      },
      "timestamp": "2026-01-15T14:05:00Z"
    },
    {
      "id": "uuid",
      "type": "transaction_approved",
      "action": "approved",
      "entity_type": "transaction",
      "entity_id": "uuid",
      "description": "Software subscription",
      "amount": "1085.0000",
      "currency": "USD",
      "user": {
        "id": "uuid",
        "full_name": "Jane Smith"
      },
      "metadata": {
        "reference_number": "TXN-2026-0002",
        "approval_notes": "Within budget"
      },
      "timestamp": "2026-01-15T14:00:00Z"
    },
    {
      "id": "uuid",
      "type": "transaction_created",
      "action": "created",
      "entity_type": "transaction",
      "entity_id": "uuid",
      "description": "Marketing campaign expense",
      "amount": "5000.0000",
      "currency": "USD",
      "user": {
        "id": "uuid",
        "full_name": "Bob Wilson"
      },
      "metadata": {
        "reference_number": "TXN-2026-0003",
        "status": "draft"
      },
      "timestamp": "2026-01-15T13:30:00Z"
    },
    {
      "id": "uuid",
      "type": "budget_updated",
      "action": "updated",
      "entity_type": "budget",
      "entity_id": "uuid",
      "description": "Q1 Marketing Budget adjusted",
      "amount": "75000.0000",
      "currency": "USD",
      "user": {
        "id": "uuid",
        "full_name": "Jane Smith"
      },
      "metadata": {
        "budget_name": "FY 2026 Operating Budget",
        "previous_amount": "50000.0000"
      },
      "timestamp": "2026-01-15T11:00:00Z"
    },
    {
      "id": "uuid",
      "type": "transaction_voided",
      "action": "voided",
      "entity_type": "transaction",
      "entity_id": "uuid",
      "description": "Duplicate entry voided",
      "amount": "250.0000",
      "currency": "USD",
      "user": {
        "id": "uuid",
        "full_name": "John Doe"
      },
      "metadata": {
        "reference_number": "TXN-2026-0000",
        "void_reason": "Duplicate entry"
      },
      "timestamp": "2026-01-15T10:00:00Z"
    }
  ],
  "pagination": {
    "limit": 10,
    "has_more": true,
    "next_cursor": "2026-01-15T10:00:00Z"
  }
}
```

Activity Types:
- `transaction_created` - New transaction draft created
- `transaction_submitted` - Transaction submitted for approval
- `transaction_approved` - Transaction approved
- `transaction_rejected` - Transaction rejected
- `transaction_posted` - Transaction posted to ledger
- `transaction_voided` - Transaction voided
- `budget_created` - New budget created
- `budget_updated` - Budget line updated
- `budget_locked` - Budget locked
- `user_invited` - User invited to organization
- `user_role_changed` - User role updated
