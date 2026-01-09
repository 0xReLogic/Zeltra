//! Fiscal year and period repository for database operations.
//!
//! Implements Requirements 1.1-1.7 for fiscal year and period management.

use chrono::{Datelike, NaiveDate};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder,
    Set, TransactionTrait,
};
use uuid::Uuid;

use crate::entities::{
    fiscal_periods, fiscal_years,
    sea_orm_active_enums::{FiscalPeriodStatus, FiscalYearStatus},
};

/// Error types for fiscal operations.
#[derive(Debug, thiserror::Error)]
pub enum FiscalError {
    /// Start date must be before end date.
    #[error("Start date must be before end date")]
    InvalidDateRange,

    /// Fiscal year overlaps with existing year.
    #[error("Fiscal year overlaps with existing year: {0}")]
    OverlappingYear(String),

    /// Fiscal year not found.
    #[error("Fiscal year not found: {0}")]
    YearNotFound(Uuid),

    /// Fiscal period not found.
    #[error("Fiscal period not found: {0}")]
    PeriodNotFound(Uuid),

    /// Cannot close period because earlier periods are still open.
    #[error("Cannot close period: earlier periods must be closed first")]
    EarlierPeriodsOpen,

    /// Invalid status transition.
    #[error("Invalid status transition from {from:?} to {to:?}")]
    InvalidStatusTransition {
        /// Current status.
        from: FiscalPeriodStatus,
        /// Target status.
        to: FiscalPeriodStatus,
    },

