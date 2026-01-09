//! Dimension repository for dimension types and values database operations.
//!
//! Implements Requirements 3.1-3.6 for dimension management.

use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, Set,
};
use uuid::Uuid;

use crate::entities::{dimension_types, dimension_values};

/// Error types for dimension operations.
#[derive(Debug, thiserror::Error)]
pub enum DimensionError {
    /// Dimension type code already exists in organization.
    #[error("Dimension type code '{0}' already exists")]
    DuplicateTypeCode(String),

    /// Dimension value code already exists for this type.
    #[error("Dimension value code '{0}' already exists for this type")]
    DuplicateValueCode(String),

    /// Dimension type not found.
    #[error("Dimension type not found: {0}")]
    TypeNotFound(Uuid),

    /// Dimension value not found.
    #[error("Dimension value not found: {0}")]
    ValueNotFound(Uuid),

    /// Parent dimension value not found.
    #[error("Parent dimension value not found: {0}")]
    ParentNotFound(Uuid),

    /// Parent belongs to different dimension type.
    #[error("Parent dimension value belongs to different type")]
    ParentWrongType,

    /// Database error.
    #[error("Database error: {0}")]
    Database(#[from] DbErr),
}

/// Input for creating a dimension type.
#[derive(Debug, Clone)]
pub struct CreateDimensionTypeInput {
    /// Organization ID.
    pub organization_id: Uuid,
    /// Dimension type code (must be unique within organization).
    pub code: String,
    /// Dimension type name.
    pub name: String,
    /// Description.
    pub description: Option<String>,
    /// Whether this dimension is required on transactions.
    pub is_required: bool,
    /// Whether this dimension type is active.
    pub is_active: bool,
    /// Sort order for display.
    pub sort_order: i16,
}

/// Input for updating a dimension type.
#[derive(Debug, Clone, Default)]
pub struct UpdateDimensionTypeInput {
    /// Dimension type code.
    pub code: Option<String>,
    /// Dimension type name.
    pub name: Option<String>,
    /// Description.
    pub description: Option<Option<String>>,
    /// Whether this dimension is required.
    pub is_required: Option<bool>,
    /// Whether this dimension type is active.
    pub is_active: Option<bool>,
    /// Sort order.
    pub sort_order: Option<i16>,
}

/// Input for creating a dimension value.
#[derive(Debug, Clone)]
pub struct CreateDimensionValueInput {
    /// Organization ID.
    pub organization_id: Uuid,
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
    /// Whether this value is active.
    pub is_active: bool,
    /// Effective from date.
    pub effective_from: Option<chrono::NaiveDate>,
    /// Effective to date.
    pub effective_to: Option<chrono::NaiveDate>,
}

/// Input for updating a dimension value.
#[derive(Debug, Clone, Default)]
pub struct UpdateDimensionValueInput {
    /// Dimension value code.
    pub code: Option<String>,
    /// Dimension value name.
    pub name: Option<String>,
    /// Description.
    pub description: Option<Option<String>>,
    /// Parent dimension value ID.
    pub parent_id: Option<Option<Uuid>>,
    /// Whether this value is active.
    pub is_active: Option<bool>,
    /// Effective from date.
    pub effective_from: Option<Option<chrono::NaiveDate>>,
    /// Effective to date.
    pub effective_to: Option<Option<chrono::NaiveDate>>,
}

/// Filter options for listing dimension types.
#[derive(Debug, Clone, Default)]
pub struct DimensionTypeFilter {
    /// Filter by active status.
    pub is_active: Option<bool>,
}

/// Filter options for listing dimension values.
#[derive(Debug, Clone, Default)]
pub struct DimensionValueFilter {
    /// Filter by dimension type ID.
    pub dimension_type_id: Option<Uuid>,
    /// Filter by active status.
    pub is_active: Option<bool>,
    /// Filter by parent ID (None = root values only).
    pub parent_id: Option<Option<Uuid>>,
}

/// Dimension repository for CRUD operations.
#[derive(Debug, Clone)]
pub struct DimensionRepository {
    db: DatabaseConnection,
}

impl DimensionRepository {
    /// Creates a new dimension repository.
    #[must_use]
    pub const fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    // ========================================================================
    // Dimension Type Operations
    // ========================================================================

