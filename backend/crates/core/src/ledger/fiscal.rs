//! Fiscal period validation for transaction posting.
//!
//! Implements Requirements 9.1-9.5 for fiscal period posting rules.

use crate::auth::UserRole;
use crate::ledger::error::LedgerError;
use crate::ledger::types::FiscalPeriodStatus;

/// Validates if a user can post to a fiscal period based on period status and user role.
///
/// Property 9: Fiscal Period Posting Rules
/// - OPEN → all authorized users can post
/// - SOFT_CLOSE → only accountant/admin/owner can post
/// - CLOSED → no one can post
///
/// Requirements: 9.3, 9.4, 9.5
///
/// # Arguments
///
/// * `period_status` - The status of the fiscal period
/// * `user_role` - The role of the user attempting to post
///
/// # Returns
///
/// * `Ok(())` if posting is allowed
/// * `Err(LedgerError::PeriodClosed)` if period is closed
/// * `Err(LedgerError::PeriodSoftClosed)` if period is soft-closed and user lacks privileges
pub fn validate_posting_permission(
    period_status: &FiscalPeriodStatus,
    user_role: &UserRole,
) -> Result<(), LedgerError> {
    match period_status {
        // Requirement 9.3: OPEN → all authorized users can post
        FiscalPeriodStatus::Open => Ok(()),
        
        // Requirement 9.4: SOFT_CLOSE → only accountant/admin/owner can post
        FiscalPeriodStatus::SoftClose => {
            if user_role.can_post_soft_close() {
                Ok(())
            } else {
                Err(LedgerError::PeriodSoftClosed)
            }
        }
        
        // Requirement 9.5: CLOSED → no one can post
        FiscalPeriodStatus::Closed => Err(LedgerError::PeriodClosed),
    }
}

/// Checks if a period status allows any posting at all.
///
/// This is a quick check without considering user role.
#[must_use]
pub fn period_allows_posting(status: &FiscalPeriodStatus) -> bool {
    !matches!(status, FiscalPeriodStatus::Closed)
}

