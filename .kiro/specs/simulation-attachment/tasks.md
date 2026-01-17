# Implementation Tasks

## Task 1: Fix OpenAPI Nullable Type Syntax (BUG-007)

### Requirements Addressed
- REQ-8: OpenAPI Specification Alignment

### Files Modified
- `contracts/split-openapi.py` - Added `fix_nullable_syntax()` function

### Solution Applied

Modified `split-openapi.py` to auto-convert utoipa's OpenAPI 3.1 nullable syntax to OpenAPI 3.0 compatible format:

```python
def fix_nullable_syntax(obj):
    """
    Converts: type: [string, 'null'] -> type: string, nullable: true
    """
    if isinstance(obj, dict):
        if 'type' in obj and isinstance(obj['type'], list):
            type_list = obj['type']
            non_null_types = [t for t in type_list if t != 'null']
            has_null = 'null' in type_list
            
            if has_null and len(non_null_types) == 1:
                obj['type'] = non_null_types[0]
                obj['nullable'] = True
    # ... recursive processing
```

### Acceptance Criteria
- [x] All nullable fields use `nullable: true` instead of array syntax
- [x] Script auto-fixes on every run
- [x] No manual editing needed after regenerating OpenAPI spec

**STATUS: COMPLETED** ✅

---

## Task 2: Add Simulation Attachment Endpoints to OpenAPI

### Requirements Addressed
- REQ-8: OpenAPI Specification Alignment

### Files to Modify
- `contracts/openapi-split/26-attachments-endpoints.yaml`

### Detailed Instructions

Add new simulation attachment endpoints:

```yaml
/organizations/{org_id}/simulations/{simulation_id}/attachments/upload:
  post:
    summary: Request simulation attachment upload URL
    operationId: requestSimulationUpload
    tags: [Attachments]
    # ... similar to transaction upload

/organizations/{org_id}/simulations/{simulation_id}/attachments:
  post:
    summary: Confirm simulation attachment upload
    operationId: confirmSimulationUpload
    tags: [Attachments]
  get:
    summary: List simulation attachments
    operationId: listSimulationAttachments
    tags: [Attachments]
```

### Acceptance Criteria
- [ ] Three new endpoints defined
- [ ] Request/response schemas reference existing types
- [ ] Security requirements included
- [ ] Run `python3 contracts/split-openapi.py` to regenerate

---

## Task 3: Add simulation_id to AttachmentResponse Schema

### Requirements Addressed
- REQ-8: OpenAPI Specification Alignment
- REQ-9: Type Alignment with Backend

### Files to Modify
- `contracts/openapi-split/10-simulation-attachments-schemas.yaml`

### Detailed Instructions

Add `simulation_id` field to `AttachmentResponse`:

```yaml
AttachmentResponse:
  properties:
    # ... existing fields
    simulation_id:
      description: Simulation ID (for simulation attachments).
      format: uuid
      type: string
      nullable: true
```

### Acceptance Criteria
- [x] simulation_id field added with nullable: true
- [x] Field is optional (not in required array)

**STATUS: COMPLETED** ✅

---

## Task 4: Backend - Add simulation_id to Attachment Entity

### Requirements Addressed
- REQ-1, REQ-2, REQ-3, REQ-4, REQ-5

### Files Modified
- `backend/crates/db/src/entities/attachments.rs` - Added simulation_id column
- `backend/crates/db/src/repositories/attachment.rs` - Added list_by_simulation method
- `backend/crates/db/src/migration/m20260116_000001_simulation_attachments.rs` - New migration
- `backend/crates/core/src/attachment/types.rs` - Added simulation_id to all types
- `backend/crates/core/src/attachment/service.rs` - Added simulation attachment methods

### Acceptance Criteria
- [x] Entity has simulation_id field
- [x] Repository can list by simulation
- [x] Migration adds column with constraint

**STATUS: COMPLETED** ✅

---

## Task 5: Backend - Add Simulation Attachment Routes

