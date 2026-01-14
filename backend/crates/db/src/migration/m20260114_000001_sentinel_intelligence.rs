use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // 1. Add 'accrual' and 'revaluation' to transaction_type enum
        // Note: PostgreSQL doesn't allow adding values to enums within transactions easily.
        // However, SeaORM migration usually wraps in transaction.
        // We use ALTER TYPE ... ADD VALUE which works if outside transaction OR in PG 12+.
        db.execute_unprepared("ALTER TYPE transaction_type ADD VALUE 'accrual'")
            .await
            .ok();
        db.execute_unprepared("ALTER TYPE transaction_type ADD VALUE 'revaluation'")
            .await
            .ok();
        db.execute_unprepared("ALTER TYPE transaction_type ADD VALUE 'intercompany'")
            .await
            .ok();

        // 2. Create accrual_schedules table
        db.execute_unprepared(ACCRUAL_SCHEDULES_SQL).await?;

        // 3. Create revaluation_logs table
        db.execute_unprepared(REVALUATION_LOGS_SQL).await?;

        // 4. Create intercompany_mappings table
        db.execute_unprepared(INTERCOMPANY_MAPPINGS_SQL).await?;

        // 5. Add compliance_metadata to ledger_entries
        db.execute_unprepared("ALTER TABLE ledger_entries ADD COLUMN compliance_metadata JSONB")
            .await?;

        // 6. RLS Policies
        db.execute_unprepared(RLS_SQL).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared("DROP TABLE IF EXISTS intercompany_mappings CASCADE")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS revaluation_logs CASCADE")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS accrual_schedules CASCADE")
            .await?;
        db.execute_unprepared(
            "ALTER TABLE ledger_entries DROP COLUMN IF EXISTS compliance_metadata",
        )
        .await?;

        // Note: Dropping enum values is not supported in Postgres.
        // We would have to recreate the type if we really wanted to.

        Ok(())
    }
}

const ACCRUAL_SCHEDULES_SQL: &str = r"
CREATE TABLE accrual_schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    total_amount NUMERIC(19, 4) NOT NULL,
    currency_id CHAR(3) NOT NULL REFERENCES currencies(code),
    debit_account_id UUID NOT NULL REFERENCES chart_of_accounts(id),
    credit_account_id UUID NOT NULL REFERENCES chart_of_accounts(id),
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    frequency VARCHAR(20) NOT NULL,
    total_periods INTEGER NOT NULL,
    periods_processed INTEGER NOT NULL DEFAULT 0,
    next_run_date DATE,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    last_transaction_id UUID REFERENCES transactions(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_accrual_dates CHECK (end_date >= start_date),
    CONSTRAINT chk_total_amount_positive CHECK (total_amount > 0)
);

CREATE INDEX idx_accrual_org_status ON accrual_schedules(organization_id, status);
CREATE INDEX idx_accrual_next_run ON accrual_schedules(next_run_date) WHERE status = 'active';
";

const REVALUATION_LOGS_SQL: &str = r"
CREATE TABLE revaluation_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    account_id UUID NOT NULL REFERENCES chart_of_accounts(id),
    revaluation_date DATE NOT NULL,
    currency_id CHAR(3) NOT NULL REFERENCES currencies(code),
    balance_in_currency NUMERIC(19, 4) NOT NULL,
    old_exchange_rate NUMERIC(19, 10) NOT NULL,
    new_exchange_rate NUMERIC(19, 10) NOT NULL,
    unrealized_gain_loss NUMERIC(19, 4) NOT NULL,
    transaction_id UUID REFERENCES transactions(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_reval_org_date ON revaluation_logs(organization_id, revaluation_date);
CREATE INDEX idx_reval_account ON revaluation_logs(account_id);
";

const INTERCOMPANY_MAPPINGS_SQL: &str = r"
CREATE TABLE intercompany_mappings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    target_org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    source_account_id UUID NOT NULL REFERENCES chart_of_accounts(id),
    target_account_id UUID NOT NULL REFERENCES chart_of_accounts(id),
    mapping_type VARCHAR(20) NOT NULL DEFAULT 'elimination',
    auto_post BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (source_org_id, target_org_id, source_account_id)
);

CREATE INDEX idx_intercompany_source ON intercompany_mappings(source_org_id);
CREATE INDEX idx_intercompany_target ON intercompany_mappings(target_org_id);
";

const RLS_SQL: &str = r"
-- 1. Accrual Schedules
ALTER TABLE accrual_schedules ENABLE ROW LEVEL SECURITY;
CREATE POLICY accrual_schedules_isolation ON accrual_schedules
    USING (organization_id = (current_setting('app.current_organization_id', true)::UUID));

-- 2. Revaluation Logs
ALTER TABLE revaluation_logs ENABLE ROW LEVEL SECURITY;
CREATE POLICY revaluation_logs_isolation ON revaluation_logs
    USING (organization_id = (current_setting('app.current_organization_id', true)::UUID));

-- 3. Intercompany Mappings
ALTER TABLE intercompany_mappings ENABLE ROW LEVEL SECURITY;
CREATE POLICY intercompany_mappings_isolation ON intercompany_mappings
    USING (
        source_org_id = (current_setting('app.current_organization_id', true)::UUID)
        OR 
        target_org_id = (current_setting('app.current_organization_id', true)::UUID)
    );
";