/// Checks if posting to a period requires elevated privileges.
#[must_use]
pub fn period_requires_elevated_privileges(status: &FiscalPeriodStatus) -> bool {
    matches!(status, FiscalPeriodStatus::SoftClose)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // ========================================================================
    // Property 9: Fiscal Period Posting Rules
    // **Validates: Requirements 1.5, 1.6, 9.3, 9.4, 9.5**
    // ========================================================================

    /// Strategy for generating fiscal period statuses
    fn period_status_strategy() -> impl Strategy<Value = FiscalPeriodStatus> {
        prop_oneof![
            Just(FiscalPeriodStatus::Open),
            Just(FiscalPeriodStatus::SoftClose),
            Just(FiscalPeriodStatus::Closed),
        ]
    }

    /// Strategy for generating user roles
    fn user_role_strategy() -> impl Strategy<Value = UserRole> {
        prop_oneof![
            Just(UserRole::Owner),
            Just(UserRole::Admin),
            Just(UserRole::Accountant),
            Just(UserRole::Approver),
            Just(UserRole::Viewer),
            Just(UserRole::Submitter),
        ]
    }

    /// Strategy for generating roles that CAN post to soft-closed periods
    fn elevated_role_strategy() -> impl Strategy<Value = UserRole> {
        prop_oneof![
            Just(UserRole::Owner),
            Just(UserRole::Admin),
            Just(UserRole::Accountant),
        ]
    }

    /// Strategy for generating roles that CANNOT post to soft-closed periods
    fn non_elevated_role_strategy() -> impl Strategy<Value = UserRole> {
        prop_oneof![
            Just(UserRole::Approver),
            Just(UserRole::Viewer),
            Just(UserRole::Submitter),
        ]
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Property 9.1: Open periods allow all users to post**
        ///
        /// *For any* user role, posting to an OPEN period SHALL be allowed.
        ///
        /// **Validates: Requirements 9.3**
        #[test]
        fn prop_open_period_allows_all_users(
            role in user_role_strategy(),
        ) {
            let result = validate_posting_permission(&FiscalPeriodStatus::Open, &role);
            prop_assert!(result.is_ok(), "Open period should allow all users to post");
        }

        /// **Property 9.2: Closed periods reject all users**
        ///
        /// *For any* user role, posting to a CLOSED period SHALL be rejected.
        ///
        /// **Validates: Requirements 9.5**
        #[test]
        fn prop_closed_period_rejects_all_users(
            role in user_role_strategy(),
        ) {
            let result = validate_posting_permission(&FiscalPeriodStatus::Closed, &role);
            prop_assert!(result.is_err(), "Closed period should reject all users");
            prop_assert!(
                matches!(result, Err(LedgerError::PeriodClosed)),
                "Should return PeriodClosed error"
            );
        }

        /// **Property 9.3: Soft-closed periods allow elevated roles**
        ///
        /// *For any* user with accountant/admin/owner role, posting to a SOFT_CLOSE
        /// period SHALL be allowed.
        ///
        /// **Validates: Requirements 1.5, 9.4**
        #[test]
        fn prop_soft_close_allows_elevated_roles(
            role in elevated_role_strategy(),
        ) {
            let result = validate_posting_permission(&FiscalPeriodStatus::SoftClose, &role);
            prop_assert!(
                result.is_ok(),
                "Soft-closed period should allow {:?} to post",
                role
            );
        }

        /// **Property 9.4: Soft-closed periods reject non-elevated roles**
        ///
        /// *For any* user without accountant/admin/owner role, posting to a SOFT_CLOSE
        /// period SHALL be rejected.
        ///
        /// **Validates: Requirements 1.5, 9.4**
        #[test]
        fn prop_soft_close_rejects_non_elevated_roles(
            role in non_elevated_role_strategy(),
        ) {
            let result = validate_posting_permission(&FiscalPeriodStatus::SoftClose, &role);
            prop_assert!(
                result.is_err(),
                "Soft-closed period should reject {:?}",
                role
            );
            prop_assert!(
                matches!(result, Err(LedgerError::PeriodSoftClosed)),
                "Should return PeriodSoftClosed error"
            );
        }

        /// **Property 9.5: Posting rules are consistent with period status**
        ///
        /// *For any* period status and user role, the posting rules SHALL be:
        /// - OPEN: always allowed
        /// - SOFT_CLOSE: allowed only for elevated roles
        /// - CLOSED: never allowed
        ///
        /// **Validates: Requirements 9.3, 9.4, 9.5**
        #[test]
        fn prop_posting_rules_consistent(
            status in period_status_strategy(),
            role in user_role_strategy(),
        ) {
            let result = validate_posting_permission(&status, &role);
            
            match status {
                FiscalPeriodStatus::Open => {
                    prop_assert!(result.is_ok(), "Open should always allow posting");
                }
                FiscalPeriodStatus::SoftClose => {
                    if role.can_post_soft_close() {
                        prop_assert!(result.is_ok(), "Elevated role should be allowed");
                    } else {
                        prop_assert!(result.is_err(), "Non-elevated role should be rejected");
                    }
                }
                FiscalPeriodStatus::Closed => {
                    prop_assert!(result.is_err(), "Closed should always reject posting");
                }
            }
        }

        /// **Property 9.6: Period allows posting check is consistent**
        ///
        /// *For any* period status, the quick check SHALL match the detailed validation.
        ///
        /// **Validates: Requirements 9.3, 9.5**
        #[test]
        fn prop_period_allows_posting_consistent(
            status in period_status_strategy(),
        ) {
            let allows = period_allows_posting(&status);
            
            // If period doesn't allow posting, all roles should be rejected
            if !allows {
                for role in [UserRole::Owner, UserRole::Admin, UserRole::Accountant] {
                    let result = validate_posting_permission(&status, &role);
                    prop_assert!(result.is_err(), "If period doesn't allow posting, all should be rejected");
                }
            }
            
            // If period allows posting, at least elevated roles should be allowed
            if allows {
                let result = validate_posting_permission(&status, &UserRole::Owner);
                prop_assert!(result.is_ok(), "If period allows posting, Owner should be allowed");
            }
        }
    }

    // ========================================================================
    // Unit tests for specific examples
    // ========================================================================

    #[test]
    fn test_open_period_all_roles() {
        let status = FiscalPeriodStatus::Open;
        
        assert!(validate_posting_permission(&status, &UserRole::Owner).is_ok());
        assert!(validate_posting_permission(&status, &UserRole::Admin).is_ok());
        assert!(validate_posting_permission(&status, &UserRole::Accountant).is_ok());
        assert!(validate_posting_permission(&status, &UserRole::Approver).is_ok());
        assert!(validate_posting_permission(&status, &UserRole::Viewer).is_ok());
        assert!(validate_posting_permission(&status, &UserRole::Submitter).is_ok());
    }

    #[test]
    fn test_soft_close_elevated_roles() {
        let status = FiscalPeriodStatus::SoftClose;
        
        assert!(validate_posting_permission(&status, &UserRole::Owner).is_ok());
        assert!(validate_posting_permission(&status, &UserRole::Admin).is_ok());
        assert!(validate_posting_permission(&status, &UserRole::Accountant).is_ok());
    }

    #[test]
    fn test_soft_close_non_elevated_roles() {
        let status = FiscalPeriodStatus::SoftClose;
        
        assert!(matches!(
            validate_posting_permission(&status, &UserRole::Approver),
            Err(LedgerError::PeriodSoftClosed)
        ));
        assert!(matches!(
            validate_posting_permission(&status, &UserRole::Viewer),
            Err(LedgerError::PeriodSoftClosed)
        ));
        assert!(matches!(
            validate_posting_permission(&status, &UserRole::Submitter),
            Err(LedgerError::PeriodSoftClosed)
        ));
    }

    #[test]
    fn test_closed_period_all_roles() {
        let status = FiscalPeriodStatus::Closed;
        
        assert!(matches!(
            validate_posting_permission(&status, &UserRole::Owner),
            Err(LedgerError::PeriodClosed)
        ));
        assert!(matches!(
            validate_posting_permission(&status, &UserRole::Admin),
            Err(LedgerError::PeriodClosed)
        ));
        assert!(matches!(
            validate_posting_permission(&status, &UserRole::Accountant),
            Err(LedgerError::PeriodClosed)
        ));
        assert!(matches!(
            validate_posting_permission(&status, &UserRole::Approver),
            Err(LedgerError::PeriodClosed)
        ));
    }

    #[test]
    fn test_period_allows_posting() {
        assert!(period_allows_posting(&FiscalPeriodStatus::Open));
        assert!(period_allows_posting(&FiscalPeriodStatus::SoftClose));
        assert!(!period_allows_posting(&FiscalPeriodStatus::Closed));
    }

    #[test]
    fn test_period_requires_elevated_privileges() {
        assert!(!period_requires_elevated_privileges(&FiscalPeriodStatus::Open));
        assert!(period_requires_elevated_privileges(&FiscalPeriodStatus::SoftClose));
        assert!(!period_requires_elevated_privileges(&FiscalPeriodStatus::Closed));
    }
}
