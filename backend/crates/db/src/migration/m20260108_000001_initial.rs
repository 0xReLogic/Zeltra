//! Initial database migration.
//!
//! Creates all core tables, enums, triggers, functions, views, and RLS policies
//! as defined in `docs/DATABASE_SCHEMA.md`.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // ============================================================
        // PART 1: ENUMS
        // ============================================================
        db.execute_unprepared(ENUMS_SQL).await?;

        // ============================================================
        // PART 2: CORE TABLES
        // ============================================================
        db.execute_unprepared(USERS_SQL).await?;
        db.execute_unprepared(ORGANIZATIONS_SQL).await?;
        db.execute_unprepared(ORGANIZATION_USERS_SQL).await?;

        // ============================================================
        // PART 3: CURRENCY MANAGEMENT
        // ============================================================
        db.execute_unprepared(CURRENCIES_SQL).await?;
        db.execute_unprepared(EXCHANGE_RATES_SQL).await?;

        // ============================================================
        // PART 4: FISCAL PERIOD MANAGEMENT
        // ============================================================
        db.execute_unprepared(FISCAL_YEARS_SQL).await?;
        db.execute_unprepared(FISCAL_PERIODS_SQL).await?;

        // ============================================================
        // PART 5: DIMENSIONAL ACCOUNTING
        // ============================================================
        db.execute_unprepared(DIMENSION_TYPES_SQL).await?;
        db.execute_unprepared(DIMENSION_VALUES_SQL).await?;

        // ============================================================
        // PART 6: CHART OF ACCOUNTS
        // ============================================================
        db.execute_unprepared(CHART_OF_ACCOUNTS_SQL).await?;

        // ============================================================
        // PART 7: TRANSACTIONS & LEDGER
        // ============================================================
        db.execute_unprepared(TRANSACTIONS_SQL).await?;
        db.execute_unprepared(LEDGER_ENTRIES_SQL).await?;
        db.execute_unprepared(ENTRY_DIMENSIONS_SQL).await?;

        // ============================================================
        // PART 8: BUDGET MANAGEMENT
        // ============================================================
        db.execute_unprepared(BUDGETS_SQL).await?;
        db.execute_unprepared(BUDGET_LINES_SQL).await?;
        db.execute_unprepared(BUDGET_LINE_DIMENSIONS_SQL).await?;

        // ============================================================
        // PART 9: ATTACHMENTS
        // ============================================================
        db.execute_unprepared(ATTACHMENTS_SQL).await?;

        // ============================================================
        // PART 10: APPROVAL WORKFLOW
        // ============================================================
        db.execute_unprepared(APPROVAL_RULES_SQL).await?;

        // ============================================================
        // PART 11: SUBSCRIPTION & TIER MANAGEMENT
        // ============================================================
        db.execute_unprepared(TIER_LIMITS_SQL).await?;
        db.execute_unprepared(ORGANIZATION_USAGE_SQL).await?;

        // ============================================================
        // PART 12: TRIGGERS & FUNCTIONS
        // ============================================================
        db.execute_unprepared(TRIGGERS_SQL).await?;

        // ============================================================
        // PART 13: VIEWS
        // ============================================================
        db.execute_unprepared(VIEWS_SQL).await?;

        // ============================================================
        // PART 14: ROW-LEVEL SECURITY
        // ============================================================
        db.execute_unprepared(RLS_SQL).await?;

        // ============================================================
        // PART 15: SEED DATA
        // ============================================================
        db.execute_unprepared(SEED_CURRENCIES_SQL).await?;
        db.execute_unprepared(SEED_TIER_LIMITS_SQL).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(DROP_ALL_SQL).await?;
        Ok(())
    }
}

// ============================================================
// SQL CONSTANTS
// ============================================================

const ENUMS_SQL: &str = r"
-- User roles
CREATE TYPE user_role AS ENUM (
    'owner',
    'admin', 
    'accountant',
    'approver',
    'viewer',
    'submitter'
);

-- Exchange rate source
CREATE TYPE rate_source AS ENUM ('manual', 'api', 'bank_feed');

-- Fiscal year status
CREATE TYPE fiscal_year_status AS ENUM ('OPEN', 'CLOSED');

-- Fiscal period status
CREATE TYPE fiscal_period_status AS ENUM (
    'OPEN',
    'SOFT_CLOSE',
    'CLOSED'
);

