//! Authentication routes for login, register, and token refresh.

use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use serde_json::json;
use tracing::{error, info};

use crate::AppState;
use zeltra_core::auth::{hash_password, verify_password};
use zeltra_db::{UserRepository, entities::sea_orm_active_enums::UserRole};
use zeltra_shared::auth::{
    LoginRequest, LoginResponse, RefreshRequest, RegisterRequest, UserInfo, UserOrganization,
};

/// Creates the auth router.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/register", post(register))
        .route("/auth/refresh", post(refresh))
}

/// POST /auth/login - Authenticate user and return tokens.
#[allow(clippy::too_many_lines)]
async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    let user_repo = UserRepository::new((*state.db).clone());

    // Find user by email
    let user = match user_repo.find_by_email(&payload.email).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            info!(email = %payload.email, "Login attempt for non-existent user");
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "invalid_credentials",
                    "message": "Invalid email or password"
                })),
            )
                .into_response();
        }
        Err(e) => {
            error!(error = %e, "Database error during login");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "An error occurred during login"
                })),
            )
                .into_response();
        }
    };

    // Check if user is active
    if !user.is_active {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "account_disabled",
                "message": "This account has been disabled"
            })),
        )
            .into_response();
    }

    // Verify password
    match verify_password(&payload.password, &user.password_hash) {
        Ok(true) => {}
        Ok(false) => {
            info!(user_id = %user.id, "Failed login attempt - invalid password");
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "invalid_credentials",
                    "message": "Invalid email or password"
                })),
            )
                .into_response();
        }
        Err(e) => {
            error!(error = %e, "Password verification error");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "An error occurred during login"
                })),
            )
                .into_response();
        }
    }

    // Get user's organizations
    let orgs = match user_repo.get_user_organizations(user.id).await {
        Ok(o) => o,
        Err(e) => {
            error!(error = %e, "Failed to get user organizations");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "An error occurred during login"
                })),
            )
                .into_response();
        }
    };

    // Get default organization for token (first one)
    let (default_org, default_membership) = match orgs.first() {
        Some((org, membership)) => (org.clone(), membership.clone()),
        None => {
            // User has no organizations - this shouldn't happen normally
            // but we handle it gracefully
            return (
                StatusCode::FORBIDDEN,
                Json(json!({
                    "error": "no_organization",
                    "message": "User is not a member of any organization"
                })),
            )
                .into_response();
        }
    };

    // Generate tokens
    let role_str = role_to_string(&default_membership.role);
    let access_token =
        match state
            .jwt_service
            .generate_access_token(user.id, default_org.id, &role_str)
        {
            Ok(t) => t,
            Err(e) => {
                error!(error = %e, "Failed to generate access token");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "internal_error",
                        "message": "An error occurred during login"
                    })),
                )
                    .into_response();
            }
        };

    let refresh_token =
        match state
            .jwt_service
            .generate_refresh_token(user.id, default_org.id, &role_str)
        {
            Ok(t) => t,
            Err(e) => {
                error!(error = %e, "Failed to generate refresh token");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "internal_error",
                        "message": "An error occurred during login"
                    })),
                )
                    .into_response();
            }
        };

    info!(user_id = %user.id, "User logged in successfully");

    // Build response
    let response = LoginResponse {
        user: UserInfo {
            id: user.id,
            email: user.email,
            full_name: user.full_name,
            organizations: orgs
                .into_iter()
                .map(|(org, membership)| UserOrganization {
                    id: org.id,
                    name: org.name,
                    slug: org.slug,
                    role: role_to_string(&membership.role),
                })
                .collect(),
        },
        access_token,
        refresh_token,
        expires_in: state.jwt_service.access_token_expires_in(),
    };

    (StatusCode::OK, Json(response)).into_response()
}

/// POST /auth/register - Register a new user.
async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    let user_repo = UserRepository::new((*state.db).clone());

    // Check if email already exists
    match user_repo.email_exists(&payload.email).await {
        Ok(true) => {
            return (
                StatusCode::CONFLICT,
                Json(json!({
                    "error": "email_exists",
                    "message": "An account with this email already exists"
                })),
            )
                .into_response();
        }
        Ok(false) => {}
        Err(e) => {
            error!(error = %e, "Database error checking email");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "An error occurred during registration"
                })),
            )
                .into_response();
        }
    }

    // Hash password
    let password_hash = match hash_password(&payload.password) {
        Ok(h) => h,
        Err(e) => {
            error!(error = %e, "Failed to hash password");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "An error occurred during registration"
                })),
            )
                .into_response();
        }
    };

    // Create user
    let user = match user_repo
        .create(&payload.email, &password_hash, &payload.full_name)
        .await
    {
        Ok(u) => u,
        Err(e) => {
            error!(error = %e, "Failed to create user");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "An error occurred during registration"
                })),
            )
                .into_response();
        }
    };

    info!(user_id = %user.id, email = %user.email, "New user registered");

    // Return user info (without tokens - they need to create/join an org first)
    (
        StatusCode::CREATED,
        Json(json!({
            "user": {
                "id": user.id,
                "email": user.email,
                "full_name": user.full_name
            },
            "message": "Registration successful. Please create or join an organization."
        })),
    )
        .into_response()
}

/// POST /auth/refresh - Refresh access token using refresh token.
async fn refresh(
    State(state): State<AppState>,
    Json(payload): Json<RefreshRequest>,
) -> impl IntoResponse {
    // Validate refresh token
    let claims = match state.jwt_service.validate_token(&payload.refresh_token) {
        Ok(c) => c,
        Err(e) => {
            let (error, message) = match e {
                zeltra_shared::JwtError::Expired => ("token_expired", "Refresh token has expired"),
                _ => ("invalid_token", "Invalid refresh token"),
            };
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": error, "message": message })),
            )
                .into_response();
        }
    };

    // Generate new access token
    let access_token = match state.jwt_service.generate_access_token(
        claims.user_id(),
        claims.organization_id(),
        &claims.role,
    ) {
        Ok(t) => t,
        Err(e) => {
            error!(error = %e, "Failed to generate access token");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "An error occurred during token refresh"
                })),
            )
                .into_response();
        }
    };

    (
        StatusCode::OK,
        Json(json!({
            "access_token": access_token,
            "expires_in": state.jwt_service.access_token_expires_in()
        })),
    )
        .into_response()
}

/// Converts `UserRole` enum to string.
fn role_to_string(role: &UserRole) -> String {
    match role {
        UserRole::Owner => "owner".to_string(),
        UserRole::Admin => "admin".to_string(),
        UserRole::Approver => "approver".to_string(),
        UserRole::Accountant => "accountant".to_string(),
        UserRole::Viewer => "viewer".to_string(),
        UserRole::Submitter => "submitter".to_string(),
    }
}
