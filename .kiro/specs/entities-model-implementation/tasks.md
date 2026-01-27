# Implementation Plan: Entities Model Implementation

## Overview

This implementation plan breaks down the entities model refactoring into 7 daily tasks, each designed to be executed independently by a subagent. Each task includes detailed sub-tasks with specific file paths, testing steps, validation commands, and research tasks for 2025-2026 best practices.

**Total Estimated Effort**: 7 days (6-8 hours per day)

**Critical Notes**:
- Research tasks use Exa/Tavily to get 2025-2026 best practices (knowledge cutoff: 2024)
- Backend binary: `zeltra` (not zeltra-api)
- Frontend package manager: `pnpm` (not npm)
- Test credentials: corp@zeltra.io / qwertyui
- Use getDiagnostics tool instead of bash commands for syntax checking
- Run cargo fmt/clippy for backend, pnpm lint/build for frontend
- Each task should be completed and tested before moving to the next

## Tasks

- [-] 1. Database Schema & Core Backend (Day 1: 6-8 hours)
  - [x] 1.1 Research SeaORM and Rust database patterns for 2025-2026
    - Use Exa or Tavily to search for "SeaORM migration best practices 2025 2026"
    - Use Exa or Tavily to search for "SeaORM entity relationships patterns 2025"
    - Use Exa or Tavily to search for "Rust database migration strategies 2025"
    - Check for any breaking changes or new patterns
    - Document findings for migration and entity model implementation
    - _Requirements: 1.1-1.12, 2.1_
  
  - [x] 1.2 Create database migration for entities table
    - Create file `backend/crates/db/src/migration/m20260125_000001_add_entities.rs`
    - Define entities table schema with all fields (id, organization_id, name, legal_name, tax_id, entity_type, base_currency, is_active, settings, created_at, updated_at)
    - Add unique constraint on (organization_id, name)
    - Add foreign key to organizations(id) with CASCADE delete
    - Add indexes on organization_id and (organization_id, is_active)
    - Register migration in `backend/crates/db/src/migration/mod.rs`
    - _Requirements: 2.1, 2.6, 2.8_
  
  - [x] 1.3 Create migration to add subscription fields to users table
    - Update migration file `backend/crates/db/src/migration/m20260125_000001_add_entities.rs`
    - Add subscription_tier column (enum, default 'starter')
    - Add subscription_status column (enum, default 'trialing')
    - Add trial_ends_at column (timestamptz, nullable)
    - Add subscription_ends_at column (timestamptz, nullable)
    - Add payment_provider column (varchar, nullable)
    - Add payment_customer_id column (varchar, nullable)
    - Add payment_subscription_id column (varchar, nullable)
    - Add foreign key to tier_limits(tier)
    - Add indexes on subscription_status and (payment_provider, payment_customer_id)
    - _Requirements: 1.1, 1.2, 1.3_
  
  - [x] 1.4 Create migration to add entity_id to existing tables
    - Update migration file `backend/crates/db/src/migration/m20260125_000001_add_entities.rs`
    - Add entity_id column to chart_of_accounts (UUID, references entities(id))
    - Add entity_id column to transactions (UUID, references entities(id))
    - Add entity_id column to ledger_entries (UUID, references entities(id))
    - Add entity_id column to budgets (UUID, references entities(id))
    - Add entity_id column to fiscal_years (UUID, references entities(id))
    - Add entity_id column to accrual_schedules (UUID, references entities(id))
    - Add entity_id column to revaluation_logs (UUID, references entities(id))
    - Add indexes on entity_id for all tables
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7_
  
  - [x] 1.5 Create migration for data migration
    - Update migration file `backend/crates/db/src/migration/m20260125_000001_add_entities.rs`
    - Migrate subscription data from organizations to users (take from first/oldest org)
    - Create default entity for each organization (name: "{org_name} (Main)", entity_type: 'main')
    - Link all chart_of_accounts records to default entity
    - Link all transactions records to default entity
    - Link all ledger_entries records to default entity
    - Link all budgets records to default entity
    - Link all fiscal_years records to default entity
    - Link all accrual_schedules records to default entity
    - Link all revaluation_logs records to default entity
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7, 5.8, 5.9_


  - [x] 1.5 Create migration to update intercompany_mappings table
    - Create file `backend/crates/db/src/migration/m20260125_000002_update_intercompany.rs`
    - Rename source_org_id column to source_entity_id
    - Rename target_org_id column to target_entity_id
    - Update foreign keys to reference entities(id)
    - Add check constraint: both entities must be in same organization
    - Register migration in `backend/crates/db/src/migration/mod.rs`
    - _Requirements: 4.1, 4.2_
  
  - [x] 1.6 Create Entity model
    - Create file `backend/crates/db/src/entities/entities.rs`
    - Define Model struct with all fields
    - Derive DeriveEntityModel, Serialize, Deserialize
    - Define Relation enum with Organization relation
    - Add to `backend/crates/db/src/entities/mod.rs`
    - _Requirements: 2.1, 2.6, 2.7, 2.8_
  
  - [x] 1.7 Update User model with subscription fields
    - Modify file `backend/crates/db/src/entities/users.rs`
    - Add subscription_tier field
    - Add subscription_status field
    - Add trial_ends_at field
    - Add subscription_ends_at field
    - Add payment_provider field
    - Add payment_customer_id field
    - Add payment_subscription_id field
    - _Requirements: 1.1, 1.2, 1.3_
  
  - [x] 1.8 Update existing entity models with entity_id
    - Modify `backend/crates/db/src/entities/chart_of_accounts.rs` - add entity_id field
    - Modify `backend/crates/db/src/entities/transactions.rs` - add entity_id field
    - Modify `backend/crates/db/src/entities/ledger_entries.rs` - add entity_id field
    - Modify `backend/crates/db/src/entities/budgets.rs` - add entity_id field
    - Modify `backend/crates/db/src/entities/fiscal_years.rs` - add entity_id field
    - Modify `backend/crates/db/src/entities/accrual_schedules.rs` - add entity_id field
    - Modify `backend/crates/db/src/entities/revaluation_logs.rs` - add entity_id field
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7_
  
  - [x] 1.9 Update intercompany_mappings model
    - Modify file `backend/crates/db/src/entities/intercompany_mappings.rs`
    - Rename source_org_id field to source_entity_id
    - Rename target_org_id field to target_entity_id
    - Update Relation enum to reference entities instead of organizations
    - _Requirements: 4.1_
  
  - [x] 1.10 Create Entity repository
    - Create file `backend/crates/db/src/repositories/entity.rs`
    - Implement create() method with tier limit validation
    - Implement list_by_organization() method
    - Implement find_by_id() method
    - Implement update() method
    - Implement delete() method (soft delete)
    - Implement count_by_organization() method
    - Implement get_org_owner_tier() helper method
    - Add to `backend/crates/db/src/repositories/mod.rs`
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.8_
  
  - [x] 1.11 Update User repository with subscription methods
    - Modify file `backend/crates/db/src/repositories/user.rs`
    - Add get_subscription_tier() method
    - Add update_subscription_tier() method
    - Add get_subscription_status() method
    - Add update_subscription_status() method
    - _Requirements: 1.1, 1.2, 1.3_
  
  - [x] 1.12 Run migrations and validate
    - Run `cargo run --bin zeltra migrate`
    - Verify entities table created
    - Verify users table has subscription fields
    - Verify all tables have entity_id column
    - Verify intercompany_mappings updated
    - Verify default entities created for existing organizations
    - Verify all entity_id foreign keys are valid
    - Use MCP postgres tool to query: `SELECT COUNT(*) FROM entities`
    - Use MCP postgres tool to query: `SELECT COUNT(*) FROM chart_of_accounts WHERE entity_id IS NULL`
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7, 5.8, 5.9, 5.10, 5.11_
  
  - [x] 1.13 Run backend validation
    - Run `cargo fmt` to format code
    - Run `cargo clippy -- -D warnings` to check for issues
    - Use getDiagnostics tool on all modified backend files
    - Fix any errors or warnings
    - Run `cargo test` to ensure existing tests pass
  
  - [ ] 1.14 Write property-based tests for entity repository
    - Create test file `backend/crates/db/tests/entity_repository_properties.rs`
    - **Property 5**: Default entity creation - test that creating org creates default entity
    - **Property 6**: Entity tier limit enforcement - test tier limits for all tiers
    - **Property 7**: Enterprise unlimited entities - test unlimited creation for enterprise
    - **Property 8**: Entity list ordering - test entities returned in created_at order
    - **Property 9**: Entity update - test all fields can be updated
    - **Property 10**: Entity soft delete - test is_active set to false
    - Use quickcheck library with 100 iterations per property
    - Tag each test: `Feature: entities-model-implementation, Property N: {property_text}`
    - _Requirements: 2.1, 2.2, 2.5, 2.6, 2.7, 2.8_
  
  - [ ] 1.15 Write unit tests for entity repository
    - Create test file `backend/crates/db/tests/entity_repository_tests.rs`
    - Test create entity with valid data
    - Test create entity with duplicate name (should fail)
    - Test create entity exceeding Starter limit (should fail)
    - Test create entity exceeding Growth limit (should fail)
    - Test list entities filters by is_active
    - Test update entity with partial data
    - Test soft delete doesn't remove from database
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.6, 2.7, 2.8_
  
  - [ ] 1.16 Checkpoint - Ensure all tests pass
    - Ensure all tests pass, ask the user if questions arise.