    /// Database error.
    #[error("Database error: {0}")]
    Database(#[from] DbErr),
}

/// Fiscal year with nested periods.
#[derive(Debug, Clone)]
pub struct FiscalYearWithPeriods {
    /// The fiscal year record.
    pub fiscal_year: fiscal_years::Model,
    /// The fiscal periods within this year.
    pub periods: Vec<fiscal_periods::Model>,
}

/// Input for creating a fiscal year.
#[derive(Debug, Clone)]
pub struct CreateFiscalYearInput {
    /// Organization ID.
    pub organization_id: Uuid,
    /// Fiscal year name (e.g., "FY 2026").
    pub name: String,
    /// Start date of the fiscal year.
    pub start_date: NaiveDate,
    /// End date of the fiscal year.
    pub end_date: NaiveDate,
}

/// Validates that start_date is strictly before end_date.
///
/// Property 10: Fiscal Year Date Validation (part 1)
/// Validates: Requirements 1.2
pub fn validate_date_range(start_date: NaiveDate, end_date: NaiveDate) -> Result<(), FiscalError> {
    if start_date >= end_date {
        return Err(FiscalError::InvalidDateRange);
    }
    Ok(())
}

/// Checks if two date ranges overlap.
///
/// Property 10: Fiscal Year Date Validation (part 2)
/// Validates: Requirements 1.3
///
/// Two ranges [a_start, a_end] and [b_start, b_end] overlap if:
/// a_start <= b_end AND a_end >= b_start
pub fn date_ranges_overlap(
    a_start: NaiveDate,
    a_end: NaiveDate,
    b_start: NaiveDate,
    b_end: NaiveDate,
) -> bool {
    a_start <= b_end && a_end >= b_start
}

/// Fiscal year and period repository.
#[derive(Debug, Clone)]
pub struct FiscalRepository {
    db: DatabaseConnection,
}

impl FiscalRepository {
    /// Creates a new fiscal repository.
    #[must_use]
    pub const fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Creates a fiscal year with auto-generated monthly periods.
    ///
    /// Requirements: 1.1, 1.2, 1.3
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - start_date >= end_date
    /// - Date range overlaps with existing fiscal year
    /// - Database operation fails
    pub async fn create_fiscal_year(
        &self,
        input: CreateFiscalYearInput,
    ) -> Result<FiscalYearWithPeriods, FiscalError> {
        // Validate date range (Requirement 1.2)
        if input.start_date >= input.end_date {
            return Err(FiscalError::InvalidDateRange);
        }

        // Check for overlapping fiscal years (Requirement 1.3)
        let overlapping = fiscal_years::Entity::find()
            .filter(fiscal_years::Column::OrganizationId.eq(input.organization_id))
            .filter(fiscal_years::Column::StartDate.lte(input.end_date))
            .filter(fiscal_years::Column::EndDate.gte(input.start_date))
            .one(&self.db)
            .await?;

        if let Some(existing) = overlapping {
            return Err(FiscalError::OverlappingYear(existing.name));
        }

        let txn = self.db.begin().await?;
        let now = chrono::Utc::now().into();
        let fiscal_year_id = Uuid::new_v4();

        // Create fiscal year
        let fiscal_year = fiscal_years::ActiveModel {
            id: Set(fiscal_year_id),
            organization_id: Set(input.organization_id),
            name: Set(input.name),
            start_date: Set(input.start_date),
            end_date: Set(input.end_date),
            status: Set(FiscalYearStatus::Open),
            closed_by: Set(None),
            closed_at: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let fiscal_year = fiscal_year.insert(&txn).await?;

        // Auto-generate monthly periods (Requirement 1.1)
        let periods = generate_monthly_periods(
            fiscal_year_id,
            input.organization_id,
            input.start_date,
            input.end_date,
        );

        let mut inserted_periods = Vec::with_capacity(periods.len());
        for period in periods {
            let period_model = fiscal_periods::ActiveModel {
                id: Set(period.id),
                organization_id: Set(period.organization_id),
                fiscal_year_id: Set(period.fiscal_year_id),
                name: Set(period.name),
                period_number: Set(period.period_number),
                start_date: Set(period.start_date),
                end_date: Set(period.end_date),
                status: Set(FiscalPeriodStatus::Open),
                is_adjustment_period: Set(period.is_adjustment_period),
                closed_by: Set(None),
                closed_at: Set(None),
                created_at: Set(now),
                updated_at: Set(now),
            };
            let inserted = period_model.insert(&txn).await?;
            inserted_periods.push(inserted);
        }

        txn.commit().await?;

        Ok(FiscalYearWithPeriods {
            fiscal_year,
            periods: inserted_periods,
        })
    }

    /// Lists fiscal years with nested periods for an organization.
    ///
    /// Requirements: 1.4
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn list_fiscal_years(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<FiscalYearWithPeriods>, FiscalError> {
        let fiscal_years = fiscal_years::Entity::find()
            .filter(fiscal_years::Column::OrganizationId.eq(organization_id))
            .order_by_desc(fiscal_years::Column::StartDate)
            .all(&self.db)
            .await?;

        let mut results = Vec::with_capacity(fiscal_years.len());

        for fy in fiscal_years {
            let periods = fiscal_periods::Entity::find()
                .filter(fiscal_periods::Column::FiscalYearId.eq(fy.id))
                .order_by_asc(fiscal_periods::Column::PeriodNumber)
                .all(&self.db)
                .await?;

            results.push(FiscalYearWithPeriods {
                fiscal_year: fy,
                periods,
            });
        }

        Ok(results)
    }

    /// Finds a fiscal year by ID with its periods.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn find_fiscal_year_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<FiscalYearWithPeriods>, FiscalError> {
        let fiscal_year = fiscal_years::Entity::find_by_id(id).one(&self.db).await?;

        let Some(fy) = fiscal_year else {
            return Ok(None);
        };

        let periods = fiscal_periods::Entity::find()
            .filter(fiscal_periods::Column::FiscalYearId.eq(fy.id))
            .order_by_asc(fiscal_periods::Column::PeriodNumber)
            .all(&self.db)
            .await?;

        Ok(Some(FiscalYearWithPeriods {
            fiscal_year: fy,
            periods,
        }))
    }

    /// Finds the fiscal period containing a specific date.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn find_period_for_date(
        &self,
        organization_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<fiscal_periods::Model>, FiscalError> {
        let period = fiscal_periods::Entity::find()
            .filter(fiscal_periods::Column::OrganizationId.eq(organization_id))
            .filter(fiscal_periods::Column::StartDate.lte(date))
            .filter(fiscal_periods::Column::EndDate.gte(date))
            .one(&self.db)
            .await?;

        Ok(period)
    }

    /// Updates a fiscal period's status.
    ///
    /// Requirements: 1.5, 1.6, 1.7
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Period not found
    /// - Earlier periods are not closed (when closing)
    /// - Invalid status transition
    /// - Database operation fails
    pub async fn update_period_status(
        &self,
        period_id: Uuid,
        new_status: FiscalPeriodStatus,
        closed_by: Option<Uuid>,
    ) -> Result<fiscal_periods::Model, FiscalError> {
        let period = fiscal_periods::Entity::find_by_id(period_id)
            .one(&self.db)
            .await?
            .ok_or(FiscalError::PeriodNotFound(period_id))?;

        // Validate status transition
        validate_status_transition(&period.status, &new_status)?;

        // If closing (SOFT_CLOSE or CLOSED), validate earlier periods are closed (Requirement 1.7)
        if matches!(
            new_status,
            FiscalPeriodStatus::SoftClose | FiscalPeriodStatus::Closed
        ) {
            let earlier_open = fiscal_periods::Entity::find()
                .filter(fiscal_periods::Column::FiscalYearId.eq(period.fiscal_year_id))
                .filter(fiscal_periods::Column::PeriodNumber.lt(period.period_number))
                .filter(fiscal_periods::Column::Status.eq(FiscalPeriodStatus::Open))
                .one(&self.db)
                .await?;

            if earlier_open.is_some() {
                return Err(FiscalError::EarlierPeriodsOpen);
            }
        }

        let now = chrono::Utc::now().into();
        let mut active: fiscal_periods::ActiveModel = period.into();

        active.status = Set(new_status.clone());
        active.updated_at = Set(now);

        if matches!(
            new_status,
            FiscalPeriodStatus::SoftClose | FiscalPeriodStatus::Closed
        ) {
            active.closed_by = Set(closed_by);
            active.closed_at = Set(Some(now));
        }

        let updated = active.update(&self.db).await?;
        Ok(updated)
    }

    /// Finds a fiscal period by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn find_period_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<fiscal_periods::Model>, FiscalError> {
        let period = fiscal_periods::Entity::find_by_id(id).one(&self.db).await?;
        Ok(period)
    }
}

/// Validates fiscal period status transitions.
fn validate_status_transition(
    from: &FiscalPeriodStatus,
    to: &FiscalPeriodStatus,
) -> Result<(), FiscalError> {
    let valid = match (from, to) {
        // Same status is a no-op - always valid
        _ if from == to => true,
        // Can go from Open to SoftClose or Closed, or from SoftClose to Closed or back to Open
        (FiscalPeriodStatus::Open, FiscalPeriodStatus::SoftClose | FiscalPeriodStatus::Closed)
        | (FiscalPeriodStatus::SoftClose, FiscalPeriodStatus::Closed | FiscalPeriodStatus::Open) => {
            true
        }
        // Cannot change from Closed (immutable) and all other transitions are invalid
        _ => false,
    };

    if valid {
        Ok(())
    } else {
        Err(FiscalError::InvalidStatusTransition {
            from: from.clone(),
            to: to.clone(),
        })
    }
}

/// Period data for generation.
struct PeriodData {
    id: Uuid,
    organization_id: Uuid,
    fiscal_year_id: Uuid,
    name: String,
    period_number: i16,
    start_date: NaiveDate,
    end_date: NaiveDate,
    is_adjustment_period: bool,
}

/// Generates monthly periods for a fiscal year.
fn generate_monthly_periods(
    fiscal_year_id: Uuid,
    organization_id: Uuid,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Vec<PeriodData> {
    let mut periods = Vec::new();
    let mut current = start_date;
    let mut period_number: i16 = 1;

    while current <= end_date {
        // Calculate period end (last day of month or fiscal year end)
        let month_end = last_day_of_month(current.year(), current.month());
        let period_end = if month_end > end_date {
            end_date
        } else {
            month_end
        };

        let name = format!("{} {}", month_name(current.month()), current.year());

        periods.push(PeriodData {
            id: Uuid::new_v4(),
            organization_id,
            fiscal_year_id,
            name,
            period_number,
            start_date: current,
            end_date: period_end,
            is_adjustment_period: false,
        });

        // Move to first day of next month
        current = if current.month() == 12 {
            NaiveDate::from_ymd_opt(current.year() + 1, 1, 1).unwrap()
        } else {
            NaiveDate::from_ymd_opt(current.year(), current.month() + 1, 1).unwrap()
        };
        period_number += 1;
    }

    periods
}

/// Returns the last day of a month.
fn last_day_of_month(year: i32, month: u32) -> NaiveDate {
    let next_month = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    };

    next_month
        .unwrap()
        .pred_opt()
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(year, month, 28).unwrap())
}

