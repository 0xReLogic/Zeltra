//! Entity management routes.

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, patch, post},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{error, info};
use uuid::Uuid;

use crate::{AppState, middleware::AuthUser};
use zeltra_db::{
    OrganizationRepository, entities::sea_orm_active_enums::UserRole,
    repositories::entity::EntityRepository,
};

/// Creates the entity routes (requires auth middleware to be applied externally).
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/organizations/{org_id}/entities", get(list_entities))
        .route("/organizations/{org_id}/entities", post(create_entity))
        .route(
            "/organizations/{org_id}/entities/{entity_id}",
            get(get_entity),
        )
        .route(
            "/organizations/{org_id}/entities/{entity_id}",
            patch(update_entity),
        )
        .route(
            "/organizations/{org_id}/entities/{entity_id}",
            delete(delete_entity),
        )
}

/// Request body for creating an entity.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateEntityRequest {
    /// Entity name (must be unique within organization).
    #[schema(example = "Acme Corp Main")]
    pub name: String,
    /// Legal entity name.
    #[schema(example = "Acme Corporation Inc.")]
    pub legal_name: Option<String>,
    /// Tax identification number (EIN, VAT, etc.).
    #[schema(example = "12-3456789")]
    pub tax_id: Option<String>,
    /// Entity type: main, subsidiary, branch, division.
    #[schema(example = "main")]
    pub entity_type: String,
    /// Base currency code (ISO 4217).
    #[schema(example = "USD")]
    pub base_currency: String,
}

/// Request body for updating an entity.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateEntityRequest {
    /// Entity name.
    pub name: Option<String>,
    /// Legal entity name.
    pub legal_name: Option<String>,
    /// Tax identification number.
    pub tax_id: Option<String>,
    /// Entity type.
    pub entity_type: Option<String>,
    /// Base currency code.
    pub base_currency: Option<String>,
}

/// Response for an entity.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct EntityResponse {
    /// Entity ID.
    pub id: Uuid,
    /// Organization ID.
    pub organization_id: Uuid,
    /// Entity name.
    pub name: String,
    /// Legal entity name.
    pub legal_name: Option<String>,
    /// Tax identification number.
    pub tax_id: Option<String>,
    /// Entity type.
    pub entity_type: String,
    /// Base currency code.
    pub base_currency: String,
    /// Whether the entity is active.
    pub is_active: bool,
    /// Entity-specific settings.
    pub settings: serde_json::Value,
    /// Creation timestamp.
    pub created_at: String,
    /// Last update timestamp.
    pub updated_at: String,
}

/// Response wrapper for list entities endpoint.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct GetEntitiesResponse {
    /// List of entities.
    pub entities: Vec<EntityResponse>,
}

/// Helper function to check organization membership.
async fn check_membership(
    org_repo: &OrganizationRepository,
    org_id: Uuid,
    user_id: Uuid,
) -> Result<(), impl IntoResponse> {
    match org_repo.get_user_role(org_id, user_id).await {
        Ok(Some(_)) => Ok(()),
        Ok(None) => Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "FORBIDDEN",
                "message": "You don't have access to this organization"
            })),
        )),
        Err(e) => {
            error!("Failed to check organization membership: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "INTERNAL_ERROR",
                    "message": "Failed to verify organization access"
                })),
            ))
        }
    }
}

/// GET `/organizations/{org_id}/entities` - List entities for an organization.
#[utoipa::path(
    get,
    path = "/organizations/{org_id}/entities",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
    ),
    responses(
        (status = 200, description = "List of entities", body = GetEntitiesResponse),
        (status = 403, description = "Forbidden")
    ),
    tag = "Entities",
    security(("bearerAuth" = []))
)]
pub async fn list_entities(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_id): Path<Uuid>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let entity_repo = EntityRepository::new((*state.db).clone());

    // List entities
    match entity_repo.list_by_organization(org_id).await {
        Ok(entities) => {
            let entity_responses: Vec<EntityResponse> = entities
                .into_iter()
                .map(|e| EntityResponse {
                    id: e.id,
                    organization_id: e.organization_id,
                    name: e.name,
                    legal_name: e.legal_name,
                    tax_id: e.tax_id,
                    entity_type: e.entity_type,
                    base_currency: e.base_currency,
                    is_active: e.is_active,
                    settings: e.settings,
                    created_at: e.created_at.to_string(),
                    updated_at: e.updated_at.to_string(),
                })
                .collect();

            (
                StatusCode::OK,
                Json(GetEntitiesResponse {
                    entities: entity_responses,
                }),
            )
                .into_response()
        }
        Err(e) => {
            error!("Failed to list entities: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "INTERNAL_ERROR",
                    "message": "Failed to list entities"
                })),
            )
                .into_response()
        }
    }
}