    /// Creates a new dimension type with validation.
    ///
    /// Requirements: 3.1, 3.2
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Dimension type code already exists in organization
    pub async fn create_dimension_type(
        &self,
        input: CreateDimensionTypeInput,
    ) -> Result<dimension_types::Model, DimensionError> {
        // Validate unique code within organization (Requirement 3.2)
        let existing = dimension_types::Entity::find()
            .filter(dimension_types::Column::OrganizationId.eq(input.organization_id))
            .filter(dimension_types::Column::Code.eq(&input.code))
            .one(&self.db)
            .await?;

        if existing.is_some() {
            return Err(DimensionError::DuplicateTypeCode(input.code));
        }

        let now = chrono::Utc::now().into();
        let dimension_type = dimension_types::ActiveModel {
            id: Set(Uuid::new_v4()),
            organization_id: Set(input.organization_id),
            code: Set(input.code),
            name: Set(input.name),
            description: Set(input.description),
            is_required: Set(input.is_required),
            is_active: Set(input.is_active),
            sort_order: Set(input.sort_order),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = dimension_type.insert(&self.db).await?;
        Ok(result)
    }

    /// Lists dimension types for an organization.
    ///
    /// Requirements: 3.3
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn list_dimension_types(
        &self,
        organization_id: Uuid,
        filter: DimensionTypeFilter,
    ) -> Result<Vec<dimension_types::Model>, DimensionError> {
        let mut query = dimension_types::Entity::find()
            .filter(dimension_types::Column::OrganizationId.eq(organization_id))
            .order_by_asc(dimension_types::Column::SortOrder)
            .order_by_asc(dimension_types::Column::Code);

        if let Some(is_active) = filter.is_active {
            query = query.filter(dimension_types::Column::IsActive.eq(is_active));
        }

        let results = query.all(&self.db).await?;
        Ok(results)
    }

    /// Finds a dimension type by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn find_dimension_type_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<dimension_types::Model>, DimensionError> {
        let result = dimension_types::Entity::find_by_id(id)
            .one(&self.db)
            .await?;
        Ok(result)
    }

    /// Updates a dimension type.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Dimension type not found
    /// - New code already exists in organization
    pub async fn update_dimension_type(
        &self,
        id: Uuid,
        input: UpdateDimensionTypeInput,
    ) -> Result<dimension_types::Model, DimensionError> {
        let dim_type = dimension_types::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or(DimensionError::TypeNotFound(id))?;

        // If changing code, validate uniqueness
        if let Some(new_code) = &input.code
            && *new_code != dim_type.code
        {
            let existing = dimension_types::Entity::find()
                .filter(dimension_types::Column::OrganizationId.eq(dim_type.organization_id))
                .filter(dimension_types::Column::Code.eq(new_code))
                .filter(dimension_types::Column::Id.ne(id))
                .one(&self.db)
                .await?;

            if existing.is_some() {
                return Err(DimensionError::DuplicateTypeCode(new_code.clone()));
            }
        }

        let now = chrono::Utc::now().into();
        let mut active: dimension_types::ActiveModel = dim_type.into();

        if let Some(code) = input.code {
            active.code = Set(code);
        }
        if let Some(name) = input.name {
            active.name = Set(name);
        }
        if let Some(description) = input.description {
            active.description = Set(description);
        }
        if let Some(is_required) = input.is_required {
            active.is_required = Set(is_required);
        }
        if let Some(is_active) = input.is_active {
            active.is_active = Set(is_active);
        }
        if let Some(sort_order) = input.sort_order {
            active.sort_order = Set(sort_order);
        }
        active.updated_at = Set(now);

        let updated = active.update(&self.db).await?;
        Ok(updated)
    }

    // ========================================================================
    // Dimension Value Operations
    // ========================================================================