/// Returns month name.
fn month_name(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_monthly_periods_full_year() {
        let start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 12, 31).unwrap();
        let org_id = Uuid::new_v4();
        let fy_id = Uuid::new_v4();

        let periods = generate_monthly_periods(fy_id, org_id, start, end);

        assert_eq!(periods.len(), 12);
        assert_eq!(periods[0].name, "January 2026");
        assert_eq!(periods[0].period_number, 1);
        assert_eq!(
            periods[0].start_date,
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
        );
        assert_eq!(
            periods[0].end_date,
            NaiveDate::from_ymd_opt(2026, 1, 31).unwrap()
        );

        assert_eq!(periods[11].name, "December 2026");
        assert_eq!(periods[11].period_number, 12);
        assert_eq!(
            periods[11].start_date,
            NaiveDate::from_ymd_opt(2026, 12, 1).unwrap()
        );
        assert_eq!(
            periods[11].end_date,
            NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()
        );
    }

    #[test]
    fn test_generate_monthly_periods_partial_year() {
        let start = NaiveDate::from_ymd_opt(2026, 4, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2027, 3, 31).unwrap();
        let org_id = Uuid::new_v4();
        let fy_id = Uuid::new_v4();

        let periods = generate_monthly_periods(fy_id, org_id, start, end);

        assert_eq!(periods.len(), 12);
        assert_eq!(periods[0].name, "April 2026");
        assert_eq!(periods[11].name, "March 2027");
    }

    #[test]
    fn test_last_day_of_month() {
        assert_eq!(
            last_day_of_month(2026, 1),
            NaiveDate::from_ymd_opt(2026, 1, 31).unwrap()
        );
        assert_eq!(
            last_day_of_month(2026, 2),
            NaiveDate::from_ymd_opt(2026, 2, 28).unwrap()
        );
        assert_eq!(
            last_day_of_month(2024, 2),
            NaiveDate::from_ymd_opt(2024, 2, 29).unwrap()
        ); // Leap year
        assert_eq!(
            last_day_of_month(2026, 4),
            NaiveDate::from_ymd_opt(2026, 4, 30).unwrap()
        );
        assert_eq!(
            last_day_of_month(2026, 12),
            NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()
        );
    }

    #[test]
    fn test_validate_status_transition_valid() {
        assert!(
            validate_status_transition(&FiscalPeriodStatus::Open, &FiscalPeriodStatus::SoftClose)
                .is_ok()
        );
        assert!(
            validate_status_transition(&FiscalPeriodStatus::Open, &FiscalPeriodStatus::Closed)
                .is_ok()
        );
        assert!(
            validate_status_transition(&FiscalPeriodStatus::SoftClose, &FiscalPeriodStatus::Closed)
                .is_ok()
        );
        assert!(
            validate_status_transition(&FiscalPeriodStatus::SoftClose, &FiscalPeriodStatus::Open)
                .is_ok()
        );
    }

    #[test]
    fn test_validate_status_transition_invalid() {
        assert!(
            validate_status_transition(&FiscalPeriodStatus::Closed, &FiscalPeriodStatus::Open)
                .is_err()
        );
        assert!(
            validate_status_transition(&FiscalPeriodStatus::Closed, &FiscalPeriodStatus::SoftClose)
                .is_err()
        );
    }
}