/// POST `/organizations/{org_id}/entities` - Create a new entity.
#[utoipa::path(
    post,
    path = "/organizations/{org_id}/entities",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
    ),
    request_body = CreateEntityRequest,
    responses(
        (status = 201, description = "Entity created successfully", body = EntityResponse),
        (status = 400, description = "Bad request - validation error or entity limit exceeded"),
        (status = 403, description = "Forbidden")
    ),
    tag = "Entities",
    security(("bearerAuth" = []))
)]
pub async fn create_entity(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_id): Path<Uuid>,
    Json(req): Json<CreateEntityRequest>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let entity_repo = EntityRepository::new((*state.db).clone());

    // Create entity (includes tier limit validation)
    match entity_repo
        .create(
            org_id,
            req.name,
            req.base_currency,
            req.entity_type,
            req.legal_name,
            req.tax_id,
        )
        .await
    {
        Ok(entity) => {
            info!("Entity created: {} for organization {}", entity.id, org_id);
            (
                StatusCode::CREATED,
                Json(EntityResponse {
                    id: entity.id,
                    organization_id: entity.organization_id,
                    name: entity.name,
                    legal_name: entity.legal_name,
                    tax_id: entity.tax_id,
                    entity_type: entity.entity_type,
                    base_currency: entity.base_currency,
                    is_active: entity.is_active,
                    settings: entity.settings,
                    created_at: entity.created_at.to_string(),
                    updated_at: entity.updated_at.to_string(),
                }),
            )
                .into_response()
        }
        Err(e) => {
            let error_msg = e.to_string();

            // Check for specific error types
            if error_msg.contains("Entity limit reached") {
                error!("Entity limit exceeded for organization {}: {}", org_id, e);
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "ENTITY_LIMIT_EXCEEDED",
                        "message": "Entity limit reached for your tier. Upgrade to create more entities."
                    })),
                )
                    .into_response()
            } else if error_msg.contains("duplicate") || error_msg.contains("unique") {
                error!("Duplicate entity name for organization {}: {}", org_id, e);
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "DUPLICATE_ENTITY_NAME",
                        "message": "An entity with this name already exists in your organization"
                    })),
                )
                    .into_response()
            } else {
                error!("Failed to create entity: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "INTERNAL_ERROR",
                        "message": "Failed to create entity"
                    })),
                )
                    .into_response()
            }
        }
    }
}

/// GET `/organizations/{org_id}/entities/{entity_id}` - Get entity details.
#[utoipa::path(
    get,
    path = "/organizations/{org_id}/entities/{entity_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("entity_id" = Uuid, Path, description = "Entity ID"),
    ),
    responses(
        (status = 200, description = "Entity details", body = EntityResponse),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Entity not found")
    ),
    tag = "Entities",
    security(("bearerAuth" = []))
)]
pub async fn get_entity(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_id, entity_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let entity_repo = EntityRepository::new((*state.db).clone());

    // Get entity
    match entity_repo.find_by_id(entity_id).await {
        Ok(Some(entity)) => {
            // Verify entity belongs to organization
            if entity.organization_id != org_id {
                return (
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "error": "FORBIDDEN",
                        "message": "Entity does not belong to this organization"
                    })),
                )
                    .into_response();
            }

            (
                StatusCode::OK,
                Json(EntityResponse {
                    id: entity.id,
                    organization_id: entity.organization_id,
                    name: entity.name,
                    legal_name: entity.legal_name,
                    tax_id: entity.tax_id,
                    entity_type: entity.entity_type,
                    base_currency: entity.base_currency,
                    is_active: entity.is_active,
                    settings: entity.settings,
                    created_at: entity.created_at.to_string(),
                    updated_at: entity.updated_at.to_string(),
                }),
            )
                .into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "ENTITY_NOT_FOUND",
                "message": "Entity not found"
            })),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to get entity: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "INTERNAL_ERROR",
                    "message": "Failed to get entity"
                })),
            )
                .into_response()
        }
    }
}

