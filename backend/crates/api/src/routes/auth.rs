//! Authentication routes for login, register, token refresh, and logout.

use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use serde_json::json;
use tracing::{error, info, warn};
use validator::Validate;

use crate::AppState;
use zeltra_core::auth::{hash_password, verify_password};
use zeltra_db::{
    EmailVerificationRepository, SessionRepository, UserRepository,
    entities::sea_orm_active_enums::UserRole,
};
use zeltra_shared::auth::{
    LoginRequest, LoginResponse, LogoutRequest, RefreshRequest, RefreshResponse, RegisterRequest,
    RegisterResponse, ResendVerificationRequest, ResendVerificationResponse, UserInfo,
    UserOrganization, VerifyEmailRequest, VerifyEmailResponse,
};

/// Creates the auth router.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/register", post(register))
        .route("/auth/refresh", post(refresh))
        .route("/auth/logout", post(logout))
        .route("/auth/verify-email", post(verify_email))
        .route("/auth/resend-verification", post(resend_verification))
}

/// POST /auth/login - Authenticate user and return tokens.
#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials")
    ),
    tag = "Auth"
)]
#[allow(clippy::too_many_lines)]
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    // Validate request payload
    if let Err(e) = payload.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "validation_error",
                "message": "Invalid request data",
                "details": e.to_string()
            })),
        )
            .into_response();
    }

    let user_repo = UserRepository::new((*state.db).clone());
    let session_repo = SessionRepository::new((*state.db).clone());

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

    // Get default organization for token (first one, or nil if no orgs)
    let (default_org_id, default_role) = match orgs.first() {
        Some((org, membership)) => (org.id, role_to_string(&membership.role)),
        None => {
            // User has no organizations - allow login but with nil org_id
            // Frontend should redirect to create organization page
            (uuid::Uuid::nil(), "none".to_string())
        }
    };

    // Generate tokens
    let access_token =
        match state
            .jwt_service
            .generate_access_token(user.id, default_org_id, &default_role)
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
            .generate_refresh_token(user.id, default_org_id, &default_role)
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

    // Store session in database (only if user has an org)
    if !default_org_id.is_nil() {
        let expires_at = chrono::Utc::now()
            + chrono::Duration::days(state.jwt_service.refresh_token_expires_days());
        if let Err(e) = session_repo
            .create(
                user.id,
                default_org_id,
                &refresh_token,
                expires_at,
                None, // TODO: Extract user agent from request headers
                None, // TODO: Extract IP from request
            )
            .await
        {
            error!(error = %e, "Failed to create session");
            // Don't fail login if session creation fails - tokens are still valid
        }
    }

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
#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully", body = RegisterResponse),
        (status = 409, description = "Email already exists")
    ),
    tag = "Auth"
)]
#[allow(clippy::too_many_lines)]
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    // Validate request payload
    if let Err(e) = payload.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "validation_error",
                "message": "Invalid request data",
                "details": e.to_string()
            })),
        )
            .into_response();
    }

    let user_repo = UserRepository::new((*state.db).clone());
    let email_verification_repo = EmailVerificationRepository::new((*state.db).clone());

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

    // Create verification token and send email
    match email_verification_repo.create_token(user.id).await {
        Ok(token) => {
            // Send verification email (don't fail registration if email fails)
            if let Err(e) = state
                .email_service
                .send_verification_email(&user.email, &user.full_name, &token)
                .await
            {
                warn!(error = %e, user_id = %user.id, "Failed to send verification email");
            } else {
                info!(user_id = %user.id, "Verification email sent");
            }
        }
        Err(e) => {
            warn!(error = %e, user_id = %user.id, "Failed to create verification token");
        }
    }

    // Return user info (without tokens - they need to verify email and create/join an org first)
    (
        StatusCode::CREATED,
        Json(RegisterResponse {
            user: UserInfo {
                id: user.id,
                email: user.email,
                full_name: user.full_name,
                organizations: vec![],
            },
            message: "Registration successful. Please check your email to verify your account."
                .to_string(),
        }),
    )
        .into_response()
}

/// POST /auth/refresh - Refresh access token using refresh token.
#[utoipa::path(
    post,
    path = "/auth/refresh",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "Token refreshed successfully", body = RefreshResponse),
        (status = 401, description = "Invalid or expired refresh token")
    ),
    tag = "Auth"
)]
pub async fn refresh(
    State(state): State<AppState>,
    Json(payload): Json<RefreshRequest>,
) -> impl IntoResponse {
    let session_repo = SessionRepository::new((*state.db).clone());

    // Check if session exists and is valid
    let session = match session_repo.find_by_token(&payload.refresh_token).await {
        Ok(Some(s)) => s,
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "invalid_token",
                    "message": "Invalid or revoked refresh token"
                })),
            )
                .into_response();
        }
        Err(e) => {
            error!(error = %e, "Database error checking session");
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

    // Check if session is expired
    if session.expires_at < chrono::Utc::now() {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "token_expired",
                "message": "Refresh token has expired"
            })),
        )
            .into_response();
    }

    // Validate refresh token JWT
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
        Json(RefreshResponse {
            access_token,
            expires_in: state.jwt_service.access_token_expires_in(),
        }),
    )
        .into_response()
}

