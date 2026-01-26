//! Sentinel Intelligence API routes.
//!
//! Implements Phase 2b endpoints for revaluation, accruals, and intercompany operations.

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::str::FromStr;
use tracing::{error, info};
use uuid::Uuid;

use crate::{AppState, middleware::AuthUser};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use zeltra_core::ledger::types::AccrualFrequency;
use zeltra_db::{
    OrganizationRepository,
    entities::{accrual_schedules, intercompany_mappings, revaluation_logs},
    repositories::{
        accrual::{AccrualError, AccrualRepository, CreateAccrualScheduleInput},
        intercompany::{IntercompanyError, IntercompanyRepository},
    },
};

/// Creates the sentinel routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/organizations/{org_id}/revaluation-logs",
            get(list_revaluation_logs),
        )
        .route(
            "/organizations/{org_id}/accrual-schedules",
            get(list_accrual_schedules).post(create_accrual_schedule),
        )
        .route(
            "/organizations/{org_id}/accrual-schedules/{schedule_id}",
            get(get_accrual_schedule),
        )
        .route(
            "/organizations/{org_id}/intercompany/mappings",
            get(list_intercompany_mappings),
        )
        .route(
            "/organizations/{org_id}/intercompany/connect",
            post(create_intercompany_mapping),
        )
}

// ============================================================================
// Request/Response Types
// ============================================================================

/// Request body for creating an accrual schedule.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateAccrualScheduleRequest {
    /// Name of the accrual schedule.
    #[schema(example = "Prepaid Insurance")]
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Total amount to accrue.
    #[schema(example = "12000.00")]
    pub total_amount: String,
    /// Currency ID (ISO code).
    #[schema(example = "USD")]
    pub currency_id: String,
    /// Debit account ID.
    pub debit_account_id: Uuid,
    /// Credit account ID.
    pub credit_account_id: Uuid,
    /// Start date (YYYY-MM-DD).
    #[schema(example = "2026-01-01")]
    pub start_date: String,
    /// End date (YYYY-MM-DD).
    #[schema(example = "2026-12-31")]
    pub end_date: String,
    /// Frequency: daily, weekly, monthly, quarterly, yearly.
    #[schema(example = "monthly")]
    pub frequency: String,
    /// Total number of periods.
    #[schema(example = 12)]
    pub total_periods: i32,
}

/// Response for an accrual schedule.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct AccrualScheduleResponse {
    /// Schedule ID.
    pub id: Uuid,
    /// Organization ID.
    pub organization_id: Uuid,
    /// Name.
    pub name: String,
    /// Description.
    pub description: Option<String>,
    /// Total amount.
    pub total_amount: String,
    /// Currency ID.
    pub currency_id: String,
    /// Debit account ID.
    pub debit_account_id: Uuid,
    /// Credit account ID.
    pub credit_account_id: Uuid,
    /// Start date.
    pub start_date: String,
    /// End date.
    pub end_date: String,
    /// Frequency.
    pub frequency: String,
    /// Total periods.
    pub total_periods: i32,
    /// Periods processed.
    pub periods_processed: i32,
    /// Status.
    pub status: String,
    /// Next run date.
    pub next_run_date: Option<String>,
    /// Created at.
    pub created_at: String,
}

/// Response for a revaluation log.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RevaluationLogResponse {
    /// Log ID.
    pub id: Uuid,
    /// Account ID.
    pub account_id: Uuid,
    /// Organization ID.
    pub organization_id: Uuid,
    /// Revaluation date.
    pub revaluation_date: String,
    /// Account currency.
    pub source_currency: String,
    /// Functional currency.
    pub functional_currency: String,
    /// Exchange rate used.
    pub exchange_rate: String,
    /// Carrying balance before revaluation.
    pub carrying_balance: String,
    /// Revalued balance.
    pub revalued_balance: String,
    /// Gain/loss amount.
    pub gain_loss_amount: String,
    /// Related transaction ID.
    pub transaction_id: Option<Uuid>,
    /// Created at.
    pub created_at: String,
}