- [-] 2. Entity API Routes & Subscription Middleware (Day 2: 6-8 hours)
  - [x] 2.1 Research Axum and utoipa patterns for 2025-2026
    - Use Exa or Tavily to search for "Axum middleware best practices 2025 2026"
    - Use Exa or Tavily to search for "utoipa OpenAPI generation patterns 2025"
    - Use Exa or Tavily to search for "Rust API error handling 2025"
    - Check for any updates to Axum or utoipa
    - Document findings for API route implementation
    - _Requirements: 2.1-2.15_
  
  - [x] 2.2 Create Entity API request/response types
    - Create file `backend/crates/api/src/routes/entities.rs`
    - Define CreateEntityRequest struct with utoipa ToSchema
    - Define UpdateEntityRequest struct with utoipa ToSchema
    - Define EntityResponse struct with utoipa ToSchema
    - Add utoipa examples for all fields
    - _Requirements: 2.1, 2.7_
  
  - [x] 2.2 Implement list_entities endpoint
    - In file `backend/crates/api/src/routes/entities.rs`
    - Implement GET /organizations/{org_id}/entities
    - Add utoipa path annotation with full documentation
    - Validate user has access to organization
    - Call EntityRepository.list_by_organization()
    - Return Vec<EntityResponse>
    - _Requirements: 2.6_
  
  - [x] 2.3 Implement create_entity endpoint
    - In file `backend/crates/api/src/routes/entities.rs`
    - Implement POST /organizations/{org_id}/entities
    - Add utoipa path annotation with full documentation
    - Validate user has access to organization
    - Call EntityRepository.create() (includes tier limit check)
    - Return EntityResponse with HTTP 201
    - Handle errors: ENTITY_LIMIT_EXCEEDED, DUPLICATE_ENTITY_NAME
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 19.1, 19.2_
  
  - [x] 2.4 Implement get_entity endpoint
    - In file `backend/crates/api/src/routes/entities.rs`
    - Implement GET /organizations/{org_id}/entities/{entity_id}
    - Add utoipa path annotation with full documentation
    - Validate user has access to organization
    - Call EntityRepository.find_by_id()
    - Return EntityResponse or 404
    - _Requirements: 2.6_
  
  - [x] 2.5 Implement update_entity endpoint
    - In file `backend/crates/api/src/routes/entities.rs`
    - Implement PATCH /organizations/{org_id}/entities/{entity_id}
    - Add utoipa path annotation with full documentation
    - Validate user has access to organization
    - Call EntityRepository.update()
    - Return EntityResponse
    - _Requirements: 2.7_
  
  - [x] 2.6 Implement delete_entity endpoint
    - In file `backend/crates/api/src/routes/entities.rs`
    - Implement DELETE /organizations/{org_id}/entities/{entity_id}
    - Add utoipa path annotation with full documentation
    - Validate user has access to organization
    - Call EntityRepository.delete() (soft delete)
    - Return HTTP 204 No Content
    - _Requirements: 2.8_
  
  - [x] 2.7 Register entity routes
    - Modify file `backend/crates/api/src/routes/mod.rs`
    - Add entities module
    - Register all entity endpoints in ApiDoc OpenAPI spec
    - Register all entity schemas in ApiDoc components
    - Add entity routes to router
    - _Requirements: 2.1, 2.6, 2.7, 2.8_
  
  - [x] 2.8 Update subscription middleware
    - Modify file `backend/crates/api/src/middleware/subscription.rs`
    - Change to query user's subscription fields instead of organization's
    - Extract user_id from Claims
    - Query users table for subscription_status
    - Allow requests for 'active' and 'trialing' status
    - Reject with HTTP 402 for 'expired' and 'cancelled' status
    - Return appropriate error messages
    - _Requirements: 1.5, 6.1, 6.2, 6.3, 6.4, 19.5_


  - [x] 2.9 Update organization routes
    - Modify file `backend/crates/api/src/routes/organizations.rs`
    - Remove subscription fields from OrganizationResponse
    - Update create_organization to create default entity
    - Update utoipa annotations
    - _Requirements: 1.4, 2.1_
  
  - [x] 2.10 Update intercompany repository
    - Modify file `backend/crates/db/src/repositories/intercompany.rs`
    - Update get_mappings() to use source_entity_id parameter
    - Update find_mapping_by_account() to use source_entity_id parameter
    - Update validate_mapping() to check entities in same organization
    - Remove cross-organization access checks
    - Simplify validation logic
    - _Requirements: 4.2, 4.3, 4.4, 4.6_
  
  - [x] 2.11 Update intercompany core logic
    - Modify file `backend/crates/core/src/ledger/intercompany.rs`
    - Update IntercompanyEngine.validate_entities() to check same organization
    - Simplify validation (no cross-org complexity)
    - Update generate_mirror_transaction() to use entity_id
    - Update generate_elimination_transaction() to use entity_id
    - _Requirements: 4.2, 4.5_
  
  - [x] 2.12 Update intercompany API routes
    - Modify file `backend/crates/api/src/routes/sentinel.rs`
    - Update CreateIntercompanyMappingRequest to use source_entity_id and target_entity_id
    - Update list_intercompany_mappings endpoint
    - Update create_intercompany_mapping endpoint
    - Update utoipa annotations
    - _Requirements: 4.1, 4.2, 4.4_
  
  - [-] 2.13 Generate OpenAPI specification
    - Run `cd backend && cargo run --bin generate-openapi`
    - Run `cd ../contracts && python3 split-openapi.py`
    - Verify openapi.yaml contains entity endpoints
    - Verify entity_id parameters in existing endpoints
    - _Requirements: 13.1, 13.2_
  
  - [x] 2.14 Run backend validation
    - Run `cargo fmt` to format code
    - Run `cargo clippy -- -D warnings` to check for issues
    - Use getDiagnostics tool on all modified backend files
    - Fix any errors or warnings
    - Run `cargo test` to ensure tests pass
  
  - [ ] 2.16 Write property-based tests for subscription middleware
    - Create test file `backend/crates/api/tests/subscription_middleware_properties.rs`
    - **Property 16**: Subscription status middleware - test all status combinations
    - Generate random users with different subscription statuses
    - Test that 'active' and 'trialing' allow requests
    - Test that 'expired' and 'cancelled' reject with HTTP 402
    - Use quickcheck library with 100 iterations
    - Tag test: `Feature: entities-model-implementation, Property 16: Subscription Status Middleware`
    - _Requirements: 6.1, 6.2, 6.3, 6.4_
  
  - [ ] 2.17 Write unit tests for subscription middleware
    - Create test file `backend/crates/api/tests/subscription_middleware_tests.rs`
    - Test active user can make request
    - Test trialing user can make request
    - Test expired user gets HTTP 402
    - Test cancelled user gets HTTP 402
    - Test error message format
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 19.5_
  
  - [ ] 2.18 Write integration tests for entity API
    - Create test file `backend/crates/api/tests/entity_api_integration.rs`
    - Test full entity CRUD flow
    - Test entity creation with different subscription tiers
    - Test entity limit enforcement via API
    - Test entity filtering and listing
    - Test error responses (400, 404, 403)
    - _Requirements: 2.1, 2.2, 2.6, 2.7, 2.8, 19.1, 19.2, 19.4_
  
  - [ ] 2.19 Checkpoint - Ensure all tests pass
    - Ensure all tests pass, ask the user if questions arise.

