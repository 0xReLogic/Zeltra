# Database Schema

Enterprise-grade multi-tenant accounting system with dimensional accounting, multi-currency support, and strict fiscal period management.

## Design Principles

1. Multi-tenant with Row-Level Security - Organization-level data isolation
2. Double-entry enforced at database level - Triggers validate balance at COMMIT
3. Immutable ledger - No UPDATE/DELETE on posted entries, corrections via reversing entries
4. Multi-currency with functional currency conversion - Store source, rate, and functional amounts
5. Dimensional accounting - Flexible, validated dimensions for enterprise reporting
6. Explicit fiscal period management - Granular control over period closing
7. Historical balance tracking - Each entry stores running balance for point-in-time queries
8. Decimal precision - NUMERIC(19,4) for all monetary values (supports up to 999 trillion with 4 decimal places)

## Multi-Tenancy Strategy

Strategy: Shared Database, Shared Schema with `organization_id` foreign key + Row-Level Security

For Enterprise On-Premise: Can migrate to schema-per-tenant if client requires physical isolation.

## Core Tables

### users

```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    full_name VARCHAR(255) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    email_verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_users_email ON users(email) WHERE is_active = true;
```

### organizations

```sql
CREATE TABLE organizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(100) NOT NULL UNIQUE,
    
    -- Immutable base currency for consolidated reporting
    base_currency CHAR(3) NOT NULL,
    
    -- Timezone for fiscal period calculations
    timezone VARCHAR(50) NOT NULL DEFAULT 'UTC',
    
    -- Settings stored as JSONB for flexibility (non-critical config only)
    settings JSONB NOT NULL DEFAULT '{}',
    
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    CONSTRAINT chk_base_currency_format CHECK (base_currency ~ '^[A-Z]{3}$')
);

CREATE INDEX idx_organizations_slug ON organizations(slug);

COMMENT ON COLUMN organizations.base_currency IS 'Immutable functional currency for consolidated reporting. All transactions converted to this currency.';
```

### organization_users

```sql
CREATE TYPE user_role AS ENUM (
    'owner',
    'admin', 
    'accountant',
    'approver',
    'viewer',
    'submitter'
);

CREATE TABLE organization_users (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    role user_role NOT NULL DEFAULT 'viewer',
    
    -- Approval limit in base currency (NULL = unlimited for role)
    approval_limit NUMERIC(19, 4),
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    PRIMARY KEY (user_id, organization_id)
);

CREATE INDEX idx_org_users_org ON organization_users(organization_id);
```

## Currency Management

### currencies

```sql
CREATE TABLE currencies (
    code CHAR(3) PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    symbol VARCHAR(10) NOT NULL,
    decimal_places SMALLINT NOT NULL DEFAULT 2,
    is_active BOOLEAN NOT NULL DEFAULT true,
    
    CONSTRAINT chk_currency_code CHECK (code ~ '^[A-Z]{3}$'),
    CONSTRAINT chk_decimal_places CHECK (decimal_places BETWEEN 0 AND 4)
);

-- Seed common currencies
INSERT INTO currencies (code, name, symbol, decimal_places) VALUES
('USD', 'US Dollar', '$', 2),
('EUR', 'Euro', '€', 2),
('GBP', 'British Pound', '£', 2),
('JPY', 'Japanese Yen', '¥', 0),
('IDR', 'Indonesian Rupiah', 'Rp', 0),
('SGD', 'Singapore Dollar', 'S$', 2),
('AUD', 'Australian Dollar', 'A$', 2),
('CNY', 'Chinese Yuan', '¥', 2);
```

### exchange_rates

```sql
CREATE TYPE rate_source AS ENUM ('manual', 'api', 'bank_feed');

CREATE TABLE exchange_rates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    
    from_currency CHAR(3) NOT NULL REFERENCES currencies(code),
    to_currency CHAR(3) NOT NULL REFERENCES currencies(code),
    
    rate NUMERIC(19, 10) NOT NULL,
    
    -- Effective date range for this rate
    effective_date DATE NOT NULL,
    
    source rate_source NOT NULL DEFAULT 'manual',
    source_reference VARCHAR(255),
    
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    CONSTRAINT chk_rate_positive CHECK (rate > 0),
    CONSTRAINT chk_different_currencies CHECK (from_currency <> to_currency),
    UNIQUE (organization_id, from_currency, to_currency, effective_date)
);

CREATE INDEX idx_exchange_rates_lookup ON exchange_rates(organization_id, from_currency, to_currency, effective_date DESC);

COMMENT ON TABLE exchange_rates IS 'Historical exchange rates. Use effective_date to find rate applicable for a transaction date.';
```

## Fiscal Period Management

### fiscal_years

```sql
CREATE TYPE fiscal_year_status AS ENUM ('OPEN', 'CLOSED');

CREATE TABLE fiscal_years (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    
    name VARCHAR(50) NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    
    status fiscal_year_status NOT NULL DEFAULT 'OPEN',
    
    closed_by UUID REFERENCES users(id),
    closed_at TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    CONSTRAINT chk_fiscal_year_dates CHECK (end_date > start_date),
    UNIQUE (organization_id, name),
    UNIQUE (organization_id, start_date)
);

CREATE INDEX idx_fiscal_years_org ON fiscal_years(organization_id, start_date);
```

### fiscal_periods

```sql
CREATE TYPE fiscal_period_status AS ENUM (
    'OPEN',       -- Normal operations, all users can post
    'SOFT_CLOSE', -- Only accountant+ can post (month-end adjustments)
    'CLOSED'      -- No posting allowed, fully locked
);

CREATE TABLE fiscal_periods (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    fiscal_year_id UUID NOT NULL REFERENCES fiscal_years(id) ON DELETE CASCADE,
    
    name VARCHAR(50) NOT NULL,
    period_number SMALLINT NOT NULL,
    
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    
    status fiscal_period_status NOT NULL DEFAULT 'OPEN',
    
    -- Adjustment period flag (for year-end adjustments)
    is_adjustment_period BOOLEAN NOT NULL DEFAULT false,
    
    closed_by UUID REFERENCES users(id),
    closed_at TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    CONSTRAINT chk_period_dates CHECK (end_date >= start_date),
    CONSTRAINT chk_period_number CHECK (period_number > 0),
    UNIQUE (organization_id, fiscal_year_id, period_number),
    UNIQUE (organization_id, start_date)
);

CREATE INDEX idx_fiscal_periods_org_date ON fiscal_periods(organization_id, start_date, end_date);
CREATE INDEX idx_fiscal_periods_status ON fiscal_periods(organization_id, status) WHERE status <> 'CLOSED';

COMMENT ON COLUMN fiscal_periods.is_adjustment_period IS 'True for period 13/14 used for year-end audit adjustments';
```