-- Account types
CREATE TYPE account_type AS ENUM (
    'asset',
    'liability', 
    'equity',
    'revenue',
    'expense'
);

-- Account subtypes
CREATE TYPE account_subtype AS ENUM (
    'cash',
    'bank',
    'accounts_receivable',
    'inventory',
    'prepaid',
    'fixed_asset',
    'accumulated_depreciation',
    'other_asset',
    'accounts_payable',
    'credit_card',
    'accrued_liability',
    'short_term_debt',
    'long_term_debt',
    'other_liability',
    'owner_equity',
    'retained_earnings',
    'common_stock',
    'other_equity',
    'operating_revenue',
    'other_revenue',
    'cost_of_goods_sold',
    'operating_expense',
    'payroll_expense',
    'depreciation_expense',
    'interest_expense',
    'tax_expense',
    'other_expense'
);

-- Transaction status
CREATE TYPE transaction_status AS ENUM (
    'draft',
    'pending',
    'approved',
    'posted',
    'voided'
);

-- Transaction type
CREATE TYPE transaction_type AS ENUM (
    'journal',
    'expense',
    'invoice',
    'bill',
    'payment',
    'transfer',
    'adjustment',
    'opening_balance',
    'reversal'
);

-- Budget type
CREATE TYPE budget_type AS ENUM (
    'annual',
    'quarterly', 
    'monthly',
    'project'
);

-- Attachment type
CREATE TYPE attachment_type AS ENUM (
    'receipt',
    'invoice',
    'contract',
    'supporting_document',
    'other'
);

-- Storage provider
CREATE TYPE storage_provider AS ENUM (
    'cloudflare_r2',
    'aws_s3',
    'azure_blob',
    'digitalocean_spaces',
    'supabase_storage',
    'local'
);

-- Subscription tier
CREATE TYPE subscription_tier AS ENUM (
    'starter',
    'growth',
    'enterprise',
    'self_hosted'
);

-- Subscription status
CREATE TYPE subscription_status AS ENUM (
    'trialing',
    'active',
    'past_due',
    'cancelled',
    'expired'
);
";

const USERS_SQL: &str = r"
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
";

const ORGANIZATIONS_SQL: &str = r"
CREATE TABLE organizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(100) NOT NULL UNIQUE,
    base_currency CHAR(3) NOT NULL,
    timezone VARCHAR(50) NOT NULL DEFAULT 'UTC',
    settings JSONB NOT NULL DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT true,
    
    -- Subscription fields
    subscription_tier subscription_tier NOT NULL DEFAULT 'starter',
    subscription_status subscription_status NOT NULL DEFAULT 'trialing',
    trial_ends_at TIMESTAMPTZ,
    subscription_ends_at TIMESTAMPTZ,
    
    -- Payment provider integration
    payment_provider VARCHAR(50),
    payment_customer_id VARCHAR(255),
    payment_subscription_id VARCHAR(255),
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    CONSTRAINT chk_base_currency_format CHECK (base_currency ~ '^[A-Z]{3}$')
);

CREATE INDEX idx_organizations_slug ON organizations(slug);
CREATE INDEX idx_organizations_subscription_status ON organizations(subscription_status);
CREATE INDEX idx_organizations_payment_customer ON organizations(payment_provider, payment_customer_id) 
    WHERE payment_customer_id IS NOT NULL;
";

const ORGANIZATION_USERS_SQL: &str = r"
CREATE TABLE organization_users (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    role user_role NOT NULL DEFAULT 'viewer',
    approval_limit NUMERIC(19, 4),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, organization_id)
);

CREATE INDEX idx_org_users_org ON organization_users(organization_id);
";

const CURRENCIES_SQL: &str = r"
CREATE TABLE currencies (
    code CHAR(3) PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    symbol VARCHAR(10) NOT NULL,
    decimal_places SMALLINT NOT NULL DEFAULT 2,
    is_active BOOLEAN NOT NULL DEFAULT true,
    CONSTRAINT chk_currency_code CHECK (code ~ '^[A-Z]{3}$'),
    CONSTRAINT chk_decimal_places CHECK (decimal_places BETWEEN 0 AND 4)
);
";