- [x] 3. Core Transaction & Account Routes (Day 3: 6-8 hours)
  - [x] 3.1 Update transaction request/response types
    - Modify file `backend/crates/api/src/routes/transactions.rs`
    - Add entity_id field to CreateTransactionRequest
    - Add entity_id field to TransactionResponse
    - Update utoipa ToSchema annotations
    - Add utoipa examples for entity_id
    - _Requirements: 3.2_
  
  - [x] 3.2 Update create_transaction endpoint
    - Modify file `backend/crates/api/src/routes/transactions.rs`
    - Add entity_id parameter to request
    - Validate entity_id is provided (return 400 if missing)
    - Validate user has access to entity
    - Pass entity_id to TransactionRepository.create()
    - Update utoipa path annotation
    - _Requirements: 3.2, 19.3_
  
  - [x] 3.3 Update list_transactions endpoint
    - Modify file `backend/crates/api/src/routes/transactions.rs`
    - Add entity_id query parameter (optional)
    - Filter transactions by entity_id if provided
    - Update utoipa path annotation
    - _Requirements: 3.9_
  
  - [x] 3.4 Update other transaction endpoints
    - Modify file `backend/crates/api/src/routes/transactions.rs`
    - Update get_transaction to include entity_id in response
    - Update update_transaction to validate entity access
    - Update delete_transaction to validate entity access
    - Update utoipa annotations
    - _Requirements: 3.2, 3.9_
  
  - [x] 3.5 Update account request/response types
    - Modify file `backend/crates/api/src/routes/accounts.rs`
    - Add entity_id field to CreateAccountRequest
    - Add entity_id field to AccountResponse
    - Update utoipa ToSchema annotations
    - Add utoipa examples for entity_id
    - _Requirements: 3.1_
  
  - [x] 3.6 Update create_account endpoint
    - Modify file `backend/crates/api/src/routes/accounts.rs`
    - Add entity_id parameter to request
    - Validate entity_id is provided (return 400 if missing)
    - Validate user has access to entity
    - Pass entity_id to AccountRepository.create()
    - Update utoipa path annotation
    - _Requirements: 3.1, 19.3_
  
  - [x] 3.7 Update list_accounts endpoint
    - Modify file `backend/crates/api/src/routes/accounts.rs`
    - Add entity_id query parameter (optional)
    - Filter accounts by entity_id if provided
    - Update utoipa path annotation
    - _Requirements: 3.8_
  
  - [x] 3.8 Update other account endpoints
    - Modify file `backend/crates/api/src/routes/accounts.rs`
    - Update get_account to include entity_id in response
    - Update update_account to validate entity access
    - Update delete_account to validate entity access
    - Update utoipa annotations
    - _Requirements: 3.1, 3.8_
  
  - [x] 3.9 Update budget request/response types
    - Modify file `backend/crates/api/src/routes/budgets.rs`
    - Add entity_id field to CreateBudgetRequest
    - Add entity_id field to BudgetResponse
    - Update utoipa ToSchema annotations
    - Add utoipa examples for entity_id
    - _Requirements: 3.4_
  
  - [x] 3.10 Update create_budget endpoint
    - Modify file `backend/crates/api/src/routes/budgets.rs`
    - Add entity_id parameter to request
    - Validate entity_id is provided (return 400 if missing)
    - Validate user has access to entity
    - Pass entity_id to BudgetRepository.create()
    - Update utoipa path annotation
    - _Requirements: 3.4, 19.3_
  
  - [x] 3.11 Update list_budgets endpoint
    - Modify file `backend/crates/api/src/routes/budgets.rs`
    - Add entity_id query parameter (optional)
    - Filter budgets by entity_id if provided
    - Update utoipa path annotation
    - _Requirements: 3.10_
  
  - [x] 3.12 Update other budget endpoints
    - Modify file `backend/crates/api/src/routes/budgets.rs`
    - Update get_budget to include entity_id in response
    - Update update_budget to validate entity access
    - Update delete_budget to validate entity access
    - Update utoipa annotations
    - _Requirements: 3.4, 3.10_
  
  - [x] 3.13 Update fiscal year routes
    - Modify file `backend/crates/api/src/routes/fiscal.rs`
    - Add entity_id to CreateFiscalYearRequest
    - Add entity_id to FiscalYearResponse
    - Add entity_id query parameter to list endpoint
    - Update all utoipa annotations
    - _Requirements: 3.5_
  
  - [x] 3.14 Generate OpenAPI specification
    - Run `cd backend && cargo run --bin generate-openapi`
    - Run `cd ../contracts && python3 split-openapi.py`
    - Verify entity_id in transaction/account/budget/fiscal endpoints
    - _Requirements: 13.1, 13.2_
  
  - [-] 3.15 Run backend validation
    - Run `cargo fmt` to format code
    - Run `cargo clippy -- -D warnings` to check for issues
    - Use getDiagnostics tool on all modified backend files
    - Fix any errors or warnings
    - Run `cargo test` to ensure tests pass
  
  - [ ] 3.17 Write property-based tests for entity-scoped data
    - Create test file `backend/crates/db/tests/entity_scoped_data_properties.rs`
    - **Property 11**: Entity-scoped data creation - test entity_id required for all data types
    - **Property 12**: Entity-scoped data filtering - test filtering by entity_id
    - Generate random entities and data
    - Test accounts, transactions, budgets require entity_id
    - Test filtering returns only matching entity_id
    - Use quickcheck library with 100 iterations per property
    - Tag tests: `Feature: entities-model-implementation, Property 11/12: {property_text}`
    - _Requirements: 3.1, 3.2, 3.4, 3.8, 3.9, 3.10_
  
  - [ ] 3.18 Write unit tests for entity-scoped routes
    - Create test file `backend/crates/api/tests/entity_scoped_routes_tests.rs`
    - Test create transaction without entity_id fails with 400
    - Test create transaction with entity_id succeeds
    - Test create account without entity_id fails with 400
    - Test create account with entity_id succeeds
    - Test list transactions filters by entity_id
    - Test list accounts filters by entity_id
    - Test unauthorized entity access returns 403
    - _Requirements: 3.1, 3.2, 3.8, 3.9, 19.3, 19.4_
  
  - [ ] 3.19 Checkpoint - Ensure all tests pass
    - Ensure all tests pass, ask the user if questions arise.

