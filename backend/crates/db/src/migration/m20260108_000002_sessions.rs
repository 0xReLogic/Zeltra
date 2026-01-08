//! Sessions migration for refresh token management.
//!
//! Creates the sessions table for tracking active user sessions.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(SESSIONS_SQL).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("DROP TABLE IF EXISTS sessions CASCADE;")
            .await?;
        Ok(())
    }
}

const SESSIONS_SQL: &str = r"
-- Sessions table for refresh token management
CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    refresh_token_hash VARCHAR(64) NOT NULL,
    user_agent TEXT,
    ip_address VARCHAR(45),
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_expires_future CHECK (expires_at > created_at)
);

-- Index for token lookup (most common operation)
CREATE INDEX idx_sessions_token_hash ON sessions(refresh_token_hash) WHERE revoked_at IS NULL;

-- Index for user's active sessions
CREATE INDEX idx_sessions_user ON sessions(user_id, created_at DESC) WHERE revoked_at IS NULL;

-- Index for cleanup of expired sessions
CREATE INDEX idx_sessions_expires ON sessions(expires_at) WHERE revoked_at IS NULL;

-- Index for organization sessions (for admin view)
CREATE INDEX idx_sessions_org ON sessions(organization_id, created_at DESC);
";