const EXCHANGE_RATES_SQL: &str = r"
CREATE TABLE exchange_rates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    from_currency CHAR(3) NOT NULL REFERENCES currencies(code),
    to_currency CHAR(3) NOT NULL REFERENCES currencies(code),
    rate NUMERIC(19, 10) NOT NULL,
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
";

const FISCAL_YEARS_SQL: &str = r"
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
";

const FISCAL_PERIODS_SQL: &str = r"
CREATE TABLE fiscal_periods (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    fiscal_year_id UUID NOT NULL REFERENCES fiscal_years(id) ON DELETE CASCADE,
    name VARCHAR(50) NOT NULL,
    period_number SMALLINT NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    status fiscal_period_status NOT NULL DEFAULT 'OPEN',
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
";

const DIMENSION_TYPES_SQL: &str = r"
CREATE TABLE dimension_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    is_required BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    sort_order SMALLINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (organization_id, code)
);

CREATE INDEX idx_dimension_types_org ON dimension_types(organization_id) WHERE is_active = true;
";

const DIMENSION_VALUES_SQL: &str = r"
CREATE TABLE dimension_values (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    dimension_type_id UUID NOT NULL REFERENCES dimension_types(id) ON DELETE CASCADE,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    parent_id UUID REFERENCES dimension_values(id),
    is_active BOOLEAN NOT NULL DEFAULT true,
    effective_from DATE,
    effective_to DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (organization_id, dimension_type_id, code),
    CONSTRAINT chk_effective_dates CHECK (effective_to IS NULL OR effective_to >= effective_from)
);

CREATE INDEX idx_dimension_values_type ON dimension_values(dimension_type_id) WHERE is_active = true;
CREATE INDEX idx_dimension_values_parent ON dimension_values(parent_id) WHERE parent_id IS NOT NULL;
";

const CHART_OF_ACCOUNTS_SQL: &str = r"
CREATE TABLE chart_of_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    code VARCHAR(20) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    account_type account_type NOT NULL,
    account_subtype account_subtype,
    parent_id UUID REFERENCES chart_of_accounts(id),
    currency CHAR(3) NOT NULL REFERENCES currencies(code),
    is_active BOOLEAN NOT NULL DEFAULT true,
    is_system_account BOOLEAN NOT NULL DEFAULT false,
    allow_direct_posting BOOLEAN NOT NULL DEFAULT true,
    is_bank_account BOOLEAN NOT NULL DEFAULT false,
    bank_account_number VARCHAR(50),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (organization_id, code)
);

CREATE INDEX idx_coa_org ON chart_of_accounts(organization_id) WHERE is_active = true;
CREATE INDEX idx_coa_type ON chart_of_accounts(organization_id, account_type);
CREATE INDEX idx_coa_parent ON chart_of_accounts(parent_id) WHERE parent_id IS NOT NULL;
";

const TRANSACTIONS_SQL: &str = r"
CREATE TABLE transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    fiscal_period_id UUID NOT NULL REFERENCES fiscal_periods(id),
    reference_number VARCHAR(100),
    transaction_type transaction_type NOT NULL,
    transaction_date DATE NOT NULL,
    description TEXT NOT NULL,
    memo TEXT,
    status transaction_status NOT NULL DEFAULT 'draft',
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
CREATE INDEX idx_txn_pending_approval ON transactions(organization_id, created_at) WHERE status = 'pending';
";

const LEDGER_ENTRIES_SQL: &str = r"
CREATE TABLE ledger_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    transaction_id UUID NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    account_id UUID NOT NULL REFERENCES chart_of_accounts(id),
    source_currency CHAR(3) NOT NULL REFERENCES currencies(code),
    source_amount NUMERIC(19, 4) NOT NULL,
    exchange_rate NUMERIC(19, 10) NOT NULL DEFAULT 1,
    functional_currency CHAR(3) NOT NULL REFERENCES currencies(code),
    functional_amount NUMERIC(19, 4) NOT NULL,
    debit NUMERIC(19, 4) NOT NULL DEFAULT 0,
    credit NUMERIC(19, 4) NOT NULL DEFAULT 0,
    account_version BIGINT NOT NULL,
    account_previous_balance NUMERIC(19, 4) NOT NULL,
    account_current_balance NUMERIC(19, 4) NOT NULL,
    memo VARCHAR(500),
    event_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
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
";