- [ ] 4. Reports, Dashboard & Sentinel Routes (Day 4: 6-8 hours)
  - [ ] 4.1 Update report routes
    - Modify file `backend/crates/api/src/routes/reports.rs`
    - Add entity_id query parameter to trial_balance endpoint
    - Add entity_id query parameter to balance_sheet endpoint
    - Add entity_id query parameter to income_statement endpoint
    - Add entity_id query parameter to dimensional_report endpoint
    - Add entity_id query parameter to account_ledger endpoint
    - Add consolidated query parameter (boolean) to all report endpoints
    - Update utoipa path annotations
    - _Requirements: 10.1, 10.2, 10.3_
  
  - [ ] 4.2 Implement entity filtering in report generation
    - Modify file `backend/crates/api/src/routes/reports.rs`
    - Update report queries to filter by entity_id when provided
    - Implement consolidated mode (query all entities, combine data)
    - Implement intercompany elimination for consolidated reports
    - _Requirements: 10.2, 10.3, 10.4_
  
  - [ ] 4.3 Update dashboard routes
    - Modify file `backend/crates/api/src/routes/dashboard.rs`
    - Add entity_id query parameter to all dashboard endpoints
    - Add consolidated query parameter to all dashboard endpoints
    - Update utoipa path annotations
    - _Requirements: 16.1, 16.2, 16.3_
  
  - [ ] 4.4 Implement entity filtering in dashboard metrics
    - Modify file `backend/crates/api/src/routes/dashboard.rs`
    - Update dashboard queries to filter by entity_id when provided
    - Implement consolidated mode for dashboard metrics
    - Implement intercompany elimination for consolidated metrics
    - _Requirements: 16.2, 16.3, 16.4_


  - [ ] 4.5 Update sentinel routes (accruals)
    - Modify file `backend/crates/api/src/routes/sentinel.rs`
    - Add entity_id to CreateAccrualScheduleRequest
    - Add entity_id to AccrualScheduleResponse
    - Add entity_id query parameter to list_accrual_schedules endpoint
    - Update utoipa annotations
    - _Requirements: 3.6_
  
  - [ ] 4.6 Update sentinel routes (revaluation)
    - Modify file `backend/crates/api/src/routes/sentinel.rs`
    - Add entity_id to RevaluationRequest
    - Add entity_id to RevaluationLogResponse
    - Add entity_id query parameter to list_revaluation_logs endpoint
    - Update utoipa annotations
    - _Requirements: 3.7_
  
  - [ ] 4.7 Update forensic routes
    - Modify file `backend/crates/api/src/routes/forensic.rs`
    - Add entity_id query parameter to all forensic endpoints
    - Update forensic analysis to filter by entity_id
    - Update utoipa annotations
    - _Requirements: 17.1, 17.2_
  
  - [ ] 4.8 Update simulation routes
    - Modify file `backend/crates/api/src/routes/simulation.rs`
    - Add entity_id query parameter to simulation endpoints
    - Update simulation logic to use entity_id
    - Update utoipa annotations
    - _Requirements: 17.3, 17.4_
  
  - [ ] 4.9 Update background jobs
    - Modify file `backend/crates/api/src/jobs/trial_expiry.rs`
    - Update to query users table for trial_ends_at
    - Update to set subscription_status on users table
    - Remove organization-based trial checks
    - _Requirements: 1.2, 14.1, 14.2_
  
  - [ ] 4.10 Update sync job
    - Modify file `backend/bins/server/src/sync.rs`
    - Update tier limit checks to use user's subscription_tier
    - Update entity count checks to count entities per organization
    - _Requirements: 14.4, 14.5_
  
  - [ ] 4.11 Generate OpenAPI specification
    - Run `cd backend && cargo run --bin generate-openapi`
    - Run `cd ../contracts && python3 split-openapi.py`
    - Verify entity_id in all updated endpoints
    - _Requirements: 13.1, 13.2_
  
  - [ ] 4.12 Run backend validation
    - Run `cargo fmt` to format code
    - Run `cargo clippy -- -D warnings` to check for issues
    - Use getDiagnostics tool on all modified backend files
    - Fix any errors or warnings
    - Run `cargo test` to ensure tests pass
  
  - [ ] 4.14 Write property-based tests for intercompany
    - Create test file `backend/crates/db/tests/intercompany_properties.rs`
    - **Property 13**: Intercompany same-organization validation
    - **Property 14**: Intercompany mapping filtering
    - **Property 15**: Intercompany transaction processing
    - Generate random entities in same/different orgs
    - Test validation succeeds for same org, fails for different orgs
    - Test mapping filtering returns only org's mappings
    - Test mirror/elimination entry generation
    - Use quickcheck library with 100 iterations per property
    - Tag tests: `Feature: entities-model-implementation, Property 13/14/15: {property_text}`
    - _Requirements: 4.2, 4.3, 4.4, 4.5_
  
  - [ ] 4.15 Write unit tests for intercompany
    - Create test file `backend/crates/db/tests/intercompany_tests.rs`
    - Test create mapping with entities in same org succeeds
    - Test create mapping with entities in different orgs fails
    - Test error message for different orgs
    - Test list mappings filters by organization
    - Test mirror transaction generation
    - Test elimination transaction generation
    - _Requirements: 4.2, 4.3, 4.4, 4.5, 19.3_
  
  - [ ] 4.16 Write property-based tests for reports
    - Create test file `backend/crates/api/tests/report_properties.rs`
    - **Property 17**: Report entity filtering
    - **Property 18**: Consolidated report generation
    - Generate random entities and transactions
    - Test report filters by entity_id correctly
    - Test consolidated mode combines all entities
    - Test intercompany elimination in consolidated reports
    - Use quickcheck library with 100 iterations per property
    - Tag tests: `Feature: entities-model-implementation, Property 17/18: {property_text}`
    - _Requirements: 10.2, 10.3, 10.4_
  
  - [ ] 4.17 Write unit tests for background jobs
    - Create test file `backend/crates/api/tests/background_jobs_tests.rs`
    - Test trial expiry job queries users table
    - Test trial expiry job updates expired trials
    - Test sync job uses user subscription tier
    - Test sync job counts entities per organization
    - _Requirements: 14.1, 14.2, 14.4, 14.5_
  
  - [ ] 4.18 Checkpoint - Ensure all tests pass
    - Ensure all tests pass, ask the user if questions arise.

