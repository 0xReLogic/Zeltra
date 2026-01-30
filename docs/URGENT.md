# URGENT: Subscription Model Refactoring - REVISED APPROACH

**Status**: ✅ ANALYSIS COMPLETE - Ready for Implementation  
**Priority**: P0 - Must Fix Before Production  
**Estimated Effort**: 10-13 days (Complete Implementation with All Features)  
**Date Identified**: 2026-01-24  
**Date Revised**: 2026-01-24
**Date Analysis Completed**: 2026-01-24

---


## 🚨 Problem Statement

### Current Implementation (WRONG)
Subscription fields (`subscription_tier`, `subscription_status`, `trial_ends_at`, etc.) are stored **per ORGANIZATION** in the `organizations` table.

### Business Model (CORRECT)
According to `docs/BUSINESS_MODEL.md`:
- **Starter**: $12/mo per USER → 1 organization
- **Growth**: $25/mo per USER → 5 organizations  
- **Enterprise**: Custom per USER → **UNLIMITED organizations**

**Subscription is per USER, not per organization.**

### Why This is Critical
1. **Inconsistent with pricing model**: User pays once, gets access to multiple orgs based on tier
2. **Trial inheritance broken**: When user creates 2nd org, trial period is copied from 1st org (workaround)
3. **Upgrade complexity**: Must update ALL orgs when user upgrades (inefficient)
4. **Delete org bug**: If user deletes their 1st org, trial period is lost
5. **Multi-tier confusion**: User could theoretically have orgs with different tiers (nonsensical)

---

## � REVISED SOLUTION: Remove Multi-Org + Add Entities Feature

### Analysis Summary

After comprehensive analysis including:
- Sequential thinking on architectural implications
- Competitor research (Xero, QuickBooks, NetSuite, Sage Intacct)
- Intercompany accounting best practices
- Industry pricing models

**Key Finding**: The "multi-organization" feature is fundamentally flawed. Industry leaders (Xero, QuickBooks) **charge per organization**, not unlimited. Our "unlimited organizations" promise is:
- More generous than competitors (economically unsustainable)
- Creates unnecessary architectural complexity
- Doesn't align with what customers actually need

### What Customers Actually Need

Customers don't need "multiple organizations" (separate workspaces, separate logins).  
They need **"multi-entity accounting"** (multiple companies/entities within ONE workspace).

This is what NetSuite and Sage Intacct do - and it's superior to multi-org model.

---

## 💡 New Approach: Entities Model

### Concept

Instead of:
- ❌ User has multiple organizations (separate workspaces)
- ❌ Each org has separate subscription
- ❌ User switches between orgs

We do:
- ✅ User has ONE organization (workspace)
- ✅ User can create multiple **entities** (companies) within that workspace
- ✅ Subscription is per user, limits apply to entities
- ✅ Unified workspace, no switching needed

### Revised Business Model

| Tier | Price | Workspace | Entities/Companies | Users | Transactions/Month | Value Proposition |
|------|-------|-----------|-------------------|-------|-------------------|-------------------|
| **Starter** | $12/mo | 1 | 1 | 50 | 1,000 | Single company accounting |
| **Growth** | $25/mo | 1 | **5** | 200 | 10,000 | Multi-company management |
| **Enterprise** | Custom | 1 | **Unlimited** | Unlimited | Unlimited | Full multi-entity + Intercompany Hub |

### Key Benefits

1. **Simpler Architecture**: No multi-org complexity, subscription naturally on user
2. **Better UX**: Unified workspace, entity selector (like QuickBooks company selector)
3. **Intercompany Hub Works Better**: Inter-entity transactions within one org are simpler
4. **Competitive Advantage**: 
   - Xero/QuickBooks: Charge per company ($45/mo per additional company)
   - Zeltra: "Manage 5 companies for $25/mo in ONE workspace"
5. **Faster Implementation**: 2-3 days vs 5 days for multi-org refactor
6. **Lower Risk**: Simpler changes, less can go wrong
7. **Easier Maintenance**: No multi-org considerations in every feature

---

## 📊 Implementation Plan

### Phase 1: Database Schema (Day 1)

#### Add Entities Table
```sql
CREATE TABLE entities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    legal_name VARCHAR(255),
    tax_id VARCHAR(100),
    entity_type VARCHAR(50), -- 'subsidiary', 'branch', 'division', 'client'
    base_currency VARCHAR(3) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    settings JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT unique_entity_name_per_org UNIQUE(organization_id, name)
);

CREATE INDEX idx_entities_organization ON entities(organization_id);
CREATE INDEX idx_entities_active ON entities(organization_id, is_active);
```

#### Move Subscription to Users Table
```sql
-- Add subscription fields to users
ALTER TABLE users ADD COLUMN subscription_tier subscription_tier NOT NULL DEFAULT 'starter';
ALTER TABLE users ADD COLUMN subscription_status subscription_status NOT NULL DEFAULT 'trialing';
ALTER TABLE users ADD COLUMN trial_ends_at TIMESTAMPTZ;
ALTER TABLE users ADD COLUMN subscription_ends_at TIMESTAMPTZ;
ALTER TABLE users ADD COLUMN payment_provider VARCHAR(255);
ALTER TABLE users ADD COLUMN payment_customer_id VARCHAR(255);
ALTER TABLE users ADD COLUMN payment_subscription_id VARCHAR(255);

-- Add foreign key to tier_limits
ALTER TABLE users ADD CONSTRAINT fk_users_tier 
  FOREIGN KEY (subscription_tier) REFERENCES tier_limits(tier);

-- Add indexes
CREATE INDEX idx_users_subscription_status ON users(subscription_status);
CREATE INDEX idx_users_payment_customer ON users(payment_provider, payment_customer_id) 
  WHERE payment_customer_id IS NOT NULL;
```

#### Add Entity References to Existing Tables
```sql
-- Add entity_id to tables that need entity isolation
ALTER TABLE chart_of_accounts ADD COLUMN entity_id UUID REFERENCES entities(id);
ALTER TABLE transactions ADD COLUMN entity_id UUID REFERENCES entities(id);
ALTER TABLE ledger_entries ADD COLUMN entity_id UUID REFERENCES entities(id);
ALTER TABLE budgets ADD COLUMN entity_id UUID REFERENCES entities(id);
ALTER TABLE fiscal_years ADD COLUMN entity_id UUID REFERENCES entities(id);
ALTER TABLE accrual_schedules ADD COLUMN entity_id UUID REFERENCES entities(id);
ALTER TABLE revaluation_logs ADD COLUMN entity_id UUID REFERENCES entities(id);

-- Add indexes
CREATE INDEX idx_accounts_entity ON chart_of_accounts(entity_id);
CREATE INDEX idx_transactions_entity ON transactions(entity_id);
CREATE INDEX idx_ledger_entries_entity ON ledger_entries(entity_id);
CREATE INDEX idx_budgets_entity ON budgets(entity_id);
CREATE INDEX idx_fiscal_years_entity ON fiscal_years(entity_id);
CREATE INDEX idx_accrual_schedules_entity ON accrual_schedules(entity_id);
CREATE INDEX idx_revaluation_logs_entity ON revaluation_logs(entity_id);
```