/// Response for an intercompany mapping.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct IntercompanyMappingResponse {
    /// Mapping ID.
    pub id: Uuid,
    /// Source entity ID.
    pub source_entity_id: Uuid,
    /// Source account ID.
    pub source_account_id: Uuid,
    /// Target entity ID.
    pub target_entity_id: Uuid,
    /// Target account ID.
    pub target_account_id: Uuid,
    /// Whether to auto-post transactions.
    pub auto_post: bool,
    /// Mapping type.
    pub mapping_type: String,
    /// Created at.
    pub created_at: String,
}

/// Request body for creating an intercompany mapping.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateIntercompanyMappingRequest {
    /// Source account ID (in source entity).
    pub source_account_id: Uuid,
    /// Source entity ID.
    pub source_entity_id: Uuid,
    /// Target entity ID (must be in same organization).
    pub target_entity_id: Uuid,
    /// Target account ID (in target entity).
    pub target_account_id: Uuid,
}

// ============================================================================
// Route Handlers
// ============================================================================

/// GET `/organizations/{org_id}/revaluation-logs` - List revaluation logs.
#[utoipa::path(
    get,
    path = "/organizations/{org_id}/revaluation-logs",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID")
    ),
    responses(
        (status = 200, description = "List of revaluation logs", body = [RevaluationLogResponse]),
        (status = 403, description = "Forbidden")
    ),
    tag = "Sentinel",
    security(("bearerAuth" = []))
)]
async fn list_revaluation_logs(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_id): Path<Uuid>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    if let Err(response) = check_tier_feature(&org_repo, org_id, "has_multi_currency").await {
        return response;
    }

    // Query revaluation logs directly

    let logs = revaluation_logs::Entity::find()
        .filter(revaluation_logs::Column::OrganizationId.eq(org_id))
        .order_by_desc(revaluation_logs::Column::CreatedAt)
        .all(&*state.db)
        .await;

    match logs {
        Ok(logs) => {
            let items: Vec<RevaluationLogResponse> = logs
                .into_iter()
                .map(|log| RevaluationLogResponse {
                    id: log.id,
                    account_id: log.account_id,
                    organization_id: log.organization_id,
                    revaluation_date: log.revaluation_date.to_string(),
                    source_currency: log.currency_id.clone(),
                    functional_currency: "N/A".to_string(), // Not stored in entity
                    exchange_rate: log.new_exchange_rate.to_string(),
                    carrying_balance: log.balance_in_currency.to_string(),
                    revalued_balance: (log.balance_in_currency * log.new_exchange_rate).to_string(),
                    gain_loss_amount: log.unrealized_gain_loss.to_string(),
                    transaction_id: log.transaction_id,
                    created_at: log.created_at.to_rfc3339(),
                })
                .collect();

            (StatusCode::OK, Json(json!({ "data": items }))).into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to list revaluation logs");
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

/// GET `/organizations/{org_id}/accrual-schedules` - List accrual schedules.
#[utoipa::path(
    get,
    path = "/organizations/{org_id}/accrual-schedules",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID")
    ),
    responses(
        (status = 200, description = "List of accrual schedules", body = [AccrualScheduleResponse]),
        (status = 403, description = "Forbidden")
    ),
    tag = "Sentinel",
    security(("bearerAuth" = []))
)]
async fn list_accrual_schedules(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_id): Path<Uuid>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let schedules = accrual_schedules::Entity::find()
        .filter(accrual_schedules::Column::OrganizationId.eq(org_id))
        .order_by_desc(accrual_schedules::Column::CreatedAt)
        .all(&*state.db)
        .await;

    match schedules {
        Ok(schedules) => {
            let items: Vec<AccrualScheduleResponse> =
                schedules.into_iter().map(schedule_to_response).collect();

            (StatusCode::OK, Json(json!({ "data": items }))).into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to list accrual schedules");
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

/// POST `/organizations/{org_id}/accrual-schedules` - Create accrual schedule.
#[utoipa::path(
    post,
    path = "/organizations/{org_id}/accrual-schedules",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID")
    ),
    request_body = CreateAccrualScheduleRequest,
    responses(
        (status = 201, description = "Accrual schedule created", body = AccrualScheduleResponse),
        (status = 400, description = "Invalid input"),
        (status = 403, description = "Forbidden")
    ),
    tag = "Sentinel",
    security(("bearerAuth" = []))
)]
async fn create_accrual_schedule(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_id): Path<Uuid>,
    Json(payload): Json<CreateAccrualScheduleRequest>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    if let Err(response) = check_admin_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    if let Err(response) = check_tier_feature(&org_repo, org_id, "has_auto_accruals").await {
        return response;
    }

    // Parse amount
    let total_amount = match Decimal::from_str(&payload.total_amount) {
        Ok(a) if a > Decimal::ZERO => a,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "invalid_amount",
                    "message": "Total amount must be a positive number"
                })),
            )
                .into_response();
        }
    };

    // Parse dates
    let Ok(start_date) = NaiveDate::parse_from_str(&payload.start_date, "%Y-%m-%d") else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_date",
                "message": "Invalid start_date format. Use YYYY-MM-DD"
            })),
        )
            .into_response();
    };

    let Ok(end_date) = NaiveDate::parse_from_str(&payload.end_date, "%Y-%m-%d") else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_date",
                "message": "Invalid end_date format. Use YYYY-MM-DD"
            })),
        )
            .into_response();
    };

    if end_date <= start_date {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_date_range",
                "message": "end_date must be after start_date"
            })),
        )
            .into_response();
    }

    // Parse frequency
    let Ok(frequency) = AccrualFrequency::from_str(&payload.frequency) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_frequency",
                "message": "Frequency must be one of: daily, weekly, monthly, quarterly, yearly"
            })),
        )
            .into_response();
    };

    let repo = AccrualRepository::new((*state.db).clone());

    let input = CreateAccrualScheduleInput {
        organization_id: org_id,
        name: payload.name,
        description: payload.description,
        total_amount,
        currency_id: payload.currency_id,
        debit_account_id: payload.debit_account_id,
        credit_account_id: payload.credit_account_id,
        start_date,
        end_date,
        frequency,
        total_periods: payload.total_periods,
        next_run_date: Some(start_date),
    };

    match repo.create_schedule(input).await {
        Ok(schedule) => {
            info!(
                org_id = %org_id,
                schedule_id = %schedule.id,
                "Accrual schedule created"
            );

            (StatusCode::CREATED, Json(schedule_to_response(schedule))).into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to create accrual schedule");
            accrual_error_response(&e)
        }
    }
}

