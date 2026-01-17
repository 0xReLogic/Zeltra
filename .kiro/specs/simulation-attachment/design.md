# Design Document

## Overview

This design extends the existing attachment system to support simulation attachments. The current system only supports transaction attachments - we need to add simulation_id support to the database schema, create new API endpoints, and build frontend components.

## Architecture

### System Context

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Frontend      │────▶│   Backend API   │────▶│   PostgreSQL    │
│   (Next.js)     │     │   (Axum/Rust)   │     │   Database      │
└─────────────────┘     └────────┬────────┘     └─────────────────┘
                                 │
                                 ▼
                        ┌─────────────────┐
                        │  Cloud Storage  │
                        │  (S3/R2/Local)  │
                        └─────────────────┘
```

### Database Schema Changes

Current `attachments` table has `transaction_id` - we need to add `simulation_id`:

```sql
ALTER TABLE attachments ADD COLUMN simulation_id UUID REFERENCES simulations(id) ON DELETE CASCADE;
ALTER TABLE attachments ADD CONSTRAINT chk_attachment_parent 
  CHECK (
    (transaction_id IS NOT NULL AND simulation_id IS NULL) OR
    (transaction_id IS NULL AND simulation_id IS NOT NULL)
  );
CREATE INDEX idx_attachments_simulation_id ON attachments(simulation_id) WHERE simulation_id IS NOT NULL;
```

### API Endpoints Design

New simulation attachment endpoints (parallel to transaction attachments):

| Method | Path | Description |
|--------|------|-------------|
| POST | `/organizations/{org_id}/simulations/{simulation_id}/attachments/upload` | Request upload URL |
| POST | `/organizations/{org_id}/simulations/{simulation_id}/attachments` | Confirm upload |
| GET | `/organizations/{org_id}/simulations/{simulation_id}/attachments` | List attachments |
| GET | `/organizations/{org_id}/attachments/{attachment_id}` | Get with download URL (existing) |
| DELETE | `/organizations/{org_id}/attachments/{attachment_id}` | Delete attachment (existing) |

### Component Hierarchy

```
SimulationPage
├── SimulationControls
├── SimulationChart
├── SimulationSummaryCards
└── SimulationAttachments (NEW)
    ├── SimulationAttachmentUpload (NEW)
    │   └── Dropzone
    └── SimulationAttachmentList (NEW)
        └── AttachmentItem (reuse existing)
```

## Detailed Design

### Component 1: Backend - Simulation Attachment Routes

Extends `backend/crates/api/src/routes/attachments.rs` with simulation-scoped endpoints.

#### Interfaces

```rust
// New route registration
pub fn simulation_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/organizations/{org_id}/simulations/{simulation_id}/attachments/upload",
            post(request_simulation_upload),
        )
        .route(
            "/organizations/{org_id}/simulations/{simulation_id}/attachments",
            post(confirm_simulation_upload).get(list_simulation_attachments),
        )
}

// Modified ConfirmUploadInput to support simulation_id
pub struct ConfirmUploadInput {
    pub attachment_id: Uuid,
    pub organization_id: Uuid,
    pub transaction_id: Option<Uuid>,  // Now optional
    pub simulation_id: Option<Uuid>,   // NEW
    // ... rest unchanged
}
```

#### Correctness Properties

- **CP1**: Attachment must belong to exactly one parent (transaction XOR simulation)
- **CP2**: User must be organization member to access simulation attachments
- **CP3**: Storage quota validation applies to simulation attachments
- **CP4**: File size limit (10MB) applies to simulation attachments

### Component 2: Backend - Database Repository

Extends `backend/crates/db/src/repositories/attachment.rs` with simulation queries.

#### Interfaces

```rust
impl AttachmentRepository {
    // NEW: List attachments by simulation
    pub async fn list_by_simulation(
        &self,
        simulation_id: Uuid,
        organization_id: Uuid,
    ) -> Result<Vec<Attachment>, DbError>;
    
    // MODIFIED: Create attachment with optional simulation_id
    pub async fn create(&self, input: CreateAttachmentInput) -> Result<Attachment, DbError>;
}

pub struct CreateAttachmentInput {
    pub organization_id: Uuid,
    pub transaction_id: Option<Uuid>,
    pub simulation_id: Option<Uuid>,  // NEW
    // ... rest unchanged
}
```

### Component 3: OpenAPI Specification

Updates to `contracts/openapi-split/10-simulation-attachments-schemas.yaml`:

#### Bug Fixes (BUG-007)

Change all `type: [T, 'null']` to proper nullable syntax:

```yaml
# BEFORE (incorrect - utoipa bug)
transaction_id:
  type:
  - string
  - 'null'