#### Data Migration
```sql
-- Step 1: Migrate subscription from organizations to users
-- Take subscription from user's FIRST (oldest) organization
WITH first_org_per_user AS (
  SELECT DISTINCT ON (ou.user_id)
    ou.user_id,
    o.subscription_tier,
    o.subscription_status,
    o.trial_ends_at,
    o.subscription_ends_at,
    o.payment_provider,
    o.payment_customer_id,
    o.payment_subscription_id
  FROM organization_users ou
  JOIN organizations o ON o.id = ou.organization_id
  ORDER BY ou.user_id, ou.created_at ASC
)
UPDATE users u
SET 
  subscription_tier = f.subscription_tier,
  subscription_status = f.subscription_status,
  trial_ends_at = f.trial_ends_at,
  subscription_ends_at = f.subscription_ends_at,
  payment_provider = f.payment_provider,
  payment_customer_id = f.payment_customer_id,
  payment_subscription_id = f.payment_subscription_id
FROM first_org_per_user f
WHERE u.id = f.user_id;

-- Step 2: Create default entity for each organization
INSERT INTO entities (organization_id, name, legal_name, base_currency, entity_type)
SELECT 
  id,
  name || ' (Main)',
  name,
  base_currency,
  'main'
FROM organizations;

-- Step 3: Link existing data to default entity
WITH org_default_entity AS (
  SELECT organization_id, id as entity_id
  FROM entities
  WHERE entity_type = 'main'
)
UPDATE chart_of_accounts coa
SET entity_id = ode.entity_id
FROM org_default_entity ode
WHERE coa.organization_id = ode.organization_id;

-- Repeat for transactions, ledger_entries, budgets, fiscal_years, accrual_schedules, revaluation_logs
```

**Step 4: Link Accrual Schedules and Revaluation Logs**
```sql
-- Link accrual_schedules
WITH org_default_entity AS (
  SELECT organization_id, id as entity_id
  FROM entities
  WHERE entity_type = 'main'
)
UPDATE accrual_schedules acs
SET entity_id = ode.entity_id
FROM org_default_entity ode
WHERE acs.organization_id = ode.organization_id;

-- Link revaluation_logs
WITH org_default_entity AS (
  SELECT organization_id, id as entity_id
  FROM entities
  WHERE entity_type = 'main'
)
UPDATE revaluation_logs rl
SET entity_id = ode.entity_id
FROM org_default_entity ode
WHERE rl.organization_id = ode.organization_id;
```

#### Remove Subscription from Organizations (Later, after testing)
```sql
-- Phase 2 migration (after full testing)
ALTER TABLE organizations DROP COLUMN subscription_tier;
ALTER TABLE organizations DROP COLUMN subscription_status;
ALTER TABLE organizations DROP COLUMN trial_ends_at;
ALTER TABLE organizations DROP COLUMN subscription_ends_at;
ALTER TABLE organizations DROP COLUMN payment_provider;
ALTER TABLE organizations DROP COLUMN payment_customer_id;
ALTER TABLE organizations DROP COLUMN payment_subscription_id;
```

#### Update Tier Limits to Apply to Entities
```sql
-- ✅ COMPLETED: Added max_entities column to tier_limits table
-- See migration: backend/crates/db/src/migration/m20260108_000001_initial.rs
-- Seed data: Starter=1, Growth=5, Enterprise=NULL (unlimited)
```

---

### Phase 2: Backend Entities (Day 1-2)

#### New Entity Files

**`backend/crates/db/src/entities/entities.rs`** (NEW)
```rust
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "entities")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub legal_name: Option<String>,
    pub tax_id: Option<String>,
    pub entity_type: String,
    pub base_currency: String,
    pub is_active: bool,
    pub settings: Json,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::organizations::Entity",
        from = "Column::OrganizationId",
        to = "super::organizations::Column::Id"
    )]
    Organization,
}
```

**`backend/crates/db/src/repositories/entity.rs`** (NEW)
```rust
pub struct EntityRepository {
    db: Arc<DatabaseConnection>,
}

impl EntityRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    /// Create a new entity
    pub async fn create(
        &self,
        organization_id: Uuid,
        name: String,
        base_currency: String,
        entity_type: String,
    ) -> Result<entities::Model, DbErr> {
        // Check entity limit based on user's subscription tier
        let entity_count = self.count_by_organization(organization_id).await?;
        let user_tier = self.get_org_owner_tier(organization_id).await?;
        
        // Check tier limits
        let limits = SubscriptionRepository::get_tier_limits(&*self.db, user_tier).await?;
        if let Some(max_entities) = limits.max_entities {
            if entity_count >= max_entities as i64 {
                return Err(DbErr::Custom("Entity limit reached for your tier".to_string()));
            }
        }
        
        let entity = entities::ActiveModel {
            id: Set(Uuid::new_v4()),
            organization_id: Set(organization_id),
            name: Set(name),
            base_currency: Set(base_currency),
            entity_type: Set(entity_type),
            is_active: Set(true),
            settings: Set(json!({})),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
            ..Default::default()
        };
        
        entity.insert(&*self.db).await
    }

    /// List entities for an organization
    pub async fn list_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<entities::Model>, DbErr> {
        entities::Entity::find()
            .filter(entities::Column::OrganizationId.eq(organization_id))
            .filter(entities::Column::IsActive.eq(true))
            .order_by_asc(entities::Column::CreatedAt)
            .all(&*self.db)
            .await
    }

    /// Get entity by ID
    pub async fn find_by_id(&self, entity_id: Uuid) -> Result<Option<entities::Model>, DbErr> {
        entities::Entity::find_by_id(entity_id).one(&*self.db).await
    }

    /// Count entities for an organization
    pub async fn count_by_organization(&self, organization_id: Uuid) -> Result<i64, DbErr> {
        entities::Entity::find()
            .filter(entities::Column::OrganizationId.eq(organization_id))
            .filter(entities::Column::IsActive.eq(true))
            .count(&*self.db)
            .await
    }

    /// Get organization owner's subscription tier
    async fn get_org_owner_tier(&self, organization_id: Uuid) -> Result<SubscriptionTier, DbErr> {
        let owner = organization_users::Entity::find()
            .filter(organization_users::Column::OrganizationId.eq(organization_id))
            .filter(organization_users::Column::Role.eq(UserRole::Owner))
            .find_also_related(users::Entity)
            .one(&*self.db)
            .await?
            .and_then(|(_, user)| user)
            .ok_or_else(|| DbErr::Custom("Organization owner not found".to_string()))?;
        
        Ok(owner.subscription_tier)
    }
}
```

#### Update Existing Repositories

**`backend/crates/db/src/repositories/organization.rs`**
- Remove subscription field assignments from `create_with_owner()`
- Create default entity when creating organization
- Simplify to one-org-per-user model

**`backend/crates/db/src/repositories/subscription.rs`**
- Change all methods from `organization_id` to `user_id`
- Update tier limit checks to apply to entities, not orgs

**`backend/crates/db/src/repositories/user.rs`**
- Add subscription methods (get_subscription_tier, update_subscription_tier, etc.)

---

### Phase 3: API Routes (Day 2)

#### New Entity Routes

**`backend/crates/api/src/routes/entities.rs`** (NEW)
```rust
/// GET /organizations/{org_id}/entities
/// List all entities for an organization
pub async fn list_entities(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    AuthUser(claims): AuthUser,
) -> Result<Json<Vec<EntityResponse>>, impl IntoResponse> {
    // Check user has access to org
    // Get entities
    // Return list
}

/// POST /organizations/{org_id}/entities
/// Create a new entity
pub async fn create_entity(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    AuthUser(claims): AuthUser,
    Json(req): Json<CreateEntityRequest>,
) -> Result<Json<EntityResponse>, impl IntoResponse> {
    // Check tier limits
    // Create entity
    // Return entity
}

/// GET /organizations/{org_id}/entities/{entity_id}
/// Get entity details
pub async fn get_entity(
    State(state): State<AppState>,
    Path((org_id, entity_id)): Path<(Uuid, Uuid)>,
    AuthUser(claims): AuthUser,
) -> Result<Json<EntityResponse>, impl IntoResponse> {
    // Get entity
    // Return entity
}

/// PATCH /organizations/{org_id}/entities/{entity_id}
/// Update entity
pub async fn update_entity(
    State(state): State<AppState>,
    Path((org_id, entity_id)): Path<(Uuid, Uuid)>,
    AuthUser(claims): AuthUser,
    Json(req): Json<UpdateEntityRequest>,
) -> Result<Json<EntityResponse>, impl IntoResponse> {
    // Update entity
    // Return entity
}

/// DELETE /organizations/{org_id}/entities/{entity_id}
/// Delete entity (soft delete)
pub async fn delete_entity(
    State(state): State<AppState>,
    Path((org_id, entity_id)): Path<(Uuid, Uuid)>,
    AuthUser(claims): AuthUser,
) -> Result<StatusCode, impl IntoResponse> {
    // Soft delete entity
    // Return 204
}
```