/// PATCH `/organizations/{org_id}/entities/{entity_id}` - Update entity.
#[utoipa::path(
    patch,
    path = "/organizations/{org_id}/entities/{entity_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("entity_id" = Uuid, Path, description = "Entity ID"),
    ),
    request_body = UpdateEntityRequest,
    responses(
        (status = 200, description = "Entity updated successfully", body = EntityResponse),
        (status = 400, description = "Bad request - validation error"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Entity not found")
    ),
    tag = "Entities",
    security(("bearerAuth" = []))
)]
pub async fn update_entity(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_id, entity_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateEntityRequest>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let entity_repo = EntityRepository::new((*state.db).clone());

    // Verify entity exists and belongs to organization
    match entity_repo.find_by_id(entity_id).await {
        Ok(Some(entity)) => {
            if entity.organization_id != org_id {
                return (
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "error": "FORBIDDEN",
                        "message": "Entity does not belong to this organization"
                    })),
                )
                    .into_response();
            }
        }
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "ENTITY_NOT_FOUND",
                    "message": "Entity not found"
                })),
            )
                .into_response();
        }
        Err(e) => {
            error!("Failed to verify entity: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "INTERNAL_ERROR",
                    "message": "Failed to verify entity"
                })),
            )
                .into_response();
        }
    }

    // Update entity
    match entity_repo
        .update(
            entity_id,
            req.name,
            req.legal_name,
            req.tax_id,
            req.entity_type,
            req.base_currency,
        )
        .await
    {
        Ok(entity) => {
            info!("Entity updated: {}", entity_id);
            (
                StatusCode::OK,
                Json(EntityResponse {
                    id: entity.id,
                    organization_id: entity.organization_id,
                    name: entity.name,
                    legal_name: entity.legal_name,
                    tax_id: entity.tax_id,
                    entity_type: entity.entity_type,
                    base_currency: entity.base_currency,
                    is_active: entity.is_active,
                    settings: entity.settings,
                    created_at: entity.created_at.to_string(),
                    updated_at: entity.updated_at.to_string(),
                }),
            )
                .into_response()
        }
        Err(e) => {
            let error_msg = e.to_string();

            if error_msg.contains("duplicate") || error_msg.contains("unique") {
                error!("Duplicate entity name: {}", e);
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "DUPLICATE_ENTITY_NAME",
                        "message": "An entity with this name already exists in your organization"
                    })),
                )
                    .into_response()
            } else {
                error!("Failed to update entity: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "INTERNAL_ERROR",
                        "message": "Failed to update entity"
                    })),
                )
                    .into_response()
            }
        }
    }
}

/// DELETE `/organizations/{org_id}/entities/{entity_id}` - Delete entity (soft delete).
#[utoipa::path(
    delete,
    path = "/organizations/{org_id}/entities/{entity_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("entity_id" = Uuid, Path, description = "Entity ID"),
    ),
    responses(
        (status = 204, description = "Entity deleted successfully"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Entity not found")
    ),
    tag = "Entities",
    security(("bearerAuth" = []))
)]
pub async fn delete_entity(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_id, entity_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let entity_repo = EntityRepository::new((*state.db).clone());

    // Verify entity exists and belongs to organization
    match entity_repo.find_by_id(entity_id).await {
        Ok(Some(entity)) => {
            if entity.organization_id != org_id {
                return (
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "error": "FORBIDDEN",
                        "message": "Entity does not belong to this organization"
                    })),
                )
                    .into_response();
            }
        }
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "ENTITY_NOT_FOUND",
                    "message": "Entity not found"
                })),
            )
                .into_response();
        }
        Err(e) => {
            error!("Failed to verify entity: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "INTERNAL_ERROR",
                    "message": "Failed to verify entity"
                })),
            )
                .into_response();
        }
    }

    // Delete entity (soft delete)
    match entity_repo.delete(entity_id).await {
        Ok(_) => {
            info!("Entity deleted: {}", entity_id);
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => {
            error!("Failed to delete entity: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "INTERNAL_ERROR",
                    "message": "Failed to delete entity"
                })),
            )
                .into_response()
        }
    }
}