const ENTRY_DIMENSIONS_SQL: &str = r"
CREATE TABLE entry_dimensions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ledger_entry_id UUID NOT NULL REFERENCES ledger_entries(id) ON DELETE CASCADE,
    dimension_value_id UUID NOT NULL REFERENCES dimension_values(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (ledger_entry_id, dimension_value_id)
);

CREATE INDEX idx_entry_dimensions_entry ON entry_dimensions(ledger_entry_id);
CREATE INDEX idx_entry_dimensions_value ON entry_dimensions(dimension_value_id);
";

const BUDGETS_SQL: &str = r"
CREATE TABLE budgets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    fiscal_year_id UUID NOT NULL REFERENCES fiscal_years(id),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    budget_type budget_type NOT NULL,
    currency CHAR(3) NOT NULL REFERENCES currencies(code),
    is_active BOOLEAN NOT NULL DEFAULT true,
    is_locked BOOLEAN NOT NULL DEFAULT false,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (organization_id, fiscal_year_id, name)
);

CREATE INDEX idx_budgets_org_year ON budgets(organization_id, fiscal_year_id);
";

const BUDGET_LINES_SQL: &str = r"
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
";

const BUDGET_LINE_DIMENSIONS_SQL: &str = r"
CREATE TABLE budget_line_dimensions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    budget_line_id UUID NOT NULL REFERENCES budget_lines(id) ON DELETE CASCADE,
    dimension_value_id UUID NOT NULL REFERENCES dimension_values(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (budget_line_id, dimension_value_id)
);

CREATE INDEX idx_budget_line_dims_line ON budget_line_dimensions(budget_line_id);
";

const ATTACHMENTS_SQL: &str = r"
CREATE TABLE attachments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    transaction_id UUID REFERENCES transactions(id) ON DELETE SET NULL,
    attachment_type attachment_type NOT NULL DEFAULT 'other',
    file_name VARCHAR(255) NOT NULL,
    file_size BIGINT NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    checksum_sha256 VARCHAR(64),
    storage_provider storage_provider NOT NULL DEFAULT 'cloudflare_r2',
    storage_bucket VARCHAR(100) NOT NULL,
    storage_key VARCHAR(500) NOT NULL,
    storage_region VARCHAR(50),
    extracted_data JSONB,
    ocr_processed_at TIMESTAMPTZ,
    uploaded_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_file_size CHECK (file_size > 0)
);

CREATE INDEX idx_attachments_transaction ON attachments(transaction_id) WHERE transaction_id IS NOT NULL;
CREATE INDEX idx_attachments_org ON attachments(organization_id);
";

const APPROVAL_RULES_SQL: &str = r"
CREATE TABLE approval_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    min_amount NUMERIC(19, 4),
    max_amount NUMERIC(19, 4),
    transaction_types transaction_type[] NOT NULL,
    required_role user_role NOT NULL,
    priority SMALLINT NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_amount_range CHECK (max_amount IS NULL OR min_amount IS NULL OR max_amount >= min_amount)
);

CREATE INDEX idx_approval_rules_org ON approval_rules(organization_id) WHERE is_active = true;
";

const TIER_LIMITS_SQL: &str = r"
CREATE TABLE tier_limits (
    tier subscription_tier PRIMARY KEY,
    max_users INTEGER,
    max_transactions_per_month INTEGER,
    max_dimensions INTEGER NOT NULL,
    max_currencies INTEGER NOT NULL,
    max_fiscal_periods INTEGER,
    max_budgets INTEGER,
    max_approval_rules INTEGER,
    has_multi_currency BOOLEAN NOT NULL DEFAULT false,
    has_simulation BOOLEAN NOT NULL DEFAULT false,
    has_api_access BOOLEAN NOT NULL DEFAULT false,
    has_sso BOOLEAN NOT NULL DEFAULT false,
    has_custom_reports BOOLEAN NOT NULL DEFAULT false,
    has_multi_entity BOOLEAN NOT NULL DEFAULT false,
    has_audit_export BOOLEAN NOT NULL DEFAULT false,
    has_priority_support BOOLEAN NOT NULL DEFAULT false,
    audit_log_retention_days INTEGER NOT NULL DEFAULT 90,
    attachment_storage_gb INTEGER NOT NULL DEFAULT 5,
    display_name VARCHAR(50) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
";

const ORGANIZATION_USAGE_SQL: &str = r"
CREATE TABLE organization_usage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    year_month CHAR(7) NOT NULL,
    transaction_count INTEGER NOT NULL DEFAULT 0,
    api_call_count INTEGER NOT NULL DEFAULT 0,
    storage_used_bytes BIGINT NOT NULL DEFAULT 0,
    active_user_count INTEGER NOT NULL DEFAULT 0,
    active_dimension_count INTEGER NOT NULL DEFAULT 0,
    active_currency_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (organization_id, year_month)
);