    /// Creates a new dimension value with validation.
    ///
    /// Requirements: 3.4, 3.5
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Dimension type does not exist
    /// - Dimension value code already exists for this type
    /// - Parent value does not exist or belongs to different type
    pub async fn create_dimension_value(
        &self,
        input: CreateDimensionValueInput,
    ) -> Result<dimension_values::Model, DimensionError> {
        // Validate dimension type exists
        let dim_type = dimension_types::Entity::find_by_id(input.dimension_type_id)
            .one(&self.db)
            .await?
            .ok_or(DimensionError::TypeNotFound(input.dimension_type_id))?;

        // Validate organization matches
        if dim_type.organization_id != input.organization_id {
            return Err(DimensionError::TypeNotFound(input.dimension_type_id));
        }

        // Validate unique code within dimension type (Requirement 3.4)
        let existing = dimension_values::Entity::find()
            .filter(dimension_values::Column::DimensionTypeId.eq(input.dimension_type_id))
            .filter(dimension_values::Column::Code.eq(&input.code))
            .one(&self.db)
            .await?;

        if existing.is_some() {
            return Err(DimensionError::DuplicateValueCode(input.code));
        }

        // Validate parent if provided (Requirement 3.5)
        if let Some(parent_id) = input.parent_id {
            let parent = dimension_values::Entity::find_by_id(parent_id)
                .one(&self.db)
                .await?;

            match parent {
                None => return Err(DimensionError::ParentNotFound(parent_id)),
                Some(p) if p.dimension_type_id != input.dimension_type_id => {
                    return Err(DimensionError::ParentWrongType);
                }
                _ => {}
            }
        }

        let now = chrono::Utc::now().into();
        let dimension_value = dimension_values::ActiveModel {
            id: Set(Uuid::new_v4()),
            organization_id: Set(input.organization_id),
            dimension_type_id: Set(input.dimension_type_id),
            code: Set(input.code),
            name: Set(input.name),
            description: Set(input.description),
            parent_id: Set(input.parent_id),
            is_active: Set(input.is_active),
            effective_from: Set(input.effective_from),
            effective_to: Set(input.effective_to),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = dimension_value.insert(&self.db).await?;
        Ok(result)
    }

    /// Lists dimension values for an organization.
    ///
    /// Requirements: 3.6
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn list_dimension_values(
        &self,
        organization_id: Uuid,
        filter: DimensionValueFilter,
    ) -> Result<Vec<dimension_values::Model>, DimensionError> {
        let mut query = dimension_values::Entity::find()
            .filter(dimension_values::Column::OrganizationId.eq(organization_id))
            .order_by_asc(dimension_values::Column::Code);

        if let Some(dimension_type_id) = filter.dimension_type_id {
            query = query.filter(dimension_values::Column::DimensionTypeId.eq(dimension_type_id));
        }

        if let Some(is_active) = filter.is_active {
            query = query.filter(dimension_values::Column::IsActive.eq(is_active));
        }

        if let Some(parent_id) = filter.parent_id {
            match parent_id {
                Some(pid) => {
                    query = query.filter(dimension_values::Column::ParentId.eq(pid));
                }
                None => {
                    query = query.filter(dimension_values::Column::ParentId.is_null());
                }
            }
        }

        let results = query.all(&self.db).await?;
        Ok(results)
    }

    /// Finds a dimension value by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn find_dimension_value_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<dimension_values::Model>, DimensionError> {
        let result = dimension_values::Entity::find_by_id(id)
            .one(&self.db)
            .await?;
        Ok(result)
    }

    /// Updates a dimension value.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Dimension value not found
    /// - New code already exists for this type
    /// - Parent validation fails
    pub async fn update_dimension_value(
        &self,
        id: Uuid,
        input: UpdateDimensionValueInput,
    ) -> Result<dimension_values::Model, DimensionError> {
        let dim_value = dimension_values::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or(DimensionError::ValueNotFound(id))?;

        // If changing code, validate uniqueness within type
        if let Some(new_code) = &input.code
            && *new_code != dim_value.code
        {
            let existing = dimension_values::Entity::find()
                .filter(dimension_values::Column::DimensionTypeId.eq(dim_value.dimension_type_id))
                .filter(dimension_values::Column::Code.eq(new_code))
                .filter(dimension_values::Column::Id.ne(id))
                .one(&self.db)
                .await?;

            if existing.is_some() {
                return Err(DimensionError::DuplicateValueCode(new_code.clone()));
            }
        }

        // If changing parent, validate
        if let Some(new_parent) = &input.parent_id
            && let Some(parent_id) = new_parent
        {
            let parent = dimension_values::Entity::find_by_id(*parent_id)
                .one(&self.db)
                .await?;

            match parent {
                None => return Err(DimensionError::ParentNotFound(*parent_id)),
                Some(p) if p.dimension_type_id != dim_value.dimension_type_id => {
                    return Err(DimensionError::ParentWrongType);
                }
                _ => {}
            }
        }

        let now = chrono::Utc::now().into();
        let mut active: dimension_values::ActiveModel = dim_value.into();

        if let Some(code) = input.code {
            active.code = Set(code);
        }
        if let Some(name) = input.name {
            active.name = Set(name);
        }
        if let Some(description) = input.description {
            active.description = Set(description);
        }
        if let Some(parent_id) = input.parent_id {
            active.parent_id = Set(parent_id);
        }
        if let Some(is_active) = input.is_active {
            active.is_active = Set(is_active);
        }
        if let Some(effective_from) = input.effective_from {
            active.effective_from = Set(effective_from);
        }
        if let Some(effective_to) = input.effective_to {
            active.effective_to = Set(effective_to);
        }
        active.updated_at = Set(now);

        let updated = active.update(&self.db).await?;
        Ok(updated)
    }

