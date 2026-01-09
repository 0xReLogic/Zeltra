//! Dimension management routes.

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{error, info};
use uuid::Uuid;

use crate::{AppState, middleware::AuthUser};
use zeltra_db::{
    OrganizationRepository,
    entities::sea_orm_active_enums::UserRole,
    repositories::dimension::{
        CreateDimensionTypeInput, CreateDimensionValueInput, DimensionRepository,
        DimensionTypeFilter, DimensionValueFilter,
    },
};

/// Creates the dimension routes (requires auth middleware to be applied externally).
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/organizations/{org_id}/dimension-types", get(list_dimension_types))
        .route("/organizations/{org_id}/dimension-types", post(create_dimension_type))
        .route("/organizations/{org_id}/dimension-values", get(list_dimension_values))
        .route("/organizations/{org_id}/dimension-values", post(create_dimension_value))
}

/// Query parameters for listing dimension types.
#[derive(Debug, Deserialize)]
pub struct ListDimensionTypesQuery {
    /// Filter by active status.
    pub active: Option<bool>,
}

/// Query parameters for listing dimension values.
#[derive(Debug, Deserialize)]
pub struct ListDimensionValuesQuery {
    /// Filter by dimension type ID.
    #[serde(rename = "type")]
    pub dimension_type_id: Option<Uuid>,
    /// Filter by active status.
    pub active: Option<bool>,
}

/// Request body for creating a dimension type.
#[derive(Debug, Deserialize)]
pub struct CreateDimensionTypeRequest {
    /// Dimension type code (must be unique within organization).
    pub code: String,
    /// Dimension type name.
    pub name: String,
    /// Description.
    pub description: Option<String>,
    /// Whether this dimension is required on transactions.
    pub is_required: Option<bool>,
    /// Whether this dimension type is active (default: true).
    pub is_active: Option<bool>,
    /// Sort order for display (default: 0).
    pub sort_order: Option<i16>,
}

/// Request body for creating a dimension value.
#[derive(Debug, Deserialize)]
pub struct CreateDimensionValueRequest {
    /// Dimension type ID.
    pub dimension_type_id: Uuid,
    /// Dimension value code (must be unique within type).
    pub code: String,
    /// Dimension value name.
    pub name: String,
    /// Description.
    pub description: Option<String>,
    /// Parent dimension value ID for hierarchical structure.
    pub parent_id: Option<Uuid>,
    /// Whether this value is active (default: true).
    pub is_active: Option<bool>,
    /// Effective from date.
    pub effective_from: Option<NaiveDate>,
    /// Effective to date.
    pub effective_to: Option<NaiveDate>,
}

/// Response for a dimension type.
#[derive(Debug, Serialize)]
pub struct DimensionTypeResponse {
    /// Dimension type ID.
    pub id: Uuid,
    /// Dimension type code.
    pub code: String,
    /// Dimension type name.
    pub name: String,
    /// Description.
    pub description: Option<String>,
    /// Whether this dimension is required.
    pub is_required: bool,
    /// Whether this dimension type is active.
    pub is_active: bool,
    /// Sort order.
    pub sort_order: i16,
}

/// Response for a dimension value.
#[derive(Debug, Serialize)]
pub struct DimensionValueResponse {
    /// Dimension value ID.
    pub id: Uuid,
    /// Dimension type ID.
    pub dimension_type_id: Uuid,
    /// Dimension value code.
    pub code: String,
    /// Dimension value name.
    pub name: String,
    /// Description.
    pub description: Option<String>,
    /// Parent dimension value ID.
    pub parent_id: Option<Uuid>,
    /// Whether this value is active.
    pub is_active: bool,
    /// Effective from date.
    pub effective_from: Option<NaiveDate>,
    /// Effective to date.
    pub effective_to: Option<NaiveDate>,
}


/// GET `/organizations/{org_id}/dimension-types` - List dimension types.
async fn list_dimension_types(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_id): Path<Uuid>,
    Query(query): Query<ListDimensionTypesQuery>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let dim_repo = DimensionRepository::new((*state.db).clone());

    let filter = DimensionTypeFilter {
        is_active: query.active,
    };

    match dim_repo.list_dimension_types(org_id, filter).await {
        Ok(types) => {
            let response: Vec<DimensionTypeResponse> = types
                .into_iter()
                .map(|t| DimensionTypeResponse {
                    id: t.id,
                    code: t.code,
                    name: t.name,
                    description: t.description,
                    is_required: t.is_required,
                    is_active: t.is_active,
                    sort_order: t.sort_order,
                })
                .collect();

            (StatusCode::OK, Json(json!({ "dimension_types": response }))).into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to list dimension types");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "An error occurred"
                })),
            )
                .into_response()
        }
    }
}