CREATE INDEX idx_org_usage_org_month ON organization_usage(organization_id, year_month DESC);
";

const TRIGGERS_SQL: &str = r"
-- ============================================================
-- FUNCTION: check_transaction_balance
-- Ensures double-entry balance (debit = credit) for posted transactions
-- ============================================================
CREATE OR REPLACE FUNCTION check_transaction_balance()
RETURNS TRIGGER AS $$
DECLARE
    total_debit NUMERIC(19, 4);
    total_credit NUMERIC(19, 4);
    txn_status transaction_status;
BEGIN
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

-- ============================================================
-- FUNCTION: validate_fiscal_period_posting
-- Validates posting rules based on fiscal period status
-- ============================================================
CREATE OR REPLACE FUNCTION validate_fiscal_period_posting()
RETURNS TRIGGER AS $$
DECLARE
    period_status fiscal_period_status;
    user_role_val user_role;
BEGIN
    SELECT fp.status INTO period_status
    FROM fiscal_periods fp
    WHERE fp.id = NEW.fiscal_period_id;
    
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

-- ============================================================
-- FUNCTION: prevent_posted_modification
-- Prevents modification of posted/voided transactions
-- ============================================================
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

-- ============================================================
-- FUNCTION: update_account_balance
-- Tracks running balance per account for historical queries
-- ============================================================
CREATE OR REPLACE FUNCTION update_account_balance()
RETURNS TRIGGER AS $$
DECLARE
    current_version BIGINT;
    current_balance NUMERIC(19, 4);
    new_balance NUMERIC(19, 4);
    account_type_val account_type;
BEGIN
    SELECT coa.account_type INTO account_type_val
    FROM chart_of_accounts coa
    WHERE coa.id = NEW.account_id
    FOR UPDATE;
    
    SELECT COALESCE(MAX(account_version), 0), COALESCE(MAX(account_current_balance), 0)
    INTO current_version, current_balance
    FROM ledger_entries
    WHERE account_id = NEW.account_id;
    
    IF account_type_val IN ('asset', 'expense') THEN
        new_balance := current_balance + NEW.debit - NEW.credit;
    ELSE
        new_balance := current_balance + NEW.credit - NEW.debit;
    END IF;
    
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

-- ============================================================
-- FUNCTION: increment_transaction_usage
-- Auto-increment transaction count for tier limit tracking
-- ============================================================
CREATE OR REPLACE FUNCTION increment_transaction_usage()
RETURNS TRIGGER AS $$
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
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_increment_transaction_usage
AFTER INSERT OR UPDATE ON transactions
FOR EACH ROW
EXECUTE FUNCTION increment_transaction_usage();

-- ============================================================
-- FUNCTION: get_account_balance_at
-- Returns account balance at a specific point in time
-- ============================================================
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

-- ============================================================
-- FUNCTION: get_exchange_rate
-- Looks up exchange rate for a given date
-- ============================================================
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

-- ============================================================
-- FUNCTION: check_tier_limit
-- Checks if organization can perform action based on tier limits
-- ============================================================
CREATE OR REPLACE FUNCTION check_tier_limit(
    p_organization_id UUID,
    p_limit_type VARCHAR(50)
) RETURNS BOOLEAN AS $$
DECLARE
    org_tier subscription_tier;
    org_status subscription_status;
    limit_value INTEGER;
    current_count INTEGER;
    current_month CHAR(7);
BEGIN
    SELECT subscription_tier, subscription_status 
    INTO org_tier, org_status
    FROM organizations 
    WHERE id = p_organization_id;
    
    IF org_status NOT IN ('trialing', 'active') THEN
        RETURN false;
    END IF;
    
    current_month := to_char(now(), 'YYYY-MM');
    
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
            SELECT COUNT(DISTINCT from_currency) + 1 INTO current_count
            FROM exchange_rates WHERE organization_id = p_organization_id;
            
        ELSE
            RETURN true;
    END CASE;
    
    IF limit_value IS NULL THEN
        RETURN true;
    END IF;
    
    RETURN current_count < limit_value;