## Dimensional Accounting

### dimension_types

```sql
CREATE TABLE dimension_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    
    code VARCHAR(50) NOT NULL,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    
    -- Validation rules
    is_required BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    
    -- Display order in UI
    sort_order SMALLINT NOT NULL DEFAULT 0,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    UNIQUE (organization_id, code)
);

CREATE INDEX idx_dimension_types_org ON dimension_types(organization_id) WHERE is_active = true;

-- Example dimension types: DEPARTMENT, PROJECT, COST_CENTER, LOCATION, CUSTOMER, VENDOR
```

### dimension_values

```sql
CREATE TABLE dimension_values (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    dimension_type_id UUID NOT NULL REFERENCES dimension_types(id) ON DELETE CASCADE,
    
    code VARCHAR(50) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    
    -- Hierarchical support (for nested dimensions like department hierarchy)
    parent_id UUID REFERENCES dimension_values(id),
    
    is_active BOOLEAN NOT NULL DEFAULT true,
    
    -- Effective date range (for time-bound dimensions)
    effective_from DATE,
    effective_to DATE,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    UNIQUE (organization_id, dimension_type_id, code),
    CONSTRAINT chk_effective_dates CHECK (effective_to IS NULL OR effective_to >= effective_from)
);

CREATE INDEX idx_dimension_values_type ON dimension_values(dimension_type_id) WHERE is_active = true;
CREATE INDEX idx_dimension_values_parent ON dimension_values(parent_id) WHERE parent_id IS NOT NULL;
```

## Chart of Accounts

### chart_of_accounts

```sql
CREATE TYPE account_type AS ENUM (
    'asset',
    'liability', 
    'equity',
    'revenue',
    'expense'
);

CREATE TYPE account_subtype AS ENUM (
    -- Assets
    'cash',
    'bank',
    'accounts_receivable',
    'inventory',
    'prepaid',
    'fixed_asset',
    'accumulated_depreciation',
    'other_asset',
    
    -- Liabilities
    'accounts_payable',
    'credit_card',
    'accrued_liability',
    'short_term_debt',
    'long_term_debt',
    'other_liability',
    
    -- Equity
    'owner_equity',
    'retained_earnings',
    'common_stock',
    'other_equity',
    
    -- Revenue
    'operating_revenue',
    'other_revenue',
    
    -- Expense
    'cost_of_goods_sold',
    'operating_expense',
    'payroll_expense',
    'depreciation_expense',
    'interest_expense',
    'tax_expense',
    'other_expense'
);

CREATE TABLE chart_of_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    
    code VARCHAR(20) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    
    account_type account_type NOT NULL,
    account_subtype account_subtype,
    
    -- Hierarchical structure
    parent_id UUID REFERENCES chart_of_accounts(id),
    
    -- Currency for this account (must match org base_currency for P&L accounts)
    currency CHAR(3) NOT NULL REFERENCES currencies(code),
    
    -- Control flags
    is_active BOOLEAN NOT NULL DEFAULT true,
    is_system_account BOOLEAN NOT NULL DEFAULT false,
    allow_direct_posting BOOLEAN NOT NULL DEFAULT true,
    
    -- Bank reconciliation
    is_bank_account BOOLEAN NOT NULL DEFAULT false,
    bank_account_number VARCHAR(50),
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    UNIQUE (organization_id, code)
);

CREATE INDEX idx_coa_org ON chart_of_accounts(organization_id) WHERE is_active = true;
CREATE INDEX idx_coa_type ON chart_of_accounts(organization_id, account_type);
CREATE INDEX idx_coa_parent ON chart_of_accounts(parent_id) WHERE parent_id IS NOT NULL;

COMMENT ON COLUMN chart_of_accounts.is_system_account IS 'System accounts cannot be deleted (e.g., Retained Earnings, Currency Gain/Loss)';
COMMENT ON COLUMN chart_of_accounts.allow_direct_posting IS 'If false, only child accounts can receive postings (header account)';
```


## Transactions & Ledger

### transactions

```sql
CREATE TYPE transaction_status AS ENUM (
    'draft',      -- Being composed, not yet submitted
    'pending',    -- Submitted, awaiting approval
    'approved',   -- Approved, ready to post
    'posted',     -- Posted to ledger, immutable
    'voided'      -- Voided via reversing entry
);

CREATE TYPE transaction_type AS ENUM (
    'journal',           -- Manual journal entry
    'expense',           -- Expense claim
    'invoice',           -- Sales invoice
    'bill',              -- Vendor bill
    'payment',           -- Payment (AR/AP)
    'transfer',          -- Bank transfer
    'adjustment',        -- Period-end adjustment
    'opening_balance',   -- Opening balance entry
    'reversal'           -- Reversing entry
);

CREATE TABLE transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    fiscal_period_id UUID NOT NULL REFERENCES fiscal_periods(id),
    
    -- Transaction identification
    reference_number VARCHAR(100),
    transaction_type transaction_type NOT NULL,
    
    -- Dates
    transaction_date DATE NOT NULL,
    
    -- Description
    description TEXT NOT NULL,
    memo TEXT,
    
    -- Status workflow
    status transaction_status NOT NULL DEFAULT 'draft',
    
    -- Audit trail
    created_by UUID NOT NULL REFERENCES users(id),
    submitted_at TIMESTAMPTZ,
    submitted_by UUID REFERENCES users(id),
    
    approved_at TIMESTAMPTZ,
    approved_by UUID REFERENCES users(id),
    approval_notes TEXT,
    
    posted_at TIMESTAMPTZ,
    posted_by UUID REFERENCES users(id),
    
    voided_at TIMESTAMPTZ,
    voided_by UUID REFERENCES users(id),
    void_reason TEXT,
    
    -- Link to reversing transaction (if voided)
    reversed_by_transaction_id UUID REFERENCES transactions(id),
    reverses_transaction_id UUID REFERENCES transactions(id),
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    UNIQUE (organization_id, reference_number)
);

CREATE INDEX idx_txn_org_date ON transactions(organization_id, transaction_date);
CREATE INDEX idx_txn_org_status ON transactions(organization_id, status);
CREATE INDEX idx_txn_fiscal_period ON transactions(fiscal_period_id);
CREATE INDEX idx_txn_created_by ON transactions(created_by);
CREATE INDEX idx_txn_pending_approval ON transactions(organization_id, created_at) 
    WHERE status = 'pending';
```

