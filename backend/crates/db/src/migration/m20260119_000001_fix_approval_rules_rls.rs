use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop the existing policy
        manager
            .get_connection()
            .execute_unprepared("DROP POLICY IF EXISTS tenant_isolation ON approval_rules;")
            .await?;

        // Recreate the policy with WITH CHECK clause for UPDATE operations
        manager
            .get_connection()
            .execute_unprepared(
                r#"
CREATE POLICY tenant_isolation ON approval_rules
    FOR ALL
    USING (organization_id = current_setting('app.current_organization_id', true)::UUID)
    WITH CHECK (organization_id = current_setting('app.current_organization_id', true)::UUID);
                "#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Revert to the old policy without WITH CHECK
        manager
            .get_connection()
            .execute_unprepared("DROP POLICY IF EXISTS tenant_isolation ON approval_rules;")
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"
CREATE POLICY tenant_isolation ON approval_rules
    USING (organization_id = current_setting('app.current_organization_id', true)::UUID);
                "#,
            )
            .await?;

        Ok(())
    }
}
