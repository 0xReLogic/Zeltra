# Requirements Document: Entities Model Implementation

## Overview

This document specifies the requirements for refactoring Zeltra's subscription model from organization-based to user-based, and replacing the multi-organization feature with an entities model. This change aligns the technical implementation with the business model where subscriptions are per user, and enables multi-entity accounting within a single workspace.

## Business Context

**Current Problem**: The system stores subscription fields per organization, but the business model charges per user:
- Starter: $12/mo per USER → 1 entity
- Growth: $25/mo per USER → 5 entities  
- Enterprise: Custom per USER → unlimited entities

This mismatch creates issues with trial inheritance, upgrade logic, and data consistency.

**Solution**: Move subscriptions to users, replace multi-organization with multi-entity accounting (similar to NetSuite/Sage Intacct), where users manage multiple companies within one workspace.

## Acceptance Criteria

### 1. User Subscription Management

**1.1** When a new user account is created, the user record MUST have subscription_tier set to 'starter', subscription_status set to 'trialing', and trial_ends_at set to a future date (default: 14 days from creation).

**1.2** When the trial expiry background job runs, any user with subscription_status 'trialing' and trial_ends_at in the past MUST have their subscription_status updated to 'expired'.

**1.3** When a user's subscription tier is updated (via admin action or payment webhook), the user's subscription_tier field MUST reflect the new tier value.

**1.4** When a new organization is created, the organization record MUST NOT contain subscription fields (subscription_tier, subscription_status, trial_ends_at, subscription_ends_at, payment_provider, payment_customer_id, payment_subscription_id).

**1.5** When a user makes an API request, the subscription middleware MUST check the user's subscription_status (not the organization's) to determine access.

### 2. Entity Management

**2.1** When a new organization is created, a default entity MUST be automatically created with name "{organization_name} (Main)", entity_type 'main', and is_active true.

**2.2** When a user attempts to create an entity, the system MUST check the user's subscription tier and count existing active entities for that organization. If the count equals or exceeds the tier limit, the creation MUST fail with error "Entity limit reached for your tier".

**2.3** When a user with subscription tier 'starter' attempts to create a second entity, the creation MUST fail with error "Entity limit reached for your tier".

**2.4** When a user with subscription tier 'growth' attempts to create a sixth entity, the creation MUST fail with error "Entity limit reached for your tier".

**2.5** When a user with subscription tier 'enterprise' creates any number of entities, all creations MUST succeed without limit checks.

**2.6** When entities are listed for an organization, the results MUST contain only entities with is_active true, ordered by created_at ascending.

**2.7** When an entity is updated with new values for name, legal_name, tax_id, entity_type, base_currency, or settings, the entity record MUST reflect the new values.

**2.8** When an entity is deleted, the entity record MUST remain in the database with is_active set to false (soft delete).

### 3. Entity-Scoped Data

**3.1** When creating a chart of accounts record, the request MUST include entity_id. If entity_id is missing, the creation MUST fail with validation error "entity_id is required". If entity_id is provided, the record MUST be associated with that entity.

**3.2** When creating a transaction record, the request MUST include entity_id. If entity_id is missing, the creation MUST fail with validation error "entity_id is required". If entity_id is provided, the record MUST be associated with that entity.

**3.3** When creating a ledger entry record, the request MUST include entity_id. If entity_id is missing, the creation MUST fail with validation error "entity_id is required". If entity_id is provided, the record MUST be associated with that entity.

**3.4** When creating a budget record, the request MUST include entity_id. If entity_id is missing, the creation MUST fail with validation error "entity_id is required". If entity_id is provided, the record MUST be associated with that entity.

**3.5** When creating a fiscal year record, the request MUST include entity_id. If entity_id is missing, the creation MUST fail with validation error "entity_id is required". If entity_id is provided, the record MUST be associated with that entity.

**3.6** When creating an accrual schedule record, the request MUST include entity_id. If entity_id is missing, the creation MUST fail with validation error "entity_id is required". If entity_id is provided, the record MUST be associated with that entity.

**3.7** When creating a revaluation log record, the request MUST include entity_id. If entity_id is missing, the creation MUST fail with validation error "entity_id is required". If entity_id is provided, the record MUST be associated with that entity.

**3.8** When querying chart of accounts with entity_id filter, the results MUST contain only records where entity_id equals the filter value.

**3.9** When querying transactions with entity_id filter, the results MUST contain only records where entity_id equals the filter value.