### ledger_entries

```sql
CREATE TABLE ledger_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    transaction_id UUID NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    account_id UUID NOT NULL REFERENCES chart_of_accounts(id),
    
    -- Multi-currency support: Store ALL three values
    -- Source amount (original transaction currency)
    source_currency CHAR(3) NOT NULL REFERENCES currencies(code),
    source_amount NUMERIC(19, 4) NOT NULL,
    
    -- Exchange rate at transaction date
    exchange_rate NUMERIC(19, 10) NOT NULL DEFAULT 1,
    
    -- Functional amount (converted to org base_currency)
    functional_currency CHAR(3) NOT NULL REFERENCES currencies(code),
    functional_amount NUMERIC(19, 4) NOT NULL,
    
    -- Debit/Credit in functional currency
    debit NUMERIC(19, 4) NOT NULL DEFAULT 0,
    credit NUMERIC(19, 4) NOT NULL DEFAULT 0,
    
    -- Running balance for this account (for historical balance queries)
    account_version BIGINT NOT NULL,
    account_previous_balance NUMERIC(19, 4) NOT NULL,
    account_current_balance NUMERIC(19, 4) NOT NULL,
    
    -- Line memo
    memo VARCHAR(500),
    
    -- Event timestamp (when the real-world event occurred, may differ from created_at)
    event_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    -- Constraints
    CONSTRAINT chk_debit_or_credit CHECK (
        (debit > 0 AND credit = 0) OR (debit = 0 AND credit > 0)
    ),
    CONSTRAINT chk_functional_matches_debit_credit CHECK (
        functional_amount = CASE WHEN debit > 0 THEN debit ELSE credit END
    ),
    CONSTRAINT chk_exchange_rate_positive CHECK (exchange_rate > 0)
);

CREATE INDEX idx_le_transaction ON ledger_entries(transaction_id);
CREATE INDEX idx_le_account ON ledger_entries(account_id);
CREATE INDEX idx_le_account_version ON ledger_entries(account_id, account_version);
CREATE INDEX idx_le_event_at ON ledger_entries(account_id, event_at);

COMMENT ON COLUMN ledger_entries.source_amount IS 'Original amount in transaction currency';
COMMENT ON COLUMN ledger_entries.functional_amount IS 'Converted amount in organization base currency';
COMMENT ON COLUMN ledger_entries.account_version IS 'Monotonically increasing version per account for optimistic locking';
COMMENT ON COLUMN ledger_entries.account_previous_balance IS 'Balance before this entry, enables point-in-time balance queries';
COMMENT ON COLUMN ledger_entries.account_current_balance IS 'Balance after this entry';
```

### entry_dimensions

```sql
CREATE TABLE entry_dimensions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ledger_entry_id UUID NOT NULL REFERENCES ledger_entries(id) ON DELETE CASCADE,
    dimension_value_id UUID NOT NULL REFERENCES dimension_values(id),
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    UNIQUE (ledger_entry_id, dimension_value_id)
);

CREATE INDEX idx_entry_dimensions_entry ON entry_dimensions(ledger_entry_id);
CREATE INDEX idx_entry_dimensions_value ON entry_dimensions(dimension_value_id);

COMMENT ON TABLE entry_dimensions IS 'Links ledger entries to dimension values for dimensional reporting';
```

## Budget Management

### budgets

```sql
CREATE TYPE budget_type AS ENUM (
    'annual',
    'quarterly', 
    'monthly',
    'project'
);

CREATE TABLE budgets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    fiscal_year_id UUID NOT NULL REFERENCES fiscal_years(id),
    
    name VARCHAR(255) NOT NULL,
    description TEXT,
    budget_type budget_type NOT NULL,
    
    -- Budget currency (usually org base_currency)
    currency CHAR(3) NOT NULL REFERENCES currencies(code),
    
    is_active BOOLEAN NOT NULL DEFAULT true,
    is_locked BOOLEAN NOT NULL DEFAULT false,
    
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    UNIQUE (organization_id, fiscal_year_id, name)
);

CREATE INDEX idx_budgets_org_year ON budgets(organization_id, fiscal_year_id);
```

### budget_lines

```sql
CREATE TABLE budget_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    budget_id UUID NOT NULL REFERENCES budgets(id) ON DELETE CASCADE,
    account_id UUID NOT NULL REFERENCES chart_of_accounts(id),
    fiscal_period_id UUID NOT NULL REFERENCES fiscal_periods(id),
    
    amount NUMERIC(19, 4) NOT NULL,
    
    notes TEXT,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    UNIQUE (budget_id, account_id, fiscal_period_id)
);

CREATE INDEX idx_budget_lines_budget ON budget_lines(budget_id);
CREATE INDEX idx_budget_lines_account ON budget_lines(account_id);
CREATE INDEX idx_budget_lines_period ON budget_lines(fiscal_period_id);
```

### budget_line_dimensions

```sql
CREATE TABLE budget_line_dimensions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    budget_line_id UUID NOT NULL REFERENCES budget_lines(id) ON DELETE CASCADE,
    dimension_value_id UUID NOT NULL REFERENCES dimension_values(id),
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    UNIQUE (budget_line_id, dimension_value_id)
);

CREATE INDEX idx_budget_line_dims_line ON budget_line_dimensions(budget_line_id);
```

## Attachments

### attachments