- [ ] 5. Frontend Types, Queries & Auth Store (Day 5: 6-8 hours)
  - [ ] 5.1 Research React Query and TypeScript patterns for 2025-2026
    - Use Exa or Tavily to search for "TanStack Query v5 best practices 2025 2026"
    - Use Exa or Tavily to search for "React Query mutation patterns 2025"
    - Use Exa or Tavily to search for "TypeScript React hooks patterns 2025"
    - Check for any breaking changes from v4 to v5
    - Document findings for query implementation
    - _Requirements: 5.6-5.16_
  
  - [ ] 5.2 Generate frontend types from OpenAPI
    - Run `cd frontend && pnpm run generate:types`
    - Verify Entity type generated in `frontend/src/types/api.generated.ts`
    - Verify entity_id in transaction/account/budget types
    - Verify CreateEntityRequest and UpdateEntityRequest types
    - _Requirements: 13.3, 13.4, 13.5_
  
  - [ ] 5.2 Create entity types file
    - Create file `frontend/src/types/entities.ts`
    - Export Entity type (re-export from api.generated.ts)
    - Export CreateEntityRequest type
    - Export UpdateEntityRequest type
    - Export EntityResponse type
    - _Requirements: 2.1, 2.7_
  
  - [ ] 5.3 Update organization types
    - Modify file `frontend/src/types/organizations.ts`
    - Remove subscription_tier field
    - Remove subscription_status field
    - Remove trial_ends_at field
    - Remove subscription_ends_at field
    - Remove payment_provider field
    - Remove payment_customer_id field
    - Remove payment_subscription_id field
    - _Requirements: 1.4_
  
  - [ ] 5.4 Add UserSubscription type
    - Modify file `frontend/src/types/auth.ts`
    - Add UserSubscription type with all subscription fields
    - Export type
    - _Requirements: 1.1, 1.2, 1.3_
  
  - [ ] 5.5 Update auth store
    - Modify file `frontend/src/lib/stores/authStore.ts`
    - Add currentEntityId state (string | null)
    - Add setCurrentEntityId action
    - Update clearAuth to clear currentEntityId
    - _Requirements: 7.3, 15.1_
  
  - [ ] 5.6 Create entity queries
    - Create file `frontend/src/lib/queries/entities.ts`
    - Implement useEntities() hook (list entities for current org)
    - Implement useEntity(entityId) hook (get single entity)
    - Implement useCreateEntity() mutation hook
    - Implement useUpdateEntity(entityId) mutation hook
    - Implement useDeleteEntity(entityId) mutation hook
    - Add query invalidation on mutations
    - _Requirements: 2.1, 2.6, 2.7, 2.8_
  
  - [ ] 5.7 Add user subscription query
    - Modify file `frontend/src/lib/queries/auth.ts`
    - Implement useUserSubscription() hook
    - Query user's subscription fields from /users/me endpoint
    - _Requirements: 11.1, 11.2, 11.3_
  
  - [ ] 5.8 Update organization queries
    - Modify file `frontend/src/lib/queries/organizations.ts`
    - Remove useCreateOrganization() hook (one org per user)
    - Simplify useOrganizations() hook
    - Update types to remove subscription fields
    - _Requirements: 1.4_
  
  - [ ] 5.9 Update transaction queries
    - Modify file `frontend/src/lib/queries/transactions.ts`
    - Add entity_id parameter to useTransactions() hook
    - Add entity_id to useCreateTransaction() mutation
    - Update types to include entity_id
    - _Requirements: 3.2, 3.9_
  
  - [ ] 5.10 Update account queries
    - Modify file `frontend/src/lib/queries/accounts.ts`
    - Add entity_id parameter to useAccounts() hook
    - Add entity_id to useCreateAccount() mutation
    - Update types to include entity_id
    - _Requirements: 3.1, 3.8_
  
  - [ ] 5.11 Update budget queries
    - Modify file `frontend/src/lib/queries/budgets.ts`
    - Add entity_id parameter to useBudgets() hook
    - Add entity_id to useCreateBudget() mutation
    - Update types to include entity_id
    - _Requirements: 3.4, 3.10_
  
  - [ ] 5.12 Update fiscal queries
    - Modify file `frontend/src/lib/queries/fiscal.ts`
    - Add entity_id parameter to useFiscalYears() hook
    - Add entity_id to useCreateFiscalYear() mutation
    - Update types to include entity_id
    - _Requirements: 3.5_
  
  - [ ] 5.13 Update sentinel queries
    - Modify file `frontend/src/lib/queries/sentinel.ts`
    - Add entity_id to accrual schedule queries
    - Add entity_id to revaluation queries
    - Update intercompany queries to use entity_id instead of org_id
    - Update types
    - _Requirements: 3.6, 3.7, 4.1_
  
  - [ ] 5.14 Update report queries
    - Modify file `frontend/src/lib/queries/reports.ts`
    - Add entity_id parameter to all report hooks
    - Add consolidated parameter to all report hooks
    - Update types
    - _Requirements: 10.1, 10.2, 10.3_
  
  - [ ] 5.15 Update dashboard queries
    - Modify file `frontend/src/lib/queries/dashboard.ts`
    - Add entity_id parameter to dashboard hooks
    - Add consolidated parameter to dashboard hooks
    - Update types
    - _Requirements: 16.1, 16.2, 16.3_
  
  - [ ] 5.16 Update forensic and simulation queries
    - Modify file `frontend/src/lib/queries/forensic.ts` - add entity_id parameter
    - Modify file `frontend/src/lib/queries/simulation.ts` - add entity_id parameter
    - Update types
    - _Requirements: 17.1, 17.3_
  
  - [ ] 5.17 Update intercompany types
    - Modify file `frontend/src/types/api-helpers.ts`
    - Update IntercompanyMapping type to use source_entity_id and target_entity_id
    - Remove source_org_id and target_org_id
    - Update CreateIntercompanyMappingRequest type
    - _Requirements: 4.1_
  
  - [ ] 5.18 Run frontend validation
    - Run `pnpm lint` to check for linting issues
    - Run `pnpm type-check` to check TypeScript types
    - Use getDiagnostics tool on all modified frontend files
    - Fix any errors or warnings
    - Run `pnpm build` to ensure build succeeds
  
  - [ ] 5.19 Checkpoint - Ensure all tests pass
    - Ensure all tests pass, ask the user if questions arise.