**3.10** When querying budgets with entity_id filter, the results MUST contain only records where entity_id equals the filter value.

### 4. Intercompany Mappings

**4.1** When an intercompany mapping is created or updated, the mapping MUST use source_entity_id and target_entity_id (not source_org_id and target_org_id).

**4.2** When creating an intercompany mapping between two entities, the creation MUST succeed if both entities belong to the same organization, and MUST fail with error "Entities must belong to the same organization" if they belong to different organizations.

**4.3** When creating an intercompany mapping between two entities in different organizations, the creation MUST fail with error "Entities must belong to the same organization".

**4.4** When listing intercompany mappings for an organization, the results MUST contain only mappings where both source_entity_id and target_entity_id belong to entities in that organization.

**4.5** When a transaction is posted to an account with an intercompany mapping, a corresponding mirror or elimination entry MUST be automatically created in the target entity according to the mapping_type.

**4.6** When validating an intercompany mapping, the system MUST NOT perform cross-organization access checks (simplified validation).

### 5. Data Migration

**5.1** When the migration runs, subscription data (subscription_tier, subscription_status, trial_ends_at, subscription_ends_at, payment_provider, payment_customer_id, payment_subscription_id) MUST be copied from each user's first (oldest) organization to the user record.

**5.2** When the migration runs, a default entity MUST be created for each existing organization with name "{organization_name} (Main)", entity_type 'main', and is_active true.

**5.3** When the migration runs, all existing chart_of_accounts records MUST be linked to their organization's default entity.

**5.4** When the migration runs, all existing transactions records MUST be linked to their organization's default entity.

**5.5** When the migration runs, all existing ledger_entries records MUST be linked to their organization's default entity.

**5.6** When the migration runs, all existing budgets records MUST be linked to their organization's default entity.

**5.7** When the migration runs, all existing fiscal_years records MUST be linked to their organization's default entity.

**5.8** When the migration runs, all existing accrual_schedules records MUST be linked to their organization's default entity.

**5.9** When the migration runs, all existing revaluation_logs records MUST be linked to their organization's default entity.

**5.10** After the migration completes, all entity_id foreign keys MUST be valid (no null values, all reference existing entities).

**5.11** After the migration completes, all users MUST have subscription data (no null values for subscription_tier or subscription_status).

### 6. Subscription Status Enforcement

**6.1** When a user with subscription_status 'active' makes an API request, the request MUST be allowed to proceed.

**6.2** When a user with subscription_status 'trialing' makes an API request, the request MUST be allowed to proceed.

**6.3** When a user with subscription_status 'expired' makes an API request, the request MUST be rejected with HTTP 402 Payment Required and error message "Your trial has expired. Please upgrade to continue."

**6.4** When a user with subscription_status 'cancelled' makes an API request, the request MUST be rejected with HTTP 402 Payment Required and error message "Your subscription has been cancelled. Please reactivate to continue."

### 7. Frontend Entity Context

**7.1** When the EntitySelector component loads and only one entity exists, that entity MUST be automatically selected.

**7.2** When a user selects an entity in the EntitySelector, the selection MUST be persisted to localStorage.

**7.3** When the EntitySelector component mounts, if a valid entity_id exists in localStorage, that entity MUST be restored as the current selection.

**7.4** When the current entity changes, all data queries MUST be refreshed to show data for the new entity.

### 8. Frontend Forms

**8.1** When creating a transaction, the form MUST include an entity_id field (either as a selector or hidden field using current entity context).

**8.2** When creating an account, the form MUST include an entity_id field (either as a selector or hidden field using current entity context).

**8.3** When creating a budget, the form MUST include an entity_id field (either as a selector or hidden field using current entity context).

**8.4** When submitting a form without entity_id, the submission MUST fail with validation error "entity_id is required".

### 9. Frontend Lists

**9.1** When viewing the transactions list, the list MUST be filtered by the currently selected entity_id.

**9.2** When viewing the accounts list, the list MUST be filtered by the currently selected entity_id.

**9.3** When viewing the budgets list, the list MUST be filtered by the currently selected entity_id.

**9.4** When the user changes the selected entity, all lists MUST refresh to show data for the new entity.

### 10. Reports

**10.1** When generating a financial report (trial balance, balance sheet, income statement, dimensional), the user MUST be able to select a specific entity or "All Entities" (consolidated).

**10.2** When generating a report for a specific entity, the report MUST include only data where entity_id equals the selected entity.