```sql
CREATE TYPE attachment_type AS ENUM (
    'receipt',
    'invoice',
    'contract',
    'supporting_document',
    'other'
);

CREATE TYPE storage_provider AS ENUM (
    'cloudflare_r2',      -- Recommended: S3-compatible, no egress fees
    'aws_s3',
    'azure_blob',
    'digitalocean_spaces',
    'supabase_storage',   -- If using Supabase
    'local'               -- For on-premise enterprise deployments
);

CREATE TABLE attachments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    transaction_id UUID REFERENCES transactions(id) ON DELETE SET NULL,
    
    attachment_type attachment_type NOT NULL DEFAULT 'other',
    
    -- File metadata
    file_name VARCHAR(255) NOT NULL,
    file_size BIGINT NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    checksum_sha256 VARCHAR(64),
    
    -- Storage reference (multi-provider support)
    storage_provider storage_provider NOT NULL DEFAULT 'cloudflare_r2',
    storage_bucket VARCHAR(100) NOT NULL,
    storage_key VARCHAR(500) NOT NULL,
    storage_region VARCHAR(50),
    
    -- Optional OCR/parsed data
    extracted_data JSONB,
    ocr_processed_at TIMESTAMPTZ,
    
    uploaded_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    CONSTRAINT chk_file_size CHECK (file_size > 0)
);

CREATE INDEX idx_attachments_transaction ON attachments(transaction_id) WHERE transaction_id IS NOT NULL;
CREATE INDEX idx_attachments_org ON attachments(organization_id);
```

## Approval Workflow

### approval_rules

```sql
CREATE TABLE approval_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    
    name VARCHAR(255) NOT NULL,
    description TEXT,
    
    -- Conditions (amount threshold in base currency)
    min_amount NUMERIC(19, 4),
    max_amount NUMERIC(19, 4),
    
    -- Which transaction types this rule applies to
    transaction_types transaction_type[] NOT NULL,
    
    -- Required approver role
    required_role user_role NOT NULL,
    
    -- Priority (lower = evaluated first)
    priority SMALLINT NOT NULL DEFAULT 0,
    
    is_active BOOLEAN NOT NULL DEFAULT true,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    CONSTRAINT chk_amount_range CHECK (max_amount IS NULL OR min_amount IS NULL OR max_amount >= min_amount)
);

CREATE INDEX idx_approval_rules_org ON approval_rules(organization_id) WHERE is_active = true;
```

## Database Constraints & Triggers

### Double-Entry Balance Enforcement

```sql
CREATE OR REPLACE FUNCTION check_transaction_balance()
RETURNS TRIGGER AS $$
DECLARE
    total_debit NUMERIC(19, 4);
    total_credit NUMERIC(19, 4);
    txn_status transaction_status;
BEGIN
    -- Only check for posted transactions
    SELECT status INTO txn_status 
    FROM transactions 
    WHERE id = NEW.transaction_id;
    
    IF txn_status = 'posted' THEN
        SELECT 
            COALESCE(SUM(debit), 0),
            COALESCE(SUM(credit), 0)
        INTO total_debit, total_credit
        FROM ledger_entries
        WHERE transaction_id = NEW.transaction_id;
        
        IF total_debit <> total_credit THEN
            RAISE EXCEPTION 'Transaction is not balanced. Debit: %, Credit: %', 
                total_debit, total_credit;
        END IF;
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE CONSTRAINT TRIGGER trg_check_balance
AFTER INSERT OR UPDATE ON ledger_entries
DEFERRABLE INITIALLY DEFERRED
FOR EACH ROW
EXECUTE FUNCTION check_transaction_balance();
```

### Fiscal Period Validation

```sql
CREATE OR REPLACE FUNCTION validate_fiscal_period_posting()
RETURNS TRIGGER AS $$
DECLARE
    period_status fiscal_period_status;
    user_role_val user_role;
BEGIN
    -- Get period status
    SELECT fp.status INTO period_status
    FROM fiscal_periods fp
    WHERE fp.id = NEW.fiscal_period_id;
    
    -- Get user role
    SELECT ou.role INTO user_role_val
    FROM organization_users ou
    JOIN transactions t ON t.organization_id = ou.organization_id
    WHERE t.id = NEW.id AND ou.user_id = NEW.posted_by;
    
    IF period_status = 'CLOSED' THEN
        RAISE EXCEPTION 'Cannot post to closed fiscal period';
    END IF;
    
    IF period_status = 'SOFT_CLOSE' AND user_role_val NOT IN ('owner', 'admin', 'accountant') THEN
        RAISE EXCEPTION 'Only accountants can post to soft-closed periods';
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_validate_fiscal_period
BEFORE UPDATE ON transactions
FOR EACH ROW
WHEN (NEW.status = 'posted' AND OLD.status <> 'posted')
EXECUTE FUNCTION validate_fiscal_period_posting();
```

### Prevent Posted Transaction Modification

```sql
CREATE OR REPLACE FUNCTION prevent_posted_modification()
RETURNS TRIGGER AS $$
BEGIN
    IF OLD.status = 'posted' AND NEW.status NOT IN ('voided') THEN
        RAISE EXCEPTION 'Cannot modify posted transaction. Create a reversing entry instead.';
    END IF;
    
    IF OLD.status = 'voided' THEN
        RAISE EXCEPTION 'Cannot modify voided transaction.';
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_prevent_posted_mod
BEFORE UPDATE ON transactions
FOR EACH ROW
EXECUTE FUNCTION prevent_posted_modification();
```

### Account Balance Tracking

```sql
CREATE OR REPLACE FUNCTION update_account_balance()
RETURNS TRIGGER AS $$
DECLARE
    current_version BIGINT;
    current_balance NUMERIC(19, 4);
    new_balance NUMERIC(19, 4);
    account_type_val account_type;
BEGIN
    -- Lock the account row for update
    SELECT coa.account_type INTO account_type_val
    FROM chart_of_accounts coa
    WHERE coa.id = NEW.account_id
    FOR UPDATE;
    
    -- Get current version and balance
    SELECT COALESCE(MAX(account_version), 0), COALESCE(MAX(account_current_balance), 0)
    INTO current_version, current_balance
    FROM ledger_entries
    WHERE account_id = NEW.account_id;
    
    -- Calculate new balance based on account type
    -- Assets & Expenses: Debit increases, Credit decreases
    -- Liabilities, Equity, Revenue: Credit increases, Debit decreases
    IF account_type_val IN ('asset', 'expense') THEN
        new_balance := current_balance + NEW.debit - NEW.credit;
    ELSE
        new_balance := current_balance + NEW.credit - NEW.debit;
    END IF;
    
    -- Set the balance tracking fields
    NEW.account_version := current_version + 1;
    NEW.account_previous_balance := current_balance;
    NEW.account_current_balance := new_balance;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_update_account_balance
BEFORE INSERT ON ledger_entries
FOR EACH ROW
EXECUTE FUNCTION update_account_balance();
```

## Row-Level Security