### Requirements Addressed
- REQ-1, REQ-2, REQ-3, REQ-4, REQ-5

### Files Modified
- `backend/crates/api/src/routes/attachments.rs` - Added 3 new handlers

### Routes Added
1. `request_simulation_upload` - POST `/organizations/{org_id}/simulations/{simulation_id}/attachments/upload`
2. `confirm_simulation_upload` - POST `/organizations/{org_id}/simulations/{simulation_id}/attachments`
3. `list_simulation_attachments` - GET `/organizations/{org_id}/simulations/{simulation_id}/attachments`

### Acceptance Criteria
- [x] Three new route handlers
- [x] Routes registered in router
- [x] Membership check for all routes
- [x] Storage quota validation

**STATUS: COMPLETED** ✅

### Detailed Instructions

1. Add `simulation_id: Option<Uuid>` to Attachment entity
2. Add `list_by_simulation()` method to repository
3. Modify `create()` to accept optional simulation_id
4. Add database migration for simulation_id column

### Acceptance Criteria
- [ ] Entity has simulation_id field
- [ ] Repository can list by simulation
- [ ] Migration adds column with constraint

---

## Task 5: Backend - Add Simulation Attachment Routes

### Requirements Addressed
- REQ-1, REQ-2, REQ-3, REQ-4, REQ-5

### Files to Modify
- `backend/crates/api/src/routes/attachments.rs`
- `backend/crates/api/src/routes/mod.rs`

### Detailed Instructions

Add simulation attachment routes parallel to transaction routes:

1. `request_simulation_upload` - POST `/organizations/{org_id}/simulations/{simulation_id}/attachments/upload`
2. `confirm_simulation_upload` - POST `/organizations/{org_id}/simulations/{simulation_id}/attachments`
3. `list_simulation_attachments` - GET `/organizations/{org_id}/simulations/{simulation_id}/attachments`

Reuse existing `get_attachment` and `delete_attachment` handlers.

### Acceptance Criteria
- [ ] Three new route handlers
- [ ] Routes registered in router
- [ ] Membership check for all routes
- [ ] Storage quota validation

---

## Task 6: Frontend - Add Simulation Attachment Types

### Requirements Addressed
- REQ-9: Type Alignment with Backend

### Files to Modify
- `frontend/src/types/attachments.ts`

### Detailed Instructions

Add/update types:

```typescript
export interface SimulationAttachmentResponse extends AttachmentResponse {
  simulation_id: string;
}
```

### Acceptance Criteria
- [ ] SimulationAttachmentResponse type defined
- [ ] Types match OpenAPI schema

---

## Task 7: Frontend - Add Simulation Attachment Query Hooks

### Requirements Addressed
- REQ-9: Type Alignment with Backend

### Files to Create
- `frontend/src/lib/queries/simulation-attachments.ts`

### Detailed Instructions

Create hooks:

```typescript
export const simulationAttachmentKeys = {
  all: ['simulation-attachments'] as const,
  simulation: (simulationId: string) => [...simulationAttachmentKeys.all, 'simulation', simulationId] as const,
};

export function useSimulationAttachments(simulationId: string);
export function useRequestSimulationUpload(simulationId: string);
export function useConfirmSimulationUpload(simulationId: string);
// useDeleteAttachment from attachments.ts can be reused
```

### Acceptance Criteria
- [ ] Query keys defined
- [ ] useSimulationAttachments hook works
- [ ] useRequestSimulationUpload hook works
- [ ] useConfirmSimulationUpload hook works

---

## Task 8: Frontend - Create SimulationAttachmentUpload Component

### Requirements Addressed
- REQ-6: Frontend Attachment Upload UI

### Files to Create
- `frontend/src/components/simulation/SimulationAttachmentUpload.tsx`

### Detailed Instructions

Create upload component with:
- Drag-and-drop zone using react-dropzone
- File type validation (PDF, PNG, JPG, DOC, DOCX, XLS, XLSX, CSV)
- File size validation (10MB max)
- Upload progress indicator
- Error handling with toast

