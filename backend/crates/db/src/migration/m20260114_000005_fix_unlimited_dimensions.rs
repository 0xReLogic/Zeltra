use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Fix: Restore "Unlimited" (999999) dimensions for Growth and Enterprise
        // to match BUSINESS_MODEL.md promise.

        // Growth: Unlimited
        db.execute_unprepared(
            "UPDATE tier_limits SET max_dimensions = 999999 WHERE tier = 'growth'::subscription_tier",
        )
        .await?;

        // Enterprise: Unlimited
        db.execute_unprepared(
            "UPDATE tier_limits SET max_dimensions = 999999 WHERE tier = 'enterprise'::subscription_tier"
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Revert to the restricted limits if needed (10 for Growth, 100 for Enterprise)
        db.execute_unprepared(
            "UPDATE tier_limits SET max_dimensions = 10 WHERE tier = 'growth'::subscription_tier",
        )
        .await?;

        db.execute_unprepared(
            "UPDATE tier_limits SET max_dimensions = 100 WHERE tier = 'enterprise'::subscription_tier"
        ).await?;

        Ok(())
    }
}