/// Property-based tests for fiscal year date validation.
///
/// Feature: ledger-core
/// Property 10: Fiscal Year Date Validation
/// Validates: Requirements 1.2, 1.3
#[cfg(test)]
mod props {
    use super::*;
    use proptest::prelude::*;

    /// Strategy to generate valid dates within a reasonable range.
    fn date_strategy() -> impl Strategy<Value = NaiveDate> {
        // Generate dates from 2020-01-01 to 2030-12-31
        (2020i32..=2030, 1u32..=12, 1u32..=28)
            .prop_map(|(year, month, day)| NaiveDate::from_ymd_opt(year, month, day).unwrap())
    }

    /// Strategy to generate a valid fiscal year (start < end).
    fn valid_fiscal_year_dates() -> impl Strategy<Value = (NaiveDate, NaiveDate)> {
        date_strategy().prop_flat_map(|start| {
            // End date is 1 to 365 days after start
            (Just(start), 1i64..=365).prop_map(move |(s, days)| {
                let end = s + chrono::Duration::days(days);
                (s, end)
            })
        })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        // =========================================================================
        // Property 10: Fiscal Year Date Validation
        // Validates: Requirements 1.2, 1.3
        // =========================================================================

        /// Property 10.1: Valid date ranges are accepted.
        ///
        /// *For any* fiscal year where start_date < end_date,
        /// validation SHALL succeed.
        #[test]
        fn prop_valid_date_range_accepted((start, end) in valid_fiscal_year_dates()) {
            let result = validate_date_range(start, end);
            prop_assert!(result.is_ok(), "Valid date range should be accepted: {} to {}", start, end);
        }

        /// Property 10.2: Invalid date ranges are rejected.
        ///
        /// *For any* fiscal year where start_date >= end_date,
        /// validation SHALL fail with InvalidDateRange error.
        #[test]
        fn prop_invalid_date_range_rejected(date in date_strategy()) {
            // Same date (start == end)
            let result = validate_date_range(date, date);
            prop_assert!(
                matches!(result, Err(FiscalError::InvalidDateRange)),
                "Same start and end date should be rejected"
            );

            // End before start
            if let Some(earlier) = date.pred_opt() {
                let result = validate_date_range(date, earlier);
                prop_assert!(
                    matches!(result, Err(FiscalError::InvalidDateRange)),
                    "End before start should be rejected"
                );
            }
        }

        /// Property 10.3: Overlapping ranges are detected.
        ///
        /// *For any* two fiscal years with overlapping date ranges,
        /// the overlap detection SHALL return true.
        #[test]
        fn prop_overlapping_ranges_detected(
            (a_start, a_end) in valid_fiscal_year_dates(),
            offset in 0i64..=180,
        ) {
            // Create a second range that overlaps with the first
            // by starting within the first range
            let b_start = a_start + chrono::Duration::days(offset);
            if b_start <= a_end {
                let b_end = b_start + chrono::Duration::days(30);

                let overlaps = date_ranges_overlap(a_start, a_end, b_start, b_end);
                prop_assert!(overlaps, "Overlapping ranges should be detected");
            }
        }

        /// Property 10.4: Non-overlapping ranges are not flagged.
        ///
        /// *For any* two fiscal years with non-overlapping date ranges,
        /// the overlap detection SHALL return false.
        #[test]
        fn prop_non_overlapping_ranges_not_flagged(
            (a_start, a_end) in valid_fiscal_year_dates(),
            gap in 1i64..=365,
        ) {
            // Create a second range that starts after the first ends
            let b_start = a_end + chrono::Duration::days(gap);
            let b_end = b_start + chrono::Duration::days(30);

            let overlaps = date_ranges_overlap(a_start, a_end, b_start, b_end);
            prop_assert!(!overlaps, "Non-overlapping ranges should not be flagged");
        }

        /// Property 10.5: Overlap detection is symmetric.
        ///
        /// *For any* two date ranges, overlap(A, B) == overlap(B, A).
        #[test]
        fn prop_overlap_is_symmetric(
            (a_start, a_end) in valid_fiscal_year_dates(),
            (b_start, b_end) in valid_fiscal_year_dates(),
        ) {
            let ab = date_ranges_overlap(a_start, a_end, b_start, b_end);
            let ba = date_ranges_overlap(b_start, b_end, a_start, a_end);
            prop_assert_eq!(ab, ba, "Overlap detection should be symmetric");
        }

        /// Property 10.6: Adjacent ranges do not overlap.
        ///
        /// *For any* fiscal year, a range ending on day D and another
        /// starting on day D+1 SHALL NOT overlap.
        #[test]
        fn prop_adjacent_ranges_do_not_overlap((a_start, a_end) in valid_fiscal_year_dates()) {
            // B starts the day after A ends
            let b_start = a_end + chrono::Duration::days(1);
            let b_end = b_start + chrono::Duration::days(30);

            let overlaps = date_ranges_overlap(a_start, a_end, b_start, b_end);
            prop_assert!(!overlaps, "Adjacent ranges should not overlap");
        }
    }