#### Update Existing Routes

**`backend/crates/api/src/routes/organizations.rs`**
- Remove subscription fields from `OrganizationResponse`
- Simplify organization creation (no multi-org logic)
- Remove organization limits (one org per user)

**`backend/crates/api/src/routes/transactions.rs`**
- Add `entity_id` to transaction creation
- Filter transactions by entity

**`backend/crates/api/src/routes/accounts.rs`**
- Add `entity_id` to account creation
- Filter accounts by entity

**`backend/crates/api/src/routes/budgets.rs`**
- Add `entity_id` to budget creation
- Filter budgets by entity

**`backend/crates/api/src/routes/fiscal.rs`**
- Add `entity_id` to fiscal year creation
- Filter fiscal years/periods by entity

**`backend/crates/api/src/routes/sentinel.rs`** (Accruals & Revaluation)
- Add `entity_id` to accrual schedule creation
- Filter accrual schedules by entity
- Add `entity_id` to revaluation operations
- Filter revaluation logs by entity

**`backend/crates/api/src/routes/reports.rs`**
- Add `entity_id` parameter to all report endpoints
- Support consolidated reports (all entities)
- Filter report data by entity

**`backend/crates/api/src/routes/dashboard.rs`**
- Add `entity_id` parameter to dashboard metrics
- Filter dashboard data by entity
- Support consolidated view (all entities)

**`backend/crates/api/src/routes/forensic.rs`**
- Add `entity_id` parameter to forensic analysis
- Filter forensic data by entity

**`backend/crates/api/src/routes/simulation.rs`**
- Add `entity_id` parameter to simulation runs
- Filter simulation data by entity

---

### Phase 4: Intercompany Hub Updates (Day 2)

#### Current Intercompany Implementation

**Database Model** (`intercompany_mappings.rs`):
```rust
pub struct Model {
    pub id: Uuid,
    pub source_org_id: Uuid,           // ← Will change to source_entity_id
    pub target_org_id: Uuid,           // ← Will change to target_entity_id
    pub source_account_id: Uuid,
    pub target_account_id: Uuid,
    pub mapping_type: String,          // 'elimination' or 'mirror'
    pub auto_post: bool,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}
```

**Current Workflow**:
1. User creates mapping between two organizations
2. When transaction posted in source org's intercompany account
3. System automatically mirrors or eliminates in target org
4. Validation ensures both orgs exist and user has access

**Current Repository Methods** (`intercompany.rs`):
- `get_mappings(source_org_id)` - Get all mappings for org
- `find_mapping_by_account(source_org_id, source_account_id)` - Find specific mapping
- `get_pending_intercompany_entries(organization_id)` - Find entries needing processing
- `process_intercompany_entries(organization_id, tx_repo)` - Process pending entries

**Current Core Engine** (`ledger/intercompany.rs`):
- `IntercompanyEngine::is_match()` - Check if two entries match for elimination
- `IntercompanyEngine::generate_elimination_transaction()` - Create elimination entry
- `IntercompanyEngine::generate_mirror_transaction()` - Create mirror entry

**Current API Endpoints** (`sentinel.rs`):
- `GET /organizations/{org_id}/intercompany/mappings` - List mappings
- `POST /organizations/{org_id}/intercompany/connect` - Create mapping

---

#### Database Schema Changes

**Current Schema**:
```sql
CREATE TABLE intercompany_mappings (
    id UUID PRIMARY KEY,
    source_org_id UUID NOT NULL REFERENCES organizations(id),
    target_org_id UUID NOT NULL REFERENCES organizations(id),
    source_account_id UUID NOT NULL REFERENCES chart_of_accounts(id),
    target_account_id UUID NOT NULL REFERENCES chart_of_accounts(id),
    mapping_type VARCHAR(20),  -- 'elimination' or 'mirror'
    auto_post BOOLEAN,
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ
);
```

**New Schema** (Entity-Based):
```sql
-- Rename columns
ALTER TABLE intercompany_mappings 
  RENAME COLUMN source_org_id TO source_entity_id;
  
ALTER TABLE intercompany_mappings 
  RENAME COLUMN target_org_id TO target_entity_id;

-- Update foreign keys
ALTER TABLE intercompany_mappings 
  DROP CONSTRAINT IF EXISTS fk_source_org,
  ADD CONSTRAINT fk_source_entity 
    FOREIGN KEY (source_entity_id) REFERENCES entities(id);

ALTER TABLE intercompany_mappings 
  DROP CONSTRAINT IF EXISTS fk_target_org,
  ADD CONSTRAINT fk_target_entity 
    FOREIGN KEY (target_entity_id) REFERENCES entities(id);

-- Add validation: both entities must be in same org
ALTER TABLE intercompany_mappings 
  ADD CONSTRAINT check_same_org CHECK (
    (SELECT organization_id FROM entities WHERE id = source_entity_id) = 
    (SELECT organization_id FROM entities WHERE id = target_entity_id)
  );
```

---

#### Backend Code Changes

**`backend/crates/db/src/entities/intercompany_mappings.rs`**:
```rust
// Change Model fields
pub struct Model {
    pub id: Uuid,
    pub source_entity_id: Uuid,        // ← Changed from source_org_id
    pub target_entity_id: Uuid,        // ← Changed from target_org_id
    pub source_account_id: Uuid,
    pub target_account_id: Uuid,
    pub mapping_type: String,
    pub auto_post: bool,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

// Update Relations
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::entities::Entity",
        from = "Column::SourceEntityId",
        to = "super::entities::Column::Id"
    )]
    SourceEntity,
    
    #[sea_orm(
        belongs_to = "super::entities::Entity",
        from = "Column::TargetEntityId",
        to = "super::entities::Column::Id"
    )]
    TargetEntity,
}
```

**`backend/crates/db/src/repositories/intercompany.rs`**:
```rust
// Update method signatures
pub async fn get_mappings(
    &self,
    source_entity_id: Uuid,  // ← Changed from source_org_id
) -> Result<Vec<intercompany_mappings::Model>, DbErr> {
    intercompany_mappings::Entity::find()
        .filter(intercompany_mappings::Column::SourceEntityId.eq(source_entity_id))
        .all(&*self.db)
        .await
}

pub async fn find_mapping_by_account(
    &self,
    source_entity_id: Uuid,  // ← Changed from source_org_id
    source_account_id: Uuid,
) -> Result<Option<intercompany_mappings::Model>, DbErr> {
    intercompany_mappings::Entity::find()
        .filter(intercompany_mappings::Column::SourceEntityId.eq(source_entity_id))
        .filter(intercompany_mappings::Column::SourceAccountId.eq(source_account_id))
        .one(&*self.db)
        .await
}

// Simplified validation (no cross-org checks)
pub async fn validate_mapping(
    &self,
    source_entity_id: Uuid,
    target_entity_id: Uuid,
) -> Result<(), DbErr> {
    let source = entities::Entity::find_by_id(source_entity_id).one(&*self.db).await?;
    let target = entities::Entity::find_by_id(target_entity_id).one(&*self.db).await?;
    
    // Both entities must exist
    let source = source.ok_or_else(|| DbErr::Custom("Source entity not found".to_string()))?;
    let target = target.ok_or_else(|| DbErr::Custom("Target entity not found".to_string()))?;
    
    // Both entities must be in same organization (enforced by DB constraint)
    if source.organization_id != target.organization_id {
        return Err(DbErr::Custom("Entities must belong to same organization".to_string()));
    }
    
    Ok(())
}
```

