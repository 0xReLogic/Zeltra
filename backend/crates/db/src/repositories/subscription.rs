//! Subscription and tier management repository.
//!
//! Handles tier limits, feature checks, and usage tracking for multi-tenant `SaaS`.

use chrono::{Datelike, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    Set,
};
use uuid::Uuid;

use crate::entities::{
    organization_usage, organization_users, organizations,
    sea_orm_active_enums::{SubscriptionStatus, SubscriptionTier},
    tier_limits,
};

/// Feature flags available in different tiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Feature {
    /// Multi-currency support.
    MultiCurrency,
    /// What-if simulation engine.
    Simulation,
    /// API access for integrations.
    ApiAccess,
    /// Single sign-on support.
    Sso,
    /// Custom report builder.
    CustomReports,
    /// Multi-entity/subsidiary support.
    MultiEntity,
    /// Audit log export.
    AuditExport,
    /// Priority support access.
    PrioritySupport,
}

/// Resource types that have limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceLimit {
    /// Maximum users per organization.
    Users,
    /// Maximum transactions per month.
    TransactionsPerMonth,
    /// Maximum dimension types.
    Dimensions,
    /// Maximum currencies.
    Currencies,
    /// Maximum fiscal periods.
    FiscalPeriods,
    /// Maximum budgets.
    Budgets,
    /// Maximum approval rules.
    ApprovalRules,
}

/// Result of a limit check.
#[derive(Debug, Clone)]
pub struct LimitCheckResult {
    /// Whether the operation is allowed.
    pub allowed: bool,
    /// Current usage count.
    pub current: i64,
    /// Maximum limit (None = unlimited).
    pub limit: Option<i64>,
    /// Human-readable message if limit exceeded.
    pub message: Option<String>,
}

/// Repository for subscription and tier operations.
pub struct SubscriptionRepository;

impl SubscriptionRepository {
    /// Get tier limits for a specific tier.
    pub async fn get_tier_limits(
        db: &DatabaseConnection,
        tier: SubscriptionTier,
    ) -> Result<Option<tier_limits::Model>, sea_orm::DbErr> {
        tier_limits::Entity::find_by_id(tier).one(db).await
    }

    /// Check if an organization has access to a specific feature.
    pub async fn has_feature(
        db: &DatabaseConnection,
        organization_id: Uuid,
        feature: Feature,
    ) -> Result<bool, sea_orm::DbErr> {
        // Get organization's tier
        let org = organizations::Entity::find_by_id(organization_id)
            .one(db)
            .await?;

        let Some(org) = org else {
            return Ok(false);
        };

        // Check subscription status - only active/trialing can use features
        if !matches!(
            org.subscription_status,
            SubscriptionStatus::Active | SubscriptionStatus::Trialing
        ) {
            return Ok(false);
        }

        // Get tier limits
        let limits = tier_limits::Entity::find_by_id(org.subscription_tier)
            .one(db)
            .await?;

        let Some(limits) = limits else {
            return Ok(false);
        };

        // Check feature flag
        let has_feature = match feature {
            Feature::MultiCurrency => limits.has_multi_currency,
            Feature::Simulation => limits.has_simulation,
            Feature::ApiAccess => limits.has_api_access,
            Feature::Sso => limits.has_sso,
            Feature::CustomReports => limits.has_custom_reports,
            Feature::MultiEntity => limits.has_multi_entity,
            Feature::AuditExport => limits.has_audit_export,
            Feature::PrioritySupport => limits.has_priority_support,
        };

        Ok(has_feature)
    }

    /// Check if an organization is within a specific resource limit.
    #[allow(clippy::cast_possible_wrap)]
    pub async fn check_limit(
        db: &DatabaseConnection,
        organization_id: Uuid,
        resource: ResourceLimit,
    ) -> Result<LimitCheckResult, sea_orm::DbErr> {
        // Get organization's tier
        let org = organizations::Entity::find_by_id(organization_id)
            .one(db)
            .await?;

        let Some(org) = org else {
            return Ok(LimitCheckResult {
                allowed: false,
                current: 0,
                limit: None,
                message: Some("Organization not found".to_string()),
            });
        };

        // Get tier limits
        let limits = tier_limits::Entity::find_by_id(org.subscription_tier.clone())
            .one(db)
            .await?;

        let Some(limits) = limits else {
            return Ok(LimitCheckResult {
                allowed: false,
                current: 0,
                limit: None,
                message: Some("Tier limits not configured".to_string()),
            });
        };

        // Get current usage
        let (current, max_limit) = match resource {
            ResourceLimit::Users => {
                let count = organization_users::Entity::find()
                    .filter(organization_users::Column::OrganizationId.eq(organization_id))
                    .count(db)
                    .await? as i64;
                (count, limits.max_users.map(i64::from))
            }
            ResourceLimit::TransactionsPerMonth => {
                let usage = Self::get_or_create_current_usage(db, organization_id).await?;
                (
                    i64::from(usage.transaction_count),
                    limits.max_transactions_per_month.map(i64::from),
                )
            }
            ResourceLimit::Dimensions => {
                let usage = Self::get_or_create_current_usage(db, organization_id).await?;
                (
                    i64::from(usage.active_dimension_count),
                    Some(i64::from(limits.max_dimensions)),
                )
            }
            ResourceLimit::Currencies => {
                let usage = Self::get_or_create_current_usage(db, organization_id).await?;
                (
                    i64::from(usage.active_currency_count),
                    Some(i64::from(limits.max_currencies)),
                )
            }
            ResourceLimit::FiscalPeriods => {
                // For fiscal periods, we'd need to count from fiscal_periods table
                // For now, use a placeholder - will be implemented when needed
                (0, limits.max_fiscal_periods.map(i64::from))
            }
            ResourceLimit::Budgets => {
                // Would count from budgets table - will be implemented when needed
                (0, limits.max_budgets.map(i64::from))
            }
            ResourceLimit::ApprovalRules => {
                // Would count from approval_rules table - will be implemented when needed
                (0, limits.max_approval_rules.map(i64::from))
            }
        };

        // None means unlimited
        let allowed = max_limit.is_none_or(|max| current < max);

        let message = if allowed {
            None
        } else {
            Some(format!(
                "Limit reached: {current}/{} for {resource:?}",
                max_limit.unwrap_or(0),
            ))
        };

        Ok(LimitCheckResult {
            allowed,
            current,
            limit: max_limit,
            message,
        })
    }

