//! Fiscal year and period management routes.

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch, post},
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{error, info};
use uuid::Uuid;

use crate::{AppState, middleware::AuthUser};
use zeltra_db::{
    OrganizationRepository,
    entities::sea_orm_active_enums::{FiscalPeriodStatus, UserRole},
    repositories::fiscal::{CreateFiscalYearInput, FiscalRepository},
};

/// Creates the fiscal routes (requires auth middleware to be applied externally).
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/organizations/{org_id}/fiscal-years", get(list_fiscal_years))
        .route("/organizations/{org_id}/fiscal-years", post(create_fiscal_year))
        .route("/organizations/{org_id}/fiscal-periods/{period_id}/status", patch(update_period_status))
}

/// Request body for creating a fiscal year.
#[derive(Debug, Deserialize)]
pub struct CreateFiscalYearRequest {
    /// Fiscal year name (e.g., "FY 2026").
    pub name: String,
    /// Start date (YYYY-MM-DD).
    pub start_date: NaiveDate,
    /// End date (YYYY-MM-DD).
    pub end_date: NaiveDate,
}

/// Request body for updating period status.
#[derive(Debug, Deserialize)]
pub struct UpdatePeriodStatusRequest {
    /// New status: "open", "soft_close", or "closed".
    pub status: String,
}

/// Response for a fiscal period.
#[derive(Debug, Serialize)]
pub struct FiscalPeriodResponse {
    /// Period ID.
    pub id: Uuid,
    /// Period name.
    pub name: String,
    /// Period number within the fiscal year.
    pub period_number: i16,
    /// Start date.
    pub start_date: NaiveDate,
    /// End date.
    pub end_date: NaiveDate,
    /// Status: open, soft_close, or closed.
    pub status: String,
    /// Whether this is an adjustment period.
    pub is_adjustment_period: bool,
}

/// Response for a fiscal year with periods.
#[derive(Debug, Serialize)]
pub struct FiscalYearResponse {
    /// Fiscal year ID.
    pub id: Uuid,
    /// Fiscal year name.
    pub name: String,
    /// Start date.
    pub start_date: NaiveDate,
    /// End date.
    pub end_date: NaiveDate,
    /// Status: open or closed.
    pub status: String,
    /// Nested periods.
    pub periods: Vec<FiscalPeriodResponse>,
}