**`backend/crates/core/src/ledger/intercompany.rs`**:
```rust
// Update IntercompanyEngine methods
impl IntercompanyEngine {
    // Validation becomes simpler (both entities in same org)
    pub fn validate_entities(
        source_entity: &entities::Model,
        target_entity: &entities::Model,
    ) -> Result<(), IntercompanyError> {
        if source_entity.organization_id != target_entity.organization_id {
            return Err(IntercompanyError::DifferentOrganizations);
        }
        Ok(())
    }
    
    // Mirror transactions stay in same org
    pub async fn generate_mirror_transaction(
        &self,
        source_entry: &ledger_entries::Model,
        mapping: &intercompany_mappings::Model,
    ) -> Result<transactions::Model, IntercompanyError> {
        // Create mirror transaction in target entity (same org)
        // No cross-org complexity
    }
    
    // Elimination entries stay in same org
    pub async fn generate_elimination_transaction(
        &self,
        source_entry: &ledger_entries::Model,
        target_entry: &ledger_entries::Model,
        mapping: &intercompany_mappings::Model,
    ) -> Result<transactions::Model, IntercompanyError> {
        // Create elimination entries (same org)
        // Consolidation becomes easier
    }
}
```

**`backend/crates/api/src/routes/sentinel.rs`**:
```rust
// Update API request/response types
#[derive(Debug, Deserialize)]
pub struct CreateIntercompanyMappingRequest {
    pub source_entity_id: Uuid,        // ← Changed from source_org_id
    pub target_entity_id: Uuid,        // ← Changed from target_org_id
    pub source_account_id: Uuid,
    pub target_account_id: Uuid,
    pub mapping_type: String,
    pub auto_post: bool,
}

// Update endpoint paths (optional - can keep /organizations/ for backward compat)
// GET /organizations/{org_id}/intercompany/mappings
pub async fn list_intercompany_mappings(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    AuthUser(claims): AuthUser,
) -> Result<Json<Vec<IntercompanyMappingResponse>>, impl IntoResponse> {
    // Get all entities for this org
    // Get mappings for all entities
    // Return list
}

// POST /organizations/{org_id}/intercompany/connect
pub async fn create_intercompany_mapping(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    AuthUser(claims): AuthUser,
    Json(req): Json<CreateIntercompanyMappingRequest>,
) -> Result<Json<IntercompanyMappingResponse>, impl IntoResponse> {
    // Validate both entities belong to org_id
    // Create mapping
    // Return mapping
}
```

---

#### Benefits of Entity-Based Intercompany

1. **Simpler Validation**: No cross-org access checks needed
2. **Faster Queries**: Same database, no joins across orgs
3. **Better UX**: No org switching needed
4. **Easier Consolidation**: All entities in one workspace
5. **Cleaner Code**: Less complexity in validation logic
6. **Better Performance**: No cross-org transaction overhead

---

#### Files Requiring Changes

**Backend Files**:
- `backend/crates/db/src/entities/intercompany_mappings.rs` - Update model fields
- `backend/crates/db/src/repositories/intercompany.rs` - Update method signatures
- `backend/crates/core/src/ledger/intercompany.rs` - Simplify validation
- `backend/crates/api/src/routes/sentinel.rs` - Update API endpoints
- `backend/crates/db/src/migration/m20260125_000002_update_intercompany.rs` (NEW) - Migration

**Frontend Files** (covered in Phase 6):
- `frontend/src/app/dashboard/intercompany/page.tsx` - Update UI
- `frontend/src/lib/queries/sentinel.ts` - Update queries
- `frontend/src/types/api-helpers.ts` - Update types

---

### Phase 5: Middleware Updates (Day 2)

**`backend/crates/api/src/middleware/subscription.rs`**
```rust
// Simplified: Check user subscription directly
pub async fn check_subscription_status(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    // Extract user_id from Claims
    let claims = request.extensions().get::<Claims>().copied();
    let Some(claims) = claims else {
        return next.run(request).await;
    };

    // Check user's subscription status
    let user = users::Entity::find_by_id(claims.sub)
        .one(&*state.db)
        .await?
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "User not found"))?;

    match user.subscription_status {
        SubscriptionStatus::Active | SubscriptionStatus::Trialing => next.run(request).await,
        SubscriptionStatus::Expired => {
            (StatusCode::PAYMENT_REQUIRED, Json(json!({
                "error": "subscription_expired",
                "message": "Your trial has expired. Please upgrade to continue."
            }))).into_response()
        }
        // ... other statuses
    }
}
```

---

### Phase 6: Frontend Updates (Day 3-4)

#### 1. Organization Switcher → Entity Selector

**Current Architecture**:
- Auth Store (`authStore.ts`): Stores `currentOrgId` and user's organizations array
- Organization Context: Stored in Zustand store
- Switcher Location: Sidebar component uses `useOrganization()` hook
- Organization Selection: Implicit - first org in array is default

**Current Flow**:
```
User Login → setAuth() → currentOrgId = first org → useOrganization() → fetch org data
```

**Changes Needed**:
- Keep `currentOrgId` (one org per user now)
- Add `currentEntityId` to auth store
- Create entity selector component (similar to org switcher)
- Update sidebar to show entity selector instead of org switcher
- Pass `entity_id` to all queries instead of relying on org context

**Files to Modify**:
- `frontend/src/lib/stores/authStore.ts` - Add `currentEntityId` state
- `frontend/src/components/layout/Sidebar.tsx` - Replace org switcher with entity selector
- `frontend/src/components/entities/EntitySelector.tsx` (NEW) - Entity selector component

---

#### 2. Type System Updates

**Current Organization Type** (`organizations.ts`):
```typescript
export type Organization = {
  id: string
  name: string
  slug: string
  base_currency: string
  timezone: string
  subscription_tier: string        // ← REMOVE
  subscription_status: string      // ← REMOVE
  trial_ends_at: DateTime          // ← REMOVE
  subscription_ends_at: DateTime   // ← REMOVE
  payment_provider: string         // ← REMOVE
  payment_customer_id: string      // ← REMOVE
  payment_subscription_id: string  // ← REMOVE
}
```

**New Types Needed**:

```typescript
// entities.ts (NEW)
export type Entity = {
  id: string
  organization_id: string
  name: string
  legal_name?: string
  tax_id?: string
  entity_type: string  // 'main', 'subsidiary', 'branch', 'division'
  base_currency: string
  is_active: boolean
  settings: Record<string, any>
  created_at: string
  updated_at: string
}

// auth.ts (ADD)
export type UserSubscription = {
  subscription_tier: string
  subscription_status: string
  trial_ends_at?: string
  subscription_ends_at?: string
  payment_provider?: string
  payment_customer_id?: string
  payment_subscription_id?: string
}
```

**Files to Create/Modify**:
- `frontend/src/types/organizations.ts` - Remove subscription fields
- `frontend/src/types/entities.ts` (NEW) - Entity types
- `frontend/src/types/auth.ts` - Add UserSubscription type

---

#### 3. Query System Updates

**Current Organization Queries** (`organizations.ts`):
```typescript
useOrganizations()        // List all orgs → REMOVE (one org per user)
useOrganization()         // Get current org → KEEP
useCreateOrganization()   // Create org → REMOVE
useUpdateOrganization()   // Update org → KEEP
useOrganizationUsers()    // Get org users → KEEP
```

**New Entity Queries** (NEW FILE: `entities.ts`):
```typescript
useEntities()             // List entities in current org
useEntity(entityId)       // Get single entity
useCreateEntity()         // Create entity
useUpdateEntity()         // Update entity
useDeleteEntity()         // Delete entity (soft delete)
```

