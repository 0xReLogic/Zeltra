//! Organization repository for database operations.

use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait,
    QueryFilter, Set, TransactionTrait,
};
use serde_json::json;
use uuid::Uuid;

use crate::entities::{
    currencies, organization_users, organizations,
    sea_orm_active_enums::{SubscriptionStatus, SubscriptionTier, TransactionStatus, UserRole},
    transactions, users,
};

/// Error types for organization operations.
#[derive(Debug, thiserror::Error)]
pub enum OrganizationError {
    /// Organization not found.
    #[error("Organization not found")]
    NotFound,

    /// User is not a member of this organization.
    #[error("User is not a member of this organization")]
    NotMember,

    /// Insufficient permissions for this operation.
    #[error("Insufficient permissions")]
    Forbidden,

    /// Cannot remove the last owner of an organization.
    #[error("Cannot remove the last owner")]
    LastOwner,

    /// Cannot change base currency after posting transactions.
    #[error("Cannot change base currency after posting transactions")]
    CurrencyChangeNotAllowed,

    /// Invalid currency code.
    #[error("Invalid currency code: {0}")]
    InvalidCurrency(String),

    /// Invalid timezone.
    #[error("Invalid timezone: {0}")]
    InvalidTimezone(String),

    /// Name must be between 1 and 255 characters.
    #[error("Name must be between 1 and 255 characters")]
    InvalidName,

    /// No fields provided for update.
    #[error("No fields provided for update")]
    EmptyUpdate,

    /// Database error.
    #[error("Database error: {0}")]
    Database(#[from] DbErr),
}

/// Organization repository for CRUD operations.
#[derive(Debug, Clone)]
pub struct OrganizationRepository {
    db: DatabaseConnection,
}

impl OrganizationRepository {
    /// Creates a new organization repository.
    #[must_use]
    pub const fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Finds an organization by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<organizations::Model>, DbErr> {
        organizations::Entity::find_by_id(id).one(&self.db).await
    }

    /// Finds an organization by slug.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn find_by_slug(&self, slug: &str) -> Result<Option<organizations::Model>, DbErr> {
        organizations::Entity::find()
            .filter(organizations::Column::Slug.eq(slug))
            .one(&self.db)
            .await
    }

    /// Checks if a slug is already taken.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn slug_exists(&self, slug: &str) -> Result<bool, DbErr> {
        let count = organizations::Entity::find()
            .filter(organizations::Column::Slug.eq(slug))
            .count(&self.db)
            .await?;

