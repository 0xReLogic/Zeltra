use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // 1. Add new feature flags to tier_limits
        db.execute_unprepared(
            "ALTER TABLE tier_limits ADD COLUMN has_auto_accruals BOOLEAN NOT NULL DEFAULT false",
        )
        .await?;

        db.execute_unprepared(
            "ALTER TABLE tier_limits ADD COLUMN has_intercompany_hub BOOLEAN NOT NULL DEFAULT false"
        ).await?;

        // 2. Update existing tiers consistent with BUSINESS_MODEL.md

        // Starter: already false by default

        // Growth: typically doesn't have these automated features yet,
        // but let's be explicit based on our plan.

        // Enterprise: has everything
        db.execute_unprepared(
            "UPDATE tier_limits SET has_auto_accruals = true, has_intercompany_hub = true WHERE tier = 'enterprise'::subscription_tier"
        ).await?;

        // Self-hosted: has everything
        db.execute_unprepared(
            "UPDATE tier_limits SET has_auto_accruals = true, has_intercompany_hub = true WHERE tier = 'self_hosted'::subscription_tier"
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared("ALTER TABLE tier_limits DROP COLUMN IF EXISTS has_auto_accruals")
            .await?;

        db.execute_unprepared("ALTER TABLE tier_limits DROP COLUMN IF EXISTS has_intercompany_hub")
            .await?;

        Ok(())
    }
}