**Queries Requiring entity_id Parameter**:
- `useTransactions()` - Add entity_id filter
- `useCreateTransaction()` - Add entity_id to payload
- `useAccounts()` - Add entity_id filter
- `useCreateAccount()` - Add entity_id to payload
- `useBudgets()` - Add entity_id filter
- `useCreateBudget()` - Add entity_id to payload

**New User Subscription Query** (`auth.ts`):
```typescript
useUserSubscription()     // Get current user's subscription
```

**Files to Create/Modify**:
- `frontend/src/lib/queries/organizations.ts` - Simplify org queries
- `frontend/src/lib/queries/entities.ts` (NEW) - Entity queries
- `frontend/src/lib/queries/transactions.ts` - Add entity_id parameter
- `frontend/src/lib/queries/accounts.ts` - Add entity_id parameter
- `frontend/src/lib/queries/budgets.ts` - Add entity_id parameter
- `frontend/src/lib/queries/auth.ts` - Add user subscription query

---

#### 4. Forms Requiring Entity Selector

**Transaction Form** (`CreateTransactionDialog.tsx`):
- **Current**: Uses `useOrganization()` to get org data
- **Changes**: 
  - Add entity selector dropdown
  - Pass `entity_id` to transaction creation
  - Update query to include `entity_id` filter
  - Line 159: `const isStarterTier = organization?.subscription_tier?.toLowerCase() === 'starter'` → Move to user subscription check

**Account Form** (`AccountForm.tsx`):
- **Current**: No org reference (uses implicit org context)
- **Changes**:
  - Add entity selector
  - Pass `entity_id` to account creation
  - Update query to filter by entity

**Budget Forms** (in `frontend/src/app/dashboard/budgets/`):
- **Changes**:
  - Add entity selector
  - Pass `entity_id` to budget creation

**Files to Modify**:
- `frontend/src/components/transactions/CreateTransactionDialog.tsx`
- `frontend/src/components/accounts/AccountForm.tsx`
- Budget form components (need to locate)

---

#### 5. Lists Requiring Entity Filter

**Transaction List** (`frontend/src/app/dashboard/transactions/`):
- Add entity filter dropdown
- Pass `entity_id` to transaction query
- Show entity name in transaction rows

**Account List** (`frontend/src/app/dashboard/accounts/`):
- Add entity filter dropdown
- Pass `entity_id` to account query
- Show entity name in account rows

**Budget List** (`frontend/src/app/dashboard/budgets/`):
- Add entity filter dropdown
- Pass `entity_id` to budget query
- Show entity name in budget rows

**Accruals List** (`frontend/src/app/dashboard/accruals/`):
- Add entity filter dropdown
- Pass `entity_id` to accrual query
- Show entity name in accrual rows

**Revaluation List** (`frontend/src/app/dashboard/revaluation/`):
- Add entity filter dropdown
- Pass `entity_id` to revaluation query
- Show entity name in revaluation rows

**Fiscal Periods List** (`frontend/src/app/dashboard/master-data/fiscal-periods/`):
- Add entity filter dropdown
- Pass `entity_id` to fiscal periods query
- Show entity name in fiscal period rows

**Files to Modify**:
- `frontend/src/app/dashboard/transactions/page.tsx`
- `frontend/src/app/dashboard/accounts/page.tsx`
- `frontend/src/app/dashboard/budgets/page.tsx`
- `frontend/src/app/dashboard/accruals/page.tsx` (NEW or existing)
- `frontend/src/app/dashboard/revaluation/page.tsx` (NEW or existing)
- `frontend/src/app/dashboard/master-data/fiscal-periods/page.tsx`

---

#### 6. Reports Requiring Entity Filter

**All Report Pages** need entity selector with two modes:
1. **Single Entity Mode**: Show report for selected entity
2. **Consolidated Mode**: Show combined report for all entities

**Report Pages**:

**Trial Balance** (`frontend/src/app/dashboard/reports/trial-balance/`):
- Add entity selector (single or consolidated)
- Pass `entity_id` or `consolidated=true` to API
- Show entity name in report header

**Balance Sheet** (`frontend/src/app/dashboard/reports/balance-sheet/`):
- Add entity selector (single or consolidated)
- Pass `entity_id` or `consolidated=true` to API
- Show entity name in report header

**Income Statement** (`frontend/src/app/dashboard/reports/income-statement/`):
- Add entity selector (single or consolidated)
- Pass `entity_id` or `consolidated=true` to API
- Show entity name in report header

**Dimensional Reports** (`frontend/src/app/dashboard/reports/dimensional/`):
- Add entity selector (single or consolidated)
- Pass `entity_id` or `consolidated=true` to API
- Show entity name in report header

**Account Ledger** (`frontend/src/app/dashboard/reports/account-ledger/`):
- Add entity selector (single entity only)
- Pass `entity_id` to API
- Show entity name in report header

**Files to Modify**:
- `frontend/src/app/dashboard/reports/trial-balance/page.tsx`
- `frontend/src/app/dashboard/reports/balance-sheet/page.tsx`
- `frontend/src/app/dashboard/reports/income-statement/page.tsx`
- `frontend/src/app/dashboard/reports/dimensional/page.tsx`
- `frontend/src/app/dashboard/reports/account-ledger/page.tsx`

---

#### 7. Dashboard Requiring Entity Filter

**Main Dashboard** (`frontend/src/app/dashboard/page.tsx`):
- Add entity selector (single or consolidated)
- Pass `entity_id` or `consolidated=true` to dashboard metrics API
- Show entity name in dashboard header
- Update all dashboard widgets to respect entity filter:
  - Cash flow chart
  - Recent activity
  - Budget vs actual
  - Quick stats

**Files to Modify**:
- `frontend/src/app/dashboard/page.tsx`
- `frontend/src/components/dashboard/BudgetVsActual.tsx`
- `frontend/src/components/dashboard/RecentActivity.tsx`
- `frontend/src/components/dashboard/UsageMeter.tsx` (already covered)

---

#### 8. Sentinel Intelligence Pages

**Forensic Analysis** (`frontend/src/app/dashboard/forensic/`):
- Add entity selector (single or consolidated)
- Pass `entity_id` or `consolidated=true` to forensic API
- Show entity name in analysis header

**Simulation** (`frontend/src/app/dashboard/simulation/`):
- Add entity selector (single entity only)
- Pass `entity_id` to simulation API
- Show entity name in simulation header

**Files to Modify**:
- `frontend/src/app/dashboard/forensic/page.tsx`
- `frontend/src/app/dashboard/simulation/page.tsx`
- `frontend/src/components/simulation/SimulationControls.tsx`
- `frontend/src/components/simulation/SimulationChart.tsx`

---

#### 9. Subscription Display Updates

**Current Locations Showing Subscription**:

1. **Settings Page** (`frontend/src/app/dashboard/settings/page.tsx`, lines 129-147)
   - Shows `org?.subscription_tier` and `org?.subscription_status`
   - Shows trial countdown
   - **Change**: Move to user settings page, fetch from user object

2. **Sidebar** (`Sidebar.tsx`, line 61)
   - Uses `org?.subscription_tier` to lock enterprise features
   - **Change**: Fetch from user subscription instead

3. **Upgrade Modal** (`UpgradeModal.tsx`)
   - References org subscription
   - **Change**: Update to use user subscription

4. **Usage Meter** (`UsageMeter.tsx`)
   - Shows subscription tier
   - **Change**: Update to use user subscription

**Implementation**:
```typescript
// Create user subscription query
const { data: userSubscription } = useUserSubscription()

// Replace all instances of:
organization?.subscription_tier
// With:
userSubscription?.subscription_tier
```