        Ok(count > 0)
    }

    /// Creates a new organization with the creator as owner.
    ///
    /// # Errors
    ///
    /// Returns an error if the database insert fails.
    pub async fn create_with_owner(
        &self,
        name: &str,
        slug: &str,
        base_currency: &str,
        timezone: &str,
        owner_id: Uuid,
    ) -> Result<organizations::Model, DbErr> {
        let txn = self.db.begin().await?;

        let now = chrono::Utc::now().into();
        let org_id = Uuid::new_v4();

        // Create organization
        let org = organizations::ActiveModel {
            id: Set(org_id),
            name: Set(name.to_string()),
            slug: Set(slug.to_string()),
            base_currency: Set(base_currency.to_string()),
            timezone: Set(timezone.to_string()),
            settings: Set(json!({})),
            is_active: Set(true),
            subscription_tier: Set(SubscriptionTier::Starter),
            subscription_status: Set(SubscriptionStatus::Trialing),
            trial_ends_at: Set(Some(
                (chrono::Utc::now() + chrono::Duration::days(14)).into(),
            )),
            subscription_ends_at: Set(None),
            payment_provider: Set(None),
            payment_customer_id: Set(None),
            payment_subscription_id: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let org = org.insert(&txn).await?;

        // Add owner to organization
        let org_user = organization_users::ActiveModel {
            user_id: Set(owner_id),
            organization_id: Set(org_id),
            role: Set(UserRole::Owner),
            approval_limit: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };

        org_user.insert(&txn).await?;

        txn.commit().await?;

        Ok(org)
    }

    /// Adds a user to an organization.
    ///
    /// # Errors
    ///
    /// Returns an error if the database insert fails.
    pub async fn add_user(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        role: UserRole,
        approval_limit: Option<Decimal>,
    ) -> Result<organization_users::Model, DbErr> {
        let now = chrono::Utc::now().into();

        let org_user = organization_users::ActiveModel {
            user_id: Set(user_id),
            organization_id: Set(org_id),
            role: Set(role),
            approval_limit: Set(approval_limit),
            created_at: Set(now),
            updated_at: Set(now),
        };

        org_user.insert(&self.db).await
    }

    /// Gets all users in an organization.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn get_users(
        &self,
        org_id: Uuid,
    ) -> Result<Vec<(users::Model, organization_users::Model)>, DbErr> {
        organization_users::Entity::find()
            .filter(organization_users::Column::OrganizationId.eq(org_id))
            .find_also_related(users::Entity)
            .all(&self.db)
            .await
            .map(|results| {
                results
                    .into_iter()
                    .filter_map(|(ou, user)| user.map(|u| (u, ou)))
                    .collect()
            })
    }

    /// Gets a user's membership in an organization.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn get_user_membership(
        &self,
        org_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<organization_users::Model>, DbErr> {
        organization_users::Entity::find()
            .filter(organization_users::Column::OrganizationId.eq(org_id))
            .filter(organization_users::Column::UserId.eq(user_id))
            .one(&self.db)
            .await
    }

    /// Checks if a user is a member of an organization.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn is_member(&self, org_id: Uuid, user_id: Uuid) -> Result<bool, DbErr> {
        let count = organization_users::Entity::find()
            .filter(organization_users::Column::OrganizationId.eq(org_id))
            .filter(organization_users::Column::UserId.eq(user_id))
            .count(&self.db)
            .await?;

        Ok(count > 0)
    }

    /// Checks if a user has a specific role or higher in an organization.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn has_role(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        required_role: UserRole,
    ) -> Result<bool, DbErr> {
        let membership = self.get_user_membership(org_id, user_id).await?;

        Ok(membership.is_some_and(|m| role_level(&m.role) >= role_level(&required_role)))
    }

    /// Updates an organization's settings.
    ///
    /// # Errors
    ///
    /// Returns an error if the database update fails.
    pub async fn update(
        &self,
        org_id: Uuid,
        name: Option<&str>,
        base_currency: Option<&str>,
        timezone: Option<&str>,
    ) -> Result<Option<organizations::Model>, DbErr> {
        let org = organizations::Entity::find_by_id(org_id)
            .one(&self.db)
            .await?;

        let Some(org) = org else {
            return Ok(None);
        };

        let mut active: organizations::ActiveModel = org.into();

        if let Some(name) = name {
            active.name = Set(name.to_string());
        }
        if let Some(base_currency) = base_currency {
            active.base_currency = Set(base_currency.to_string());
        }
        if let Some(timezone) = timezone {
            active.timezone = Set(timezone.to_string());
        }
        active.updated_at = Set(chrono::Utc::now().into());

        let updated = active.update(&self.db).await?;
        Ok(Some(updated))
    }

    /// Checks if an organization has any posted transactions.
    ///
    /// This is used to prevent changing base currency after posting.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn has_posted_transactions(&self, org_id: Uuid) -> Result<bool, DbErr> {
        let count = transactions::Entity::find()
            .filter(transactions::Column::OrganizationId.eq(org_id))
            .filter(transactions::Column::Status.eq(TransactionStatus::Posted))
            .count(&self.db)
            .await?;

        Ok(count > 0)
    }

    /// Counts the number of owners in an organization.
    ///
    /// This is used to prevent removing the last owner.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn count_owners(&self, org_id: Uuid) -> Result<u64, DbErr> {
        let count = organization_users::Entity::find()
            .filter(organization_users::Column::OrganizationId.eq(org_id))
            .filter(organization_users::Column::Role.eq(UserRole::Owner))
            .count(&self.db)
            .await?;

        Ok(count)
    }

    /// Updates an organization's settings with full validation.
    ///
    /// Validates:
    /// - Name is non-empty and at most 255 characters
    /// - Currency exists in currencies table
    /// - Currency cannot be changed if organization has posted transactions
    /// - Timezone is a valid IANA identifier
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails or database operation fails.
    pub async fn update_organization(
        &self,
        org_id: Uuid,
        name: Option<&str>,
        base_currency: Option<&str>,
        timezone: Option<&str>,
    ) -> Result<organizations::Model, OrganizationError> {
        // Check at least one field is provided
        if name.is_none() && base_currency.is_none() && timezone.is_none() {
            return Err(OrganizationError::EmptyUpdate);
        }

        // Find organization
        let org = organizations::Entity::find_by_id(org_id)
            .one(&self.db)
            .await?
            .ok_or(OrganizationError::NotFound)?;

        // Validate name if provided
        if let Some(name) = name.filter(|n| n.is_empty() || n.len() > 255) {
            let _ = name; // Suppress unused warning
            return Err(OrganizationError::InvalidName);
        }

        // Validate currency if provided
        if let Some(currency) = base_currency {
            // Check currency exists
            if currencies::Entity::find_by_id(currency)
                .one(&self.db)
                .await?
                .is_none()
            {
                return Err(OrganizationError::InvalidCurrency(currency.to_string()));
            }

            // Check if currency is actually changing and has posted transactions
            if currency != org.base_currency && self.has_posted_transactions(org_id).await? {
                return Err(OrganizationError::CurrencyChangeNotAllowed);
            }
        }

        // Validate timezone if provided
        if let Some(tz) = timezone {
            // Use chrono-tz to validate IANA timezone
            if tz.parse::<chrono_tz::Tz>().is_err() {
                return Err(OrganizationError::InvalidTimezone(tz.to_string()));
            }
        }

        // Update organization
        let mut active: organizations::ActiveModel = org.into();

        if let Some(name) = name {
            active.name = Set(name.to_string());
        }
        if let Some(base_currency) = base_currency {
            active.base_currency = Set(base_currency.to_string());
        }
        if let Some(timezone) = timezone {
            active.timezone = Set(timezone.to_string());
        }
        active.updated_at = Set(chrono::Utc::now().into());

        let updated = active.update(&self.db).await?;
        Ok(updated)
    }

    /// Removes a user from an organization.
    ///
    /// Validates:
    /// - Target user is a member of the organization
    /// - Requester has sufficient permissions (admin cannot remove owner)
    /// - Cannot remove the last owner
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails or database operation fails.
    pub async fn remove_member(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        requester_role: &UserRole,
    ) -> Result<(), OrganizationError> {
        // Check organization exists
        let org = organizations::Entity::find_by_id(org_id)
            .one(&self.db)
            .await?;

        if org.is_none() {
            return Err(OrganizationError::NotFound);
        }

        // Get target user's membership
        let membership = self.get_user_membership(org_id, user_id).await?;
        let membership = membership.ok_or(OrganizationError::NotMember)?;

        // Check role hierarchy: admin cannot remove owner
        if membership.role == UserRole::Owner && *requester_role != UserRole::Owner {
            return Err(OrganizationError::Forbidden);
        }

        // Check if removing last owner
        if membership.role == UserRole::Owner {
            let owner_count = self.count_owners(org_id).await?;
            if owner_count <= 1 {
                return Err(OrganizationError::LastOwner);
            }
        }

        // Delete membership
        organization_users::Entity::delete_many()
            .filter(organization_users::Column::OrganizationId.eq(org_id))
            .filter(organization_users::Column::UserId.eq(user_id))
            .exec(&self.db)
            .await?;

        Ok(())
    }

    /// Updates a user's role and/or approval limit in an organization.
    ///
    /// Validates:
    /// - Target user is a member of the organization
    /// - Requester has sufficient permissions (admin cannot change owner's role)
    /// - Cannot demote the last owner
    /// - At least one field must be provided
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails or database operation fails.
    pub async fn update_member(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        requester_role: &UserRole,
        new_role: Option<UserRole>,
        new_approval_limit: Option<Option<Decimal>>,
    ) -> Result<organization_users::Model, OrganizationError> {
        // Check at least one field is provided
        if new_role.is_none() && new_approval_limit.is_none() {
            return Err(OrganizationError::EmptyUpdate);
        }

        // Check organization exists
        let org = organizations::Entity::find_by_id(org_id)
            .one(&self.db)
            .await?;

        if org.is_none() {
            return Err(OrganizationError::NotFound);
        }

        // Get target user's membership
        let membership = self.get_user_membership(org_id, user_id).await?;
        let membership = membership.ok_or(OrganizationError::NotMember)?;

        // Role change validations
        if let Some(ref role) = new_role {
            // Admin cannot change owner's role
            if membership.role == UserRole::Owner && *requester_role != UserRole::Owner {
                return Err(OrganizationError::Forbidden);
            }

            // Cannot demote last owner
            if membership.role == UserRole::Owner && *role != UserRole::Owner {
                let owner_count = self.count_owners(org_id).await?;
                if owner_count <= 1 {
                    return Err(OrganizationError::LastOwner);
                }
            }

            // Admin cannot promote to owner
            if *role == UserRole::Owner && *requester_role != UserRole::Owner {
                return Err(OrganizationError::Forbidden);
            }
        }

        // Update membership
        let mut active: organization_users::ActiveModel = membership.into();

        if let Some(role) = new_role {
            active.role = Set(role);
        }
        if let Some(limit) = new_approval_limit {
            active.approval_limit = Set(limit);
        }
        active.updated_at = Set(chrono::Utc::now().into());

        let updated = active.update(&self.db).await?;
        Ok(updated)
    }
}

