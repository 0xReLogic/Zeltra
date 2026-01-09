//! Phase 1 auth & organization focused tests.

#[cfg(test)]
mod tests {
    use super::super::{
        Claims, JwtConfig, TokenPair,
        auth::{
            AddUserRequest, CreateOrganizationRequest, LogoutRequest, RefreshRequest,
            RegisterRequest, UpdateMemberRequest, UpdateOrganizationRequest, VerifyEmailRequest,
        },
        jwt::JwtService,
    };
    use chrono::{Duration, Utc};
    use serde_json::json;
    use uuid::Uuid;

    fn test_service() -> JwtService {
        JwtService::new(JwtConfig {
            secret: "phase1-secret".to_string(),
            access_token_expires_minutes: 15,
            refresh_token_expires_days: 3,
        })
    }

    #[test]
    fn claims_sets_expiration_and_iat() {
        let user = Uuid::new_v4();
        let org = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::minutes(30);
        let before = Utc::now().timestamp();
        let claims = Claims::new(user, org, "admin", expires_at);
        let after = Utc::now().timestamp();

        assert_eq!(claims.sub, user);
        assert_eq!(claims.org, org);
        assert!(claims.iat >= before);
        assert!(claims.iat <= after);
        assert_eq!(claims.exp, expires_at.timestamp());
    }

    #[test]
    fn claims_preserves_arbitrary_role() {
        let claims = Claims::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "custom_role-with-scope",
            Utc::now() + Duration::hours(1),
        );

        assert_eq!(claims.role, "custom_role-with-scope");
    }

    #[test]
    fn claims_for_different_users_are_distinct() {
        let org = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::hours(1);
        let first = Claims::new(Uuid::new_v4(), org, "viewer", expires_at);
        let second = Claims::new(Uuid::new_v4(), org, "viewer", expires_at);

        assert_ne!(first.sub, second.sub);
        assert_eq!(first.org, second.org);
    }

    #[test]
    fn token_pair_builder_sets_fields() {
        let pair = TokenPair::new("access".into(), "refresh".into(), 900);
        assert_eq!(pair.access_token, "access");
        assert_eq!(pair.refresh_token, "refresh");
        assert_eq!(pair.expires_in, 900);
    }

    #[test]
    fn token_pair_allows_zero_expiration() {
        let pair = TokenPair::new("a".into(), "b".into(), 0);
        assert_eq!(pair.expires_in, 0);
    }

    #[test]
    fn jwt_access_token_expiration_in_seconds() {
        let service = test_service();
        assert_eq!(service.access_token_expires_in(), 15 * 60);
    }

    #[test]
    fn jwt_refresh_token_days_matches_config() {
        let service = test_service();
        assert_eq!(service.refresh_token_expires_days(), 3);
    }

    #[test]
    fn jwt_refresh_token_last_longer_than_access() {
        let service = test_service();
        let user = Uuid::new_v4();
        let org = Uuid::new_v4();

        let access = service
            .generate_access_token(user, org, "admin")
            .expect("access token");
        let refresh = service
            .generate_refresh_token(user, org, "admin")
            .expect("refresh token");

        let access_claims = service.validate_token(&access).expect("access claims");
        let refresh_claims = service.validate_token(&refresh).expect("refresh claims");

        assert!(refresh_claims.exp > access_claims.exp);
    }

    #[test]
    fn jwt_access_and_refresh_tokens_differ() {
        let service = test_service();
        let user = Uuid::new_v4();
        let org = Uuid::new_v4();

        let access = service
            .generate_access_token(user, org, "viewer")
            .expect("access");
        let refresh = service
            .generate_refresh_token(user, org, "viewer")
            .expect("refresh");

        assert_ne!(access, refresh);
    }

    #[test]
    fn jwt_validation_fails_with_wrong_secret() {
        let service = test_service();
        let other_service = JwtService::new(JwtConfig {
            secret: "different-secret".into(),
            access_token_expires_minutes: 15,
            refresh_token_expires_days: 3,
        });

        let token = service
            .generate_access_token(Uuid::new_v4(), Uuid::new_v4(), "viewer")
            .expect("token");

        assert!(matches!(
            other_service.validate_token(&token),
            Err(crate::JwtError::DecodingError(_))
        ));
    }

    #[test]
    fn create_org_request_defaults_timezone_to_utc() {
        let json = json!({
            "name": "Org",
            "slug": "org-slug",
            "base_currency": "USD"
        });
        let req: CreateOrganizationRequest =
            serde_json::from_value(json).expect("deserialize request");
        assert_eq!(req.timezone, "UTC");
    }

    #[test]
    fn create_org_request_allows_custom_timezone() {
        let json = json!({
            "name": "Org",
            "slug": "org-slug",
            "base_currency": "USD",
            "timezone": "Asia/Jakarta"
        });
        let req: CreateOrganizationRequest =
            serde_json::from_value(json).expect("deserialize request");
        assert_eq!(req.timezone, "Asia/Jakarta");
    }

    #[test]
    fn add_user_request_preserves_optional_approval_limit() {
        let req = AddUserRequest {
            email: "member@example.com".to_string(),
            role: "approver".to_string(),
            approval_limit: Some("1000".to_string()),
        };
        assert_eq!(req.approval_limit.as_deref(), Some("1000"));
    }

    #[test]
    fn update_org_request_supports_partial_updates() {
        let req = UpdateOrganizationRequest {
            name: None,
            base_currency: Some("IDR".to_string()),
            timezone: None,
        };
        assert!(req.name.is_none());
        assert_eq!(req.base_currency.as_deref(), Some("IDR"));
        assert!(req.timezone.is_none());
    }

    #[test]
    fn update_member_request_allows_clearing_limit() {
        let req = UpdateMemberRequest {
            role: Some("viewer".to_string()),
            approval_limit: Some(None),
        };
        assert_eq!(req.role.as_deref(), Some("viewer"));
        assert!(req.approval_limit.is_some());
        assert!(req.approval_limit.unwrap().is_none());
    }

    #[test]
    fn refresh_request_holds_token() {
        let req = RefreshRequest {
            refresh_token: "refresh-123".to_string(),
        };
        assert_eq!(req.refresh_token, "refresh-123");
    }

    #[test]
    fn logout_request_carries_refresh_token() {
        let req = LogoutRequest {
            refresh_token: "logout-token".to_string(),
        };
        assert_eq!(req.refresh_token, "logout-token");
    }

    #[test]
    fn verify_email_request_keeps_token() {
        let req = VerifyEmailRequest {
            token: "verify-token".to_string(),
        };
        assert_eq!(req.token, "verify-token");
    }

    #[test]
    fn register_request_preserves_all_fields() {
        let req = RegisterRequest {
            email: "user@example.com".to_string(),
            password: "secure".to_string(),
            full_name: "User Example".to_string(),
        };
        assert_eq!(req.email, "user@example.com");
        assert_eq!(req.password, "secure");
        assert_eq!(req.full_name, "User Example");
    }
}
