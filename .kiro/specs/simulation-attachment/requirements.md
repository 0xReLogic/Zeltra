# Requirements Document

## Introduction

This document specifies the requirements for the Simulation Attachment feature in Zeltra. The feature enables users to attach supporting documents (spreadsheets, reports, assumptions) to simulation runs for documentation and audit purposes. Currently, the attachment system only supports transaction attachments - this spec extends it to support simulation attachments.

## Glossary

- **Simulation**: A financial projection based on historical data with configurable growth rates
- **Simulation_Attachment**: A file attached to a simulation run for documentation
- **Presigned_URL**: A temporary URL for secure file upload/download to cloud storage
- **Storage_Provider**: Cloud storage service (S3, R2, Azure Blob, etc.)
- **Attachment_Type**: Classification of attachment (assumption, report, spreadsheet, etc.)

## Requirements

### Requirement 1: Request Simulation Attachment Upload URL

**User Story:** As a finance manager, I want to request an upload URL for a simulation attachment, so that I can securely upload supporting documents.

#### Acceptance Criteria

1. WHEN a user requests an upload URL for a simulation, THE System SHALL generate a presigned upload URL
2. WHEN generating the upload URL, THE System SHALL validate file size against organization storage quota
3. WHEN generating the upload URL, THE System SHALL validate file type against allowed MIME types (PDF, PNG, JPG, DOC, DOCX, XLS, XLSX, CSV)
4. IF the file size exceeds 10MB, THEN THE System SHALL reject the request with appropriate error message
5. IF the storage quota is exceeded, THEN THE System SHALL return a 402 Payment Required error with quota details
6. THE System SHALL return attachment_id, upload_url, upload_method, upload_headers, expires_at, and storage_key

### Requirement 2: Confirm Simulation Attachment Upload

**User Story:** As a finance manager, I want to confirm my upload completed successfully, so that the attachment is recorded in the system.

#### Acceptance Criteria

1. WHEN a user confirms an upload, THE System SHALL verify the file exists in storage
2. WHEN confirming the upload, THE System SHALL create an attachment record linked to the simulation
3. WHEN confirming the upload, THE System SHALL validate file size matches the original request
4. IF the file does not exist in storage, THEN THE System SHALL return a 400 Bad Request error
5. THE System SHALL return the complete attachment metadata including download URL

### Requirement 3: List Simulation Attachments

**User Story:** As a finance manager, I want to see all attachments for a simulation, so that I can review supporting documentation.

#### Acceptance Criteria

1. WHEN a user requests simulation attachments, THE System SHALL return all attachments for that simulation
2. WHEN listing attachments, THE System SHALL include filename, file_size, mime_type, uploaded_by, and created_at
3. WHEN listing attachments, THE System SHALL NOT include download URLs (use get single attachment for that)
4. IF the simulation has no attachments, THEN THE System SHALL return an empty array

### Requirement 4: Get Simulation Attachment with Download URL

**User Story:** As a finance manager, I want to download a simulation attachment, so that I can review the supporting document.

#### Acceptance Criteria

1. WHEN a user requests a single attachment, THE System SHALL return attachment metadata with presigned download URL
2. WHEN generating download URL, THE System SHALL set expiration to 1 hour
3. IF the attachment does not exist, THEN THE System SHALL return a 404 Not Found error
4. IF the user is not a member of the organization, THEN THE System SHALL return a 403 Forbidden error

### Requirement 5: Delete Simulation Attachment

**User Story:** As a finance manager, I want to delete a simulation attachment, so that I can remove outdated or incorrect documents.

#### Acceptance Criteria

1. WHEN a user deletes an attachment, THE System SHALL remove the file from storage
2. WHEN deleting an attachment, THE System SHALL remove the attachment record from database
3. WHEN deleting an attachment, THE System SHALL update organization storage usage
4. IF the attachment does not exist, THEN THE System SHALL return a 404 Not Found error
5. IF the storage service is unavailable, THEN THE System SHALL return a 503 Service Unavailable error

### Requirement 6: Frontend Attachment Upload UI

**User Story:** As a finance manager, I want to upload attachments through the simulation page, so that I can document my simulation assumptions.

#### Acceptance Criteria

1. WHEN viewing a simulation result, THE Simulation_Page SHALL display an attachment upload area
2. WHEN uploading a file, THE Simulation_Page SHALL show upload progress indicator
3. WHEN upload completes, THE Simulation_Page SHALL refresh the attachment list
4. WHEN a file is rejected, THE Simulation_Page SHALL display the error message
5. THE Simulation_Page SHALL support drag-and-drop file upload
6. THE Simulation_Page SHALL display file type restrictions and size limit

### Requirement 7: Frontend Attachment List UI

**User Story:** As a finance manager, I want to see and manage attachments on the simulation page, so that I can review and organize documentation.

#### Acceptance Criteria

1. WHEN viewing a simulation, THE Simulation_Page SHALL display a list of attachments
2. WHEN an attachment is displayed, THE Simulation_Page SHALL show filename, size, upload date, and uploader
3. WHEN clicking an attachment, THE Simulation_Page SHALL download the file
4. WHEN clicking delete on an attachment, THE Simulation_Page SHALL confirm before deleting
5. IF no attachments exist, THE Simulation_Page SHALL display an empty state message

### Requirement 8: OpenAPI Specification Alignment

**User Story:** As a developer, I want the OpenAPI spec to accurately define simulation attachment endpoints, so that generated types are correct.

#### Acceptance Criteria

1. THE OpenAPI spec SHALL define simulation attachment endpoints with correct path structure
2. THE OpenAPI spec SHALL use `nullable: true` instead of `type: [T, 'null']` for optional fields
3. THE OpenAPI spec SHALL mark optional query parameters as `required: false`
4. THE OpenAPI spec SHALL include SimulationAttachmentResponse schema with simulation_id field

### Requirement 9: Type Alignment with Backend

**User Story:** As a developer, I want frontend types to match backend responses, so that data is correctly parsed and displayed.

#### Acceptance Criteria

1. THE Frontend types SHALL match the OpenAPI schema definitions exactly
2. THE Frontend SHALL have useSimulationAttachments() hook for listing attachments
3. THE Frontend SHALL have useRequestSimulationUpload() hook for upload URL
4. THE Frontend SHALL have useConfirmSimulationUpload() hook for confirming upload
5. THE Frontend SHALL have useDeleteSimulationAttachment() hook for deletion