/// POST /auth/logout - Logout and invalidate refresh token.
#[utoipa::path(
    post,
    path = "/auth/logout",
    request_body = LogoutRequest,
    responses(
        (status = 200, description = "Logged out successfully")
    ),
    tag = "Auth"
)]
pub async fn logout(
    State(state): State<AppState>,
    Json(payload): Json<LogoutRequest>,
) -> impl IntoResponse {
    let session_repo = SessionRepository::new((*state.db).clone());

    // Revoke the session
    match session_repo.revoke_by_token(&payload.refresh_token).await {
        Ok(true) => {
            info!("Session revoked successfully");
            (
                StatusCode::OK,
                Json(json!({
                    "message": "Logged out successfully"
                })),
            )
                .into_response()
        }
        Ok(false) => {
            // Token not found or already revoked - still return success
            (
                StatusCode::OK,
                Json(json!({
                    "message": "Logged out successfully"
                })),
            )
                .into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to revoke session");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "An error occurred during logout"
                })),
            )
                .into_response()
        }
    }
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

/// POST /auth/verify-email - Verify user's email with token.
#[utoipa::path(
    post,
    path = "/auth/verify-email",
    request_body = VerifyEmailRequest,
    responses(
        (status = 200, description = "Email verified successfully", body = VerifyEmailResponse),
        (status = 400, description = "Invalid or expired token")
    ),
    tag = "Auth"
)]
pub async fn verify_email(
    State(state): State<AppState>,
    Json(payload): Json<VerifyEmailRequest>,
) -> impl IntoResponse {
    let email_verification_repo = EmailVerificationRepository::new((*state.db).clone());

    match email_verification_repo.verify_token(&payload.token).await {
        Ok(user) => {
            info!(user_id = %user.id, "Email verified successfully");
            (
                StatusCode::OK,
                Json(VerifyEmailResponse {
                    message: "Email verified successfully".to_string(),
                    verified: true,
                }),
            )
                .into_response()
        }
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("Invalid or expired") {
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "invalid_token",
                        "message": "Invalid or expired verification token"
                    })),
                )
                    .into_response()
            } else {
                error!(error = %e, "Failed to verify email");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "internal_error",
                        "message": "An error occurred during email verification"
                    })),
                )
                    .into_response()
            }
        }
    }
}

/// POST /auth/resend-verification - Resend verification email.
#[utoipa::path(
    post,
    path = "/auth/resend-verification",
    request_body = ResendVerificationRequest,
    responses(
        (status = 200, description = "Verification email sent if account exists", body = ResendVerificationResponse)
    ),
    tag = "Auth"
)]
pub async fn resend_verification(
    State(state): State<AppState>,
    Json(payload): Json<ResendVerificationRequest>,
) -> impl IntoResponse {
    let user_repo = UserRepository::new((*state.db).clone());
    let email_verification_repo = EmailVerificationRepository::new((*state.db).clone());

    // Find user by email
    let user = match user_repo.find_by_email(&payload.email).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            // Don't reveal if email exists or not for security
            return (
                StatusCode::OK,
                Json(json!({
                    "message": "If an account exists with this email, a verification link has been sent."
                })),
            )
                .into_response();
        }
        Err(e) => {
            error!(error = %e, "Database error finding user");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "An error occurred"
                })),
            )
                .into_response();
        }
    };

    // Check if already verified
    if user.email_verified_at.is_some() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "already_verified",
                "message": "Email is already verified"
            })),
        )
            .into_response();
    }

    // Create new verification token
    let token = match email_verification_repo.create_token(user.id).await {
        Ok(t) => t,
        Err(e) => {
            error!(error = %e, "Failed to create verification token");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "An error occurred"
                })),
            )
                .into_response();
        }
    };

    // Send verification email
    if let Err(e) = state
        .email_service
        .send_verification_email(&user.email, &user.full_name, &token)
        .await
    {
        error!(error = %e, user_id = %user.id, "Failed to send verification email");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "email_error",
                "message": "Failed to send verification email"
            })),
        )
            .into_response();
    }

    info!(user_id = %user.id, "Verification email resent");

    (
        StatusCode::OK,
        Json(ResendVerificationResponse {
            message: "If an account exists with this email, a verification link has been sent."
                .to_string(),
        }),
    )
        .into_response()
}