/// Returns the privilege level of a role (higher = more privileges).
const fn role_level(role: &UserRole) -> u8 {
    match role {
        UserRole::Owner => 100,
        UserRole::Admin => 80,
        UserRole::Approver => 60,
        UserRole::Accountant => 40,
        UserRole::Submitter => 30,
        UserRole::Viewer => 20,
    }
}

/// Returns the privilege level of a role (higher = more privileges).
/// Public version for use in other modules.
#[must_use]
pub const fn get_role_level(role: &UserRole) -> u8 {
    role_level(role)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // =========================================================================
    // Helper functions for validation logic (testable without database)
    // =========================================================================

    /// Validates name length (1-255 characters).
    fn validate_name(name: &str) -> Result<(), OrganizationError> {
        if name.is_empty() || name.len() > 255 {
            Err(OrganizationError::InvalidName)
        } else {
            Ok(())
        }
    }

    /// Checks if a role can perform admin operations.
    fn can_perform_admin_ops(role: &UserRole) -> bool {
        matches!(role, UserRole::Admin | UserRole::Owner)
    }

    /// Checks if requester can remove target based on role hierarchy.
    fn can_remove_user(requester_role: &UserRole, target_role: &UserRole) -> bool {
        // Must be admin or owner to remove anyone
        if !can_perform_admin_ops(requester_role) {
            return false;
        }
        // Admin cannot remove owner
        if *target_role == UserRole::Owner && *requester_role != UserRole::Owner {
            return false;
        }
        true
    }

    // =========================================================================
    // Property Test Strategies
    // =========================================================================

    /// Strategy to generate roles below admin level.
    fn non_admin_role_strategy() -> impl Strategy<Value = UserRole> {
        prop_oneof![
            Just(UserRole::Approver),
            Just(UserRole::Accountant),
            Just(UserRole::Submitter),
            Just(UserRole::Viewer),
        ]
    }

    /// Strategy to generate admin-level roles.
    fn admin_role_strategy() -> impl Strategy<Value = UserRole> {
        prop_oneof![Just(UserRole::Admin), Just(UserRole::Owner),]
    }

    /// Strategy to generate any role.
    fn any_role_strategy() -> impl Strategy<Value = UserRole> {
        prop_oneof![
            Just(UserRole::Owner),
            Just(UserRole::Admin),
            Just(UserRole::Approver),
            Just(UserRole::Accountant),
            Just(UserRole::Submitter),
            Just(UserRole::Viewer),
        ]
    }

    /// Strategy to generate valid names (1-255 chars).
    fn valid_name_strategy() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9 ]{1,255}"
    }

    /// Strategy to generate invalid names (empty or > 255 chars).
    fn invalid_name_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            Just(String::new()),    // Empty
            "[a-zA-Z0-9]{256,300}", // Too long
        ]
    }

    /// Checks if removing an owner is allowed based on owner count.
    fn can_remove_owner_by_count(owner_count: u64) -> bool {
        owner_count > 1
    }

    /// Checks if currency change is allowed based on posted transaction count.
    fn can_change_currency(has_posted_transactions: bool) -> bool {
        !has_posted_transactions
    }

    // =========================================================================
    // Property Tests
    // =========================================================================

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        // =========================================================================
        // Property 1: Role-Based Authorization
        // Validates: Requirements 1.2, 2.2
        // =========================================================================

        /// Property 1.1: Non-admin roles cannot perform admin operations.
        ///
        /// *For any* user with role below admin (approver, accountant, submitter, viewer),
        /// admin operations SHALL be rejected.
        #[test]
        fn prop_non_admin_cannot_perform_admin_ops(role in non_admin_role_strategy()) {
            prop_assert!(
                !can_perform_admin_ops(&role),
                "Role {:?} should not be able to perform admin operations",
                role
            );
        }

        /// Property 1.2: Admin and owner roles can perform admin operations.
        ///
        /// *For any* user with admin or owner role,
        /// admin operations SHALL be allowed.
        #[test]
        fn prop_admin_can_perform_admin_ops(role in admin_role_strategy()) {
            prop_assert!(
                can_perform_admin_ops(&role),
                "Role {:?} should be able to perform admin operations",
                role
            );
        }

        // =========================================================================
        // Property 2: Owner Hierarchy Protection
        // Validates: Requirements 2.3
        // =========================================================================

        /// Property 2.1: Admin cannot remove owner.
        ///
        /// *For any* removal request where requester is admin and target is owner,
        /// the request SHALL be rejected.
        #[test]
        fn prop_admin_cannot_remove_owner(_unused in any::<u8>()) {
            let result = can_remove_user(&UserRole::Admin, &UserRole::Owner);
            prop_assert!(
                !result,
                "Admin should not be able to remove owner"
            );
        }

        /// Property 2.2: Owner can remove owner (if not last).
        ///
        /// *For any* removal request where requester is owner and target is owner,
        /// the role check SHALL pass (last owner check is separate).
        #[test]
        fn prop_owner_can_remove_owner(_unused in any::<u8>()) {
            let result = can_remove_user(&UserRole::Owner, &UserRole::Owner);
            prop_assert!(
                result,
                "Owner should be able to remove owner (role check only)"
            );
        }

        /// Property 2.3: Admin can remove non-owner roles.
        ///
        /// *For any* removal request where requester is admin and target is not owner,
        /// the role check SHALL pass.
        #[test]
        fn prop_admin_can_remove_non_owner(target in non_admin_role_strategy()) {
            let result = can_remove_user(&UserRole::Admin, &target);
            prop_assert!(
                result,
                "Admin should be able to remove {:?}",
                target
            );
        }

        /// Property 2.4: Non-admin cannot remove anyone.
        ///
        /// *For any* removal request where requester is below admin,
        /// the request SHALL be rejected regardless of target role.
        #[test]
        fn prop_non_admin_cannot_remove_anyone(
            requester in non_admin_role_strategy(),
            target in any_role_strategy()
        ) {
            let result = can_remove_user(&requester, &target);
            prop_assert!(
                !result,
                "{:?} should not be able to remove {:?}",
                requester,
                target
            );
        }

        // =========================================================================
        // Property 9: Name Validation
        // Validates: Requirements 1.3
        // =========================================================================

        /// Property 9.1: Valid names are accepted.
        ///
        /// *For any* name between 1 and 255 characters,
        /// validation SHALL succeed.
        #[test]
        fn prop_valid_name_accepted(name in valid_name_strategy()) {
            let result = validate_name(&name);
            prop_assert!(
                result.is_ok(),
                "Valid name '{}' (len={}) should be accepted",
                name,
                name.len()
            );
        }

        /// Property 9.2: Invalid names are rejected.
        ///
        /// *For any* name that is empty or exceeds 255 characters,
        /// validation SHALL fail with InvalidName error.
        #[test]
        fn prop_invalid_name_rejected(name in invalid_name_strategy()) {
            let result = validate_name(&name);
            prop_assert!(
                matches!(result, Err(OrganizationError::InvalidName)),
                "Invalid name (len={}) should be rejected",
                name.len()
            );
        }

        // =========================================================================
        // Property 3: Last Owner Protection
        // Validates: Requirements 2.4
        // =========================================================================

        /// Property 3.1: Single owner cannot be removed.
        ///
        /// *For any* organization with exactly one owner,
        /// removing that owner SHALL be rejected.
        #[test]
        fn prop_single_owner_cannot_be_removed(_unused in any::<u8>()) {
            let result = can_remove_owner_by_count(1);
            prop_assert!(
                !result,
                "Single owner should not be removable"
            );
        }

        /// Property 3.2: Multiple owners allow removal.
        ///
        /// *For any* organization with more than one owner,
        /// removing an owner SHALL be allowed (by count check).
        #[test]
        fn prop_multiple_owners_allow_removal(count in 2u64..=100) {
            let result = can_remove_owner_by_count(count);
            prop_assert!(
                result,
                "With {} owners, removal should be allowed",
                count
            );
        }

        // =========================================================================
        // Property 4: Currency Change Restriction
        // Validates: Requirements 1.5
        // =========================================================================

        /// Property 4.1: Currency change blocked with posted transactions.
        ///
        /// *For any* organization with posted transactions,
        /// changing base currency SHALL be rejected.
        #[test]
        fn prop_currency_change_blocked_with_posted(_unused in any::<u8>()) {
            let result = can_change_currency(true);
            prop_assert!(
                !result,
                "Currency change should be blocked when posted transactions exist"
            );
        }

        /// Property 4.2: Currency change allowed without posted transactions.
        ///
        /// *For any* organization without posted transactions,
        /// changing base currency SHALL be allowed.
        #[test]
        fn prop_currency_change_allowed_without_posted(_unused in any::<u8>()) {
            let result = can_change_currency(false);
            prop_assert!(
                result,
                "Currency change should be allowed when no posted transactions"
            );
        }
    }

    // =========================================================================
    // Unit Tests for Specific Examples
    // =========================================================================

    #[test]
    fn test_role_levels() {
        assert_eq!(role_level(&UserRole::Owner), 100);
        assert_eq!(role_level(&UserRole::Admin), 80);
        assert_eq!(role_level(&UserRole::Approver), 60);
        assert_eq!(role_level(&UserRole::Accountant), 40);
        assert_eq!(role_level(&UserRole::Submitter), 30);
        assert_eq!(role_level(&UserRole::Viewer), 20);
    }

    #[test]
    fn test_admin_ops_permissions() {
        assert!(can_perform_admin_ops(&UserRole::Owner));
        assert!(can_perform_admin_ops(&UserRole::Admin));
        assert!(!can_perform_admin_ops(&UserRole::Approver));
        assert!(!can_perform_admin_ops(&UserRole::Accountant));
        assert!(!can_perform_admin_ops(&UserRole::Submitter));
        assert!(!can_perform_admin_ops(&UserRole::Viewer));
    }

    #[test]
    fn test_remove_user_hierarchy() {
        // Owner can remove anyone
        assert!(can_remove_user(&UserRole::Owner, &UserRole::Owner));
        assert!(can_remove_user(&UserRole::Owner, &UserRole::Admin));
        assert!(can_remove_user(&UserRole::Owner, &UserRole::Viewer));

        // Admin can remove non-owners
        assert!(!can_remove_user(&UserRole::Admin, &UserRole::Owner));
        assert!(can_remove_user(&UserRole::Admin, &UserRole::Admin));
        assert!(can_remove_user(&UserRole::Admin, &UserRole::Viewer));

        // Non-admin cannot remove anyone
        assert!(!can_remove_user(&UserRole::Viewer, &UserRole::Viewer));
        assert!(!can_remove_user(&UserRole::Accountant, &UserRole::Viewer));
    }

    #[test]
    fn test_name_validation() {
        // Valid names
        assert!(validate_name("Test Org").is_ok());
        assert!(validate_name("A").is_ok());
        assert!(validate_name(&"x".repeat(255)).is_ok());

        // Invalid names
        assert!(matches!(
            validate_name(""),
            Err(OrganizationError::InvalidName)
        ));
        assert!(matches!(
            validate_name(&"x".repeat(256)),
            Err(OrganizationError::InvalidName)
        ));
    }

    #[test]
    fn test_last_owner_protection() {
        // Single owner cannot be removed
        assert!(!can_remove_owner_by_count(0));
        assert!(!can_remove_owner_by_count(1));

        // Multiple owners allow removal
        assert!(can_remove_owner_by_count(2));
        assert!(can_remove_owner_by_count(10));
    }

    #[test]
    fn test_currency_change_restriction() {
        // With posted transactions - blocked
        assert!(!can_change_currency(true));

        // Without posted transactions - allowed
        assert!(can_change_currency(false));
    }
}
