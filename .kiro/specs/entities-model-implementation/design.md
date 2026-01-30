# Design Document: Entities Model Implementation

## Overview

This design document specifies the technical architecture for refactoring Zeltra's subscription model from organization-based to user-based, and replacing the multi-organization feature with an entities model.

### Problem Statement

The current implementation stores subscription fields (`subscription_tier`, `subscription_status`, `trial_ends_at`, etc.) per organization in the `organizations` table. However, the business model specifies that subscriptions are per user:
- Starter: $12/mo per USER → 1 entity
- Growth: $25/mo per USER → 5 entities
- Enterprise: Custom per USER → unlimited entities

This mismatch creates several issues:
1. Inconsistent with pricing model
2. Trial inheritance broken when users create multiple orgs
3. Complex upgrade logic (must update all orgs)
4. Data loss risk if user deletes their first org
5. Potential for multi-tier confusion

Additionally, the multi-organization feature adds unnecessary complexity. Industry analysis shows that customers don't need "multiple organizations" (separate workspaces), they need "multi-entity accounting" (multiple companies within one workspace), similar to NetSuite and Sage Intacct.

### Solution Approach

We will:
1. Move subscription fields from `organizations` table to `users` table
2. Create an `entities` table to represent companies/subsidiaries within an organization
3. Add `entity_id` foreign keys to all accounting data tables
4. Update intercompany mappings to use `entity_id` instead of `org_id`
5. Migrate existing data: organizations → entities, org subscriptions → user subscriptions
6. Update all backend APIs to accept and filter by `entity_id`
7. Update all frontend components to support entity selection and filtering

### Key Benefits

1. **Simpler Architecture**: No multi-org complexity, subscription naturally on user
2. **Better UX**: Unified workspace, entity selector (like QuickBooks company selector)
3. **Intercompany Hub Works Better**: Inter-entity transactions within one org are simpler
4. **Competitive Advantage**: Manage 5 companies for $25/mo in one workspace (vs Xero/QuickBooks charging per company)
5. **Faster Implementation**: Simpler changes, less can go wrong
6. **Lower Risk**: Fewer edge cases, easier to test
7. **Easier Maintenance**: No multi-org considerations in every feature

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         Frontend                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ Entity       │  │ Forms with   │  │ Lists with   │      │
│  │ Selector     │  │ Entity Field │  │ Entity Filter│      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│         │                  │                  │              │
│         └──────────────────┴──────────────────┘              │
│                            │                                 │
└────────────────────────────┼─────────────────────────────────┘
                             │ HTTP/JSON
┌────────────────────────────┼─────────────────────────────────┐
│                         Backend API                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ Subscription │  │ Entity       │  │ Transaction  │      │
│  │ Middleware   │  │ Routes       │  │ Routes       │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│         │                  │                  │              │
│         └──────────────────┴──────────────────┘              │
│                            │                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ Entity       │  │ Subscription │  │ Transaction  │      │
│  │ Repository   │  │ Repository   │  │ Repository   │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│                            │                                 │
└────────────────────────────┼─────────────────────────────────┘
                             │ SQL
┌────────────────────────────┼─────────────────────────────────┐
│                        PostgreSQL                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ users        │  │ entities     │  │ transactions │      │
│  │ + sub fields │  │              │  │ + entity_id  │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
```

### Component Interaction Flow

**Entity Creation Flow**:
```
User → EntitySelector → POST /entities → EntityRepository.create()
  → Check user tier → Count existing entities → Validate limit
  → Insert entity → Return entity
```

**Transaction Creation Flow**:
```
User → TransactionForm (with entity_id) → POST /transactions
  → Validate entity access → TransactionRepository.create()
  → Insert transaction with entity_id → Return transaction
```

**Report Generation Flow**:
```
User → ReportPage (select entity) → GET /reports/balance-sheet?entity_id=X
  → Filter data by entity_id → Generate report → Return report
```

**Consolidated Report Flow**:
```
User → ReportPage (select "All Entities") → GET /reports/balance-sheet?consolidated=true
  → Query all entities → Combine data → Eliminate intercompany
  → Generate report → Return report
```

### Database Schema Changes

**New Table: entities**
```sql
CREATE TABLE entities (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL REFERENCES organizations(id),
    name VARCHAR(255) NOT NULL,
    legal_name VARCHAR(255),
    tax_id VARCHAR(100),
    entity_type VARCHAR(50),  -- 'main', 'subsidiary', 'branch', 'division'
    base_currency VARCHAR(3) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    settings JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    UNIQUE(organization_id, name)
);
```

**Modified Table: users**
```sql
ALTER TABLE users 
  ADD COLUMN subscription_tier subscription_tier NOT NULL DEFAULT 'starter',
  ADD COLUMN subscription_status subscription_status NOT NULL DEFAULT 'trialing',
  ADD COLUMN trial_ends_at TIMESTAMPTZ,
  ADD COLUMN subscription_ends_at TIMESTAMPTZ,
  ADD COLUMN payment_provider VARCHAR(255),
  ADD COLUMN payment_customer_id VARCHAR(255),
  ADD COLUMN payment_subscription_id VARCHAR(255);