/// GET `/organizations/{org_id}/fiscal-years` - List fiscal years with nested periods.
async fn list_fiscal_years(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_id): Path<Uuid>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let fiscal_repo = FiscalRepository::new((*state.db).clone());

    match fiscal_repo.list_fiscal_years(org_id).await {
        Ok(years) => {
            let response: Vec<FiscalYearResponse> = years
                .into_iter()
                .map(|fy| FiscalYearResponse {
                    id: fy.fiscal_year.id,
                    name: fy.fiscal_year.name,
                    start_date: fy.fiscal_year.start_date,
                    end_date: fy.fiscal_year.end_date,
                    status: fiscal_year_status_to_string(&fy.fiscal_year.status),
                    periods: fy
                        .periods
                        .into_iter()
                        .map(|p| FiscalPeriodResponse {
                            id: p.id,
                            name: p.name,
                            period_number: p.period_number,
                            start_date: p.start_date,
                            end_date: p.end_date,
                            status: period_status_to_string(&p.status),
                            is_adjustment_period: p.is_adjustment_period,
                        })
                        .collect(),
                })
                .collect();

            (StatusCode::OK, Json(json!({ "fiscal_years": response }))).into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to list fiscal years");
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

/// POST `/organizations/{org_id}/fiscal-years` - Create a fiscal year with auto-generated periods.
async fn create_fiscal_year(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_id): Path<Uuid>,
    Json(payload): Json<CreateFiscalYearRequest>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check admin/owner role
    if let Err(response) = check_admin_role(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let fiscal_repo = FiscalRepository::new((*state.db).clone());

    let input = CreateFiscalYearInput {
        organization_id: org_id,
        name: payload.name,
        start_date: payload.start_date,
        end_date: payload.end_date,
    };

    match fiscal_repo.create_fiscal_year(input).await {
        Ok(fy) => {
            info!(
                org_id = %org_id,
                fiscal_year_id = %fy.fiscal_year.id,
                "Fiscal year created"
            );

            let response = FiscalYearResponse {
                id: fy.fiscal_year.id,
                name: fy.fiscal_year.name,
                start_date: fy.fiscal_year.start_date,
                end_date: fy.fiscal_year.end_date,
                status: fiscal_year_status_to_string(&fy.fiscal_year.status),
                periods: fy
                    .periods
                    .into_iter()
                    .map(|p| FiscalPeriodResponse {
                        id: p.id,
                        name: p.name,
                        period_number: p.period_number,
                        start_date: p.start_date,
                        end_date: p.end_date,
                        status: period_status_to_string(&p.status),
                        is_adjustment_period: p.is_adjustment_period,
                    })
                    .collect(),
            };

            (StatusCode::CREATED, Json(json!(response))).into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to create fiscal year");
            match e {
                zeltra_db::repositories::fiscal::FiscalError::InvalidDateRange => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "invalid_date_range",
                        "message": "Start date must be before end date"
                    })),
                )
                    .into_response(),
                zeltra_db::repositories::fiscal::FiscalError::OverlappingYear(name) => (
                    StatusCode::CONFLICT,
                    Json(json!({
                        "error": "overlapping_year",
                        "message": format!("Fiscal year overlaps with existing year: {}", name)
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

/// PATCH `/organizations/{org_id}/fiscal-periods/{period_id}/status` - Update period status.
async fn update_period_status(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_id, period_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdatePeriodStatusRequest>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check admin/owner role
    if let Err(response) = check_admin_role(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let fiscal_repo = FiscalRepository::new((*state.db).clone());

    // Verify period belongs to this organization
    match fiscal_repo.find_period_by_id(period_id).await {
        Ok(Some(p)) if p.organization_id == org_id => {}
        Ok(Some(_)) => {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({
                    "error": "forbidden",
                    "message": "Period does not belong to this organization"
                })),
            )
                .into_response();
        }
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "not_found",
                    "message": "Fiscal period not found"
                })),
            )
                .into_response();
        }
        Err(e) => {
            error!(error = %e, "Database error finding period");
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

    // Parse status
    let Some(new_status) = string_to_period_status(&payload.status) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_status",
                "message": "Invalid status. Must be one of: open, soft_close, closed"
            })),
        )
            .into_response();
    };

    // Determine closed_by
    let closed_by = if matches!(new_status, FiscalPeriodStatus::SoftClose | FiscalPeriodStatus::Closed) {
        Some(auth.user_id())
    } else {
        None
    };

    match fiscal_repo.update_period_status(period_id, new_status, closed_by).await {
        Ok(updated) => {
            info!(
                org_id = %org_id,
                period_id = %period_id,
                new_status = %payload.status,
                "Fiscal period status updated"
            );

            (
                StatusCode::OK,
                Json(json!({
                    "id": updated.id,
                    "name": updated.name,
                    "period_number": updated.period_number,
                    "start_date": updated.start_date,
                    "end_date": updated.end_date,
                    "status": period_status_to_string(&updated.status),
                    "is_adjustment_period": updated.is_adjustment_period,
                    "closed_by": updated.closed_by,
                    "closed_at": updated.closed_at
                })),
            )
                .into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to update period status");
            match e {
                zeltra_db::repositories::fiscal::FiscalError::EarlierPeriodsOpen => (
                    StatusCode::CONFLICT,
                    Json(json!({
                        "error": "earlier_periods_open",
                        "message": "Cannot close period: earlier periods must be closed first"
                    })),
                )
                    .into_response(),
                zeltra_db::repositories::fiscal::FiscalError::InvalidStatusTransition { from, to } => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "invalid_status_transition",
                        "message": format!("Invalid status transition from {:?} to {:?}", from, to)
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

fn fiscal_year_status_to_string(status: &zeltra_db::entities::sea_orm_active_enums::FiscalYearStatus) -> String {
    match status {
        zeltra_db::entities::sea_orm_active_enums::FiscalYearStatus::Open => "open".to_string(),
        zeltra_db::entities::sea_orm_active_enums::FiscalYearStatus::Closed => "closed".to_string(),
    }
}

fn period_status_to_string(status: &FiscalPeriodStatus) -> String {
    match status {
        FiscalPeriodStatus::Open => "open".to_string(),
        FiscalPeriodStatus::SoftClose => "soft_close".to_string(),
        FiscalPeriodStatus::Closed => "closed".to_string(),
    }
}

fn string_to_period_status(s: &str) -> Option<FiscalPeriodStatus> {
    match s.to_lowercase().as_str() {
        "open" => Some(FiscalPeriodStatus::Open),
        "soft_close" => Some(FiscalPeriodStatus::SoftClose),
        "closed" => Some(FiscalPeriodStatus::Closed),
        _ => None,
    }
}