END;
$$ LANGUAGE plpgsql STABLE;

-- ============================================================
-- FUNCTION: has_feature
-- Checks if organization has access to a specific feature
-- ============================================================
CREATE OR REPLACE FUNCTION has_feature(
    p_organization_id UUID,
    p_feature VARCHAR(50)
) RETURNS BOOLEAN AS $$
DECLARE
    org_tier subscription_tier;
    org_status subscription_status;
    result BOOLEAN;
BEGIN
    SELECT subscription_tier, subscription_status 
    INTO org_tier, org_status
    FROM organizations 
    WHERE id = p_organization_id;
    
    IF org_status NOT IN ('trialing', 'active') THEN
        RETURN false;
    END IF;
    
    EXECUTE format(
        'SELECT %I FROM tier_limits WHERE tier = $1',
        'has_' || p_feature
    ) INTO result USING org_tier;
    
    RETURN COALESCE(result, false);
END;
$$ LANGUAGE plpgsql STABLE;
";

const VIEWS_SQL: &str = r"
-- ============================================================
-- VIEW: account_balances_view
-- Current balance for each account
-- ============================================================
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

-- ============================================================
-- VIEW: trial_balance_view
-- Trial balance report with debit/credit totals
-- ============================================================
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

-- ============================================================
-- VIEW: budget_vs_actual_view
-- Budget vs actual comparison with variance
-- ============================================================
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

-- ============================================================
-- VIEW: dimensional_report_view
-- Dimensional analysis for reporting
-- ============================================================
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
";

const RLS_SQL: &str = r"
-- ============================================================
-- ROW-LEVEL SECURITY POLICIES
-- Enable RLS on all tenant tables
-- ============================================================

-- Enable RLS
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
ALTER TABLE budget_line_dimensions ENABLE ROW LEVEL SECURITY;
ALTER TABLE attachments ENABLE ROW LEVEL SECURITY;
ALTER TABLE exchange_rates ENABLE ROW LEVEL SECURITY;
ALTER TABLE approval_rules ENABLE ROW LEVEL SECURITY;
ALTER TABLE organization_usage ENABLE ROW LEVEL SECURITY;
ALTER TABLE entry_dimensions ENABLE ROW LEVEL SECURITY;

-- Create policies for tenant isolation
-- Application sets context before queries: SET app.current_organization_id = 'org-uuid';

CREATE POLICY tenant_isolation ON organizations
    USING (id = current_setting('app.current_organization_id', true)::UUID);

CREATE POLICY tenant_isolation ON organization_users
    USING (organization_id = current_setting('app.current_organization_id', true)::UUID);

CREATE POLICY tenant_isolation ON fiscal_years
    USING (organization_id = current_setting('app.current_organization_id', true)::UUID);

CREATE POLICY tenant_isolation ON fiscal_periods
    USING (organization_id = current_setting('app.current_organization_id', true)::UUID);

CREATE POLICY tenant_isolation ON dimension_types
    USING (organization_id = current_setting('app.current_organization_id', true)::UUID);

CREATE POLICY tenant_isolation ON dimension_values
    USING (organization_id = current_setting('app.current_organization_id', true)::UUID);

CREATE POLICY tenant_isolation ON chart_of_accounts
    USING (organization_id = current_setting('app.current_organization_id', true)::UUID);

CREATE POLICY tenant_isolation ON transactions
    USING (organization_id = current_setting('app.current_organization_id', true)::UUID);

CREATE POLICY tenant_isolation ON budgets
    USING (organization_id = current_setting('app.current_organization_id', true)::UUID);

CREATE POLICY tenant_isolation ON attachments
    USING (organization_id = current_setting('app.current_organization_id', true)::UUID);

CREATE POLICY tenant_isolation ON exchange_rates
    USING (organization_id = current_setting('app.current_organization_id', true)::UUID);

CREATE POLICY tenant_isolation ON approval_rules
    USING (organization_id = current_setting('app.current_organization_id', true)::UUID);

CREATE POLICY tenant_isolation ON organization_usage
    USING (organization_id = current_setting('app.current_organization_id', true)::UUID);

