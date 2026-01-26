//! Entities Model Implementation Migration
//!
//! This migration implements the entities model refactoring:
//! 1. Creates entities table for multi-entity accounting
//! 2. Moves subscription fields from organizations to users
//! 3. Adds entity_id to all accounting data tables
//! 4. Migrates existing data (orgs -> entities, org subscriptions -> user subscriptions)
//! 5. Updates intercompany_mappings to use entity_id instead of org_id

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // ============================================================
        // STEP 1: Create entities table
        // ============================================================
        db.execute_unprepared(
            r"
            CREATE TABLE entities (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
                name VARCHAR(255) NOT NULL,
                legal_name VARCHAR(255),
                tax_id VARCHAR(100),
                entity_type VARCHAR(50) NOT NULL DEFAULT 'main',
                base_currency CHAR(3) NOT NULL,
                is_active BOOLEAN NOT NULL DEFAULT true,
                settings JSONB NOT NULL DEFAULT '{}',
                created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                CONSTRAINT unique_entity_name_per_org UNIQUE(organization_id, name),
                CONSTRAINT check_entity_type CHECK (entity_type IN ('main', 'subsidiary', 'branch', 'division'))
            );
            
            CREATE INDEX idx_entities_organization ON entities(organization_id);
            CREATE INDEX idx_entities_active ON entities(organization_id, is_active);
            ",
        )
        .await?;

        // ============================================================
        // STEP 2: Add subscription fields to users table
        // ============================================================
        db.execute_unprepared(
            r"
            ALTER TABLE users 
                ADD COLUMN subscription_tier subscription_tier NOT NULL DEFAULT 'starter',
                ADD COLUMN subscription_status subscription_status NOT NULL DEFAULT 'trialing',
                ADD COLUMN trial_ends_at TIMESTAMPTZ,
                ADD COLUMN subscription_ends_at TIMESTAMPTZ,
                ADD COLUMN payment_provider VARCHAR(50),
                ADD COLUMN payment_customer_id VARCHAR(255),
                ADD COLUMN payment_subscription_id VARCHAR(255);
            
            CREATE INDEX idx_users_subscription_status ON users(subscription_status);
            CREATE INDEX idx_users_payment_customer ON users(payment_provider, payment_customer_id) 
                WHERE payment_customer_id IS NOT NULL;
            CREATE INDEX idx_users_trial_ends ON users(trial_ends_at) 
                WHERE trial_ends_at IS NOT NULL;
            ",
        )
        .await?;

        // ============================================================
        // STEP 3: Add entity_id to existing tables
        // ============================================================

        // Add entity_id columns (nullable initially for data migration)
        db.execute_unprepared(
            r"
            ALTER TABLE chart_of_accounts ADD COLUMN entity_id UUID REFERENCES entities(id);
            ALTER TABLE transactions ADD COLUMN entity_id UUID REFERENCES entities(id);
            ALTER TABLE ledger_entries ADD COLUMN entity_id UUID REFERENCES entities(id);
            ALTER TABLE budgets ADD COLUMN entity_id UUID REFERENCES entities(id);
            ALTER TABLE fiscal_years ADD COLUMN entity_id UUID REFERENCES entities(id);
            ALTER TABLE accrual_schedules ADD COLUMN entity_id UUID REFERENCES entities(id);
            ALTER TABLE revaluation_logs ADD COLUMN entity_id UUID REFERENCES entities(id);
            ",
        )
        .await?;

        // ============================================================
        // STEP 4: Migrate subscription data from organizations to users
        // ============================================================

        // Copy subscription from user's first (oldest) organization to user account
        db.execute_unprepared(
            r"
            WITH first_org_per_user AS (
                SELECT DISTINCT ON (ou.user_id)
                    ou.user_id,
                    o.subscription_tier,
                    o.subscription_status,
                    o.trial_ends_at,
                    o.subscription_ends_at,
                    o.payment_provider,
                    o.payment_customer_id,
                    o.payment_subscription_id
                FROM organization_users ou
                JOIN organizations o ON o.id = ou.organization_id
                ORDER BY ou.user_id, ou.created_at ASC
            )
            UPDATE users u
            SET 
                subscription_tier = f.subscription_tier,
                subscription_status = f.subscription_status,
                trial_ends_at = f.trial_ends_at,
                subscription_ends_at = f.subscription_ends_at,
                payment_provider = f.payment_provider,
                payment_customer_id = f.payment_customer_id,
                payment_subscription_id = f.payment_subscription_id
            FROM first_org_per_user f
            WHERE u.id = f.user_id;
            ",
        )
        .await?;

        // ============================================================
        // STEP 5: Create default entity for each organization
        // ============================================================

        db.execute_unprepared(
            r"
            INSERT INTO entities (organization_id, name, legal_name, base_currency, entity_type, is_active, created_at, updated_at)
            SELECT 
                id,
                name || ' (Main)',
                name,
                base_currency,
                'main',
                true,
                created_at,
                updated_at
            FROM organizations;
            ",
        )
        .await?;

        // ============================================================
        // STEP 6: Link existing data to default entities
        // ============================================================

        // Link chart_of_accounts
        db.execute_unprepared(
            r"
            WITH org_default_entity AS (
                SELECT organization_id, id as entity_id
                FROM entities
                WHERE entity_type = 'main'
            )
            UPDATE chart_of_accounts coa
            SET entity_id = ode.entity_id
            FROM org_default_entity ode
            WHERE coa.organization_id = ode.organization_id;
            ",
        )
        .await?;

        // Link transactions
        db.execute_unprepared(
            r"
            WITH org_default_entity AS (
                SELECT organization_id, id as entity_id
                FROM entities
                WHERE entity_type = 'main'
            )
            UPDATE transactions t
            SET entity_id = ode.entity_id
            FROM org_default_entity ode
            WHERE t.organization_id = ode.organization_id;
            ",
        )
        .await?;

        // Link ledger_entries
        db.execute_unprepared(
            r"
            WITH org_default_entity AS (
                SELECT organization_id, id as entity_id
                FROM entities
                WHERE entity_type = 'main'
            )
            UPDATE ledger_entries le
            SET entity_id = ode.entity_id
            FROM org_default_entity ode
            JOIN transactions t ON t.id = le.transaction_id
            WHERE t.organization_id = ode.organization_id;
            ",
        )
        .await?;

        // Link budgets
        db.execute_unprepared(
            r"
            WITH org_default_entity AS (
                SELECT organization_id, id as entity_id
                FROM entities
                WHERE entity_type = 'main'
            )
            UPDATE budgets b
            SET entity_id = ode.entity_id
            FROM org_default_entity ode
            WHERE b.organization_id = ode.organization_id;
            ",
        )
        .await?;

        // Link fiscal_years
        db.execute_unprepared(
            r"
            WITH org_default_entity AS (
                SELECT organization_id, id as entity_id
                FROM entities
                WHERE entity_type = 'main'
            )
            UPDATE fiscal_years fy
            SET entity_id = ode.entity_id
            FROM org_default_entity ode
            WHERE fy.organization_id = ode.organization_id;
            ",
        )
        .await?;

        // Link accrual_schedules
        db.execute_unprepared(
            r"
            WITH org_default_entity AS (
                SELECT organization_id, id as entity_id
                FROM entities
                WHERE entity_type = 'main'
            )
            UPDATE accrual_schedules acs
            SET entity_id = ode.entity_id
            FROM org_default_entity ode
            WHERE acs.organization_id = ode.organization_id;
            ",
        )
        .await?;

        // Link revaluation_logs
        db.execute_unprepared(
            r"
            WITH org_default_entity AS (
                SELECT organization_id, id as entity_id
                FROM entities
                WHERE entity_type = 'main'
            )
            UPDATE revaluation_logs rl
            SET entity_id = ode.entity_id
            FROM org_default_entity ode
            WHERE rl.organization_id = ode.organization_id;
            ",
        )
        .await?;

        // ============================================================
        // STEP 7: Make entity_id NOT NULL after data migration
        // ============================================================

        db.execute_unprepared(
            r"
            ALTER TABLE chart_of_accounts ALTER COLUMN entity_id SET NOT NULL;
            ALTER TABLE transactions ALTER COLUMN entity_id SET NOT NULL;
            ALTER TABLE ledger_entries ALTER COLUMN entity_id SET NOT NULL;
            ALTER TABLE budgets ALTER COLUMN entity_id SET NOT NULL;
            ALTER TABLE fiscal_years ALTER COLUMN entity_id SET NOT NULL;
            ALTER TABLE accrual_schedules ALTER COLUMN entity_id SET NOT NULL;
            ALTER TABLE revaluation_logs ALTER COLUMN entity_id SET NOT NULL;
            ",
        )
        .await?;

        // ============================================================
        // STEP 8: Add indexes on entity_id for performance
        // ============================================================

        db.execute_unprepared(
            r"
            CREATE INDEX idx_accounts_entity ON chart_of_accounts(entity_id);
            CREATE INDEX idx_transactions_entity ON transactions(entity_id);
            CREATE INDEX idx_transactions_entity_date ON transactions(entity_id, transaction_date);
            CREATE INDEX idx_ledger_entries_entity ON ledger_entries(entity_id);
            CREATE INDEX idx_ledger_entries_entity_account ON ledger_entries(entity_id, account_id);
            CREATE INDEX idx_budgets_entity ON budgets(entity_id);
            CREATE INDEX idx_fiscal_years_entity ON fiscal_years(entity_id);
            CREATE INDEX idx_accrual_schedules_entity ON accrual_schedules(entity_id);
            CREATE INDEX idx_revaluation_logs_entity ON revaluation_logs(entity_id);
            ",
        )
        .await?;

        // ============================================================
        // STEP 9: Update intercompany_mappings table
        // ============================================================

        // Rename columns
        db.execute_unprepared(
            r"
            ALTER TABLE intercompany_mappings 
                RENAME COLUMN source_org_id TO source_entity_id;
            
            ALTER TABLE intercompany_mappings 
                RENAME COLUMN target_org_id TO target_entity_id;
            ",
        )
        .await?;

        // Drop old foreign key constraints and add new ones
        db.execute_unprepared(
            r"
            ALTER TABLE intercompany_mappings 
                DROP CONSTRAINT IF EXISTS intercompany_mappings_source_org_id_fkey,
                DROP CONSTRAINT IF EXISTS intercompany_mappings_target_org_id_fkey,
                ADD CONSTRAINT fk_source_entity 
                    FOREIGN KEY (source_entity_id) REFERENCES entities(id) ON DELETE CASCADE,
                ADD CONSTRAINT fk_target_entity 
                    FOREIGN KEY (target_entity_id) REFERENCES entities(id) ON DELETE CASCADE;
            ",
        )
        .await?;

        // Update unique constraint
        db.execute_unprepared(
            r"
            ALTER TABLE intercompany_mappings 
                DROP CONSTRAINT IF EXISTS intercompany_mappings_source_org_id_target_org_id_source_acc_key;
            
            ALTER TABLE intercompany_mappings 
                ADD CONSTRAINT unique_intercompany_mapping 
                    UNIQUE (source_entity_id, target_entity_id, source_account_id);
            ",
        )
        .await?;

        // Update indexes
        db.execute_unprepared(
            r"
            DROP INDEX IF EXISTS idx_intercompany_source;
            DROP INDEX IF EXISTS idx_intercompany_target;
            
            CREATE INDEX idx_intercompany_source ON intercompany_mappings(source_entity_id);
            CREATE INDEX idx_intercompany_target ON intercompany_mappings(target_entity_id);
            ",
        )
        .await?;

        // ============================================================
        // STEP 10: Add RLS policies for entities table
        // ============================================================

        db.execute_unprepared(
            r"
            ALTER TABLE entities ENABLE ROW LEVEL SECURITY;
            
            CREATE POLICY entities_isolation ON entities
                USING (organization_id = (current_setting('app.current_organization_id', true)::UUID));
            ",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Rollback in reverse order

        // Drop RLS policies
        db.execute_unprepared(
            r"
            DROP POLICY IF EXISTS entities_isolation ON entities;
            ALTER TABLE entities DISABLE ROW LEVEL SECURITY;
            ",
        )
        .await?;

        // Restore intercompany_mappings
        db.execute_unprepared(
            r"
            DROP INDEX IF EXISTS idx_intercompany_source;
            DROP INDEX IF EXISTS idx_intercompany_target;
            
            ALTER TABLE intercompany_mappings 
                DROP CONSTRAINT IF EXISTS unique_intercompany_mapping,
                DROP CONSTRAINT IF EXISTS fk_source_entity,
                DROP CONSTRAINT IF EXISTS fk_target_entity;
            
            ALTER TABLE intercompany_mappings 
                RENAME COLUMN source_entity_id TO source_org_id;
            
            ALTER TABLE intercompany_mappings 
                RENAME COLUMN target_entity_id TO target_org_id;
            
            ALTER TABLE intercompany_mappings 
                ADD CONSTRAINT intercompany_mappings_source_org_id_fkey 
                    FOREIGN KEY (source_org_id) REFERENCES organizations(id) ON DELETE CASCADE,
                ADD CONSTRAINT intercompany_mappings_target_org_id_fkey 
                    FOREIGN KEY (target_org_id) REFERENCES organizations(id) ON DELETE CASCADE;
            
            CREATE INDEX idx_intercompany_source ON intercompany_mappings(source_org_id);
            CREATE INDEX idx_intercompany_target ON intercompany_mappings(target_org_id);
            ",
        )
        .await?;

        // Drop entity_id indexes
        db.execute_unprepared(
            r"
            DROP INDEX IF EXISTS idx_accounts_entity;
            DROP INDEX IF EXISTS idx_transactions_entity;
            DROP INDEX IF EXISTS idx_transactions_entity_date;
            DROP INDEX IF EXISTS idx_ledger_entries_entity;
            DROP INDEX IF EXISTS idx_ledger_entries_entity_account;
            DROP INDEX IF EXISTS idx_budgets_entity;
            DROP INDEX IF EXISTS idx_fiscal_years_entity;
            DROP INDEX IF EXISTS idx_accrual_schedules_entity;
            DROP INDEX IF EXISTS idx_revaluation_logs_entity;
            ",
        )
        .await?;

        // Drop entity_id columns
        db.execute_unprepared(
            r"
            ALTER TABLE chart_of_accounts DROP COLUMN IF EXISTS entity_id;
            ALTER TABLE transactions DROP COLUMN IF EXISTS entity_id;
            ALTER TABLE ledger_entries DROP COLUMN IF EXISTS entity_id;
            ALTER TABLE budgets DROP COLUMN IF EXISTS entity_id;
            ALTER TABLE fiscal_years DROP COLUMN IF EXISTS entity_id;
            ALTER TABLE accrual_schedules DROP COLUMN IF EXISTS entity_id;
            ALTER TABLE revaluation_logs DROP COLUMN IF EXISTS entity_id;
            ",
        )
        .await?;

        // Drop subscription fields from users
        db.execute_unprepared(
            r"
            DROP INDEX IF EXISTS idx_users_subscription_status;
            DROP INDEX IF EXISTS idx_users_payment_customer;
            DROP INDEX IF EXISTS idx_users_trial_ends;
            
            ALTER TABLE users 
                DROP COLUMN IF EXISTS subscription_tier,
                DROP COLUMN IF EXISTS subscription_status,
                DROP COLUMN IF EXISTS trial_ends_at,
                DROP COLUMN IF EXISTS subscription_ends_at,
                DROP COLUMN IF EXISTS payment_provider,
                DROP COLUMN IF EXISTS payment_customer_id,
                DROP COLUMN IF EXISTS payment_subscription_id;
            ",
        )
        .await?;

        // Drop entities table
        db.execute_unprepared(
            r"
            DROP TABLE IF EXISTS entities CASCADE;
            ",
        )
        .await?;

        Ok(())
    }
}