**Files to Modify**:
- `frontend/src/app/dashboard/settings/page.tsx` - Remove org subscription display
- `frontend/src/app/dashboard/settings/subscription/page.tsx` (NEW) - Add user subscription page
- `frontend/src/components/layout/Sidebar.tsx` - Use user subscription for tier checks
- `frontend/src/components/modals/UpgradeModal.tsx` - Use user subscription
- `frontend/src/components/dashboard/UsageMeter.tsx` - Use user subscription

---

#### 10. Intercompany UI Updates

**Current Implementation** (`frontend/src/app/dashboard/intercompany/page.tsx`):

**Current Features**:
- List intercompany mappings in table
- Create new mapping dialog
- Select source account from current org
- Select target **organization** from user's orgs
- Select target account (from any org)
- Show mapping type (elimination/mirror)
- Show auto-post status

**Current Flow**:
```
1. User selects source account (from current org)
2. User selects target organization (from their orgs)  ← REMOVE
3. User selects target account (from target org)
4. System creates mapping between orgs
```

**New Flow**:
```
1. User selects source entity (from current org)
2. User selects source account (from source entity)
3. User selects target entity (from current org)  ← CHANGE
4. User selects target account (from target entity)
5. System creates mapping between entities (same org)
```

**Specific Changes**:
- Line 47: `useIntercompanyMappings()` - Still works, returns entity-based mappings
- Line 48: `useCreateIntercompanyMapping()` - Update to accept `entity_ids` instead of `org_ids`
- Line 49: `userOrganizations` - No longer needed for org selection
- Line 50-60: Form fields - Replace org selector with entity selector
- Line 150-160: Table display - Show entity names instead of org names
- Line 170-180: Helper functions - Update to get entity names instead of org names

**Benefits of Entity-Based Intercompany**:
- Simpler validation (no cross-org checks needed)
- Faster queries (same database, no joins across orgs)
- Better UX (no org switching needed)
- Easier consolidation (all entities in one workspace)

**Files to Modify**:
- `frontend/src/app/dashboard/intercompany/page.tsx` - Update UI for entities
- `frontend/src/lib/queries/sentinel.ts` - Update intercompany queries
- `frontend/src/types/api-helpers.ts` - Update intercompany types

---

---

## 📋 OpenAPI Generation Workflow & Impact

### Overview

Our backend uses **utoipa** to automatically generate OpenAPI specifications from Rust code. This ensures type safety between backend and frontend, but requires understanding the workflow and a critical bug fix.

### Complete Workflow

```
1. Rust Code (utoipa annotations)
   ↓
2. Generate OpenAPI (cargo run --bin generate-openapi)
   ↓
3. Fix Nullable Bug & Split (python3 split-openapi.py)
   ↓
4. Generate Frontend Types (pnpm run generate:types)
```

### The utoipa Nullable Bug

**Problem**: utoipa generates OpenAPI 3.1 syntax for nullable fields:
```yaml
# utoipa generates (OpenAPI 3.1):
type: [string, 'null']
```

**Issue**: Many tools only support OpenAPI 3.0:
```yaml
# OpenAPI 3.0 compatible:
type: string
nullable: true
```

**Solution**: `contracts/split-openapi.py` automatically fixes this:
- Converts `type: [string, 'null']` → `type: string, nullable: true`
- Converts `oneOf: [{type: 'null'}, {$ref}]` → `allOf: [{$ref}], nullable: true`

### Impact on Entities Implementation

When adding `entity_id` to routes, you must update:

**1. utoipa Annotations**:
```rust
// Before
#[utoipa::path(
    post,
    path = "/organizations/{org_id}/transactions",
    params(("org_id" = Uuid, Path, description = "Organization ID")),
    request_body = CreateTransactionRequest,
    // ...
)]

// After
#[utoipa::path(
    post,
    path = "/organizations/{org_id}/transactions",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("entity_id" = Option<Uuid>, Query, description = "Filter by entity")
    ),
    request_body = CreateTransactionRequest,
    // ...
)]
```

**2. Request/Response Schemas**:
```rust
// Before
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateTransactionRequest {
    pub description: String,
    pub entries: Vec<CreateEntryRequest>,
}

// After
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateTransactionRequest {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub entity_id: Uuid,
    pub description: String,
    pub entries: Vec<CreateEntryRequest>,
}
```

**3. Register in ApiDoc** (`backend/crates/api/src/routes/mod.rs`):
```rust
#[derive(OpenApi)]
#[openapi(
    paths(
        transactions::create_transaction,  // Ensure listed
        // ...
    ),
    components(
        schemas(
            transactions::CreateTransactionRequest,  // Ensure listed
            // ...
        )
    ),
)]
pub struct ApiDoc;
```

### Regeneration Commands

After any backend changes:

```bash
# 1. Generate OpenAPI from Rust
cd backend
cargo run --bin generate-openapi

# 2. Fix nullable syntax and split
cd ../contracts
python3 split-openapi.py

# 3. Generate frontend TypeScript types
cd ../frontend
pnpm run generate:types
```

### Checklist: Adding entity_id to Routes

For EACH route that needs entity_id:

**Backend**:
- [ ] Add `entity_id` to request/response structs
- [ ] Update `#[utoipa::path]` annotations
- [ ] Register schemas in `ApiDoc`
- [ ] Regenerate OpenAPI spec

**Frontend**:
- [ ] Regenerate TypeScript types
- [ ] Update API calls with `entity_id`
- [ ] Add entity selector UI
- [ ] Test type safety

### Scope for Entities Implementation

| Module | Endpoints | Priority |
|--------|-----------|----------|
| transactions | 10 | 🔴 Critical |
| accounts | 5 | 🔴 Critical |
| budgets | 5 | 🟡 High |
| fiscal | 3 | 🟡 High |
| reports | 4 | 🟡 High |
| dashboard | 4 | 🟡 High |
| sentinel | 5 | 🟢 Medium |
| forensic | 3 | 🟢 Medium |
| simulation | 1 | 🟢 Medium |

**Total**: ~40 endpoints need OpenAPI updates

### Common Pitfalls

1. **Forgetting to register in ApiDoc**: Route won't appear in OpenAPI spec
2. **Missing schema registration**: OpenAPI generation fails
3. **Inconsistent path parameters**: Frontend generates incorrect URLs
4. **Forgetting to regenerate types**: Frontend uses old types, runtime errors

### Key Files

- `backend/crates/api/src/routes/mod.rs` - Central OpenAPI config
- `backend/bins/generate-openapi/src/main.rs` - Generation binary
- `contracts/openapi.yaml` - Generated OpenAPI spec
- `contracts/split-openapi.py` - Nullable fix + split script
- `frontend/src/types/api.generated.ts` - Generated TypeScript types

---

## 📁 Files Requiring Changes

### Backend Files (30 files)

#### New Files (5 files)
- `backend/crates/db/src/entities/entities.rs` - Entity model
- `backend/crates/db/src/repositories/entity.rs` - Entity repository
- `backend/crates/api/src/routes/entities.rs` - Entity API routes
- `backend/crates/db/src/migration/m20260125_000001_add_entities.rs` - Entities migration
- `backend/crates/db/src/migration/m20260125_000002_update_intercompany.rs` - Intercompany migration

#### Modified Files (25 files)

**Database Entities (9 files)**:
- `backend/crates/db/src/entities/users.rs` - Add subscription fields
- `backend/crates/db/src/entities/organizations.rs` - Remove subscription fields
- `backend/crates/db/src/entities/chart_of_accounts.rs` - Add entity_id
- `backend/crates/db/src/entities/transactions.rs` - Add entity_id
- `backend/crates/db/src/entities/ledger_entries.rs` - Add entity_id
- `backend/crates/db/src/entities/budgets.rs` - Add entity_id
- `backend/crates/db/src/entities/fiscal_years.rs` - Add entity_id
- `backend/crates/db/src/entities/accrual_schedules.rs` - Add entity_id
- `backend/crates/db/src/entities/revaluation_logs.rs` - Add entity_id
- `backend/crates/db/src/entities/intercompany_mappings.rs` - Change org_id to entity_id