- [ ] 6. Frontend Components & Forms (Day 6: 6-8 hours)
  - [ ] 6.1 Create EntitySelector component
    - Create file `frontend/src/components/entities/EntitySelector.tsx`
    - Implement dropdown to select entity
    - Use useEntities() hook to fetch entities
    - Use useAuth() to get/set currentEntityId
    - Auto-select if only one entity
    - Persist selection to localStorage
    - Restore from localStorage on mount
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 15.1, 15.2, 15.3, 15.4_
  
  - [ ] 6.2 Update Sidebar component
    - Modify file `frontend/src/components/layout/Sidebar.tsx`
    - Replace organization switcher with EntitySelector
    - Update tier checks to use useUserSubscription() instead of org subscription
    - _Requirements: 7.1, 11.6_
  
  - [ ] 6.3 Update CreateTransactionDialog
    - Modify file `frontend/src/components/transactions/CreateTransactionDialog.tsx`
    - Add entity selector field
    - Pass entity_id to useCreateTransaction() mutation
    - Add validation: entity_id required
    - Update tier check to use user subscription (line 159)
    - _Requirements: 8.1, 8.4, 8.5_


  - [ ] 6.4 Update AccountForm component
    - Modify file `frontend/src/components/accounts/AccountForm.tsx`
    - Add entity selector field
    - Pass entity_id to useCreateAccount() mutation
    - Add validation: entity_id required
    - _Requirements: 8.2, 8.4, 8.5_
  
  - [ ] 6.5 Update budget forms
    - Find and modify budget form components in `frontend/src/app/dashboard/budgets/`
    - Add entity selector field
    - Pass entity_id to useCreateBudget() mutation
    - Add validation: entity_id required
    - _Requirements: 8.3, 8.4, 8.5_
  
  - [ ] 6.6 Update UpgradeModal component
    - Modify file `frontend/src/components/modals/UpgradeModal.tsx`
    - Use useUserSubscription() instead of organization subscription
    - Update tier display
    - _Requirements: 11.1, 11.2_
  
  - [ ] 6.7 Update UsageMeter component
    - Modify file `frontend/src/components/dashboard/UsageMeter.tsx`
    - Use useUserSubscription() instead of organization subscription
    - Update tier display
    - Display entity count and limit
    - _Requirements: 11.1, 11.4, 11.5_
  
  - [ ] 6.8 Update BudgetVsActual component
    - Modify file `frontend/src/components/dashboard/BudgetVsActual.tsx`
    - Add entity_id parameter to data queries
    - Filter data by selected entity
    - _Requirements: 16.7_
  
  - [ ] 6.9 Update RecentActivity component
    - Modify file `frontend/src/components/dashboard/RecentActivity.tsx`
    - Add entity_id parameter to data queries
    - Filter data by selected entity
    - _Requirements: 16.6_
  
  - [ ] 6.10 Update simulation components
    - Modify file `frontend/src/components/simulation/SimulationControls.tsx` - add entity selector
    - Modify file `frontend/src/components/simulation/SimulationChart.tsx` - add entity filter
    - _Requirements: 17.3, 17.4_
  
  - [ ] 6.11 Write unit tests for EntitySelector component
    - Create test file `frontend/src/components/entities/__tests__/EntitySelector.test.tsx`
    - Test component renders with entities list
    - Test auto-select when only one entity
    - Test localStorage persistence
    - Test localStorage restoration
    - Test entity selection updates context
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 15.1, 15.2, 15.3_
  
  - [ ] 6.12 Write unit tests for entity queries
    - Create test file `frontend/src/lib/queries/__tests__/entities.test.ts`
    - Test useEntities hook fetches entities
    - Test useCreateEntity mutation
    - Test useUpdateEntity mutation
    - Test useDeleteEntity mutation
    - Test query invalidation after mutations
    - _Requirements: 2.1, 2.6, 2.7, 2.8_
  
  - [ ] 6.13 Write unit tests for auth store
    - Create test file `frontend/src/lib/stores/__tests__/authStore.test.ts`
    - Test setCurrentEntityId updates state
    - Test clearAuth clears currentEntityId
    - Test entity context persistence
    - _Requirements: 7.3, 15.1_
  
  - [ ] 6.14 Run frontend validation
    - Run `pnpm lint` to check for linting issues
    - Run `pnpm type-check` to check TypeScript types
    - Use getDiagnostics tool on all modified frontend files
    - Fix any errors or warnings
    - Run `pnpm build` to ensure build succeeds
  
  - [ ] 6.15 Checkpoint - Ensure all tests pass
    - Ensure all tests pass, ask the user if questions arise.