# AFTER (correct)
transaction_id:
  type: string
  format: uuid
  nullable: true
```

Fields to fix:
1. `RunSimulationRequest.account_adjustments`
2. `RunSimulationRequest.dimension_filters`
3. `RunSimulationRequest.expense_growth_rate`
4. `RunSimulationRequest.revenue_growth_rate`
5. `AttachmentResponse.download_url`
6. `AttachmentResponse.download_url_expires_at`
7. `AttachmentResponse.transaction_id`
8. `RequestUploadRequest.attachment_type`
9. `ConfirmUploadRequest.attachment_type`

#### New Schema

```yaml
SimulationAttachmentResponse:
  description: Response for a simulation attachment.
  allOf:
    - $ref: '#/components/schemas/AttachmentResponse'
    - type: object
      properties:
        simulation_id:
          description: Simulation ID.
          format: uuid
          type: string
```

### Component 4: Frontend - Query Hooks

New file: `frontend/src/lib/queries/simulation-attachments.ts`

#### Interfaces

```typescript
// Query keys
export const simulationAttachmentKeys = {
  all: ['simulation-attachments'] as const,
  simulation: (simulationId: string) =>
    [...simulationAttachmentKeys.all, 'simulation', simulationId] as const,
};

// Hooks
export function useSimulationAttachments(simulationId: string);
export function useRequestSimulationUpload(simulationId: string);
export function useConfirmSimulationUpload(simulationId: string);
export function useDeleteSimulationAttachment();
```

### Component 5: Frontend - SimulationAttachmentUpload Component

New file: `frontend/src/components/simulation/SimulationAttachmentUpload.tsx`

#### Interfaces

```typescript
interface SimulationAttachmentUploadProps {
  simulationId: string;
  onUploadComplete?: () => void;
}
```

#### Behavior

1. Drag-and-drop or click to select file
2. Validate file type (PDF, PNG, JPG, DOC, DOCX, XLS, XLSX, CSV)
3. Validate file size (max 10MB)
4. Show upload progress
5. Handle errors with toast notifications
6. Refresh attachment list on success

### Component 6: Frontend - SimulationAttachmentList Component

New file: `frontend/src/components/simulation/SimulationAttachmentList.tsx`

#### Interfaces

```typescript
interface SimulationAttachmentListProps {
  simulationId: string;
}
```

#### Behavior

1. Display list of attachments with filename, size, date, uploader
2. Click to download (fetches presigned URL)
3. Delete with confirmation dialog
4. Empty state when no attachments

## Sequence Diagrams

### Upload Flow

```
User          Frontend           Backend            Storage
 │               │                  │                  │
 │──Select File──▶                  │                  │
 │               │──Request URL────▶│                  │
 │               │◀──Upload URL─────│                  │
 │               │──PUT File───────────────────────────▶
 │               │◀──200 OK────────────────────────────│
 │               │──Confirm Upload──▶                  │
 │               │                  │──Verify File────▶│
 │               │                  │◀──File Exists────│
 │               │◀──Attachment─────│                  │
 │◀──Success─────│                  │                  │
```

### Download Flow

```
User          Frontend           Backend            Storage
 │               │                  │                  │
 │──Click DL────▶│                  │                  │
 │               │──Get Attachment──▶                  │
 │               │                  │──Generate URL───▶│
 │               │                  │◀──Presigned URL──│
 │               │◀──Attachment+URL─│                  │
 │               │──Redirect to URL────────────────────▶
 │◀──File Download─────────────────────────────────────│
```

## Error Handling

| Error | HTTP Status | User Message |
|-------|-------------|--------------|
| File too large | 400 | "File exceeds 10MB limit" |
| Invalid file type | 400 | "File type not allowed" |
| Storage quota exceeded | 402 | "Storage quota exceeded. Please upgrade." |
| Simulation not found | 404 | "Simulation not found" |
| Attachment not found | 404 | "Attachment not found" |
| Not organization member | 403 | "Access denied" |
| Storage unavailable | 503 | "Storage service unavailable" |

## Testing Strategy

### Unit Tests
- Attachment type parsing
- File validation logic
- Query key generation

### Integration Tests
- Upload flow (request → upload → confirm)
- List attachments
- Delete attachment
- Permission checks

### E2E Tests (Playwright)
- Upload file via drag-and-drop
- Upload file via click
- View attachment list
- Download attachment
- Delete attachment with confirmation
- Error states (file too large, wrong type)