**Repositories (4 files)**:
- `backend/crates/db/src/repositories/user.rs` - Add subscription methods
- `backend/crates/db/src/repositories/organization.rs` - Simplify, remove multi-org
- `backend/crates/db/src/repositories/subscription.rs` - Change to user_id
- `backend/crates/db/src/repositories/intercompany.rs` - Use entity_id, simplify validation

**Core Logic (1 file)**:
- `backend/crates/core/src/ledger/intercompany.rs` - Simplify entity validation

**API Routes (9 files)**:
- `backend/crates/api/src/middleware/subscription.rs` - Check user subscription
- `backend/crates/api/src/routes/organizations.rs` - Remove subscription fields
- `backend/crates/api/src/routes/transactions.rs` - Add entity_id
- `backend/crates/api/src/routes/accounts.rs` - Add entity_id
- `backend/crates/api/src/routes/budgets.rs` - Add entity_id
- `backend/crates/api/src/routes/fiscal.rs` - Add entity_id
- `backend/crates/api/src/routes/sentinel.rs` - Update intercompany, accruals, revaluation endpoints
- `backend/crates/api/src/routes/reports.rs` - Add entity_id parameter to all reports
- `backend/crates/api/src/routes/dashboard.rs` - Add entity_id parameter
- `backend/crates/api/src/routes/forensic.rs` - Add entity_id parameter
- `backend/crates/api/src/routes/simulation.rs` - Add entity_id parameter

**Background Jobs (2 files)**:
- `backend/crates/api/src/jobs/trial_expiry.rs` - Check user trials
- `backend/bins/server/src/sync.rs` - Check user tier

---

### Frontend Files (35+ files)

#### New Files (5 files)
- `frontend/src/types/entities.ts` - Entity types
- `frontend/src/lib/queries/entities.ts` - Entity queries
- `frontend/src/components/entities/EntitySelector.tsx` - Entity selector component
- `frontend/src/app/dashboard/settings/subscription/page.tsx` - User subscription page

#### Modified Files (30+ files)

**Types (3 files)**:
- `frontend/src/types/organizations.ts` - Remove subscription fields
- `frontend/src/types/auth.ts` - Add UserSubscription type
- `frontend/src/types/api-helpers.ts` - Update intercompany types

**Queries (10 files)**:
- `frontend/src/lib/queries/organizations.ts` - Simplify org queries, remove create
- `frontend/src/lib/queries/transactions.ts` - Add entity_id parameter
- `frontend/src/lib/queries/accounts.ts` - Add entity_id parameter
- `frontend/src/lib/queries/budgets.ts` - Add entity_id parameter
- `frontend/src/lib/queries/fiscal.ts` - Add entity_id parameter
- `frontend/src/lib/queries/sentinel.ts` - Update intercompany, accruals, revaluation queries
- `frontend/src/lib/queries/reports.ts` - Add entity_id parameter to all reports
- `frontend/src/lib/queries/dashboard.ts` - Add entity_id parameter
- `frontend/src/lib/queries/forensic.ts` - Add entity_id parameter
- `frontend/src/lib/queries/simulation.ts` - Add entity_id parameter
- `frontend/src/lib/queries/auth.ts` - Add user subscription query

**Store (1 file)**:
- `frontend/src/lib/stores/authStore.ts` - Add currentEntityId

**Components (7 files)**:
- `frontend/src/components/layout/Sidebar.tsx` - Use user subscription for tier checks
- `frontend/src/components/transactions/CreateTransactionDialog.tsx` - Add entity selector
- `frontend/src/components/accounts/AccountForm.tsx` - Add entity selector
- `frontend/src/components/modals/UpgradeModal.tsx` - Use user subscription
- `frontend/src/components/dashboard/UsageMeter.tsx` - Use user subscription
- `frontend/src/components/dashboard/BudgetVsActual.tsx` - Add entity filter
- `frontend/src/components/dashboard/RecentActivity.tsx` - Add entity filter
- `frontend/src/components/simulation/SimulationControls.tsx` - Add entity selector
- `frontend/src/components/simulation/SimulationChart.tsx` - Add entity filter

**Pages (14+ files)**:
- `frontend/src/app/dashboard/page.tsx` - Add entity selector to main dashboard
- `frontend/src/app/dashboard/settings/page.tsx` - Remove org subscription display
- `frontend/src/app/dashboard/transactions/page.tsx` - Add entity filter
- `frontend/src/app/dashboard/accounts/page.tsx` - Add entity filter
- `frontend/src/app/dashboard/budgets/page.tsx` - Add entity filter
- `frontend/src/app/dashboard/accruals/page.tsx` - Add entity filter
- `frontend/src/app/dashboard/revaluation/page.tsx` - Add entity filter
- `frontend/src/app/dashboard/master-data/fiscal-periods/page.tsx` - Add entity filter
- `frontend/src/app/dashboard/intercompany/page.tsx` - Update for entity-based mappings
- `frontend/src/app/dashboard/reports/trial-balance/page.tsx` - Add entity selector
- `frontend/src/app/dashboard/reports/balance-sheet/page.tsx` - Add entity selector
- `frontend/src/app/dashboard/reports/income-statement/page.tsx` - Add entity selector
- `frontend/src/app/dashboard/reports/dimensional/page.tsx` - Add entity selector
- `frontend/src/app/dashboard/reports/account-ledger/page.tsx` - Add entity selector
- `frontend/src/app/dashboard/forensic/page.tsx` - Add entity selector
- `frontend/src/app/dashboard/simulation/page.tsx` - Add entity selector

---

### Summary by Complexity

#### High Complexity (4-6 hours each)
- Auth Store Refactoring (`authStore.ts`) - Affects entire app state management
- Query System Updates (10 query files) - Need to add entity_id to 40+ queries
- Intercompany Refactoring (4 backend files) - Complex validation logic changes
- Reports Implementation (5 report pages + backend) - Entity filtering + consolidation
- Dashboard Implementation (main page + widgets) - Entity filtering + consolidation

#### Medium Complexity (2-3 hours each)
- Form Updates (3 form components) - Add entity selectors
- Settings Pages (2 pages) - Move subscription display
- Sidebar Updates (`Sidebar.tsx`) - Change tier checking logic
- Entity Repository (`entity.rs`) - CRUD operations with tier limits
- Accruals/Revaluation Pages (2 pages) - Add entity filters
- Forensic/Simulation Pages (2 pages) - Add entity selectors

#### Low Complexity (1-2 hours each)
- Type Updates (3 type files) - Straightforward type changes
- Component Creation (`EntitySelector.tsx`) - Similar to org switcher
- List Page Updates (6 list pages) - Add entity filters
- Migration Scripts (2 migrations) - SQL schema changes
- Database Entity Updates (9 entity files) - Add entity_id column

---

### Total Effort Estimate

**Backend**: 4-5 days
- Day 1: Database migration + Entity repository (6-8 hours)
- Day 2: API routes (transactions, accounts, budgets, fiscal) (6-8 hours)
- Day 3: API routes (reports, dashboard, sentinel) (6-8 hours)
- Day 4: Intercompany updates + Testing (6-8 hours)

**Frontend**: 6-8 days
- Day 1: Types + Queries + Auth Store (6-8 hours)
- Day 2: Entity Selector + Forms (6-8 hours)
- Day 3: Lists (transactions, accounts, budgets, accruals, revaluation, fiscal) (6-8 hours)
- Day 4: Reports (5 report pages) (6-8 hours)
- Day 5: Dashboard + Widgets (6-8 hours)
- Day 6: Intercompany + Forensic + Simulation (6-8 hours)
- Day 7-8: Testing + Bug fixes (8-12 hours)

**Total**: 10-13 days for complete implementation