/// GET `/organizations/{org_id}/accrual-schedules/{schedule_id}` - Get accrual schedule.
#[utoipa::path(
    get,
    path = "/organizations/{org_id}/accrual-schedules/{schedule_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("schedule_id" = Uuid, Path, description = "Schedule ID")
    ),
    responses(
        (status = 200, description = "Accrual schedule details", body = AccrualScheduleResponse),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Schedule not found")
    ),
    tag = "Sentinel",
    security(("bearerAuth" = []))
)]
async fn get_accrual_schedule(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_id, schedule_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let schedule = accrual_schedules::Entity::find()
        .filter(accrual_schedules::Column::Id.eq(schedule_id))
        .filter(accrual_schedules::Column::OrganizationId.eq(org_id))
        .one(&*state.db)
        .await;

    match schedule {
        Ok(Some(schedule)) => {
            (StatusCode::OK, Json(schedule_to_response(schedule))).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "not_found",
                "message": "Accrual schedule not found"
            })),
        )
            .into_response(),
        Err(e) => {
            error!(error = %e, "Failed to get accrual schedule");
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

/// GET `/organizations/{org_id}/intercompany/mappings` - List intercompany mappings.
#[utoipa::path(
    get,
    path = "/organizations/{org_id}/intercompany/mappings",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID")
    ),
    responses(
        (status = 200, description = "List of intercompany mappings", body = [IntercompanyMappingResponse]),
        (status = 403, description = "Forbidden")
    ),
    tag = "Sentinel",
    security(("bearerAuth" = []))
)]
async fn list_intercompany_mappings(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_id): Path<Uuid>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    // Query all mappings for entities in this organization
    let mappings = intercompany_mappings::Entity::find()
        .filter(
            intercompany_mappings::Column::SourceEntityId.in_subquery(
                zeltra_db::entities::entities::Entity::find()
                    .filter(zeltra_db::entities::entities::Column::OrganizationId.eq(org_id))
                    .select_only()
                    .column(zeltra_db::entities::entities::Column::Id)
                    .into_query(),
            ),
        )
        .all(&*state.db)
        .await;

    match mappings {
        Ok(mappings) => {
            let items: Vec<IntercompanyMappingResponse> = mappings
                .into_iter()
                .map(|m| IntercompanyMappingResponse {
                    id: m.id,
                    source_entity_id: m.source_entity_id,
                    source_account_id: m.source_account_id,
                    target_entity_id: m.target_entity_id,
                    target_account_id: m.target_account_id,
                    auto_post: m.auto_post,
                    mapping_type: m.mapping_type.clone(),
                    created_at: m.created_at.to_rfc3339(),
                })
                .collect();

            (StatusCode::OK, Json(json!({ "data": items }))).into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to list intercompany mappings");
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

/// POST `/organizations/{org_id}/intercompany/connect` - Create intercompany mapping.
#[utoipa::path(
    post,
    path = "/organizations/{org_id}/intercompany/connect",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID")
    ),
    request_body = CreateIntercompanyMappingRequest,
    responses(
        (status = 201, description = "Intercompany mapping created", body = IntercompanyMappingResponse),
        (status = 400, description = "Invalid input"),
        (status = 403, description = "Forbidden")
    ),
    tag = "Sentinel",
    security(("bearerAuth" = []))
)]
async fn create_intercompany_mapping(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_id): Path<Uuid>,
    Json(payload): Json<CreateIntercompanyMappingRequest>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    if let Err(response) = check_admin_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    if let Err(response) = check_tier_feature(&org_repo, org_id, "has_intercompany_hub").await {
        return response;
    }

    // Verify both entities belong to the same organization
    let source_entity = zeltra_db::entities::entities::Entity::find_by_id(payload.source_entity_id)
        .one(&*state.db)
        .await;

    let target_entity = zeltra_db::entities::entities::Entity::find_by_id(payload.target_entity_id)
        .one(&*state.db)
        .await;

    match (source_entity, target_entity) {
        (Ok(Some(source)), Ok(Some(target))) => {
            if source.organization_id != org_id || target.organization_id != org_id {
                return (
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "error": "forbidden",
                        "message": "Both entities must belong to the specified organization"
                    })),
                )
                    .into_response();
            }

            if source.organization_id != target.organization_id {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "invalid_mapping",
                        "message": "Intercompany mappings can only be created between entities in the same organization"
                    })),
                )
                    .into_response();
            }
        }
        (Ok(None), _) | (_, Ok(None)) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "entity_not_found",
                    "message": "One or both entities not found"
                })),
            )
                .into_response();
        }
        (Err(e), _) | (_, Err(e)) => {
            error!(error = %e, "Failed to verify entities");
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

    let mapping = intercompany_mappings::ActiveModel {
        id: Set(Uuid::new_v4()),
        source_entity_id: Set(payload.source_entity_id),
        source_account_id: Set(payload.source_account_id),
        target_entity_id: Set(payload.target_entity_id),
        target_account_id: Set(payload.target_account_id),
        mapping_type: Set("mirror".to_string()),
        auto_post: Set(true),
        created_at: Set(chrono::Utc::now().fixed_offset()),
        updated_at: Set(chrono::Utc::now().fixed_offset()),
    };

    match mapping.insert(&*state.db).await {
        Ok(m) => {
            info!(
                source_entity_id = %payload.source_entity_id,
                target_entity_id = %payload.target_entity_id,
                mapping_id = %m.id,
                "Intercompany mapping created"
            );

            (
                StatusCode::CREATED,
                Json(IntercompanyMappingResponse {
                    id: m.id,
                    source_entity_id: m.source_entity_id,
                    source_account_id: m.source_account_id,
                    target_entity_id: m.target_entity_id,
                    target_account_id: m.target_account_id,
                    auto_post: m.auto_post,
                    mapping_type: m.mapping_type,
                    created_at: m.created_at.to_rfc3339(),
                }),
            )
                .into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to create intercompany mapping");
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

// ============================================================================
// Helper Functions
// ============================================================================

fn schedule_to_response(
    schedule: zeltra_db::entities::accrual_schedules::Model,
) -> AccrualScheduleResponse {
    AccrualScheduleResponse {
        id: schedule.id,
        organization_id: schedule.organization_id,
        name: schedule.name,
        description: schedule.description,
        total_amount: schedule.total_amount.to_string(),
        currency_id: schedule.currency_id,
        debit_account_id: schedule.debit_account_id,
        credit_account_id: schedule.credit_account_id,
        start_date: schedule.start_date.to_string(),
        end_date: schedule.end_date.to_string(),
        frequency: schedule.frequency,
        total_periods: schedule.total_periods,
        periods_processed: schedule.periods_processed,
        status: schedule.status,
        next_run_date: schedule.next_run_date.map(|d| d.to_string()),
        created_at: schedule.created_at.to_rfc3339(),
    }
}

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

async fn check_admin_membership(
    org_repo: &OrganizationRepository,
    org_id: Uuid,
    user_id: Uuid,
) -> Result<(), axum::response::Response> {
    match org_repo.get_user_membership(org_id, user_id).await {
        Ok(Some(membership)) => {
            use zeltra_db::entities::sea_orm_active_enums::UserRole;
            match membership.role {
                UserRole::Admin | UserRole::Owner => Ok(()),
                _ => Err((
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "error": "admin_required",
                        "message": "Admin or Owner role required for this operation"
                    })),
                )
                    .into_response()),
            }
        }
        Ok(None) => Err((
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

async fn check_tier_feature(
    org_repo: &OrganizationRepository,
    org_id: Uuid,
    feature: &str,
) -> Result<(), axum::response::Response> {
    match org_repo.get_tier_limits(org_id).await {
        Ok(Some(limits)) => {
            let has_access = match feature {
                "has_multi_currency" => limits.has_multi_currency,
                "has_auto_accruals" => limits.has_auto_accruals,
                "has_intercompany_hub" => limits.has_intercompany_hub,
                "has_simulation" => limits.has_simulation,
                _ => false,
            };

            if has_access {
                Ok(())
            } else {
                Err((
                    StatusCode::PAYMENT_REQUIRED,
                    Json(json!({
                        "error": "tier_limit_reached",
                        "message": format!("Your current tier does not include the '{}' feature. Please upgrade to unlock.", feature.replace("has_", "").replace('_', " ")),
                        "feature": feature
                    })),
                )
                    .into_response())
            }
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "organization_not_found",
                "message": "Organization or tier limits not found"
            })),
        )
            .into_response()),
        Err(e) => {
            error!(error = %e, "Database error checking tier limits");
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

fn accrual_error_response(e: &AccrualError) -> axum::response::Response {
    match e {
        AccrualError::NotFound => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "not_found",
                "message": "Accrual schedule not found"
            })),
        )
            .into_response(),
        AccrualError::Database(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "internal_error",
                "message": "An error occurred"
            })),
        )
            .into_response(),
    }
}

fn intercompany_error_response(e: &IntercompanyError) -> axum::response::Response {
    match e {
        IntercompanyError::MappingNotFound => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "not_found",
                "message": "Intercompany mapping not found"
            })),
        )
            .into_response(),
        IntercompanyError::Database(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "internal_error",
                "message": "An error occurred"
            })),
        )
            .into_response(),
    }
}