```

**Modified Tables: Add entity_id**
```sql
ALTER TABLE chart_of_accounts ADD COLUMN entity_id UUID REFERENCES entities(id);
ALTER TABLE transactions ADD COLUMN entity_id UUID REFERENCES entities(id);
ALTER TABLE ledger_entries ADD COLUMN entity_id UUID REFERENCES entities(id);
ALTER TABLE budgets ADD COLUMN entity_id UUID REFERENCES entities(id);
ALTER TABLE fiscal_years ADD COLUMN entity_id UUID REFERENCES entities(id);
ALTER TABLE accrual_schedules ADD COLUMN entity_id UUID REFERENCES entities(id);
ALTER TABLE revaluation_logs ADD COLUMN entity_id UUID REFERENCES entities(id);
```

**Modified Table: intercompany_mappings**
```sql
ALTER TABLE intercompany_mappings 
  RENAME COLUMN source_org_id TO source_entity_id;
ALTER TABLE intercompany_mappings 
  RENAME COLUMN target_org_id TO target_entity_id;
```

## Components and Interfaces

### Backend Components

#### Entity Model (`backend/crates/db/src/entities/entities.rs`)

```rust
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
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
```

#### Entity Repository (`backend/crates/db/src/repositories/entity.rs`)

```rust
pub struct EntityRepository {
    db: Arc<DatabaseConnection>,
}

impl EntityRepository {
    // Create entity with tier limit validation
    pub async fn create(
        &self,
        organization_id: Uuid,
        name: String,
        base_currency: String,
        entity_type: String,
    ) -> Result<entities::Model, DbErr>
    
    // List entities for organization
    pub async fn list_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<entities::Model>, DbErr>
    
    // Get entity by ID
    pub async fn find_by_id(
        &self,
        entity_id: Uuid,
    ) -> Result<Option<entities::Model>, DbErr>
    
    // Update entity
    pub async fn update(
        &self,
        entity_id: Uuid,
        name: Option<String>,
        legal_name: Option<String>,
        tax_id: Option<String>,
        entity_type: Option<String>,
        base_currency: Option<String>,
    ) -> Result<entities::Model, DbErr>
    
    // Soft delete entity
    pub async fn delete(
        &self,
        entity_id: Uuid,
    ) -> Result<(), DbErr>
    
    // Count entities for organization
    pub async fn count_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<i64, DbErr>
    
    // Get organization owner's subscription tier
    async fn get_org_owner_tier(
        &self,
        organization_id: Uuid,
    ) -> Result<SubscriptionTier, DbErr>
}
```

#### Entity API Routes (`backend/crates/api/src/routes/entities.rs`)

```rust
// List entities for organization
pub async fn list_entities(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    AuthUser(claims): AuthUser,
) -> Result<Json<Vec<EntityResponse>>, impl IntoResponse>

// Create entity
pub async fn create_entity(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    AuthUser(claims): AuthUser,
    Json(req): Json<CreateEntityRequest>,
) -> Result<Json<EntityResponse>, impl IntoResponse>

// Get entity
pub async fn get_entity(
    State(state): State<AppState>,
    Path((org_id, entity_id)): Path<(Uuid, Uuid)>,
    AuthUser(claims): AuthUser,
) -> Result<Json<EntityResponse>, impl IntoResponse>

// Update entity
pub async fn update_entity(
    State(state): State<AppState>,
    Path((org_id, entity_id)): Path<(Uuid, Uuid)>,
    AuthUser(claims): AuthUser,
    Json(req): Json<UpdateEntityRequest>,
) -> Result<Json<EntityResponse>, impl IntoResponse>