- [ ] 7. Frontend Pages & E2E Testing (Day 7: 8-12 hours)
  - [ ] 7.1 Research Next.js 16 and Playwright patterns for 2025-2026
    - Use Exa or Tavily to search for "Next.js 16 App Router best practices 2025 2026"
    - Use Exa or Tavily to search for "Next.js 16 server components patterns 2025"
    - Use Exa or Tavily to search for "Playwright E2E testing best practices 2025"
    - Use Exa or Tavily to search for "Playwright component testing 2025"
    - Check for any breaking changes in Next.js 16
    - Document findings for page and E2E test implementation
    - _Requirements: 7.1-7.26_
  
  - [ ] 7.2 Update main dashboard page
    - Modify file `frontend/src/app/dashboard/page.tsx`
    - Add EntitySelector component
    - Add entity_id parameter to all dashboard queries
    - Support consolidated mode (all entities)
    - Update all dashboard widgets to respect entity filter
    - _Requirements: 16.1, 16.2, 16.3_
  
  - [ ] 7.2 Update settings page
    - Modify file `frontend/src/app/dashboard/settings/page.tsx`
    - Remove organization subscription display (lines 129-147)
    - _Requirements: 11.6_
  
  - [ ] 7.3 Create user subscription settings page
    - Create file `frontend/src/app/dashboard/settings/subscription/page.tsx`
    - Display user subscription tier
    - Display user subscription status
    - Display trial end date if trialing
    - Display entity limits and current count
    - _Requirements: 11.1, 11.2, 11.3, 11.4, 11.5_
  
  - [ ] 7.4 Update transactions list page
    - Modify file `frontend/src/app/dashboard/transactions/page.tsx`
    - Add entity filter dropdown
    - Pass entity_id to useTransactions() query
    - Show entity name in transaction rows
    - _Requirements: 9.1, 9.7_
  
  - [ ] 7.5 Update accounts list page
    - Modify file `frontend/src/app/dashboard/accounts/page.tsx`
    - Add entity filter dropdown
    - Pass entity_id to useAccounts() query
    - Show entity name in account rows
    - _Requirements: 9.2, 9.7_
  
  - [ ] 7.6 Update budgets list page
    - Modify file `frontend/src/app/dashboard/budgets/page.tsx`
    - Add entity filter dropdown
    - Pass entity_id to useBudgets() query
    - Show entity name in budget rows
    - _Requirements: 9.3, 9.7_
  
  - [ ] 7.7 Update accruals list page
    - Modify file `frontend/src/app/dashboard/accruals/page.tsx` (if exists)
    - Add entity filter dropdown
    - Pass entity_id to accrual queries
    - Show entity name in accrual rows
    - _Requirements: 9.4, 9.7_
  
  - [ ] 7.8 Update revaluation list page
    - Modify file `frontend/src/app/dashboard/revaluation/page.tsx` (if exists)
    - Add entity filter dropdown
    - Pass entity_id to revaluation queries
    - Show entity name in revaluation rows
    - _Requirements: 9.5, 9.7_
  
  - [ ] 7.9 Update fiscal periods page
    - Modify file `frontend/src/app/dashboard/master-data/fiscal-periods/page.tsx`
    - Add entity filter dropdown
    - Pass entity_id to fiscal queries
    - Show entity name in fiscal period rows
    - _Requirements: 9.6, 9.7_
  
  - [ ] 7.10 Update trial balance report page
    - Modify file `frontend/src/app/dashboard/reports/trial-balance/page.tsx`
    - Add entity selector with "All Entities (Consolidated)" option
    - Pass entity_id or consolidated=true to report query
    - Show entity name in report header
    - _Requirements: 10.1, 10.2, 10.3, 10.5_
  
  - [ ] 7.11 Update balance sheet report page
    - Modify file `frontend/src/app/dashboard/reports/balance-sheet/page.tsx`
    - Add entity selector with "All Entities (Consolidated)" option
    - Pass entity_id or consolidated=true to report query
    - Show entity name in report header
    - _Requirements: 10.1, 10.2, 10.3, 10.6_
  
  - [ ] 7.12 Update income statement report page
    - Modify file `frontend/src/app/dashboard/reports/income-statement/page.tsx`
    - Add entity selector with "All Entities (Consolidated)" option
    - Pass entity_id or consolidated=true to report query
    - Show entity name in report header
    - _Requirements: 10.1, 10.2, 10.3, 10.7_
  
  - [ ] 7.13 Update dimensional report page
    - Modify file `frontend/src/app/dashboard/reports/dimensional/page.tsx`
    - Add entity selector with "All Entities (Consolidated)" option
    - Pass entity_id or consolidated=true to report query
    - Show entity name in report header
    - _Requirements: 10.1, 10.2, 10.3, 10.8_
  
  - [ ] 7.14 Update account ledger report page
    - Modify file `frontend/src/app/dashboard/reports/account-ledger/page.tsx`
    - Add entity selector (single entity only, no consolidated)
    - Pass entity_id to report query
    - Show entity name in report header
    - _Requirements: 10.1, 10.2, 10.9_
  
  - [ ] 7.15 Update intercompany page
    - Modify file `frontend/src/app/dashboard/intercompany/page.tsx`
    - Update to use entity selectors instead of organization selectors
    - Update CreateIntercompanyMappingRequest to use entity_ids
    - Show entity names in mapping list
    - Remove organization selection logic
    - _Requirements: 18.1, 18.2, 18.3, 18.4, 18.5_
  
  - [ ] 7.16 Update forensic page
    - Modify file `frontend/src/app/dashboard/forensic/page.tsx`
    - Add entity selector
    - Pass entity_id to forensic queries
    - Show entity name in analysis header
    - _Requirements: 17.1, 17.2, 17.5_
  
  - [ ] 7.17 Update simulation page
    - Modify file `frontend/src/app/dashboard/simulation/page.tsx`
    - Add entity selector
    - Pass entity_id to simulation queries
    - Show entity name in simulation header
    - _Requirements: 17.3, 17.4, 17.6_
  
  - [ ] 7.18 Run frontend validation
    - Run `pnpm lint` to check for linting issues
    - Run `pnpm type-check` to check TypeScript types
    - Use getDiagnostics tool on all modified frontend files
    - Fix any errors or warnings
    - Run `pnpm build` to ensure build succeeds
  
  - [ ] 7.19 Write integration tests for entity selection flow
    - Create test file `frontend/src/__tests__/integration/entity-selection.test.tsx`
    - Test: Select entity → Form updates → Data filtered
    - Test: Create entity → Entity appears in selector
    - Test: Switch entity → Data refreshes
    - Test: Entity persistence across page navigation
    - _Requirements: 7.3, 8.4, 9.7, 15.1_
  
  - [ ] 7.20 Write E2E test: Create entity and transaction
    - Create test file or update existing E2E tests
    - Test: Login → Create entity → Create transaction with entity → View transaction in list
    - Verify entity name appears in transaction
    - Use test credentials: corp@zeltra.io / qwertyui
    - _Requirements: 20.7_
  
  - [ ] 7.21 Write E2E test: Intercompany mapping
    - Create test file or update existing E2E tests
    - Test: Create two entities → Create intercompany mapping → Post transaction → Verify mirror entry
    - Verify mapping appears in list with entity names
    - _Requirements: 20.8_
  
  - [ ] 7.22 Write E2E test: Entity filtering in reports
    - Create test file or update existing E2E tests
    - Test: Create entity → Create transactions → Generate report for entity → Verify data
    - Test: Generate consolidated report → Verify combined data
    - _Requirements: 20.9, 20.10_
  
  - [ ] 7.23 Write E2E test: Entity tier limits
    - Create test file or update existing E2E tests
    - Test: Create entities up to tier limit → Attempt to create one more → Verify error
    - Test: Upgrade tier → Create entity successfully
    - _Requirements: 2.2, 2.3, 2.4_
  
  - [ ] 7.24 Run E2E tests
    - Start backend: `cargo run --bin zeltra`
    - Start frontend: `pnpm dev`
    - Run E2E tests: `pnpm test:e2e`
    - Verify all tests pass
    - _Requirements: 20.7, 20.8, 20.9, 20.10_
  
  - [ ] 7.25 Manual testing
    - Login with test credentials: corp@zeltra.io / qwertyui
    - Test entity creation
    - Test entity selection in forms
    - Test entity filtering in lists
    - Test entity filtering in reports
    - Test consolidated reports
    - Test intercompany mappings
    - Test subscription display
    - Test tier limits
  
  - [ ] 7.26 Final checkpoint - Ensure all tests pass
    - Ensure all tests pass, ask the user if questions arise.

## Notes

- All tasks are required for complete implementation (no optional tasks)
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties
- Unit tests validate specific examples and edge cases
- Integration tests validate component interactions
- E2E tests validate complete user flows
- Use MCP tools (postgres, sequential thinking) for testing and validation
- Backend binary is `zeltra` (not zeltra-api)
- Frontend package manager is `pnpm` (not npm)
- Test credentials: corp@zeltra.io / qwertyui