-- Policies for tables that reference parent tables (need join-based isolation)
CREATE POLICY tenant_isolation ON ledger_entries
    USING (transaction_id IN (
        SELECT id FROM transactions 
        WHERE organization_id = current_setting('app.current_organization_id', true)::UUID
    ));

CREATE POLICY tenant_isolation ON budget_lines
    USING (budget_id IN (
        SELECT id FROM budgets 
        WHERE organization_id = current_setting('app.current_organization_id', true)::UUID
    ));

CREATE POLICY tenant_isolation ON budget_line_dimensions
    USING (budget_line_id IN (
        SELECT bl.id FROM budget_lines bl
        JOIN budgets b ON b.id = bl.budget_id
        WHERE b.organization_id = current_setting('app.current_organization_id', true)::UUID
    ));

CREATE POLICY tenant_isolation ON entry_dimensions
    USING (ledger_entry_id IN (
        SELECT le.id FROM ledger_entries le
        JOIN transactions t ON t.id = le.transaction_id
        WHERE t.organization_id = current_setting('app.current_organization_id', true)::UUID
    ));
";

const SEED_CURRENCIES_SQL: &str = r"
-- ============================================================
-- SEED: Common currencies
-- ============================================================
INSERT INTO currencies (code, name, symbol, decimal_places) VALUES
('USD', 'US Dollar', '$', 2),
('EUR', 'Euro', '€', 2),
('GBP', 'British Pound', '£', 2),
('JPY', 'Japanese Yen', '¥', 0),
('IDR', 'Indonesian Rupiah', 'Rp', 0),
('SGD', 'Singapore Dollar', 'S$', 2),
('AUD', 'Australian Dollar', 'A$', 2),
('CNY', 'Chinese Yuan', '¥', 2),
('MYR', 'Malaysian Ringgit', 'RM', 2),
('THB', 'Thai Baht', '฿', 2),
('PHP', 'Philippine Peso', '₱', 2),
('VND', 'Vietnamese Dong', '₫', 0),
('KRW', 'South Korean Won', '₩', 0),
('INR', 'Indian Rupee', '₹', 2),
('HKD', 'Hong Kong Dollar', 'HK$', 2),
('TWD', 'Taiwan Dollar', 'NT$', 2),
('CHF', 'Swiss Franc', 'CHF', 2),
('CAD', 'Canadian Dollar', 'C$', 2),
('NZD', 'New Zealand Dollar', 'NZ$', 2),
('SAR', 'Saudi Riyal', 'SAR', 2),
('AED', 'UAE Dirham', 'AED', 2),
('BRL', 'Brazilian Real', 'R$', 2),
('MXN', 'Mexican Peso', 'MX$', 2),
('ZAR', 'South African Rand', 'R', 2),
('RUB', 'Russian Ruble', '₽', 2),
('TRY', 'Turkish Lira', '₺', 2),
('PLN', 'Polish Zloty', 'zł', 2),
('SEK', 'Swedish Krona', 'kr', 2),
('NOK', 'Norwegian Krone', 'kr', 2),
('DKK', 'Danish Krone', 'kr', 2)
ON CONFLICT (code) DO NOTHING;
";

const SEED_TIER_LIMITS_SQL: &str = r#"
-- ============================================================
-- SEED: Subscription tier limits
-- Based on BUSINESS_MODEL.md pricing tiers
-- Note: Using large numbers (999999) to represent "unlimited" for NOT NULL columns
-- ============================================================
INSERT INTO tier_limits (
    tier, 
    max_users, max_transactions_per_month,
    max_dimensions, max_currencies, max_fiscal_periods, max_budgets, max_approval_rules,
    has_multi_currency, has_simulation, has_api_access, has_sso, has_custom_reports, 
    has_multi_entity, has_audit_export, has_priority_support,
    audit_log_retention_days, attachment_storage_gb,
    display_name, description
) VALUES 
(
    'starter',
    50, 1000,
    2, 1, 24, 3, 3,
    false, false, false, false, false, false, false, false,
    90, 5,
    'Starter', 'For small teams getting started with expense tracking'
),
(
    'growth',
    200, 10000,
    999999, 999999, NULL, NULL, NULL,
    true, false, true, false, true, false, true, false,
    365, 50,
    'Growth', 'For growing companies needing multi-currency and dimensional accounting'
),
(
    'enterprise',
    NULL, NULL,
    999999, 999999, NULL, NULL, NULL,
    true, true, true, true, true, true, true, true,
    2555, 500,
    'Enterprise', 'Full-featured solution with simulation, SSO, and dedicated support'
),
(
    'self_hosted',
    NULL, NULL,
    999999, 999999, NULL, NULL, NULL,
    true, true, true, true, true, true, true, true,
    3650, 10000,
    'Self-Hosted', 'On-premise deployment with unlimited features'
)
ON CONFLICT (tier) DO NOTHING;
"#;

