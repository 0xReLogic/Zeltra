use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Update dimension limits to match BUSINESS_MODEL.md

        // Starter: 2 (already set usually, but let's be safe)
        db.execute_unprepared(
            "UPDATE tier_limits SET max_dimensions = 2 WHERE tier = 'starter'::subscription_tier",
        )
        .await?;

        // Growth: 10
        db.execute_unprepared(
            "UPDATE tier_limits SET max_dimensions = 10 WHERE tier = 'growth'::subscription_tier",
        )
        .await?;

        // Enterprise: 100
        db.execute_unprepared(
            "UPDATE tier_limits SET max_dimensions = 100 WHERE tier = 'enterprise'::subscription_tier"
        ).await?;

        // Self-hosted: keep unlimited if desired, or 1000
        db.execute_unprepared(
            "UPDATE tier_limits SET max_dimensions = 1000 WHERE tier = 'self_hosted'::subscription_tier"
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Revert to "unlimited" (999999) for Growth/Enterprise
        db.execute_unprepared(
            "UPDATE tier_limits SET max_dimensions = 999999 WHERE tier IN ('growth', 'enterprise', 'self_hosted')::subscription_tier"
        ).await?;

        Ok(())
    }
}