    /// Get or create usage record for current month.
    pub async fn get_or_create_current_usage(
        db: &DatabaseConnection,
        organization_id: Uuid,
    ) -> Result<organization_usage::Model, sea_orm::DbErr> {
        let now = Utc::now();
        let year_month = format!("{:04}-{:02}", now.year(), now.month());

        // Try to find existing
        let existing = organization_usage::Entity::find()
            .filter(organization_usage::Column::OrganizationId.eq(organization_id))
            .filter(organization_usage::Column::YearMonth.eq(&year_month))
            .one(db)
            .await?;

        if let Some(usage) = existing {
            return Ok(usage);
        }

        // Create new usage record
        let usage = organization_usage::ActiveModel {
            id: Set(Uuid::new_v4()),
            organization_id: Set(organization_id),
            year_month: Set(year_month),
            transaction_count: Set(0),
            api_call_count: Set(0),
            storage_used_bytes: Set(0),
            active_user_count: Set(0),
            active_dimension_count: Set(0),
            active_currency_count: Set(0),
            ..Default::default()
        };

        usage.insert(db).await
    }

    /// Increment transaction count for current month.
    pub async fn increment_transaction_count(
        db: &DatabaseConnection,
        organization_id: Uuid,
    ) -> Result<(), sea_orm::DbErr> {
        let usage = Self::get_or_create_current_usage(db, organization_id).await?;

        let mut active: organization_usage::ActiveModel = usage.into();
        active.transaction_count = Set(active.transaction_count.unwrap().saturating_add(1));
        active.update(db).await?;

        Ok(())
    }

    /// Increment API call count for current month.
    pub async fn increment_api_call_count(
        db: &DatabaseConnection,
        organization_id: Uuid,
    ) -> Result<(), sea_orm::DbErr> {
        let usage = Self::get_or_create_current_usage(db, organization_id).await?;

        let mut active: organization_usage::ActiveModel = usage.into();
        active.api_call_count = Set(active.api_call_count.unwrap().saturating_add(1));
        active.update(db).await?;

        Ok(())
    }

    /// Update active counts (users, dimensions, currencies).
    pub async fn update_active_counts(
        db: &DatabaseConnection,
        organization_id: Uuid,
        user_count: i32,
        dimension_count: i32,
        currency_count: i32,
    ) -> Result<(), sea_orm::DbErr> {
        let usage = Self::get_or_create_current_usage(db, organization_id).await?;

        let mut active: organization_usage::ActiveModel = usage.into();
        active.active_user_count = Set(user_count);
        active.active_dimension_count = Set(dimension_count);
        active.active_currency_count = Set(currency_count);
        active.update(db).await?;

        Ok(())
    }

    /// Check if organization's trial has expired.
    pub async fn is_trial_expired(
        db: &DatabaseConnection,
        organization_id: Uuid,
    ) -> Result<bool, sea_orm::DbErr> {
        let org = organizations::Entity::find_by_id(organization_id)
            .one(db)
            .await?;

        let Some(org) = org else {
            return Ok(true);
        };

        if org.subscription_status != SubscriptionStatus::Trialing {
            return Ok(false);
        }

        let Some(trial_ends_at) = org.trial_ends_at else {
            return Ok(true); // No trial end date means expired
        };

        Ok(Utc::now() > trial_ends_at)
    }

    /// Update organization subscription status.
    pub async fn update_subscription_status(
        db: &DatabaseConnection,
        organization_id: Uuid,
        status: SubscriptionStatus,
    ) -> Result<(), sea_orm::DbErr> {
        let org = organizations::Entity::find_by_id(organization_id)
            .one(db)
            .await?;

        let Some(org) = org else {
            return Ok(());
        };

        let mut active: organizations::ActiveModel = org.into();
        active.subscription_status = Set(status);
        active.update(db).await?;

        Ok(())
    }

    /// Upgrade organization to a new tier.
    pub async fn upgrade_tier(
        db: &DatabaseConnection,
        organization_id: Uuid,
        new_tier: SubscriptionTier,
    ) -> Result<(), sea_orm::DbErr> {
        let org = organizations::Entity::find_by_id(organization_id)
            .one(db)
            .await?;

        let Some(org) = org else {
            return Ok(());
        };

        let mut active: organizations::ActiveModel = org.into();
        active.subscription_tier = Set(new_tier);
        active.subscription_status = Set(SubscriptionStatus::Active);
        active.update(db).await?;

        Ok(())
    }
}
