//! Organization management routes.

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch, post},
};
use serde_json::json;
use tracing::{error, info};

use crate::{AppState, middleware::AuthUser};
use zeltra_db::repositories::organization::OrganizationError;
use zeltra_db::{
    OrganizationRepository, SessionRepository, UserRepository,
    entities::sea_orm_active_enums::UserRole,
};
use zeltra_shared::auth::{
    AddUserRequest, CreateOrganizationRequest, UpdateMemberRequest, UpdateOrganizationRequest,
};

/// Creates the organizations router (requires auth middleware to be applied externally).
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/organizations", post(create_organization))
        .route("/organizations/{org_id}", get(get_organization))
        .route("/organizations/{org_id}", patch(update_organization))
        .route("/organizations/{org_id}/users", get(list_users))
        .route("/organizations/{org_id}/users", post(add_user))
        .route(
            "/organizations/{org_id}/users/{user_id}",
            patch(update_member),
        )
        .route(
            "/organizations/{org_id}/users/{user_id}",
            axum::routing::delete(remove_user),
        )
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

/// PATCH `/organizations/{org_id}` - Update organization settings.
#[allow(clippy::too_many_lines)]
async fn update_organization(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_id): Path<uuid::Uuid>,
    Json(payload): Json<UpdateOrganizationRequest>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check if user has admin or owner role
    match org_repo
        .has_role(org_id, auth.user_id(), UserRole::Admin)
        .await
    {
        Ok(false) => {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({
                    "error": "forbidden",
                    "message": "You need admin or owner role to update organization settings"
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

    // Update organization with full validation
    let org = match org_repo
        .update_organization(
            org_id,
            payload.name.as_deref(),
            payload.base_currency.as_deref(),
            payload.timezone.as_deref(),
        )
        .await
    {
        Ok(o) => o,
        Err(OrganizationError::NotFound) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "not_found",
                    "message": "Organization not found"
                })),
            )
                .into_response();
        }
        Err(OrganizationError::EmptyUpdate) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "empty_update",
                    "message": "No fields provided for update"
                })),
            )
                .into_response();
        }
        Err(OrganizationError::InvalidName) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "invalid_name",
                    "message": "Name must be between 1 and 255 characters"
                })),
            )
                .into_response();
        }
        Err(OrganizationError::InvalidCurrency(code)) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "invalid_currency",
                    "message": format!("Invalid currency code: {}", code)
                })),
            )
                .into_response();
        }
        Err(OrganizationError::CurrencyChangeNotAllowed) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "currency_change_not_allowed",
                    "message": "Cannot change base currency after posting transactions"
                })),
            )
                .into_response();
        }
        Err(OrganizationError::InvalidTimezone(tz)) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "invalid_timezone",
                    "message": format!("Invalid timezone: {}", tz)
                })),
            )
                .into_response();
        }
        Err(e) => {
            error!(error = %e, "Failed to update organization");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "An error occurred updating the organization"
                })),
            )
                .into_response();
        }
    };

    info!(org_id = %org_id, "Organization updated");

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
            "updated_at": org.updated_at
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

