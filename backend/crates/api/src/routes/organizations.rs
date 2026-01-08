//! Organization management routes.

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde_json::json;
use tracing::{error, info};

use crate::{AppState, middleware::AuthUser};
use zeltra_db::{OrganizationRepository, UserRepository, entities::sea_orm_active_enums::UserRole};
use zeltra_shared::auth::{AddUserRequest, CreateOrganizationRequest};

/// Creates the organizations router (requires auth middleware to be applied externally).
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/organizations", post(create_organization))
        .route("/organizations/{org_id}", get(get_organization))
        .route("/organizations/{org_id}/users", get(list_users))
        .route("/organizations/{org_id}/users", post(add_user))
}

/// POST /organizations - Create a new organization.
async fn create_organization(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<CreateOrganizationRequest>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check if slug is available
    match org_repo.slug_exists(&payload.slug).await {
        Ok(true) => {
            return (
                StatusCode::CONFLICT,
                Json(json!({
                    "error": "slug_exists",
                    "message": "An organization with this slug already exists"
                })),
            )
                .into_response();
        }
        Ok(false) => {}
        Err(e) => {
            error!(error = %e, "Database error checking slug");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "An error occurred"
                })),
            )
                .into_response();
        }
    }

    // Create organization with current user as owner
    let org = match org_repo
        .create_with_owner(
            &payload.name,
            &payload.slug,
            &payload.base_currency,
            &payload.timezone,
            auth.user_id(),
        )
        .await
    {
        Ok(o) => o,
        Err(e) => {
            error!(error = %e, "Failed to create organization");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "An error occurred creating the organization"
                })),
            )
                .into_response();
        }
    };

    info!(
        org_id = %org.id,
        slug = %org.slug,
        owner_id = %auth.user_id(),
        "Organization created"
    );

    (
        StatusCode::CREATED,
        Json(json!({
            "id": org.id,
            "name": org.name,
            "slug": org.slug,
            "base_currency": org.base_currency,
            "timezone": org.timezone,
            "subscription_tier": format!("{:?}", org.subscription_tier).to_lowercase(),
            "subscription_status": format!("{:?}", org.subscription_status).to_lowercase(),
            "created_at": org.created_at
        })),
    )
        .into_response()
}

/// GET `/organizations/{org_id}` - Get organization details.
async fn get_organization(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_id): Path<uuid::Uuid>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check if user is a member
    match org_repo.is_member(org_id, auth.user_id()).await {
        Ok(false) => {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({
                    "error": "forbidden",
                    "message": "You are not a member of this organization"
                })),
            )
                .into_response();
        }
        Err(e) => {
            error!(error = %e, "Database error checking membership");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "An error occurred"
                })),
            )
                .into_response();
        }
        Ok(true) => {}
    }

    // Get organization
    let org = match org_repo.find_by_id(org_id).await {
        Ok(Some(o)) => o,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "not_found",
                    "message": "Organization not found"
                })),
            )
                .into_response();
        }
        Err(e) => {
            error!(error = %e, "Database error fetching organization");
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

    (
        StatusCode::OK,
        Json(json!({
            "id": org.id,
            "name": org.name,
            "slug": org.slug,
            "base_currency": org.base_currency,
            "timezone": org.timezone,
            "subscription_tier": format!("{:?}", org.subscription_tier).to_lowercase(),
            "subscription_status": format!("{:?}", org.subscription_status).to_lowercase(),
            "created_at": org.created_at
        })),
    )
        .into_response()
}

/// GET `/organizations/{org_id}/users` - List organization users.
async fn list_users(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_id): Path<uuid::Uuid>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check if user is a member
    match org_repo.is_member(org_id, auth.user_id()).await {
        Ok(false) => {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({
                    "error": "forbidden",
                    "message": "You are not a member of this organization"
                })),
            )
                .into_response();
        }
        Err(e) => {
            error!(error = %e, "Database error checking membership");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "An error occurred"
                })),
            )
                .into_response();
        }
        Ok(true) => {}
    }

    // Get users
    let users = match org_repo.get_users(org_id).await {
        Ok(u) => u,
        Err(e) => {
            error!(error = %e, "Database error fetching users");
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

    let users_json: Vec<_> = users
        .into_iter()
        .map(|(user, membership)| {
            json!({
                "id": user.id,
                "email": user.email,
                "full_name": user.full_name,
                "role": role_to_string(&membership.role),
                "approval_limit": membership.approval_limit,
                "created_at": membership.created_at
            })
        })
        .collect();

    (StatusCode::OK, Json(json!({ "users": users_json }))).into_response()
}

/// POST `/organizations/{org_id}/users` - Add user to organization.
#[allow(clippy::too_many_lines)]
async fn add_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_id): Path<uuid::Uuid>,
    Json(payload): Json<AddUserRequest>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());
    let user_repo = UserRepository::new((*state.db).clone());

    // Check if current user has admin or owner role
    match org_repo
        .has_role(org_id, auth.user_id(), UserRole::Admin)
        .await
    {
        Ok(false) => {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({
                    "error": "forbidden",
                    "message": "You need admin or owner role to add users"
                })),
            )
                .into_response();
        }
        Err(e) => {
            error!(error = %e, "Database error checking role");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "An error occurred"
                })),
            )
                .into_response();
        }
        Ok(true) => {}
    }

    // Find user by email
    let user = match user_repo.find_by_email(&payload.email).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "user_not_found",
                    "message": "No user found with this email"
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

    // Check if user is already a member
    match org_repo.is_member(org_id, user.id).await {
        Ok(true) => {
            return (
                StatusCode::CONFLICT,
                Json(json!({
                    "error": "already_member",
                    "message": "User is already a member of this organization"
                })),
            )
                .into_response();
        }
        Err(e) => {
            error!(error = %e, "Database error checking membership");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "An error occurred"
                })),
            )
                .into_response();
        }
        Ok(false) => {}
    }

    // Parse role
    let Some(role) = string_to_role(&payload.role) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_role",
                "message": "Invalid role. Must be one of: owner, admin, approver, accountant, viewer"
            })),
        )
            .into_response();
    };

    // Parse approval limit
    let approval_limit = payload.approval_limit.as_ref().and_then(|s| s.parse().ok());

    // Add user to organization
    let membership = match org_repo
        .add_user(org_id, user.id, role, approval_limit)
        .await
    {
        Ok(m) => m,
        Err(e) => {
            error!(error = %e, "Failed to add user to organization");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "An error occurred adding the user"
                })),
            )
                .into_response();
        }
    };

    info!(
        org_id = %org_id,
        user_id = %user.id,
        role = %payload.role,
        "User added to organization"
    );

    (
        StatusCode::CREATED,
        Json(json!({
            "user_id": user.id,
            "organization_id": org_id,
            "role": role_to_string(&membership.role),
            "approval_limit": membership.approval_limit,
            "created_at": membership.created_at
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

/// Converts string to `UserRole` enum.
fn string_to_role(s: &str) -> Option<UserRole> {
    match s.to_lowercase().as_str() {
        "owner" => Some(UserRole::Owner),
        "admin" => Some(UserRole::Admin),
        "approver" => Some(UserRole::Approver),
        "accountant" => Some(UserRole::Accountant),
        "viewer" => Some(UserRole::Viewer),
        "submitter" => Some(UserRole::Submitter),
        _ => None,
    }
}