**10.3** When generating a consolidated report, the report MUST combine data from all entities in the organization.

**10.4** When generating a consolidated report, the report MUST eliminate intercompany transactions based on intercompany mappings.

### 11. Dashboard

**11.1** When viewing the dashboard, the user MUST be able to select a specific entity or "All Entities" (consolidated).

**11.2** When viewing the dashboard for a specific entity, all metrics MUST be calculated using only data where entity_id equals the selected entity.

**11.3** When viewing the consolidated dashboard, all metrics MUST be calculated using data from all entities in the organization.

**11.4** When viewing the consolidated dashboard, intercompany transactions MUST be eliminated from metrics based on intercompany mappings.

### 12. Tier Limits

**12.1** When counting entities for tier limit checks, the count MUST include only entities where is_active is true.

**12.2** When a user with subscription tier 'starter' has 1 active entity, attempting to create another entity MUST fail with error "Entity limit reached for your tier".

**12.3** When a user with subscription tier 'growth' has 5 active entities, attempting to create another entity MUST fail with error "Entity limit reached for your tier".

**12.4** When counting entities for tier limit checks, entities with is_active false MUST NOT be included in the count.

### 13. API Documentation

**13.1** When the OpenAPI specification is generated, it MUST include all entity endpoints (list, create, get, update, delete).

**13.2** When the OpenAPI specification is generated, all entity-scoped endpoints MUST document the entity_id parameter (either in request body or query string).

### 14. Background Jobs

**14.1** When the trial expiry job runs, it MUST query the users table for trial_ends_at (not the organizations table).

**14.2** When the trial expiry job runs, it MUST update subscription_status on the users table (not the organizations table).

**14.3** When the session cleanup job runs, it MUST continue to function correctly (no changes required).

**14.4** When the sync job runs, it MUST use the user's subscription_tier for tier limit checks (not the organization's).

**14.5** When the sync job runs, it MUST count entities per organization for tier limit validation.

### 15. Authentication

**15.1** When a user logs in, the JWT token MUST include the user_id (no changes required).

**15.2** When a user accesses an entity, the system MUST verify the user has access to the organization that owns the entity.

**15.3** When a user attempts to access an entity they don't have access to, the request MUST fail with HTTP 403 Forbidden and error message "You don't have access to this entity".

### 16. Forensic Analysis

**16.1** When running forensic analysis, the user MUST be able to filter by entity_id.

**16.2** When forensic analysis is filtered by entity_id, the results MUST include only data for that entity.

**16.3** When forensic analysis is run without entity_id filter, the results MUST include data from all entities in the organization.

### 17. Simulation

**17.1** When running a simulation, the user MUST be able to specify an entity_id.

**17.2** When a simulation is run for a specific entity, the simulation MUST use only data from that entity.

**17.3** When a simulation is run without entity_id, the simulation MUST use data from all entities in the organization.

**17.4** When viewing simulation results, the results MUST clearly indicate which entity or entities were included.

### 18. Settings

**18.1** When viewing organization settings, subscription information MUST NOT be displayed (subscription is now per user).

**18.2** When viewing user profile settings, subscription information MUST be displayed (subscription_tier, subscription_status, trial_ends_at).

**18.3** When a user upgrades their subscription, the user's subscription_tier MUST be updated (not the organization's).

### 19. Error Handling

**19.1** When entity creation fails due to tier limit, the error response MUST have HTTP status 400 and error code "ENTITY_LIMIT_EXCEEDED" with message "Entity limit reached for your tier. Upgrade to create more entities."

**19.2** When entity creation fails due to duplicate name, the error response MUST have HTTP status 400 and error code "DUPLICATE_ENTITY_NAME" with message "An entity with this name already exists in your organization".

**19.3** When creating entity-scoped data without entity_id, the error response MUST have HTTP status 400 and error code "MISSING_ENTITY_ID" with message "entity_id is required".

**19.4** When accessing an entity without permission, the error response MUST have HTTP status 403 and error code "UNAUTHORIZED_ENTITY_ACCESS" with message "You don't have access to this entity".

**19.5** When a user with expired subscription makes a request, the error response MUST have HTTP status 402 and error code "TRIAL_EXPIRED" or "SUBSCRIPTION_EXPIRED" with appropriate message.

## Non-Functional Requirements

### Performance

**NFR-1** Entity listing queries MUST complete in less than 100ms for organizations with up to 100 entities.