const DROP_ALL_SQL: &str = r"
-- ============================================================
-- DROP ALL: Rollback migration
-- Order matters due to foreign key constraints
-- ============================================================

-- Drop views first
DROP VIEW IF EXISTS dimensional_report_view CASCADE;
DROP VIEW IF EXISTS budget_vs_actual_view CASCADE;
DROP VIEW IF EXISTS trial_balance_view CASCADE;
DROP VIEW IF EXISTS account_balances_view CASCADE;

-- Drop triggers
DROP TRIGGER IF EXISTS trg_increment_transaction_usage ON transactions;
DROP TRIGGER IF EXISTS trg_update_account_balance ON ledger_entries;
DROP TRIGGER IF EXISTS trg_prevent_posted_mod ON transactions;
DROP TRIGGER IF EXISTS trg_validate_fiscal_period ON transactions;
DROP TRIGGER IF EXISTS trg_check_balance ON ledger_entries;

-- Drop functions
DROP FUNCTION IF EXISTS has_feature(UUID, VARCHAR);
DROP FUNCTION IF EXISTS check_tier_limit(UUID, VARCHAR);
DROP FUNCTION IF EXISTS get_exchange_rate(UUID, CHAR, CHAR, DATE);
DROP FUNCTION IF EXISTS get_account_balance_at(UUID, TIMESTAMPTZ);
DROP FUNCTION IF EXISTS increment_transaction_usage();
DROP FUNCTION IF EXISTS update_account_balance();
DROP FUNCTION IF EXISTS prevent_posted_modification();
DROP FUNCTION IF EXISTS validate_fiscal_period_posting();
DROP FUNCTION IF EXISTS check_transaction_balance();

-- Drop tables (reverse order of creation)
DROP TABLE IF EXISTS organization_usage CASCADE;
DROP TABLE IF EXISTS tier_limits CASCADE;
DROP TABLE IF EXISTS approval_rules CASCADE;
DROP TABLE IF EXISTS attachments CASCADE;
DROP TABLE IF EXISTS budget_line_dimensions CASCADE;
DROP TABLE IF EXISTS budget_lines CASCADE;
DROP TABLE IF EXISTS budgets CASCADE;
DROP TABLE IF EXISTS entry_dimensions CASCADE;
DROP TABLE IF EXISTS ledger_entries CASCADE;
DROP TABLE IF EXISTS transactions CASCADE;
DROP TABLE IF EXISTS chart_of_accounts CASCADE;
DROP TABLE IF EXISTS dimension_values CASCADE;
DROP TABLE IF EXISTS dimension_types CASCADE;
DROP TABLE IF EXISTS fiscal_periods CASCADE;
DROP TABLE IF EXISTS fiscal_years CASCADE;
DROP TABLE IF EXISTS exchange_rates CASCADE;
DROP TABLE IF EXISTS currencies CASCADE;
DROP TABLE IF EXISTS organization_users CASCADE;
DROP TABLE IF EXISTS organizations CASCADE;
DROP TABLE IF EXISTS users CASCADE;

-- Drop enums
DROP TYPE IF EXISTS subscription_status CASCADE;
DROP TYPE IF EXISTS subscription_tier CASCADE;
DROP TYPE IF EXISTS storage_provider CASCADE;
DROP TYPE IF EXISTS attachment_type CASCADE;
DROP TYPE IF EXISTS budget_type CASCADE;
DROP TYPE IF EXISTS transaction_type CASCADE;
DROP TYPE IF EXISTS transaction_status CASCADE;
DROP TYPE IF EXISTS account_subtype CASCADE;
DROP TYPE IF EXISTS account_type CASCADE;
DROP TYPE IF EXISTS fiscal_period_status CASCADE;
DROP TYPE IF EXISTS fiscal_year_status CASCADE;
DROP TYPE IF EXISTS rate_source CASCADE;
DROP TYPE IF EXISTS user_role CASCADE;
";
