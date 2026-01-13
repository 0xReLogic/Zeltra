# Zeltra Frontend - AI Prompt

Role: Senior Frontend Developer (Next.js 16, TypeScript, TanStack Query)

Task: Convert frontend dari Mock API ke Real API Integration jangan pernah menggunakan emotikon 

CONTEXT:

Backend sudah COMPLETE dengan semua API endpoints
Frontend Phase 6-7 sudah ada tapi masih pake Mock API
E2E Testing sudah ada di MCP - tidak perlu setup!
Ada beberapa mismatches yang perlu di-fix, bisa di baca di roadmap.md dan proggres.md  wajib gunakan shearch biar menghemat token dari pada baca semua line, dan openapi.ymal sebagai kontrak mati jika api ga ada di schema juga ga ada minta ke backend di requests.md kalau sudah selesai wajib update proggres di roadmap di phase 6-7 dan juga proggres bahwa api udh ke sambung , kamu bisa hidupkan backendnya sendiri untuk db udh jalan di docker gunakan semua mcp yg di butuh kan
GANTI RULE: Frontend harus pake REAL API, bukan mock!
🚨 CURRENT ISSUES TO FIX:
1. Mock API Removal (URGENT)
Problem: Frontend masih pake mock data, tidak konek ke backend Action:

Disable MSW mock handlers di browser.ts
Update API client untuk selalu call real backend
Hapus fallback mock logic di client.ts
2. Organization Creation Missing
Problem: Frontend tidak ada UI/form untuk buat organization Backend: POST /api/v1/organizations ✅ ready Action:

Add CreateOrganizationRequest type
Add organization creation form/dialog
Add useCreateOrganization mutation
Add ke navigation menu
3. Role Type Mismatch
Problem: Frontend types missing submitter role Backend: 6 roles: owner, admin, approver, accountant, viewer, submitter Frontend: Cuma 5 roles (missing submitter) Action:

Update OrganizationUser['role'] type
Update semua role-related UI components
Add submitter ke role options
🎯 PHASE 6-7 INTEGRATION TASKS:
Phase 6: Foundation Integration
Switch Auth flows ke Real API
Login, register, refresh, logout
Test dengan backend JWT tokens
Fix token storage dan refresh logic
Organization Management Integration
List organizations (real data)
Create organization (tambah UI)
Update organization settings
User management dengan real roles
Remove Mock Dependencies
Disable MSW browser.ts
Remove mock data dari client.ts
Ensure API calls always ke backend
Phase 7: Features Integration
Master Data Integration
Accounts, fiscal periods, dimensions
Exchange rates dengan real data
Budget management
Transaction Workflow Integration
Real transaction CRUD
Approval workflow dengan real permissions
Reports dengan real backend data
🧪 E2E TESTING (SUDAH ADA!)
Existing E2E Tests Available:
✅ Authentication Flows (auth.spec.ts)

Login dengan valid credentials
Error handling dengan invalid credentials
Logout flow
✅ Smoke Tests (smoke.spec.ts)

Page loading dan navigation
Responsive design
Accessibility checks
Console error monitoring
✅ Transaction Management (transactions.spec.ts)

Create multi-currency transaction
File attachment upload
Form validation
✅ Approval Workflows (approvals.spec.ts)

Transaction approval/rejection
E2E Integration Tasks:
Update existing tests untuk Real API
Remove mock dependencies
Test dengan real backend data
Update test data untuk match backend schema
Add missing E2E scenarios
Organization creation flow
User management dengan role changes
Multi-tenancy isolation
Role-based access control
Configure E2E Environment
Backend URL: http://localhost:8080/api/v1
Frontend URL: http://localhost:3000
Test data cleanup
🔧 TECHNICAL REQUIREMENTS:
API Integration
✅ Gunakan real backend API: http://localhost:8080/api/v1
✅ Remove semua mock dependencies
✅ Proper error handling dengan backend error format
✅ JWT token management dengan refresh logic
✅ Organization context switching
Type Safety
✅ Semua types match backend response schemas
✅ OpenAPI spec compliance
✅ Runtime type validation jika perlu
📋 CHECKLIST FOR COMPLETION:
Phase 6 Foundation
All auth flows pake real API
Organization creation UI ada dan working
Role types match backend (6 roles)
Mock dependencies removed
API client optimized untuk production
Phase 7 Features
All CRUD operations pake real API
Dashboard dengan real backend data
Reports dan analytics working
Transaction workflow complete
Multi-tenancy isolation verified
E2E Testing
Playwright setup complete (SUDAH ADA!)
Update existing tests untuk real API
Add organization creation E2E
Add role management E2E
Add multi-tenancy E2E
CI/CD pipeline dengan E2E tests
 CRITICAL INSTRUCTIONS:
JANGAN PAKE MOCK API - Backend sudah ready!
FIX SEMUA CONTRACT MISMATCHES - Types, responses, errors
ADD MISSING UI - Organization creation, role management
TEST DENGAN REAL BACKEND - Update existing E2E tests
E2E SUDAH ADA - Cukup update dan tambah test scenarios
Frontend harus production-ready dengan real backend integration! 