    /// Checks if a dimension type code exists in an organization.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn type_code_exists(
        &self,
        organization_id: Uuid,
        code: &str,
    ) -> Result<bool, DimensionError> {
        let count = dimension_types::Entity::find()
            .filter(dimension_types::Column::OrganizationId.eq(organization_id))
            .filter(dimension_types::Column::Code.eq(code))
            .count(&self.db)
            .await?;

        Ok(count > 0)
    }

    /// Checks if a dimension value code exists for a dimension type.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn value_code_exists(
        &self,
        dimension_type_id: Uuid,
        code: &str,
    ) -> Result<bool, DimensionError> {
        let count = dimension_values::Entity::find()
            .filter(dimension_values::Column::DimensionTypeId.eq(dimension_type_id))
            .filter(dimension_values::Column::Code.eq(code))
            .count(&self.db)
            .await?;

        Ok(count > 0)
    }
}

// ============================================================================
// Pure validation functions for property testing
// ============================================================================

/// Represents a dimension type code entry for uniqueness checking.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DimensionTypeCodeEntry {
    /// Organization ID.
    pub organization_id: Uuid,
    /// Dimension type code.
    pub code: String,
}

/// Represents a dimension value code entry for uniqueness checking.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DimensionValueCodeEntry {
    /// Dimension type ID.
    pub dimension_type_id: Uuid,
    /// Dimension value code.
    pub code: String,
}

/// Checks if a dimension type code would be unique within an organization.
///
/// This is a pure function that can be tested without database access.
#[must_use]
pub fn is_type_code_unique<S: std::hash::BuildHasher>(
    existing_codes: &std::collections::HashSet<DimensionTypeCodeEntry, S>,
    org_id: Uuid,
    code: &str,
) -> bool {
    let entry = DimensionTypeCodeEntry {
        organization_id: org_id,
        code: code.to_string(),
    };
    !existing_codes.contains(&entry)
}