```sql
-- Enable RLS on all tenant tables
ALTER TABLE organizations ENABLE ROW LEVEL SECURITY;
ALTER TABLE organization_users ENABLE ROW LEVEL SECURITY;
ALTER TABLE fiscal_years ENABLE ROW LEVEL SECURITY;
ALTER TABLE fiscal_periods ENABLE ROW LEVEL SECURITY;
ALTER TABLE dimension_types ENABLE ROW LEVEL SECURITY;
ALTER TABLE dimension_values ENABLE ROW LEVEL SECURITY;
ALTER TABLE chart_of_accounts ENABLE ROW LEVEL SECURITY;
ALTER TABLE transactions ENABLE ROW LEVEL SECURITY;
ALTER TABLE ledger_entries ENABLE ROW LEVEL SECURITY;
ALTER TABLE budgets ENABLE ROW LEVEL SECURITY;
ALTER TABLE budget_lines ENABLE ROW LEVEL SECURITY;
ALTER TABLE attachments ENABLE ROW LEVEL SECURITY;
ALTER TABLE exchange_rates ENABLE ROW LEVEL SECURITY;
ALTER TABLE approval_rules ENABLE ROW LEVEL SECURITY;

-- Example policy (applied to each table)
CREATE POLICY tenant_isolation ON chart_of_accounts
    USING (organization_id = current_setting('app.current_organization_id')::UUID);

-- Application sets context before queries:
-- SET app.current_organization_id = 'org-uuid';
```

## Useful Views

### account_balances_view

```sql
CREATE VIEW account_balances_view AS
SELECT 
    coa.id AS account_id,
    coa.organization_id,
    coa.code,
    coa.name,
    coa.account_type,
    coa.currency,
    COALESCE(
        (SELECT account_current_balance 
         FROM ledger_entries le
         JOIN transactions t ON t.id = le.transaction_id
         WHERE le.account_id = coa.id AND t.status = 'posted'
         ORDER BY le.account_version DESC
         LIMIT 1),
        0
    ) AS balance
FROM chart_of_accounts coa;
```

### trial_balance_view

```sql
CREATE VIEW trial_balance_view AS
SELECT 
    coa.organization_id,
    coa.id AS account_id,
    coa.code,
    coa.name,
    coa.account_type,
    COALESCE(SUM(le.debit), 0) AS total_debit,
    COALESCE(SUM(le.credit), 0) AS total_credit,
    CASE 
        WHEN coa.account_type IN ('asset', 'expense') 
            THEN COALESCE(SUM(le.debit), 0) - COALESCE(SUM(le.credit), 0)
        ELSE COALESCE(SUM(le.credit), 0) - COALESCE(SUM(le.debit), 0)
    END AS balance
FROM chart_of_accounts coa
LEFT JOIN ledger_entries le ON le.account_id = coa.id
LEFT JOIN transactions t ON t.id = le.transaction_id AND t.status = 'posted'
GROUP BY coa.id, coa.organization_id, coa.code, coa.name, coa.account_type;
```

### budget_vs_actual_view

```sql
CREATE VIEW budget_vs_actual_view AS
SELECT 
    bl.id AS budget_line_id,
    b.organization_id,
    b.name AS budget_name,
    coa.code AS account_code,
    coa.name AS account_name,
    fp.name AS period_name,
    fp.start_date,
    fp.end_date,
    bl.amount AS budgeted,
    COALESCE(SUM(
        CASE 
            WHEN coa.account_type IN ('asset', 'expense') THEN le.debit - le.credit
            ELSE le.credit - le.debit
        END
    ), 0) AS actual,
    bl.amount - COALESCE(SUM(
        CASE 
            WHEN coa.account_type IN ('asset', 'expense') THEN le.debit - le.credit
            ELSE le.credit - le.debit
        END
    ), 0) AS variance,
    CASE 
        WHEN bl.amount = 0 THEN 0
        ELSE ROUND((COALESCE(SUM(
            CASE 
                WHEN coa.account_type IN ('asset', 'expense') THEN le.debit - le.credit
                ELSE le.credit - le.debit
            END
        ), 0) / bl.amount) * 100, 2)
    END AS utilization_percent
FROM budget_lines bl
JOIN budgets b ON b.id = bl.budget_id
JOIN chart_of_accounts coa ON coa.id = bl.account_id
JOIN fiscal_periods fp ON fp.id = bl.fiscal_period_id
LEFT JOIN ledger_entries le ON le.account_id = bl.account_id
LEFT JOIN transactions t ON t.id = le.transaction_id 
    AND t.status = 'posted'
    AND t.transaction_date BETWEEN fp.start_date AND fp.end_date
WHERE b.is_active = true
GROUP BY bl.id, b.organization_id, b.name, coa.code, coa.name, 
         fp.name, fp.start_date, fp.end_date, bl.amount;
```

### dimensional_report_view

```sql
CREATE VIEW dimensional_report_view AS
SELECT 
    t.organization_id,
    t.transaction_date,
    fp.name AS fiscal_period,
    coa.code AS account_code,
    coa.name AS account_name,
    coa.account_type,
    dt.code AS dimension_type,
    dv.code AS dimension_code,
    dv.name AS dimension_name,
    le.source_currency,
    le.source_amount,
    le.functional_currency,
    le.functional_amount,
    le.debit,
    le.credit
FROM ledger_entries le
JOIN transactions t ON t.id = le.transaction_id
JOIN fiscal_periods fp ON fp.id = t.fiscal_period_id
JOIN chart_of_accounts coa ON coa.id = le.account_id
LEFT JOIN entry_dimensions ed ON ed.ledger_entry_id = le.id
LEFT JOIN dimension_values dv ON dv.id = ed.dimension_value_id
LEFT JOIN dimension_types dt ON dt.id = dv.dimension_type_id
WHERE t.status = 'posted';
```

## Historical Balance Query Function

```sql
CREATE OR REPLACE FUNCTION get_account_balance_at(
    p_account_id UUID,
    p_as_of TIMESTAMPTZ
) RETURNS NUMERIC(19, 4) AS $$
DECLARE
    balance NUMERIC(19, 4);
BEGIN
    SELECT le.account_current_balance INTO balance
    FROM ledger_entries le
    JOIN transactions t ON t.id = le.transaction_id
    WHERE le.account_id = p_account_id
      AND t.status = 'posted'
      AND le.created_at <= p_as_of
    ORDER BY le.account_version DESC
    LIMIT 1;
    
    RETURN COALESCE(balance, 0);
END;
$$ LANGUAGE plpgsql STABLE;

-- Usage: SELECT get_account_balance_at('account-uuid', '2025-06-30 23:59:59');
```