Reference: `frontend/src/components/attachments/AttachmentUpload.tsx`

### Acceptance Criteria
- [ ] Drag-and-drop works
- [ ] Click to select works
- [ ] File validation works
- [ ] Upload flow completes
- [ ] Errors shown properly

---

## Task 9: Frontend - Create SimulationAttachmentList Component

### Requirements Addressed
- REQ-7: Frontend Attachment List UI

### Files to Create
- `frontend/src/components/simulation/SimulationAttachmentList.tsx`

### Detailed Instructions

Create list component with:
- Display filename, size, date, uploader
- Download button (fetches presigned URL)
- Delete button with confirmation dialog
- Empty state message
- Loading state

### Acceptance Criteria
- [ ] List displays attachments
- [ ] Download works
- [ ] Delete with confirmation works
- [ ] Empty state shown when no attachments

---

## Task 10: Frontend - Integrate Attachments into Simulation Page

### Requirements Addressed
- REQ-6, REQ-7

### Files to Modify
- `frontend/src/app/dashboard/simulation/page.tsx`

### Detailed Instructions

Add attachment section to simulation page:

```tsx
{simulation.data && (
  <Card>
    <CardHeader>
      <CardTitle>Attachments</CardTitle>
    </CardHeader>
    <CardContent>
      <SimulationAttachmentUpload 
        simulationId={simulation.data.simulation_id} 
        onUploadComplete={() => refetchAttachments()}
      />
      <SimulationAttachmentList 
        simulationId={simulation.data.simulation_id}
      />
    </CardContent>
  </Card>
)}
```

### Acceptance Criteria
- [ ] Attachment section visible after simulation runs
- [ ] Upload component integrated
- [ ] List component integrated
- [ ] Components refresh properly

---

## Task 11: E2E Testing - Simulation Attachment Flow

### Requirements Addressed
- REQ-6, REQ-7

### Test Credentials
- Email: `corp@zeltra.io`
- Password: `qwertyui`
- URL: `http://10.0.0.5:3000`

### Detailed Instructions

Use Playwright MCP to test:

1. Login with credentials above
2. Navigate to simulation page
3. Run a simulation (set base period, growth rates, projection months)
4. Verify attachment section appears
5. Test file upload (if UI exists)
6. Test attachment list display
7. Test download functionality
8. Test delete with confirmation

### Acceptance Criteria
- [ ] Login successful
- [ ] Simulation runs
- [ ] Attachment UI visible (or document if missing)
- [ ] Upload flow works
- [ ] List displays correctly
- [ ] Download works
- [ ] Delete works

---

## Task 12: Save Bugs to Cognio Memory

### Requirements Addressed
- Documentation and learning

### Detailed Instructions

Use Cognio MCP to save discovered bugs to project `zeltra-bug`:

```
action: save
project: zeltra-bug
text: |
  BUG-007: OpenAPI Nullable Type Syntax
  - File: contracts/openapi-split/10-simulation-attachments-schemas.yaml
  - Issue: utoipa generates `type: [T, 'null']` instead of `nullable: true`
  - Affected: 9 fields (account_adjustments, dimension_filters, expense_growth_rate, revenue_growth_rate, download_url, download_url_expires_at, transaction_id, attachment_type x2)
  - Fix: Manually change to `type: T` + `nullable: true`
  
  BUG-008: Missing Simulation Attachment Feature
  - Issue: Backend attachment system only supports transaction_id, no simulation_id
  - Impact: Cannot attach documents to simulation runs
  - Fix: Add simulation_id column, new endpoints, frontend components
tags: ["utoipa", "openapi", "nullable", "simulation", "attachment"]
```

### Acceptance Criteria
- [x] BUG-007 saved to Cognio
- [x] BUG-008 saved to Cognio
- [x] Tags applied for searchability

**STATUS: COMPLETED** ✅