/// DELETE `/organizations/{org_id}/users/{user_id}` - Remove user from organization.
#[allow(clippy::too_many_lines)]
async fn remove_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_id, user_id)): Path<(uuid::Uuid, uuid::Uuid)>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());
    let session_repo = SessionRepository::new((*state.db).clone());

    // Get requester's membership to check role
    let requester_membership = match org_repo.get_user_membership(org_id, auth.user_id()).await {
        Ok(Some(m)) => m,
        Ok(None) => {
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
    };

    // Check if requester has admin or owner role
    if !matches!(requester_membership.role, UserRole::Admin | UserRole::Owner) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "forbidden",
                "message": "You need admin or owner role to remove users"
            })),
        )
            .into_response();
    }

    // Remove user from organization
    match org_repo
        .remove_member(org_id, user_id, &requester_membership.role)
        .await
    {
        Ok(()) => {}
        Err(OrganizationError::NotFound) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "not_found",
                    "message": "Organization not found"
                })),
            )
                .into_response();
        }
        Err(OrganizationError::NotMember) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "not_member",
                    "message": "User is not a member of this organization"
                })),
            )
                .into_response();
        }
        Err(OrganizationError::Forbidden) => {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({
                    "error": "forbidden",
                    "message": "Admins cannot remove owners"
                })),
            )
                .into_response();
        }
        Err(OrganizationError::LastOwner) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "last_owner",
                    "message": "Cannot remove the last owner of the organization"
                })),
            )
                .into_response();
        }
        Err(e) => {
            error!(error = %e, "Failed to remove user from organization");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "An error occurred removing the user"
                })),
            )
                .into_response();
        }
    }

    // Revoke all sessions for the removed user in this organization
    if let Err(e) = session_repo.revoke_user_org_sessions(user_id, org_id).await {
        error!(error = %e, "Failed to revoke sessions for removed user");
        // Don't fail the request, user is already removed
    }

    info!(
        org_id = %org_id,
        user_id = %user_id,
        removed_by = %auth.user_id(),
        "User removed from organization"
    );

    StatusCode::NO_CONTENT.into_response()
}

/// PATCH `/organizations/{org_id}/users/{user_id}` - Update user's role and/or approval limit.
#[allow(clippy::too_many_lines)]
async fn update_member(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_id, user_id)): Path<(uuid::Uuid, uuid::Uuid)>,
    Json(payload): Json<UpdateMemberRequest>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Get requester's membership to check role
    let requester_membership = match org_repo.get_user_membership(org_id, auth.user_id()).await {
        Ok(Some(m)) => m,
        Ok(None) => {
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
    };

    // Check if requester has admin or owner role
    if !matches!(requester_membership.role, UserRole::Admin | UserRole::Owner) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "forbidden",
                "message": "You need admin or owner role to update users"
            })),
        )
            .into_response();
    }

    // Parse role if provided
    let new_role = if let Some(ref role_str) = payload.role {
        match string_to_role(role_str) {
            Some(r) => Some(r),
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "invalid_role",
                        "message": "Invalid role. Must be one of: owner, admin, approver, accountant, submitter, viewer"
                    })),
                )
                    .into_response();
            }
        }
    } else {
        None
    };

    // Parse approval limit if provided
    let new_approval_limit = payload
        .approval_limit
        .map(|opt| opt.and_then(|s| s.parse().ok()));

    // Update member
    let membership = match org_repo
        .update_member(
            org_id,
            user_id,
            &requester_membership.role,
            new_role,
            new_approval_limit,
        )
        .await
    {
        Ok(m) => m,
        Err(OrganizationError::NotFound) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "not_found",
                    "message": "Organization not found"
                })),
            )
                .into_response();
        }
        Err(OrganizationError::NotMember) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "not_member",
                    "message": "User is not a member of this organization"
                })),
            )
                .into_response();
        }
        Err(OrganizationError::EmptyUpdate) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "empty_update",
                    "message": "No fields provided for update"
                })),
            )
                .into_response();
        }
        Err(OrganizationError::Forbidden) => {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({
                    "error": "forbidden",
                    "message": "Insufficient permissions to change this user's role"
                })),
            )
                .into_response();
        }
        Err(OrganizationError::LastOwner) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "last_owner",
                    "message": "Cannot demote the last owner of the organization"
                })),
            )
                .into_response();
        }
        Err(e) => {
            error!(error = %e, "Failed to update member");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "An error occurred updating the user"
                })),
            )
                .into_response();
        }
    };

    info!(
        org_id = %org_id,
        user_id = %user_id,
        updated_by = %auth.user_id(),
        "Member updated"
    );

    (
        StatusCode::OK,
        Json(json!({
            "user_id": membership.user_id,
            "organization_id": membership.organization_id,
            "role": role_to_string(&membership.role),
            "approval_limit": membership.approval_limit,
            "updated_at": membership.updated_at
        })),
    )
        .into_response()
}