## Exchange Rate Lookup Function

```sql
CREATE OR REPLACE FUNCTION get_exchange_rate(
    p_organization_id UUID,
    p_from_currency CHAR(3),
    p_to_currency CHAR(3),
    p_date DATE
) RETURNS NUMERIC(19, 10) AS $$
DECLARE
    rate NUMERIC(19, 10);
BEGIN
    IF p_from_currency = p_to_currency THEN
        RETURN 1;
    END IF;
    
    SELECT er.rate INTO rate
    FROM exchange_rates er
    WHERE er.organization_id = p_organization_id
      AND er.from_currency = p_from_currency
      AND er.to_currency = p_to_currency
      AND er.effective_date <= p_date
    ORDER BY er.effective_date DESC
    LIMIT 1;
    
    IF rate IS NULL THEN
        RAISE EXCEPTION 'No exchange rate found for % to % on %', 
            p_from_currency, p_to_currency, p_date;
    END IF;
    
    RETURN rate;
END;
$$ LANGUAGE plpgsql STABLE;
```


## Subscription & Tier Management

### Enums

```sql
CREATE TYPE subscription_tier AS ENUM (
    'starter',      -- $12/user/mo - basic features
    'growth',       -- $25/user/mo - multi-currency, dimensions, budgets
    'enterprise',   -- $45/user/mo - simulation, API, SSO
    'self_hosted'   -- License-based, deployed on client's server (limits configurable by client)
);

-- NOTE: Untuk SaaS, kita cuma pake starter/growth/enterprise.
-- self_hosted ada di enum supaya codebase sama antara SaaS dan self-hosted deployment.
-- Self-hosted client bisa set tier mereka sendiri di database mereka.

CREATE TYPE subscription_status AS ENUM (
    'trialing',     -- Free trial period
    'active',       -- Paid and active
    'past_due',     -- Payment failed, grace period
    'cancelled',    -- User cancelled, access until period end
    'expired'       -- Access revoked
);
```

### organizations (Additional Columns)

```sql
ALTER TABLE organizations ADD COLUMN subscription_tier subscription_tier NOT NULL DEFAULT 'starter';
ALTER TABLE organizations ADD COLUMN subscription_status subscription_status NOT NULL DEFAULT 'trialing';

-- Trial management
ALTER TABLE organizations ADD COLUMN trial_ends_at TIMESTAMPTZ;
ALTER TABLE organizations ADD COLUMN subscription_ends_at TIMESTAMPTZ;

-- Payment provider integration (Stripe, LemonSqueezy, Paddle, etc.)
ALTER TABLE organizations ADD COLUMN payment_provider VARCHAR(50);           -- 'stripe', 'lemonsqueezy', 'paddle', etc.
ALTER TABLE organizations ADD COLUMN payment_customer_id VARCHAR(255);       -- Customer ID from provider
ALTER TABLE organizations ADD COLUMN payment_subscription_id VARCHAR(255);   -- Subscription ID from provider

-- Indexes
CREATE INDEX idx_organizations_payment_customer ON organizations(payment_provider, payment_customer_id) 
    WHERE payment_customer_id IS NOT NULL;

COMMENT ON COLUMN organizations.payment_provider IS 'Payment provider: stripe, lemonsqueezy, paddle, manual (for enterprise invoicing)';
COMMENT ON COLUMN organizations.payment_customer_id IS 'Customer ID from payment provider (cus_xxx for Stripe, etc.)';
COMMENT ON COLUMN organizations.payment_subscription_id IS 'Subscription ID from payment provider';
CREATE INDEX idx_organizations_subscription_status ON organizations(subscription_status);

COMMENT ON COLUMN organizations.trial_ends_at IS '14-day trial period end. NULL if never trialed.';
COMMENT ON COLUMN organizations.subscription_ends_at IS 'When current billing period ends. For cancelled subs, access until this date.';
```

### tier_limits

Static table defining what each tier can do. Seeded once, rarely changed.

```sql
CREATE TABLE tier_limits (
    tier subscription_tier PRIMARY KEY,
    
    -- User limits
    max_users INTEGER,                          -- NULL = unlimited
    
    -- Transaction limits
    max_transactions_per_month INTEGER,         -- NULL = unlimited
    
    -- Feature limits
    max_dimensions INTEGER NOT NULL,            -- How many dimension types
    max_currencies INTEGER NOT NULL,            -- How many active currencies
    max_fiscal_periods INTEGER,                 -- NULL = unlimited
    max_budgets INTEGER,                        -- NULL = unlimited
    max_approval_rules INTEGER,                 -- NULL = unlimited
    
    -- Feature flags
    has_multi_currency BOOLEAN NOT NULL DEFAULT false,
    has_simulation BOOLEAN NOT NULL DEFAULT false,
    has_api_access BOOLEAN NOT NULL DEFAULT false,
    has_sso BOOLEAN NOT NULL DEFAULT false,
    has_custom_reports BOOLEAN NOT NULL DEFAULT false,
    has_multi_entity BOOLEAN NOT NULL DEFAULT false,  -- Future: multiple orgs under one account
    has_audit_export BOOLEAN NOT NULL DEFAULT false,
    has_priority_support BOOLEAN NOT NULL DEFAULT false,
    
    -- Retention
    audit_log_retention_days INTEGER NOT NULL DEFAULT 90,
    attachment_storage_gb INTEGER NOT NULL DEFAULT 5,
    
    -- Display
    display_name VARCHAR(50) NOT NULL,
    description TEXT,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Seed tier limits (based on BUSINESS_MODEL.md)
-- NOTE: self_hosted tier gak ada di sini karena mereka deploy database sendiri.
--       License validation untuk self-hosted di-handle terpisah (license key check).
INSERT INTO tier_limits (
    tier, 
    max_users, max_transactions_per_month,
    max_dimensions, max_currencies, max_fiscal_periods, max_budgets, max_approval_rules,
    has_multi_currency, has_simulation, has_api_access, has_sso, has_custom_reports, has_multi_entity, has_audit_export, has_priority_support,
    audit_log_retention_days, attachment_storage_gb,
    display_name, description
) VALUES 
(
    'starter',
    50, 1000,                                   -- max 50 users, 1000 txn/month
    2, 1, 24, 3, 3,                             -- 2 dimensions, single currency, 2 years periods, 3 budgets
    false, false, false, false, false, false, false, false,
    90, 5,
    'Starter', 'For small teams getting started with expense tracking'
),
(
    'growth',
    200, 10000,                                 -- max 200 users, 10k txn/month
    NULL, NULL, NULL, NULL, NULL,              -- unlimited dimensions, currencies, etc.
    true, false, true, false, true, false, true, false,
    365, 50,
    'Growth', 'For growing companies needing multi-currency and dimensional accounting'
),
(
    'enterprise',
    NULL, NULL,                                 -- unlimited
    NULL, NULL, NULL, NULL, NULL,              -- unlimited everything
    true, true, true, true, true, true, true, true,
    2555, 500,                                  -- 7 years retention, 500GB storage
    'Enterprise', 'Full-featured solution with simulation, SSO, and dedicated support'
);

-- Self-hosted: Mereka punya database sendiri, tier_limits di-seed pas deployment.
-- Default self-hosted = unlimited semua, tapi bisa di-customize client.
```

