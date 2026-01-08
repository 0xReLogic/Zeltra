//! Unit tests for JWT functionality.

#[cfg(test)]
mod tests {
    use crate::auth::Claims;
    use chrono::{Duration, Utc};
    use uuid::Uuid;

    #[test]
    fn test_claims_new_sets_correct_fields() {
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let role = "admin";
        let expires_at = Utc::now() + Duration::hours(1);

        let claims = Claims::new(user_id, org_id, role, expires_at);

        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.org, org_id);
        assert_eq!(claims.role, "admin");
        assert!(claims.iat <= Utc::now().timestamp());
        assert_eq!(claims.exp, expires_at.timestamp());
    }

    #[test]
    fn test_claims_user_id_returns_sub() {
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::hours(1);

        let claims = Claims::new(user_id, org_id, "viewer", expires_at);

        assert_eq!(claims.user_id(), user_id);
    }

    #[test]
    fn test_claims_organization_id_returns_org() {
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::hours(1);

        let claims = Claims::new(user_id, org_id, "viewer", expires_at);

        assert_eq!(claims.organization_id(), org_id);
    }

    #[test]
    fn test_claims_with_different_roles() {
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::hours(1);

        let roles = [
            "owner",
            "admin",
            "approver",
            "accountant",
            "viewer",
            "submitter",
        ];

        for role in roles {
            let claims = Claims::new(user_id, org_id, role, expires_at);
            assert_eq!(claims.role, role);
        }
    }

    #[test]
    fn test_claims_iat_is_current_time() {
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let before = Utc::now().timestamp();
        let expires_at = Utc::now() + Duration::hours(1);

        let claims = Claims::new(user_id, org_id, "admin", expires_at);

        let after = Utc::now().timestamp();
        assert!(claims.iat >= before);
        assert!(claims.iat <= after);
    }
}