**NFR-2** Entity-filtered transaction queries MUST complete in less than 200ms for entities with up to 10,000 transactions.

**NFR-3** Consolidated report generation MUST complete in less than 2 seconds for organizations with up to 5 entities and 50,000 total transactions.

**NFR-4** Tier limit validation MUST complete in less than 100ms.

**NFR-5** Data migration MUST complete in less than 5 minutes for databases with up to 100,000 records.

### Scalability

**NFR-6** The system MUST support organizations with up to 100 entities (enterprise tier).

**NFR-7** The system MUST support entities with up to 1,000,000 transactions each.

**NFR-8** The system MUST support up to 10,000 concurrent users.

### Reliability

**NFR-9** Data migration MUST be idempotent (can be run multiple times safely).

**NFR-10** Data migration MUST include validation checks to ensure data integrity.

**NFR-11** Data migration MUST include rollback capability in case of failure.

**NFR-12** Entity soft delete MUST preserve all related data (transactions, accounts, etc.).

### Security

**NFR-13** Users MUST NOT be able to access entities from organizations they don't belong to.

**NFR-14** Entity access checks MUST be enforced at the database level (RLS policies).

**NFR-15** Subscription status checks MUST be enforced at the middleware level for all protected routes.

### Maintainability

**NFR-16** All database schema changes MUST be implemented as versioned migrations.

**NFR-17** All API endpoints MUST be documented in the OpenAPI specification.

**NFR-18** All entity-related code MUST include comprehensive unit and property-based tests.

**NFR-19** Migration scripts MUST include detailed logging for troubleshooting.

### Compatibility

**NFR-20** The migration MUST preserve all existing data (no data loss).

**NFR-21** The migration MUST maintain backward compatibility with existing API clients during a transition period.

**NFR-22** Frontend changes MUST be backward compatible with the current backend until migration is complete.

## Out of Scope

The following items are explicitly out of scope for this implementation:

**OS-1** Payment integration updates (Stripe webhooks will be updated in a separate task)

**OS-2** Email notification updates for subscription changes

**OS-3** Admin dashboard for managing user subscriptions

**OS-4** Bulk entity import/export functionality

**OS-5** Entity-level permissions (all organization members have access to all entities)

**OS-6** Entity archiving/unarchiving UI (soft delete only via API)

**OS-7** Historical subscription tier tracking

**OS-8** Multi-currency consolidation (entities can have different base currencies, but consolidated reports use organization's base currency)

**OS-9** Advanced intercompany elimination rules (only basic mirror and elimination supported)

**OS-10** Entity templates or cloning functionality

## Assumptions

**A-1** All existing organizations have at least one user (the owner).

**A-2** All existing organizations have valid subscription data that can be migrated to the owner's user account.

**A-3** Users understand that subscription is per user, not per organization.

**A-4** Organizations will not need more than 100 entities (enterprise tier limit).

**A-5** The migration will be performed during a maintenance window with no active users.

**A-6** All existing data has valid organization_id foreign keys.

**A-7** The frontend will be deployed after the backend migration is complete.

**A-8** Test credentials (corp@zeltra.io / qwertyui) will continue to work after migration.

**A-9** The backend binary is named `zeltra` (not zeltra-api).

**A-10** The frontend uses `pnpm` as the package manager (not npm).

## Dependencies

**D-1** SeaORM for database migrations and entity models

**D-2** Axum for API routing and middleware

**D-3** utoipa for OpenAPI specification generation

**D-4** quickcheck for property-based testing (Rust)

**D-5** fast-check for property-based testing (TypeScript)

**D-6** TanStack Query (React Query) for data fetching

**D-7** Zustand for frontend state management

**D-8** PostgreSQL database with RLS support

**D-9** Existing authentication and authorization system

**D-10** Existing tier_limits table with subscription tier definitions

## Success Metrics

**SM-1** All 20 correctness properties pass with 100 iterations each

**SM-2** All unit tests pass (target: 100% of critical paths covered)

**SM-3** All integration tests pass (target: all critical user flows covered)

**SM-4** Migration completes successfully with zero data loss

**SM-5** All API endpoints documented in OpenAPI specification

**SM-6** Zero production incidents related to subscription or entity management in first 30 days

**SM-7** Performance targets met for all NFRs

**SM-8** Code review approval from at least 2 team members

**SM-9** QA sign-off on all acceptance criteria

**SM-10** Successful deployment to production with zero rollbacks