### organization_usage

Track monthly usage for limit enforcement and billing.

```sql
CREATE TABLE organization_usage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    
    -- Period (monthly tracking)
    year_month CHAR(7) NOT NULL,               -- Format: '2026-01'
    
    -- Counts
    transaction_count INTEGER NOT NULL DEFAULT 0,
    api_call_count INTEGER NOT NULL DEFAULT 0,
    storage_used_bytes BIGINT NOT NULL DEFAULT 0,
    
    -- Snapshot of active counts (updated periodically)
    active_user_count INTEGER NOT NULL DEFAULT 0,
    active_dimension_count INTEGER NOT NULL DEFAULT 0,
    active_currency_count INTEGER NOT NULL DEFAULT 0,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    UNIQUE (organization_id, year_month)
);

CREATE INDEX idx_org_usage_org_month ON organization_usage(organization_id, year_month DESC);

COMMENT ON TABLE organization_usage IS 'Monthly usage tracking for tier limit enforcement and overage billing';
```

### Tier Check Functions

```sql
-- Check if organization can perform action based on tier limits
CREATE OR REPLACE FUNCTION check_tier_limit(
    p_organization_id UUID,
    p_limit_type VARCHAR(50)
) RETURNS BOOLEAN AS $
DECLARE
    org_tier subscription_tier;
    org_status subscription_status;
    limit_value INTEGER;
    current_count INTEGER;
    current_month CHAR(7);
BEGIN
    -- Get org subscription info
    SELECT subscription_tier, subscription_status 
    INTO org_tier, org_status
    FROM organizations 
    WHERE id = p_organization_id;
    
    -- Check subscription is active
    IF org_status NOT IN ('trialing', 'active') THEN
        RETURN false;
    END IF;
    
    current_month := to_char(now(), 'YYYY-MM');
    
    -- Check specific limit
    CASE p_limit_type
        WHEN 'users' THEN
            SELECT max_users INTO limit_value FROM tier_limits WHERE tier = org_tier;
            SELECT COUNT(*) INTO current_count FROM organization_users WHERE organization_id = p_organization_id;
            
        WHEN 'transactions' THEN
            SELECT max_transactions_per_month INTO limit_value FROM tier_limits WHERE tier = org_tier;
            SELECT COALESCE(transaction_count, 0) INTO current_count 
            FROM organization_usage 
            WHERE organization_id = p_organization_id AND year_month = current_month;
            
        WHEN 'dimensions' THEN
            SELECT max_dimensions INTO limit_value FROM tier_limits WHERE tier = org_tier;
            SELECT COUNT(*) INTO current_count FROM dimension_types WHERE organization_id = p_organization_id AND is_active = true;
            
        WHEN 'currencies' THEN
            SELECT max_currencies INTO limit_value FROM tier_limits WHERE tier = org_tier;
            -- Count distinct currencies used in exchange_rates
            SELECT COUNT(DISTINCT from_currency) + 1 INTO current_count  -- +1 for base currency
            FROM exchange_rates WHERE organization_id = p_organization_id;
            
        ELSE
            RETURN true;  -- Unknown limit type, allow
    END CASE;
    
    -- NULL limit means unlimited
    IF limit_value IS NULL THEN
        RETURN true;
    END IF;
    
    RETURN current_count < limit_value;
END;
$ LANGUAGE plpgsql STABLE;

-- Check if organization has specific feature
CREATE OR REPLACE FUNCTION has_feature(
    p_organization_id UUID,
    p_feature VARCHAR(50)
) RETURNS BOOLEAN AS $
DECLARE
    org_tier subscription_tier;
    org_status subscription_status;
    result BOOLEAN;
BEGIN
    -- Get org subscription info
    SELECT subscription_tier, subscription_status 
    INTO org_tier, org_status
    FROM organizations 
    WHERE id = p_organization_id;
    
    -- Check subscription is active
    IF org_status NOT IN ('trialing', 'active') THEN
        RETURN false;
    END IF;
    
    -- Check feature flag
    EXECUTE format(
        'SELECT %I FROM tier_limits WHERE tier = $1',
        'has_' || p_feature
    ) INTO result USING org_tier;
    
    RETURN COALESCE(result, false);
END;
$ LANGUAGE plpgsql STABLE;

-- Usage examples:
-- SELECT check_tier_limit('org-uuid', 'users');        -- Can add more users?
-- SELECT check_tier_limit('org-uuid', 'transactions'); -- Can create more transactions this month?
-- SELECT has_feature('org-uuid', 'simulation');        -- Can use simulation?
-- SELECT has_feature('org-uuid', 'multi_currency');    -- Can use multi-currency?
```

### Increment Usage Trigger

```sql
-- Auto-increment transaction count when transaction is posted
CREATE OR REPLACE FUNCTION increment_transaction_usage()
RETURNS TRIGGER AS $
DECLARE
    current_month CHAR(7);
BEGIN
    IF NEW.status = 'posted' AND (OLD.status IS NULL OR OLD.status <> 'posted') THEN
        current_month := to_char(now(), 'YYYY-MM');
        
        INSERT INTO organization_usage (organization_id, year_month, transaction_count)
        VALUES (NEW.organization_id, current_month, 1)
        ON CONFLICT (organization_id, year_month) 
        DO UPDATE SET 
            transaction_count = organization_usage.transaction_count + 1,
            updated_at = now();
    END IF;
    
    RETURN NEW;
END;
$ LANGUAGE plpgsql;

CREATE TRIGGER trg_increment_transaction_usage
AFTER INSERT OR UPDATE ON transactions
FOR EACH ROW
EXECUTE FUNCTION increment_transaction_usage();
```