    // Unit tests for specific examples
    mod unit_tests {
        use super::*;

        #[test]
        fn test_valid_calendar_year() {
            let start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
            let end = NaiveDate::from_ymd_opt(2026, 12, 31).unwrap();
            assert!(validate_date_range(start, end).is_ok());
        }

        #[test]
        fn test_valid_fiscal_year_apr_mar() {
            let start = NaiveDate::from_ymd_opt(2026, 4, 1).unwrap();
            let end = NaiveDate::from_ymd_opt(2027, 3, 31).unwrap();
            assert!(validate_date_range(start, end).is_ok());
        }

        #[test]
        fn test_same_date_rejected() {
            let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
            assert!(matches!(
                validate_date_range(date, date),
                Err(FiscalError::InvalidDateRange)
            ));
        }

        #[test]
        fn test_end_before_start_rejected() {
            let start = NaiveDate::from_ymd_opt(2026, 12, 31).unwrap();
            let end = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
            assert!(matches!(
                validate_date_range(start, end),
                Err(FiscalError::InvalidDateRange)
            ));
        }

        #[test]
        fn test_overlapping_years() {
            // FY 2026: Jan 1 - Dec 31
            let a_start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
            let a_end = NaiveDate::from_ymd_opt(2026, 12, 31).unwrap();

            // FY 2026-2027: Jul 1 - Jun 30 (overlaps)
            let b_start = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
            let b_end = NaiveDate::from_ymd_opt(2027, 6, 30).unwrap();

            assert!(date_ranges_overlap(a_start, a_end, b_start, b_end));
        }

        #[test]
        fn test_non_overlapping_years() {
            // FY 2025: Jan 1 - Dec 31
            let a_start = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
            let a_end = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();

            // FY 2026: Jan 1 - Dec 31 (adjacent, not overlapping)
            let b_start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
            let b_end = NaiveDate::from_ymd_opt(2026, 12, 31).unwrap();

            assert!(!date_ranges_overlap(a_start, a_end, b_start, b_end));
        }
    }
}
