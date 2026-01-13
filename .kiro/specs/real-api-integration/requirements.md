# Requirements Document

## Introduction

This feature integrates the Zeltra frontend application with the real backend API, removing all mock data dependencies. The integration covers authentication flows, organization management, role-based access control, and all CRUD operations for master data, transactions, and reports.

## Glossary

- **Frontend**: The Next.js 16 application with TypeScript and TanStack Query
- **Backend**: The Rust-based API server running at http://localhost:8080/api/v1
- **MSW**: Mock Service Worker - browser-based API mocking library currently used for development
- **API_Client**: The centralized HTTP client module in frontend/src/lib/api/client.ts
- **Organization**: A tenant entity that owns accounts, transactions, and users
- **Role**: User permission level within an organization (owner, admin, approver, accountant, viewer, submitter)
- **JWT**: JSON Web Token used for authentication

## Requirements

### Requirement 1: Remove Mock API Dependencies

**User Story:** As a developer, I want to remove all mock API dependencies, so that the frontend always communicates with the real backend.

#### Acceptance Criteria

1. WHEN the application starts, THE Frontend SHALL NOT initialize MSW mock handlers
2. WHEN an API request is made, THE API_Client SHALL send the request directly to the Backend without fallback mock logic
3. WHEN the MOCK_DATA constant exists in client.ts, THE Frontend SHALL have it removed entirely
4. WHEN the NEXT_PUBLIC_API_MOCK environment variable is checked, THE API_Client SHALL ignore it and always use real API
5. IF an API request fails, THEN THE API_Client SHALL propagate the error without attempting mock fallback

### Requirement 2: Fix Role Type Mismatch

**User Story:** As a developer, I want the frontend role types to match the backend, so that all 6 roles are properly supported.

#### Acceptance Criteria

1. THE Frontend SHALL define OrganizationUser role type with exactly 6 values: owner, admin, approver, accountant, viewer, submitter
2. WHEN displaying role selection UI, THE Frontend SHALL show all 6 role options
3. WHEN inviting a user, THE Frontend SHALL allow selecting the submitter role
4. WHEN updating a user role, THE Frontend SHALL allow changing to submitter role

### Requirement 3: Organization Creation UI

**User Story:** As a user, I want to create a new organization, so that I can start using Zeltra for my business.

#### Acceptance Criteria

1. WHEN a user navigates to organization settings, THE Frontend SHALL display a "Create Organization" button
2. WHEN a user clicks "Create Organization", THE Frontend SHALL display a form dialog with name, slug, base_currency, and timezone fields
3. WHEN a user submits a valid organization form, THE Frontend SHALL send POST request to /api/v1/organizations
4. WHEN organization creation succeeds, THE Frontend SHALL update the organization list and switch to the new organization
5. IF organization creation fails, THEN THE Frontend SHALL display the error message from the Backend
6. WHEN the slug field is edited, THE Frontend SHALL validate it contains only lowercase letters, numbers, and hyphens

### Requirement 4: Authentication Integration

**User Story:** As a user, I want to authenticate with the real backend, so that my session is secure and persistent.

#### Acceptance Criteria

1. WHEN a user submits login credentials, THE Frontend SHALL send POST request to /api/v1/auth/login
2. WHEN login succeeds, THE Frontend SHALL store access_token and refresh_token securely
3. WHEN a user registers, THE Frontend SHALL send POST request to /api/v1/auth/register
4. WHEN access_token expires, THE Frontend SHALL automatically refresh using the refresh_token
5. WHEN a user logs out, THE Frontend SHALL send POST request to /api/v1/auth/logout and clear stored tokens
6. IF authentication fails, THEN THE Frontend SHALL display the error message from the Backend

### Requirement 5: API Client Optimization

**User Story:** As a developer, I want the API client to be production-ready, so that it handles errors and authentication properly.

#### Acceptance Criteria

1. THE API_Client SHALL include Authorization header with Bearer token for authenticated requests
2. THE API_Client SHALL include X-Organization-ID header for organization-scoped requests
3. WHEN a 401 response is received, THE API_Client SHALL attempt token refresh before failing
4. WHEN a 403 response is received, THE API_Client SHALL propagate a permission denied error
5. WHEN a network error occurs, THE API_Client SHALL provide a user-friendly error message
6. THE API_Client SHALL use configurable timeout (default 30 seconds) instead of 3 seconds