---

## 🎯 Implementation Timeline

### Day 1: Database & Core Backend (6-8 hours)
- ✅ Create entities table migration
- ✅ Move subscription to users table
- ✅ Add entity_id to existing tables (accounts, transactions, ledger, budgets, fiscal, accruals, revaluation)
- ✅ Update intercompany_mappings table (org_id → entity_id)
- ✅ Migrate existing data (orgs → entities, org subscription → user subscription)
- ✅ Create entity model (`entities.rs`)
- ✅ Create entity repository (`entity.rs`) with tier limit checks
- ✅ Update user repository with subscription methods

### Day 2: API Routes - Core Features (6-8 hours)
- ✅ Create entity API routes (`entities.rs`)
- ✅ Update subscription middleware (check user subscription)
- ✅ Update organization routes (remove subscription fields)
- ✅ Update transaction routes (add entity_id)
- ✅ Update account routes (add entity_id)
- ✅ Update budget routes (add entity_id)
- ✅ Update fiscal routes (add entity_id)

### Day 3: API Routes - Reports & Dashboard (6-8 hours)
- ✅ Update reports routes (add entity_id parameter to all 5 endpoints)
- ✅ Update dashboard routes (add entity_id parameter to all 4 endpoints)
- ✅ Add consolidated report support (all entities)
- ✅ Add consolidated dashboard support (all entities)

### Day 4: API Routes - Sentinel Intelligence (6-8 hours)
- ✅ Update sentinel routes (accruals - add entity_id)
- ✅ Update sentinel routes (revaluation - add entity_id)
- ✅ Update intercompany repository (entity-based validation)
- ✅ Update intercompany core logic (simplify validation)
- ✅ Update intercompany API endpoints (entity_id instead of org_id)
- ✅ Update forensic routes (add entity_id)
- ✅ Update simulation routes (add entity_id)
- ✅ Update background jobs (trial_expiry checks user trials)

### Day 5: Frontend Types & Queries (6-8 hours)
- ✅ Update organization types (remove subscription fields)
- ✅ Create entity types (`entities.ts`)
- ✅ Add UserSubscription type to auth types
- ✅ Update auth store (add currentEntityId)
- ✅ Simplify organization queries (remove create, list)
- ✅ Create entity queries (list, create, update, delete)
- ✅ Add user subscription query
- ✅ Update transaction queries (add entity_id parameter)
- ✅ Update account queries (add entity_id parameter)
- ✅ Update budget queries (add entity_id parameter)
- ✅ Update fiscal queries (add entity_id parameter)
- ✅ Update sentinel queries (accruals, revaluation, intercompany - add entity_id)
- ✅ Update report queries (add entity_id parameter to all 5 reports)
- ✅ Update dashboard queries (add entity_id parameter)
- ✅ Update forensic queries (add entity_id parameter)
- ✅ Update simulation queries (add entity_id parameter)

### Day 6: Frontend Components & Forms (6-8 hours)
- ✅ Create EntitySelector component
- ✅ Update Sidebar (replace org switcher with entity selector)
- ✅ Update Sidebar (use user subscription for tier checks)
- ✅ Update CreateTransactionDialog (add entity selector)
- ✅ Update AccountForm (add entity selector)
- ✅ Update budget forms (add entity selector)
- ✅ Update UpgradeModal (use user subscription)
- ✅ Update UsageMeter (use user subscription)
- ✅ Update BudgetVsActual (add entity filter)
- ✅ Update RecentActivity (add entity filter)

### Day 7: Frontend Lists & Master Data (6-8 hours)
- ✅ Update transaction list page (add entity filter)
- ✅ Update account list page (add entity filter)
- ✅ Update budget list page (add entity filter)
- ✅ Update accruals list page (add entity filter)
- ✅ Update revaluation list page (add entity filter)
- ✅ Update fiscal periods page (add entity filter)

### Day 8: Frontend Reports (6-8 hours)
- ✅ Update trial balance page (add entity selector + consolidated mode)
- ✅ Update balance sheet page (add entity selector + consolidated mode)
- ✅ Update income statement page (add entity selector + consolidated mode)
- ✅ Update dimensional report page (add entity selector + consolidated mode)
- ✅ Update account ledger page (add entity selector)

### Day 9: Frontend Dashboard & Sentinel (6-8 hours)
- ✅ Update main dashboard page (add entity selector + consolidated mode)
- ✅ Update dashboard widgets (entity filtering)
- ✅ Update settings page (remove org subscription display)
- ✅ Create user subscription page
- ✅ Update intercompany page (entity-based mappings)
- ✅ Update forensic page (add entity selector)
- ✅ Update simulation page (add entity selector)
- ✅ Update simulation controls (add entity selector)

### Day 10-11: Testing & Bug Fixes (12-16 hours)
- ✅ Backend unit tests (entity repository, intercompany validation)
- ✅ Backend integration tests (entity CRUD, intercompany mappings)
- ✅ Backend API tests (all 40+ endpoints with entity_id)
- ✅ Frontend component tests (EntitySelector, forms)
- ✅ Frontend integration tests (entity selection flow)
- ✅ E2E tests (create entity, create transaction with entity, intercompany mapping)
- ✅ E2E tests (reports with entity filter, dashboard with entity filter)
- ✅ Manual testing (full user flow)
- ✅ Bug fixes and refinements
- ✅ Performance testing
- ✅ Documentation updates

---

## ⚠️ Risks & Mitigation

### Risk 1: Existing Multi-Org Users
**Impact**: Users with multiple orgs need migration  
**Mitigation**: 
- Keep first org, convert others to entities
- Communicate change clearly via email/in-app notification
- Provide migration guide and FAQ
- Offer support during transition period
**Effort**: Requires data migration script + user communication

### Risk 2: Subscription Tier Checks Scattered Throughout Codebase
**Impact**: Many components check `org?.subscription_tier` directly  
**Mitigation**: 
- Create utility function `useUserTier()` hook for tier checks
- Search and replace all instances of org subscription checks
- Centralize tier checking logic
**Effort**: Search across codebase, ~20-30 locations to update

### Risk 3: Intercompany Mapping Migration
**Impact**: Existing org-to-org mappings need conversion to entity-to-entity  
**Mitigation**:
- Migrate mappings during deployment (org_id → entity_id)
- For users with multiple orgs, map to default entity of each org
- Test thoroughly with sample data
- Provide rollback plan
**Effort**: Database migration + validation + testing

### Risk 4: Entity Selector UX Complexity
**Impact**: Users need to select entity for every operation  
**Mitigation**: 
- Remember last selected entity in localStorage
- Show current entity in sidebar prominently
- Auto-select if user only has one entity
- Provide entity context throughout app
**Effort**: Add entity persistence to store + UI polish

### Risk 5: Frontend Query Refactoring
**Impact**: Need to add entity_id to 10+ query hooks  
**Mitigation**:
- Update queries incrementally
- Use TypeScript to catch missing entity_id parameters
- Test each query after update
- Maintain backward compatibility during transition
**Effort**: Systematic update of query files

### Risk 6: Intercompany Validation Logic Changes
**Impact**: Complex validation logic needs simplification  
**Mitigation**:
- Write comprehensive tests before refactoring
- Simplify validation step-by-step
- Keep old validation as fallback initially
- Test with real intercompany scenarios
**Effort**: Careful refactoring + extensive testing

### Risk 7: Data Migration Failures
**Impact**: Migration could fail for edge cases  
**Mitigation**:
- Test migration on copy of production data
- Handle edge cases (users with no orgs, deleted orgs, etc.)
- Implement rollback mechanism
- Run migration in transaction
- Monitor migration progress
**Effort**: Robust migration script + monitoring

---

**Last Updated**: 2026-01-24  
**Author**: Kiro AI Assistant  
**Status**: ✅ COMPLETE - Ready for Implementation Approval