### RLS for Tier Tables

```sql
-- tier_limits is public read (no org-specific data)
-- organization_usage needs RLS
ALTER TABLE organization_usage ENABLE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation ON organization_usage
    USING (organization_id = current_setting('app.current_organization_id')::UUID);
```


## Self-Hosted License Management (Admin/Internal)

> **NOTE:** Table ini untuk internal tracking di SaaS database kita.
> Self-hosted clients TIDAK hit database ini - mereka punya DB sendiri.
> Ini murni untuk admin: track siapa beli license, kapan expire, renewal reminder.

### self_hosted_licenses

```sql
CREATE TYPE license_type AS ENUM (
    'annual',           -- $25k/year, stops updates on expiry
    'perpetual',        -- $75k one-time, optional maintenance
    'enterprise_plus'   -- $100k/year, premium support + custom dev
);

CREATE TYPE license_status AS ENUM (
    'active',           -- Valid and current
    'expiring_soon',    -- <90 days to expiry (trigger renewal outreach)
    'expired',          -- Past expiry, no updates (app still works)
    'revoked'           -- Terminated (breach of contract, etc.)
);

CREATE TABLE self_hosted_licenses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Customer info
    customer_name VARCHAR(255) NOT NULL,
    customer_email VARCHAR(255) NOT NULL,
    customer_company VARCHAR(255) NOT NULL,
    customer_country VARCHAR(100),
    
    -- License details
    license_key VARCHAR(100) NOT NULL UNIQUE,  -- The actual key given to customer
    license_type license_type NOT NULL,
    status license_status NOT NULL DEFAULT 'active',
    
    -- Dates
    issued_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    starts_at DATE NOT NULL,
    expires_at DATE,                            -- NULL for perpetual (license never expires)
    maintenance_until DATE,                     -- For perpetual: when maintenance/updates stop
    
    -- Financials (for internal tracking, NOT payment processing)
    deal_value_usd NUMERIC(12, 2) NOT NULL,     -- Total contract value
    payment_method VARCHAR(50),                 -- 'wire_transfer', 'invoice', 'check'
    payment_reference VARCHAR(255),             -- Invoice number, wire ref, etc.
    paid_at DATE,
    
    -- Delivery
    delivery_method VARCHAR(50),                -- 'docker_registry', 'github_repo', 'manual'
    delivered_at TIMESTAMPTZ,
    delivered_version VARCHAR(50),              -- e.g., 'v1.2.0'
    
    -- Notes
    notes TEXT,                                 -- Internal notes, special terms, etc.
    
    -- Audit
    created_by UUID REFERENCES users(id),       -- Admin who created this
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_sh_licenses_status ON self_hosted_licenses(status);
CREATE INDEX idx_sh_licenses_expires ON self_hosted_licenses(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX idx_sh_licenses_customer ON self_hosted_licenses(customer_company);

COMMENT ON TABLE self_hosted_licenses IS 'Internal tracking of self-hosted license sales. NOT used by self-hosted deployments.';
COMMENT ON COLUMN self_hosted_licenses.license_key IS 'The signed license key/file content given to customer';
COMMENT ON COLUMN self_hosted_licenses.expires_at IS 'NULL for perpetual licenses (they never expire, just lose updates)';
```

### license_audit_log

Track semua aktivitas terkait license (untuk audit trail internal).

```sql
CREATE TABLE license_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    license_id UUID NOT NULL REFERENCES self_hosted_licenses(id) ON DELETE CASCADE,
    
    action VARCHAR(50) NOT NULL,               -- 'created', 'renewed', 'expired', 'revoked', 'delivered', 'updated'
    details JSONB,                             -- Action-specific details
    
    performed_by UUID REFERENCES users(id),    -- Admin who did this
    performed_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_license_audit_license ON license_audit_log(license_id);

-- Example actions:
-- { "action": "created", "details": { "type": "annual", "value": 25000 } }
-- { "action": "renewed", "details": { "old_expires": "2026-03-15", "new_expires": "2027-03-15", "value": 25000 } }
-- { "action": "delivered", "details": { "method": "docker_registry", "version": "v1.2.0" } }
-- { "action": "revoked", "details": { "reason": "Non-payment after 90 days" } }
```

### Useful Views

```sql
-- Licenses expiring soon (for renewal outreach)
CREATE VIEW licenses_expiring_soon AS
SELECT 
    id,
    customer_company,
    customer_email,
    license_type,
    expires_at,
    (expires_at - CURRENT_DATE) AS days_until_expiry,
    deal_value_usd
FROM self_hosted_licenses
WHERE status = 'active'
  AND expires_at IS NOT NULL
  AND expires_at <= CURRENT_DATE + INTERVAL '90 days'
ORDER BY expires_at;

-- Revenue summary by year
CREATE VIEW license_revenue_by_year AS
SELECT 
    EXTRACT(YEAR FROM paid_at) AS year,
    license_type,
    COUNT(*) AS deals,
    SUM(deal_value_usd) AS total_revenue
FROM self_hosted_licenses
WHERE paid_at IS NOT NULL
GROUP BY EXTRACT(YEAR FROM paid_at), license_type
ORDER BY year DESC, license_type;
```

### Cron Job: Update Expiring Status

```sql
-- Run daily to update status for licenses expiring soon
CREATE OR REPLACE FUNCTION update_license_expiry_status()
RETURNS void AS $
BEGIN
    -- Mark as expiring_soon (90 days before)
    UPDATE self_hosted_licenses
    SET status = 'expiring_soon', updated_at = now()
    WHERE status = 'active'
      AND expires_at IS NOT NULL
      AND expires_at <= CURRENT_DATE + INTERVAL '90 days';
    
    -- Mark as expired
    UPDATE self_hosted_licenses
    SET status = 'expired', updated_at = now()
    WHERE status IN ('active', 'expiring_soon')
      AND expires_at IS NOT NULL
      AND expires_at < CURRENT_DATE;
END;
$ LANGUAGE plpgsql;

-- Call via pg_cron or application cron:
-- SELECT update_license_expiry_status();
```