/// POST `/organizations/{org_id}/dimension-types` - Create a dimension type.
async fn create_dimension_type(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_id): Path<Uuid>,
    Json(payload): Json<CreateDimensionTypeRequest>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check admin/owner role
    if let Err(response) = check_admin_role(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let dim_repo = DimensionRepository::new((*state.db).clone());

    let input = CreateDimensionTypeInput {
        organization_id: org_id,
        code: payload.code,
        name: payload.name,
        description: payload.description,
        is_required: payload.is_required.unwrap_or(false),
        is_active: payload.is_active.unwrap_or(true),
        sort_order: payload.sort_order.unwrap_or(0),
    };

    match dim_repo.create_dimension_type(input).await {
        Ok(dim_type) => {
            info!(
                org_id = %org_id,
                dimension_type_id = %dim_type.id,
                code = %dim_type.code,
                "Dimension type created"
            );

            (
                StatusCode::CREATED,
                Json(json!({
                    "id": dim_type.id,
                    "code": dim_type.code,
                    "name": dim_type.name,
                    "description": dim_type.description,
                    "is_required": dim_type.is_required,
                    "is_active": dim_type.is_active,
                    "sort_order": dim_type.sort_order,
                    "created_at": dim_type.created_at
                })),
            )
                .into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to create dimension type");
            match e {
                zeltra_db::repositories::dimension::DimensionError::DuplicateTypeCode(code) => (
                    StatusCode::CONFLICT,
                    Json(json!({
                        "error": "duplicate_code",
                        "message": format!("Dimension type code '{}' already exists", code)
                    })),
                )
                    .into_response(),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "internal_error",
                        "message": "An error occurred"
                    })),
                )
                    .into_response(),
            }
        }
    }
}

/// GET `/organizations/{org_id}/dimension-values` - List dimension values.
async fn list_dimension_values(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_id): Path<Uuid>,
    Query(query): Query<ListDimensionValuesQuery>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let dim_repo = DimensionRepository::new((*state.db).clone());

    let filter = DimensionValueFilter {
        dimension_type_id: query.dimension_type_id,
        is_active: query.active,
        parent_id: None,
    };

    match dim_repo.list_dimension_values(org_id, filter).await {
        Ok(values) => {
            let response: Vec<DimensionValueResponse> = values
                .into_iter()
                .map(|v| DimensionValueResponse {
                    id: v.id,
                    dimension_type_id: v.dimension_type_id,
                    code: v.code,
                    name: v.name,
                    description: v.description,
                    parent_id: v.parent_id,
                    is_active: v.is_active,
                    effective_from: v.effective_from,
                    effective_to: v.effective_to,
                })
                .collect();

            (StatusCode::OK, Json(json!({ "dimension_values": response }))).into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to list dimension values");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "An error occurred"
                })),
            )
                .into_response()
        }
    }
}

/// POST `/organizations/{org_id}/dimension-values` - Create a dimension value.
async fn create_dimension_value(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_id): Path<Uuid>,
    Json(payload): Json<CreateDimensionValueRequest>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check admin/owner role
    if let Err(response) = check_admin_role(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let dim_repo = DimensionRepository::new((*state.db).clone());

    let input = CreateDimensionValueInput {
        organization_id: org_id,
        dimension_type_id: payload.dimension_type_id,
        code: payload.code,
        name: payload.name,
        description: payload.description,
        parent_id: payload.parent_id,
        is_active: payload.is_active.unwrap_or(true),
        effective_from: payload.effective_from,
        effective_to: payload.effective_to,
    };

    match dim_repo.create_dimension_value(input).await {
        Ok(dim_value) => {
            info!(
                org_id = %org_id,
                dimension_value_id = %dim_value.id,
                code = %dim_value.code,
                "Dimension value created"
            );

            (
                StatusCode::CREATED,
                Json(json!({
                    "id": dim_value.id,
                    "dimension_type_id": dim_value.dimension_type_id,
                    "code": dim_value.code,
                    "name": dim_value.name,
                    "description": dim_value.description,
                    "parent_id": dim_value.parent_id,
                    "is_active": dim_value.is_active,
                    "effective_from": dim_value.effective_from,
                    "effective_to": dim_value.effective_to,
                    "created_at": dim_value.created_at
                })),
            )
                .into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to create dimension value");
            match e {
                zeltra_db::repositories::dimension::DimensionError::DuplicateValueCode(code) => (
                    StatusCode::CONFLICT,
                    Json(json!({
                        "error": "duplicate_code",
                        "message": format!("Dimension value code '{}' already exists for this type", code)
                    })),
                )
                    .into_response(),
                zeltra_db::repositories::dimension::DimensionError::TypeNotFound(id) => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "type_not_found",
                        "message": format!("Dimension type not found: {}", id)
                    })),
                )
                    .into_response(),
                zeltra_db::repositories::dimension::DimensionError::ParentNotFound(id) => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "parent_not_found",
                        "message": format!("Parent dimension value not found: {}", id)
                    })),
                )
                    .into_response(),
                zeltra_db::repositories::dimension::DimensionError::ParentWrongType => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "parent_wrong_type",
                        "message": "Parent dimension value belongs to different type"
                    })),
                )
                    .into_response(),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "internal_error",
                        "message": "An error occurred"
                    })),
                )
                    .into_response(),
            }
        }
    }
}

// Helper functions

async fn check_membership(
    org_repo: &OrganizationRepository,
    org_id: Uuid,
    user_id: Uuid,
) -> Result<(), axum::response::Response> {
    match org_repo.is_member(org_id, user_id).await {
        Ok(true) => Ok(()),
        Ok(false) => Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "forbidden",
                "message": "You are not a member of this organization"
            })),
        )
            .into_response()),
        Err(e) => {
            error!(error = %e, "Database error checking membership");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "An error occurred"
                })),
            )
                .into_response())
        }
    }
}

async fn check_admin_role(
    org_repo: &OrganizationRepository,
    org_id: Uuid,
    user_id: Uuid,
) -> Result<(), axum::response::Response> {
    match org_repo.has_role(org_id, user_id, UserRole::Admin).await {
        Ok(true) => Ok(()),
        Ok(false) => Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "forbidden",
                "message": "You need admin or owner role to perform this action"
            })),
        )
            .into_response()),
        Err(e) => {
            error!(error = %e, "Database error checking role");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "An error occurred"
                })),
            )
                .into_response())
        }
    }
}