/// Checks if a dimension value code would be unique within a dimension type.
///
/// This is a pure function that can be tested without database access.
#[must_use]
pub fn is_value_code_unique<S: std::hash::BuildHasher>(
    existing_codes: &std::collections::HashSet<DimensionValueCodeEntry, S>,
    dimension_type_id: Uuid,
    code: &str,
) -> bool {
    let entry = DimensionValueCodeEntry {
        dimension_type_id,
        code: code.to_string(),
    };
    !existing_codes.contains(&entry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::HashSet;

    // ========================================================================
    // Property 11: Uniqueness Constraints (Dimension part)
    // **Validates: Requirements 3.2, 3.4**
    // ========================================================================

    /// Strategy for generating valid dimension codes (alphanumeric, 1-20 chars)
    fn dimension_code_strategy() -> impl Strategy<Value = String> {
        "[A-Z0-9_]{1,10}"
    }

    // ------------------------------------------------------------------------
    // Property 11.6: Duplicate dimension type codes in same org rejected
    // ------------------------------------------------------------------------

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Property 11.6: Duplicate dimension type codes in same org rejected**
        ///
        /// *For any* existing dimension type code in an organization,
        /// attempting to create another type with the same code
        /// in the same organization SHALL be rejected.
        ///
        /// **Validates: Requirements 3.2**
        #[test]
        fn prop_duplicate_type_code_same_org_rejected(
            org_bits in any::<u128>(),
            code in dimension_code_strategy(),
        ) {
            let org_id = Uuid::from_u128(org_bits);

            let mut existing = HashSet::new();
            existing.insert(DimensionTypeCodeEntry {
                organization_id: org_id,
                code: code.clone(),
            });

            let is_unique = is_type_code_unique(&existing, org_id, &code);
            prop_assert!(!is_unique, "Duplicate type code in same org should be rejected");
        }

        /// **Property 11.7: Same dimension type code in different orgs allowed**
        ///
        /// *For any* dimension type code, the same code CAN exist in different
        /// organizations (uniqueness is per-organization).
        ///
        /// **Validates: Requirements 3.2**
        #[test]
        fn prop_same_type_code_different_org_allowed(
            org1_bits in any::<u128>(),
            org2_bits in any::<u128>(),
            code in dimension_code_strategy(),
        ) {
            prop_assume!(org1_bits != org2_bits);

            let org1_id = Uuid::from_u128(org1_bits);
            let org2_id = Uuid::from_u128(org2_bits);

            let mut existing = HashSet::new();
            existing.insert(DimensionTypeCodeEntry {
                organization_id: org1_id,
                code: code.clone(),
            });

            let is_unique = is_type_code_unique(&existing, org2_id, &code);
            prop_assert!(is_unique, "Same type code in different org should be allowed");
        }

        /// **Property 11.8: Duplicate dimension value codes in same type rejected**
        ///
        /// *For any* existing dimension value code in a dimension type,
        /// attempting to create another value with the same code
        /// in the same type SHALL be rejected.
        ///
        /// **Validates: Requirements 3.4**
        #[test]
        fn prop_duplicate_value_code_same_type_rejected(
            type_bits in any::<u128>(),
            code in dimension_code_strategy(),
        ) {
            let type_id = Uuid::from_u128(type_bits);

            let mut existing = HashSet::new();
            existing.insert(DimensionValueCodeEntry {
                dimension_type_id: type_id,
                code: code.clone(),
            });

            let is_unique = is_value_code_unique(&existing, type_id, &code);
            prop_assert!(!is_unique, "Duplicate value code in same type should be rejected");
        }

        /// **Property 11.9: Same dimension value code in different types allowed**
        ///
        /// *For any* dimension value code, the same code CAN exist in different
        /// dimension types (uniqueness is per-type).
        ///
        /// **Validates: Requirements 3.4**
        #[test]
        fn prop_same_value_code_different_type_allowed(
            type1_bits in any::<u128>(),
            type2_bits in any::<u128>(),
            code in dimension_code_strategy(),
        ) {
            prop_assume!(type1_bits != type2_bits);

            let type1_id = Uuid::from_u128(type1_bits);
            let type2_id = Uuid::from_u128(type2_bits);

            let mut existing = HashSet::new();
            existing.insert(DimensionValueCodeEntry {
                dimension_type_id: type1_id,
                code: code.clone(),
            });

            let is_unique = is_value_code_unique(&existing, type2_id, &code);
            prop_assert!(is_unique, "Same value code in different type should be allowed");
        }
    }

    // ========================================================================
    // Unit tests for edge cases
    // ========================================================================

    #[test]
    fn test_empty_existing_type_codes_allows_any() {
        let existing = HashSet::new();
        let org_id = Uuid::new_v4();

        assert!(is_type_code_unique(&existing, org_id, "DEPARTMENT"));
        assert!(is_type_code_unique(&existing, org_id, "PROJECT"));
        assert!(is_type_code_unique(&existing, org_id, "COST_CENTER"));
    }

    #[test]
    fn test_empty_existing_value_codes_allows_any() {
        let existing = HashSet::new();
        let type_id = Uuid::new_v4();

        assert!(is_value_code_unique(&existing, type_id, "ENG"));
        assert!(is_value_code_unique(&existing, type_id, "SALES"));
        assert!(is_value_code_unique(&existing, type_id, "HR"));
    }

    #[test]
    fn test_type_code_case_sensitive() {
        let org_id = Uuid::new_v4();
        let mut existing = HashSet::new();
        existing.insert(DimensionTypeCodeEntry {
            organization_id: org_id,
            code: "DEPARTMENT".to_string(),
        });

        // Same case = duplicate
        assert!(!is_type_code_unique(&existing, org_id, "DEPARTMENT"));

        // Different case = unique (codes are case-sensitive)
        assert!(is_type_code_unique(&existing, org_id, "department"));
        assert!(is_type_code_unique(&existing, org_id, "Department"));
    }

    #[test]
    fn test_value_code_case_sensitive() {
        let type_id = Uuid::new_v4();
        let mut existing = HashSet::new();
        existing.insert(DimensionValueCodeEntry {
            dimension_type_id: type_id,
            code: "ENG".to_string(),
        });

        // Same case = duplicate
        assert!(!is_value_code_unique(&existing, type_id, "ENG"));

        // Different case = unique
        assert!(is_value_code_unique(&existing, type_id, "eng"));
        assert!(is_value_code_unique(&existing, type_id, "Eng"));
    }
}