// Delete entity
pub async fn delete_entity(
    State(state): State<AppState>,
    Path((org_id, entity_id)): Path<(Uuid, Uuid)>,
    AuthUser(claims): AuthUser,
) -> Result<StatusCode, impl IntoResponse>
```

#### Subscription Middleware (`backend/crates/api/src/middleware/subscription.rs`)

```rust
pub async fn check_subscription_status(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    // Extract user_id from Claims
    let claims = request.extensions().get::<Claims>().copied();
    
    // Check user's subscription status (not org's)
    let user = users::Entity::find_by_id(claims.sub)
        .one(&*state.db)
        .await?;
    
    match user.subscription_status {
        SubscriptionStatus::Active | SubscriptionStatus::Trialing => {
            next.run(request).await
        }
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

#### Intercompany Repository Updates (`backend/crates/db/src/repositories/intercompany.rs`)

```rust
// Updated to use entity_id instead of org_id
pub async fn get_mappings(
    &self,
    source_entity_id: Uuid,
) -> Result<Vec<intercompany_mappings::Model>, DbErr>

pub async fn find_mapping_by_account(
    &self,
    source_entity_id: Uuid,
    source_account_id: Uuid,
) -> Result<Option<intercompany_mappings::Model>, DbErr>

// Simplified validation (no cross-org checks)
pub async fn validate_mapping(
    &self,
    source_entity_id: Uuid,
    target_entity_id: Uuid,
) -> Result<(), DbErr> {
    let source = entities::Entity::find_by_id(source_entity_id).one(&*self.db).await?;
    let target = entities::Entity::find_by_id(target_entity_id).one(&*self.db).await?;
    
    // Both entities must exist and be in same organization
    let source = source.ok_or_else(|| DbErr::Custom("Source entity not found".to_string()))?;
    let target = target.ok_or_else(|| DbErr::Custom("Target entity not found".to_string()))?;
    
    if source.organization_id != target.organization_id {
        return Err(DbErr::Custom("Entities must belong to same organization".to_string()));
    }
    
    Ok(())
}
```

### Frontend Components

#### Entity Type (`frontend/src/types/entities.ts`)

```typescript
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

export type CreateEntityRequest = {
  name: string
  legal_name?: string
  tax_id?: string
  entity_type: string
  base_currency: string
}

export type UpdateEntityRequest = {
  name?: string
  legal_name?: string
  tax_id?: string
  entity_type?: string
  base_currency?: string
}
```

#### User Subscription Type (`frontend/src/types/auth.ts`)

```typescript
export type UserSubscription = {
  subscription_tier: 'starter' | 'growth' | 'enterprise'
  subscription_status: 'trialing' | 'active' | 'expired' | 'cancelled'
  trial_ends_at?: string
  subscription_ends_at?: string
  payment_provider?: string
  payment_customer_id?: string
  payment_subscription_id?: string
}
```

#### Entity Queries (`frontend/src/lib/queries/entities.ts`)

```typescript
// List entities for current organization
export function useEntities() {
  const { currentOrgId } = useAuth()
  return useQuery({
    queryKey: ['entities', currentOrgId],
    queryFn: () => api.get(`/organizations/${currentOrgId}/entities`),
  })
}

// Get single entity
export function useEntity(entityId: string) {
  const { currentOrgId } = useAuth()
  return useQuery({
    queryKey: ['entity', entityId],
    queryFn: () => api.get(`/organizations/${currentOrgId}/entities/${entityId}`),
  })
}

// Create entity
export function useCreateEntity() {
  const { currentOrgId } = useAuth()
  const queryClient = useQueryClient()
  
  return useMutation({
    mutationFn: (data: CreateEntityRequest) =>
      api.post(`/organizations/${currentOrgId}/entities`, data),
    onSuccess: () => {
      queryClient.invalidateQueries(['entities', currentOrgId])
    },
  })
}

// Update entity
export function useUpdateEntity(entityId: string) {
  const { currentOrgId } = useAuth()
  const queryClient = useQueryClient()
  
  return useMutation({
    mutationFn: (data: UpdateEntityRequest) =>
      api.patch(`/organizations/${currentOrgId}/entities/${entityId}`, data),
    onSuccess: () => {
      queryClient.invalidateQueries(['entities', currentOrgId])
      queryClient.invalidateQueries(['entity', entityId])
    },
  })
}

// Delete entity
export function useDeleteEntity(entityId: string) {
  const { currentOrgId } = useAuth()
  const queryClient = useQueryClient()
  
  return useMutation({
    mutationFn: () =>
      api.delete(`/organizations/${currentOrgId}/entities/${entityId}`),
    onSuccess: () => {
      queryClient.invalidateQueries(['entities', currentOrgId])
    },
  })
}
```

#### Entity Selector Component (`frontend/src/components/entities/EntitySelector.tsx`)

```typescript
export function EntitySelector() {
  const { currentEntityId, setCurrentEntityId } = useAuth()
  const { data: entities } = useEntities()
  
  // Auto-select if only one entity
  useEffect(() => {
    if (entities?.length === 1 && !currentEntityId) {
      setCurrentEntityId(entities[0].id)
    }
  }, [entities, currentEntityId])
  
  // Persist selection to localStorage
  useEffect(() => {
    if (currentEntityId) {
      localStorage.setItem('currentEntityId', currentEntityId)
    }
  }, [currentEntityId])
  
  // Restore from localStorage on mount
  useEffect(() => {
    const stored = localStorage.getItem('currentEntityId')
    if (stored && entities?.some(e => e.id === stored)) {
      setCurrentEntityId(stored)
    }
  }, [entities])
  
  return (
    <Select value={currentEntityId} onValueChange={setCurrentEntityId}>
      <SelectTrigger>
        <SelectValue placeholder="Select entity" />
      </SelectTrigger>
      <SelectContent>
        {entities?.map(entity => (
          <SelectItem key={entity.id} value={entity.id}>
            {entity.name}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  )
}
```

#### Auth Store Updates (`frontend/src/lib/stores/authStore.ts`)

```typescript
interface AuthState {
  user: User | null
  currentOrgId: string | null
  currentEntityId: string | null  // NEW
  setAuth: (user: User, orgId: string) => void
  setCurrentEntityId: (entityId: string) => void  // NEW
  clearAuth: () => void
}

export const useAuthStore = create<AuthState>((set) => ({
  user: null,
  currentOrgId: null,
  currentEntityId: null,
  
  setAuth: (user, orgId) => set({ user, currentOrgId }),
  
  setCurrentEntityId: (entityId) => set({ currentEntityId: entityId }),
  
  clearAuth: () => set({ 
    user: null, 
    currentOrgId: null, 
    currentEntityId: null 
  }),
}))
```

## Data Models

### Entity Model

**Purpose**: Represents a legal or operational unit (company, subsidiary, branch, division) within an organization.

**Fields**:
- `id`: UUID primary key
- `organization_id`: Foreign key to organizations table
- `name`: Display name (e.g., "Acme Corp Main")
- `legal_name`: Legal entity name (e.g., "Acme Corporation Inc.")
- `tax_id`: Tax identification number (EIN, VAT, etc.)
- `entity_type`: Type of entity ('main', 'subsidiary', 'branch', 'division')
- `base_currency`: ISO 4217 currency code (e.g., "USD")
- `is_active`: Soft delete flag
- `settings`: JSONB for entity-specific configuration
- `created_at`: Timestamp of creation
- `updated_at`: Timestamp of last update

**Constraints**:
- Unique constraint on (organization_id, name)
- Foreign key to organizations(id) with CASCADE delete
- Check constraint: entity_type IN ('main', 'subsidiary', 'branch', 'division')
- Check constraint: base_currency matches ISO 4217 format

**Indexes**:
- Primary key on id
- Index on (organization_id, is_active) for filtering active entities
- Index on organization_id for joins

### User Subscription Model

**Purpose**: Stores subscription information for a user account.

**New Fields on users table**:
- `subscription_tier`: Enum ('starter', 'growth', 'enterprise')
- `subscription_status`: Enum ('trialing', 'active', 'expired', 'cancelled')
- `trial_ends_at`: Timestamp when trial expires
- `subscription_ends_at`: Timestamp when paid subscription expires
- `payment_provider`: Payment processor name (e.g., "stripe")
- `payment_customer_id`: Customer ID in payment system
- `payment_subscription_id`: Subscription ID in payment system

**Constraints**:
- Foreign key to tier_limits(tier) for subscription_tier
- Check constraint: trial_ends_at > created_at
- Check constraint: subscription_ends_at > created_at

**Indexes**:
- Index on subscription_status for filtering active users
- Index on (payment_provider, payment_customer_id) for payment lookups
- Index on trial_ends_at for trial expiry job

### Entity-Scoped Data Models

All accounting data models are updated to include `entity_id`:

**chart_of_accounts**:
- Add `entity_id UUID REFERENCES entities(id)`
- Index on (entity_id, account_code) for lookups
- Unique constraint on (entity_id, account_code)

**transactions**:
- Add `entity_id UUID REFERENCES entities(id)`
- Index on (entity_id, transaction_date) for filtering
- Index on (entity_id, status) for status queries

**ledger_entries**:
- Add `entity_id UUID REFERENCES entities(id)`
- Index on (entity_id, account_id) for account ledger
- Index on (entity_id, transaction_id) for transaction details

**budgets**:
- Add `entity_id UUID REFERENCES entities(id)`
- Index on (entity_id, fiscal_year_id) for budget queries
- Unique constraint on (entity_id, name, fiscal_year_id)

**fiscal_years**:
- Add `entity_id UUID REFERENCES entities(id)`
- Index on (entity_id, start_date) for period lookups
- Unique constraint on (entity_id, name)

**accrual_schedules**:
- Add `entity_id UUID REFERENCES entities(id)`
- Index on (entity_id, status) for processing
- Index on (entity_id, next_recognition_date) for scheduling

**revaluation_logs**:
- Add `entity_id UUID REFERENCES entities(id)`
- Index on (entity_id, revaluation_date) for history
- Index on (entity_id, account_id) for account revaluation history

### Intercompany Mapping Model

**Purpose**: Defines relationships between accounts in different entities for automatic transaction mirroring or elimination.

**Updated Fields**:
- `source_entity_id`: Foreign key to entities(id) - replaces source_org_id
- `target_entity_id`: Foreign key to entities(id) - replaces target_org_id
- `source_account_id`: Foreign key to chart_of_accounts(id)
- `target_account_id`: Foreign key to chart_of_accounts(id)
- `mapping_type`: Enum ('elimination', 'mirror')
- `auto_post`: Boolean flag for automatic posting

**Constraints**:
- Foreign keys to entities(id) for source and target
- Check constraint: source_entity_id != target_entity_id
- Check constraint: Both entities must be in same organization (enforced by trigger or application logic)
- Unique constraint on (source_entity_id, source_account_id)

**Indexes**:
- Index on source_entity_id for mapping lookups
- Index on (source_entity_id, source_account_id) for account mapping
- Index on target_entity_id for reverse lookups

### Tier Limits Model

**Purpose**: Defines limits for each subscription tier.

**Relevant Fields**:
- `tier`: Enum ('starter', 'growth', 'enterprise')
- `max_entities`: Maximum entities per organization (integer or null for unlimited)
- `max_users`: Maximum users per organization
- `max_transactions_per_month`: Transaction limit
- `max_storage_gb`: Storage limit in gigabytes

**Entity Limit Mapping**:
- Starter: max_entities = 1 (1 entity)
- Growth: max_entities = 5 (5 entities)
- Enterprise: max_entities = NULL (unlimited entities)

## Data Migration Strategy

### Migration Steps

**Step 1: Add New Columns**
```sql
-- Add subscription fields to users
ALTER TABLE users ADD COLUMN subscription_tier subscription_tier NOT NULL DEFAULT 'starter';
ALTER TABLE users ADD COLUMN subscription_status subscription_status NOT NULL DEFAULT 'trialing';
-- ... other subscription fields

-- Create entities table
CREATE TABLE entities (...);

-- Add entity_id to existing tables
ALTER TABLE chart_of_accounts ADD COLUMN entity_id UUID REFERENCES entities(id);
-- ... other tables
```

**Step 2: Migrate Subscription Data**
```sql
-- Copy subscription from user's first (oldest) organization to user account
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
```

**Step 3: Create Default Entities**
```sql
-- Create default entity for each organization
INSERT INTO entities (id, organization_id, name, legal_name, base_currency, entity_type, is_active, settings, created_at, updated_at)
SELECT 
  gen_random_uuid(),
  id,
  name || ' (Main)',
  name,
  base_currency,
  'main',
  true,
  '{}',
  NOW(),
  NOW()
FROM organizations;
```

**Step 4: Link Existing Data to Entities**
```sql
-- Link chart_of_accounts
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

**Step 5: Update Intercompany Mappings**
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
```

**Step 6: Validate Migration**
```sql
-- Check all entity_id foreign keys are valid
SELECT COUNT(*) FROM chart_of_accounts WHERE entity_id IS NULL;
SELECT COUNT(*) FROM transactions WHERE entity_id IS NULL;
-- ... other tables

-- Check all users have subscription data
SELECT COUNT(*) FROM users WHERE subscription_tier IS NULL;
SELECT COUNT(*) FROM users WHERE subscription_status IS NULL;
```

### Rollback Strategy

If migration fails:
1. Drop entity_id columns from all tables
2. Drop entities table
3. Drop subscription columns from users table
4. Restore intercompany_mappings column names
5. Restore from backup if data corruption occurred

### Migration Testing

Before production migration:
1. Create copy of production database
2. Run migration on copy
3. Validate data integrity
4. Test application with migrated data
5. Measure migration duration
6. Document any edge cases encountered


## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property Reflection

After analyzing all acceptance criteria, I identified the following redundancies:
- Requirements 2.3 and 2.4 are specific cases of 2.2 (tier limit enforcement) - can be combined
- Requirements 3.1-3.7 all test the same pattern (entity_id required) - can be combined
- Requirements 3.8-3.10 all test the same pattern (entity filtering) - can be combined
- Requirements 6.1-6.4 all test subscription status checks - can be combined
- Requirements 12.1-12.3 are covered by requirements 2.2-2.5 - redundant
- Requirement 4.3 is the negative statement of 4.2 - redundant

The following properties provide unique validation value without redundancy:

### Property 1: User Subscription Initialization

*For any* new user account, when the user is created, the subscription_tier should be 'starter', subscription_status should be 'trialing', and trial_ends_at should be set to a future date.

**Validates: Requirements 1.1**

### Property 2: Trial Expiration Processing

*For any* user with subscription_status 'trialing' and trial_ends_at in the past, when the trial expiry job runs, the user's subscription_status should be updated to 'expired'.

**Validates: Requirements 1.2, 14.1, 14.2**

### Property 3: Subscription Tier Update

*For any* user and any valid subscription tier, when the user's subscription tier is updated, the user's subscription_tier field should reflect the new tier.

**Validates: Requirements 1.3**

### Property 4: Organization Creation Without Subscription

*For any* new organization, when the organization is created, the organization record should not contain subscription fields (subscription_tier, subscription_status, etc.).

**Validates: Requirements 1.4**

### Property 5: Default Entity Creation

*For any* new organization with name N, when the organization is created, an entity should be automatically created with name "{N} (Main)", entity_type 'main', and is_active true.

**Validates: Requirements 2.1**

### Property 6: Entity Tier Limit Enforcement

*For any* user with subscription tier T and organization O, when the user attempts to create an entity in O, the creation should succeed if the current active entity count is less than the tier limit for T, and should fail with error "Entity limit reached for your tier" if the count equals or exceeds the limit.

**Validates: Requirements 2.2, 2.3, 2.4, 12.1, 12.2**

### Property 7: Enterprise Unlimited Entities

*For any* user with subscription tier 'enterprise', when the user creates any number of entities, all creations should succeed without limit checks.

**Validates: Requirements 2.5, 12.3**

### Property 8: Entity List Ordering and Filtering

*For any* organization with multiple entities, when entities are listed, the result should contain only entities with is_active true, ordered by created_at ascending.

**Validates: Requirements 2.6**

### Property 9: Entity Update

*For any* entity and any valid update data (name, legal_name, tax_id, entity_type, base_currency, settings), when the entity is updated, the entity's fields should reflect the new values.

**Validates: Requirements 2.7**

### Property 10: Entity Soft Delete

*For any* entity, when the entity is deleted, the entity should still exist in the database with is_active set to false.

**Validates: Requirements 2.8**

### Property 11: Entity-Scoped Data Creation

*For any* data type (chart_of_accounts, transactions, ledger_entries, budgets, fiscal_years, accrual_schedules, revaluation_logs), when creating a record without entity_id, the creation should fail with a validation error, and when creating with a valid entity_id, the record should be associated with that entity.

**Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7**

### Property 12: Entity-Scoped Data Filtering

*For any* data type (chart_of_accounts, transactions, budgets) and any entity E, when querying data with entity_id filter set to E, the results should contain only records where entity_id equals E.

**Validates: Requirements 3.8, 3.9, 3.10**

### Property 13: Intercompany Same-Organization Validation

*For any* two entities E1 and E2, when creating an intercompany mapping between E1 and E2, the creation should succeed if both entities belong to the same organization, and should fail with error "Entities must belong to the same organization" if they belong to different organizations.

**Validates: Requirements 4.2, 4.3**

### Property 14: Intercompany Mapping Filtering

*For any* organization O with multiple entities, when listing intercompany mappings for O, the results should contain only mappings where both source_entity_id and target_entity_id belong to entities in O.

**Validates: Requirements 4.4**

### Property 15: Intercompany Transaction Processing

*For any* intercompany mapping M with source entity E1 and target entity E2, when a transaction is posted to the source account specified in M, a corresponding mirror or elimination entry should be automatically created in E2 according to the mapping_type.

**Validates: Requirements 4.5**

### Property 16: Subscription Status Middleware

*For any* user with subscription_status S, when the user makes an API request, the request should be allowed if S is 'active' or 'trialing', and should be rejected with HTTP 402 if S is 'expired' or 'cancelled'.

**Validates: Requirements 6.1, 6.2, 6.3, 6.4**

### Property 17: Report Entity Filtering

*For any* financial report type (trial balance, balance sheet, income statement, dimensional) and any entity E, when generating the report with entity_id filter set to E, the report should include only data where entity_id equals E.

**Validates: Requirements 10.2**

### Property 18: Consolidated Report Generation

*For any* financial report type and any organization O with multiple entities, when generating the report with consolidated mode enabled, the report should combine data from all entities in O and eliminate intercompany transactions based on intercompany mappings.

**Validates: Requirements 10.3, 10.4**

### Property 19: Active Entity Count for Limits

*For any* organization O, when counting entities for tier limit checks, the count should include only entities where is_active is true.

**Validates: Requirements 12.4**

### Property 20: Error Message Consistency

*For any* error condition (entity limit exceeded, duplicate entity name, entities in different orgs, unauthorized access, expired subscription), when the error occurs, the system should return the appropriate HTTP status code and a clear error message describing the problem.

**Validates: Requirements 19.1, 19.2, 19.3, 19.4, 19.5**

## Error Handling

### Entity Creation Errors

**Entity Limit Exceeded**:
- HTTP Status: 400 Bad Request
- Error Code: `ENTITY_LIMIT_EXCEEDED`
- Message: "Entity limit reached for your tier. Upgrade to create more entities."
- Recovery: User must upgrade subscription tier

**Duplicate Entity Name**:
- HTTP Status: 400 Bad Request
- Error Code: `DUPLICATE_ENTITY_NAME`
- Message: "An entity with this name already exists in your organization"
- Recovery: User must choose a different name

**Invalid Entity Type**:
- HTTP Status: 400 Bad Request
- Error Code: `INVALID_ENTITY_TYPE`
- Message: "Entity type must be one of: main, subsidiary, branch, division"
- Recovery: User must provide valid entity type

### Intercompany Mapping Errors

**Entities in Different Organizations**:
- HTTP Status: 400 Bad Request
- Error Code: `ENTITIES_DIFFERENT_ORGS`
- Message: "Entities must belong to the same organization"
- Recovery: User must select entities from the same organization

**Source Entity Not Found**:
- HTTP Status: 404 Not Found
- Error Code: `SOURCE_ENTITY_NOT_FOUND`
- Message: "Source entity not found"
- Recovery: User must provide valid source entity ID

**Target Entity Not Found**:
- HTTP Status: 404 Not Found
- Error Code: `TARGET_ENTITY_NOT_FOUND`
- Message: "Target entity not found"
- Recovery: User must provide valid target entity ID

**Duplicate Mapping**:
- HTTP Status: 400 Bad Request
- Error Code: `DUPLICATE_MAPPING`
- Message: "An intercompany mapping already exists for this source account"
- Recovery: User must delete existing mapping or choose different account

### Subscription Errors

**Trial Expired**:
- HTTP Status: 402 Payment Required
- Error Code: `TRIAL_EXPIRED`
- Message: "Your trial has expired. Please upgrade to continue."
- Recovery: User must upgrade subscription

**Subscription Cancelled**:
- HTTP Status: 402 Payment Required
- Error Code: `SUBSCRIPTION_CANCELLED`
- Message: "Your subscription has been cancelled. Please reactivate to continue."
- Recovery: User must reactivate subscription

**Subscription Expired**:
- HTTP Status: 402 Payment Required
- Error Code: `SUBSCRIPTION_EXPIRED`
- Message: "Your subscription has expired. Please renew to continue."
- Recovery: User must renew subscription

### Data Access Errors

**Entity Not Found**:
- HTTP Status: 404 Not Found
- Error Code: `ENTITY_NOT_FOUND`
- Message: "Entity not found"
- Recovery: User must provide valid entity ID

**Unauthorized Entity Access**:
- HTTP Status: 403 Forbidden
- Error Code: `UNAUTHORIZED_ENTITY_ACCESS`
- Message: "You don't have access to this entity"
- Recovery: User must request access or use an entity they have access to

**Missing Entity ID**:
- HTTP Status: 400 Bad Request
- Error Code: `MISSING_ENTITY_ID`
- Message: "entity_id is required"
- Recovery: User must provide entity_id in request

### Migration Errors

**Migration Validation Failed**:
- Error: Migration script exits with non-zero code
- Message: "Migration validation failed: {specific error}"
- Recovery: Review migration logs, fix data issues, retry migration

**Orphaned Data**:
- Error: Records with null entity_id after migration
- Message: "Found {count} records with null entity_id"
- Recovery: Investigate orphaned records, manually assign entity_id or delete

**Invalid Foreign Keys**:
- Error: Foreign key constraint violations
- Message: "Invalid foreign key: {table}.{column} references non-existent entity"
- Recovery: Fix data integrity issues, ensure all referenced entities exist

## Testing Strategy

### Dual Testing Approach

We will use both unit tests and property-based tests to ensure comprehensive coverage:

**Unit Tests**: Verify specific examples, edge cases, and error conditions
- Test specific tier limits (Starter: 1 entity, Growth: 5 entities)
- Test specific error messages and HTTP status codes
- Test migration script with sample data
- Test UI component rendering and interactions

**Property Tests**: Verify universal properties across all inputs
- Test entity creation with random data and all tier combinations
- Test entity filtering with random entity sets
- Test subscription status checks with random user states
- Test intercompany validation with random entity pairs

### Property-Based Testing Configuration

**Library**: We will use `quickcheck` for Rust backend tests and `fast-check` for TypeScript frontend tests.

**Configuration**:
- Minimum 100 iterations per property test
- Each property test must reference its design document property
- Tag format: `Feature: entities-model-implementation, Property {number}: {property_text}`

**Example Property Test** (Rust):
```rust
#[quickcheck]
// Feature: entities-model-implementation, Property 6: Entity Tier Limit Enforcement
fn test_entity_tier_limit_enforcement(
    tier: SubscriptionTier,
    entity_count: u32,
) -> TestResult {
    // Generate random tier and entity count
    // Create user with tier
    // Create entities up to limit
    // Attempt to create one more
    // Verify success/failure based on tier limit
}
```

**Example Property Test** (TypeScript):
```typescript
// Feature: entities-model-implementation, Property 12: Entity-Scoped Data Filtering
fc.assert(
  fc.property(
    fc.array(fc.record({ entity_id: fc.uuid(), name: fc.string() })),
    fc.uuid(),
    (accounts, filterEntityId) => {
      // Create accounts with random entity_ids
      // Query with filterEntityId
      // Verify all results have entity_id === filterEntityId
    }
  ),
  { numRuns: 100 }
)
```

### Unit Test Coverage

**Backend Unit Tests**:
- Entity repository CRUD operations
- Entity tier limit validation
- Subscription middleware status checks
- Intercompany mapping validation
- Migration script data transformation
- Error message formatting

**Frontend Unit Tests**:
- EntitySelector component behavior
- Entity query hooks
- Auth store entity context
- Form validation with entity_id
- Error display components

### Integration Tests

**Backend Integration Tests**:
- Entity creation with different subscription tiers
- Entity filtering across multiple data types
- Intercompany mapping creation and validation
- Subscription status enforcement across API routes
- Report generation with entity filtering
- Consolidated report generation

**Frontend Integration Tests**:
- Entity selection flow (select entity → update context → refresh data)
- Form submission with entity_id (select entity → fill form → submit → verify)
- List filtering by entity (select filter → verify results)
- Report generation with entity selection

### End-to-End Tests

**Critical User Flows**:
1. Create entity → Create transaction with entity → View transaction in list
2. Create two entities → Create intercompany mapping → Post transaction → Verify mirror entry
3. Create entity → Generate report for entity → Verify data
4. Create multiple entities → Generate consolidated report → Verify combined data
5. Reach entity limit → Attempt to create entity → Verify error → Upgrade tier → Create entity successfully

**E2E Test Configuration**:
- Use Playwright for browser automation
- Test credentials: corp@zeltra.io / qwertyui
- Backend binary: `zeltra` (not zeltra-api)
- Frontend package manager: `pnpm` (not npm)

### Migration Testing

**Pre-Migration Tests**:
- Backup production database
- Create test copy of production data
- Validate data integrity before migration

**Migration Tests**:
- Run migration on test database
- Verify all entity_id foreign keys are valid
- Verify all users have subscription data
- Verify default entities created for all organizations
- Verify intercompany mappings updated correctly
- Measure migration duration

**Post-Migration Tests**:
- Run full test suite against migrated database
- Verify application functionality with migrated data
- Check for orphaned records
- Validate data consistency

### Performance Testing

**Entity Query Performance**:
- Measure query time for listing entities (target: < 100ms)
- Measure query time for filtering transactions by entity (target: < 200ms)
- Measure query time for consolidated reports (target: < 2s)

**Tier Limit Check Performance**:
- Measure time to count entities (target: < 50ms)
- Measure time to validate tier limits (target: < 100ms)

**Migration Performance**:
- Measure total migration time (target: < 5 minutes for 100k records)
- Measure time per table update (target: < 1 minute per table)

### Test Execution Order

1. **Unit Tests**: Run first, fastest feedback
2. **Property Tests**: Run after unit tests, comprehensive coverage
3. **Integration Tests**: Run after property tests, test component interactions
4. **E2E Tests**: Run last, slowest but most realistic
5. **Migration Tests**: Run separately on test database copy

### Continuous Integration

**Pre-Commit**:
- Run cargo fmt and cargo clippy for backend
- Run pnpm lint for frontend
- Run unit tests

**Pull Request**:
- Run all unit tests
- Run all property tests
- Run integration tests
- Run E2E tests for critical flows

**Pre-Deployment**:
- Run full test suite
- Run migration tests on production data copy
- Run performance tests
- Manual QA testing

### Test Data Management

**Test Fixtures**:
- Create reusable test data for common scenarios
- Use factories for generating random test data
- Clean up test data after each test

**Test Database**:
- Use separate test database
- Reset database state between tests
- Use transactions for test isolation

**Test Users**:
- Create test users with different subscription tiers
- Create test organizations with different entity counts
- Create test entities with different types and currencies

### Validation Steps

**Backend Validation**:
```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Run unit tests
cargo test

# Run property tests
cargo test --features quickcheck

# Check diagnostics
# Use getDiagnostics tool on modified files
```

**Frontend Validation**:
```bash
# Lint code
pnpm lint

# Type check
pnpm type-check

# Run unit tests
pnpm test

# Run property tests
pnpm test:property

# Build
pnpm build

# Check diagnostics
# Use getDiagnostics tool on modified files
```

**Integration Validation**:
```bash
# Start backend
cargo run --bin zeltra

# Start frontend
pnpm dev

# Run E2E tests
pnpm test:e2e
```

### Test Maintenance

**Test Review**:
- Review test coverage after each implementation
- Update tests when requirements change
- Remove obsolete tests
- Refactor duplicate test logic

**Test Documentation**:
- Document test setup and teardown
- Document test data requirements
- Document known test failures and workarounds
- Document test execution time expectations

**Test Monitoring**:
- Track test execution time trends
- Track test failure rates
- Track test coverage metrics
- Alert on test regressions
