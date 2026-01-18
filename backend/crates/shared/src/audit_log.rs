//! Audit logging for tracking system changes.
//!
//! Provides structured logging for create, update, and delete operations
//! with actor information and change tracking.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;
use uuid::Uuid;

/// Audit log entry for tracking system changes.
#[derive(Debug, Serialize, Deserialize)]
pub struct AuditLogEntry {
    /// ID of the user performing the action.
    pub actor_id: Uuid,
    /// Organization ID where the action occurred.
    pub org_id: Uuid,
    /// ID of the resource being modified.
    pub resource_id: Uuid,
    /// Type of resource (e.g., "approval_rule").
    pub resource_type: String,
    /// Action performed (create, update, delete).
    pub action: String,
    /// Changes made (for updates) or full data (for creates).
    pub changes: Option<Value>,
    /// Timestamp of the action.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Audit logger for structured logging of system changes.
pub struct AuditLogger;

impl AuditLogger {
    /// Log a create operation.
    pub fn log_create(
        actor_id: Uuid,
        org_id: Uuid,
        resource_id: Uuid,
        resource_type: &str,
        data: Value,
    ) {
        let entry = AuditLogEntry {
            actor_id,
            org_id,
            resource_id,
            resource_type: resource_type.to_string(),
            action: "create".to_string(),
            changes: Some(data),
            timestamp: chrono::Utc::now(),
        };

        info!(
            target: "audit",
            actor_id = %entry.actor_id,
            org_id = %entry.org_id,
            resource_id = %entry.resource_id,
            resource_type = %entry.resource_type,
            action = %entry.action,
            changes = %serde_json::to_string(&entry.changes).unwrap_or_default(),
            timestamp = %entry.timestamp.to_rfc3339(),
            "{}",
            serde_json::to_string(&entry).unwrap_or_default()
        );
    }

    /// Log an update operation.
    pub fn log_update(
        actor_id: Uuid,
        org_id: Uuid,
        resource_id: Uuid,
        resource_type: &str,
        changes: Value,
    ) {
        let entry = AuditLogEntry {
            actor_id,
            org_id,
            resource_id,
            resource_type: resource_type.to_string(),
            action: "update".to_string(),
            changes: Some(changes),
            timestamp: chrono::Utc::now(),
        };

        info!(
            target: "audit",
            actor_id = %entry.actor_id,
            org_id = %entry.org_id,
            resource_id = %entry.resource_id,
            resource_type = %entry.resource_type,
            action = %entry.action,
            changes = %serde_json::to_string(&entry.changes).unwrap_or_default(),
            timestamp = %entry.timestamp.to_rfc3339(),
            "{}",
            serde_json::to_string(&entry).unwrap_or_default()
        );
    }

    /// Log a delete operation.
    pub fn log_delete(
        actor_id: Uuid,
        org_id: Uuid,
        resource_id: Uuid,
        resource_type: &str,
    ) {
        let entry = AuditLogEntry {
            actor_id,
            org_id,
            resource_id,
            resource_type: resource_type.to_string(),
            action: "delete".to_string(),
            changes: None,
            timestamp: chrono::Utc::now(),
        };

        info!(
            target: "audit",
            actor_id = %entry.actor_id,
            org_id = %entry.org_id,
            resource_id = %entry.resource_id,
            resource_type = %entry.resource_type,
            action = %entry.action,
            timestamp = %entry.timestamp.to_rfc3339(),
            "{}",
            serde_json::to_string(&entry).unwrap_or_default()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_audit_log_entry_serialization() {
        let entry = AuditLogEntry {
            actor_id: Uuid::new_v4(),
            org_id: Uuid::new_v4(),
            resource_id: Uuid::new_v4(),
            resource_type: "approval_rule".to_string(),
            action: "create".to_string(),
            changes: Some(json!({"name": "Test Rule"})),
            timestamp: chrono::Utc::now(),
        };

        let serialized = serde_json::to_string(&entry).unwrap();
        assert!(serialized.contains("approval_rule"));
        assert!(serialized.contains("create"));
        assert!(serialized.contains("Test Rule"));
    }

    #[test]
    fn test_audit_logger_methods() {
        let actor_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let resource_id = Uuid::new_v4();

        // Test create logging
        AuditLogger::log_create(
            actor_id,
            org_id,
            resource_id,
            "approval_rule",
            json!({"name": "Test Rule"}),
        );

        // Test update logging
        AuditLogger::log_update(
            actor_id,
            org_id,
            resource_id,
            "approval_rule",
            json!({"name": {"old": "Old Name", "new": "New Name"}}),
        );

        // Test delete logging
        AuditLogger::log_delete(actor_id, org_id, resource_id, "approval_rule");
    }
}