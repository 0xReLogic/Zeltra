# Design Document: API Polish & Attachments (Phase 5)

## Overview

Phase 5 completes the Zeltra backend API with file attachment support, live exchange rate integration, missing master data endpoints, dashboard analytics, and production-ready polish including rate limiting and comprehensive error handling.

The design prioritizes:
1. **Vendor-agnostic storage** via Apache OpenDAL
2. **Presigned URLs** for direct client-to-storage transfers
3. **Consistent API patterns** across all endpoints
4. **Production readiness** with rate limiting and load testing

## Architecture

### High-Level Component Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                         API Layer (Axum)                        │
├─────────────────────────────────────────────────────────────────┤
│  Rate Limiter │ Request Logger │ Auth Middleware │ Error Handler│
└───────┬───────────────┬────────────────┬────────────────┬───────┘
        │               │                │                │
        ▼               ▼                ▼                ▼
┌───────────────┐ ┌───────────────┐ ┌───────────────┐ ┌───────────────┐
│  Attachment   │ │  Exchange     │ │  Dashboard    │ │  Master Data  │
│  Routes       │ │  Rate Routes  │ │  Routes       │ │  Routes       │
└───────┬───────┘ └───────┬───────┘ └───────┬───────┘ └───────┬───────┘
        │               │                │                │
        ▼               ▼                ▼                ▼
┌───────────────┐ ┌───────────────┐ ┌───────────────┐ ┌───────────────┐
│  Storage      │ │  Rate         │ │  Dashboard    │ │  Account/     │
│  Service      │ │  Fetcher      │ │  Service      │ │  Dimension    │
│  (OpenDAL)    │ │  (Frankfurter)│ │               │ │  Service      │
└───────┬───────┘ └───────┬───────┘ └───────┬───────┘ └───────┬───────┘
        │               │                │                │
        ▼               ▼                ▼                ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Database (PostgreSQL)                       │
│  attachments │ exchange_rates │ ledger_entries │ chart_of_accounts│
└─────────────────────────────────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Object Storage (OpenDAL)                      │
│     Azure Blob    │    Cloudflare R2    │    Local FS (dev)     │
└─────────────────────────────────────────────────────────────────┘
```

### Storage Architecture with OpenDAL

```
┌─────────────────────────────────────────────────────────────────┐
│                      Apache OpenDAL                              │
│                   (Unified Storage API)                          │
├─────────────────────────────────────────────────────────────────┤
│ op.write("key", data)      │ op.presign_read("key", duration)   │
│ op.read("key")             │ op.presign_write("key", duration)  │
│ op.delete("key")           │ op.stat("key")                     │
└─────────────────────────────────────────────────────────────────┘
     ▲           ▲           ▲           ▲           ▲
     │           │           │           │           │
┌────┴────┐ ┌────┴────┐ ┌────┴────┐ ┌────┴────┐ ┌────┴────┐
│  Azure  │ │Cloudflare│ │ Vercel │ │Supabase │ │  Local  │
│  Blob   │ │   R2    │ │  Blob  │ │ Storage │ │   FS    │
│ (Free*) │ │(10GB Fr)│ │(Vercel)│ │ (1GB Fr)│ │  (Dev)  │
└─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘
     │           │           │           │
     └───────────┴───────────┴───────────┘
                      │
              S3-Compatible API
```

### Supported Storage Providers

| Provider | Free Tier | Pricing | OpenDAL Service | Notes |
|----------|-----------|---------|-----------------|-------|
| **Cloudflare R2** | 10GB storage, 10M reads/mo | $0.015/GB/mo | `services::S3` | Zero egress fees, S3-compatible |
| **Vercel Blob** | Included in Vercel plan | $0.15/GB/mo | `services::VercelBlob` | Native integration if using Vercel |
| **Supabase Storage** | 1GB | $0.021/GB/mo | `services::S3` | S3-compatible, good for Supabase users |
| **Azure Blob** | 5GB (Student) | $0.018/GB/mo | `services::Azblob` | Enterprise-grade, good for Azure users |
| **AWS S3** | 5GB (12 months) | $0.023/GB/mo | `services::S3` | Industry standard |
| **DigitalOcean Spaces** | 250GB ($5/mo) | $0.02/GB/mo | `services::S3` | S3-compatible, simple pricing |
| **Local Filesystem** | Unlimited | Free | `services::Fs` | Development only |

### Provider Configuration Examples

```env
# Cloudflare R2 (Recommended - zero egress fees)
STORAGE_TYPE=s3
S3_ENDPOINT=https://<account_id>.r2.cloudflarestorage.com
S3_BUCKET=attachments
S3_ACCESS_KEY_ID=xxx
S3_SECRET_ACCESS_KEY=xxx
S3_REGION=auto

# Vercel Blob (if already on Vercel)
STORAGE_TYPE=vercel_blob
VERCEL_BLOB_TOKEN=xxx

# Supabase Storage (S3-compatible)
STORAGE_TYPE=s3
S3_ENDPOINT=https://<project_ref>.supabase.co/storage/v1/s3
S3_BUCKET=attachments
S3_ACCESS_KEY_ID=xxx
S3_SECRET_ACCESS_KEY=xxx
S3_REGION=auto

# Azure Blob (Enterprise)
STORAGE_TYPE=azblob
AZURE_STORAGE_ACCOUNT=zeltradev
AZURE_STORAGE_ACCESS_KEY=xxx
AZURE_CONTAINER=attachments

# Local (Development)
STORAGE_TYPE=fs
FS_ROOT=./storage/attachments
```

## Components and Interfaces

### 1. Storage Service

```rust
// backend/crates/core/src/storage/mod.rs

use opendal::{Operator, Result as OpendalResult};
use std::time::Duration;

pub struct StorageService {
    operator: Operator,
    bucket: String,
    max_file_size: u64,
    allowed_mime_types: Vec<String>,
}

pub struct PresignedUrl {
    pub url: String,
    pub method: String,
    pub expires_at: DateTime<Utc>,
    pub headers: HashMap<String, String>,
}

pub struct UploadRequest {
    pub organization_id: Uuid,
    pub transaction_id: Option<Uuid>,
    pub filename: String,
    pub content_type: String,
    pub file_size: u64,
}

pub struct AttachmentMetadata {
    pub id: Uuid,
    pub storage_key: String,
    pub filename: String,
    pub content_type: String,
    pub file_size: u64,
    pub checksum: Option<String>,
}

impl StorageService {
    /// Create storage service from config
    pub async fn from_config(config: &StorageConfig) -> Result<Self, StorageError>;
    
    /// Generate presigned URL for upload
    pub async fn presign_upload(&self, req: &UploadRequest) -> Result<PresignedUrl, StorageError>;
    
    /// Generate presigned URL for download
    pub async fn presign_download(&self, key: &str) -> Result<PresignedUrl, StorageError>;
    
    /// Verify file exists in storage
    pub async fn verify_upload(&self, key: &str) -> Result<AttachmentMetadata, StorageError>;
    
    /// Delete file from storage
    pub async fn delete(&self, key: &str) -> Result<(), StorageError>;
    
    /// Validate file type and size
    pub fn validate_upload(&self, content_type: &str, size: u64) -> Result<(), StorageError>;
}

#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub provider: StorageProvider,
    pub bucket: String,
    pub max_file_size_mb: u64,
    pub presign_upload_ttl_secs: u64,
    pub presign_download_ttl_secs: u64,
}

#[derive(Debug, Clone)]
pub enum StorageProvider {
    /// Azure Blob Storage (Enterprise)
    AzureBlob {
        account: String,
        access_key: String,
        container: String,
    },
    /// S3-compatible: Cloudflare R2, Supabase Storage, AWS S3, DigitalOcean Spaces
    S3 {
        endpoint: String,
        bucket: String,
        access_key_id: String,
        secret_access_key: String,
        region: String,
    },
    /// Vercel Blob (native integration)
    VercelBlob {
        token: String,
    },
    /// Local filesystem (development only)
    LocalFs {
        root: PathBuf,
    },
}
```

### 2. Attachment Service

```rust
// backend/crates/core/src/attachment/mod.rs

pub struct AttachmentService {
    storage: Arc<StorageService>,
    repo: Arc<dyn AttachmentRepository>,
}

pub struct CreateAttachmentRequest {
    pub organization_id: Uuid,
    pub transaction_id: Uuid,
    pub filename: String,
    pub content_type: String,
    pub file_size: u64,
    pub uploaded_by: Uuid,
}

pub struct Attachment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub transaction_id: Option<Uuid>,
    pub filename: String,
    pub content_type: String,
    pub file_size: u64,
    pub storage_key: String,
    pub checksum: Option<String>,
    pub uploaded_by: Uuid,
    pub created_at: DateTime<Utc>,
}

impl AttachmentService {
    /// Request upload URL - returns presigned URL for direct upload
    pub async fn request_upload(
        &self,
        org_id: Uuid,
        transaction_id: Uuid,
        filename: &str,
        content_type: &str,
        file_size: u64,
        user_id: Uuid,
    ) -> Result<(Uuid, PresignedUrl), AttachmentError>;
    
    /// Confirm upload - verify file exists and create DB record
    pub async fn confirm_upload(
        &self,
        attachment_id: Uuid,
        org_id: Uuid,
    ) -> Result<Attachment, AttachmentError>;
    
    /// Get download URL
    pub async fn get_download_url(
        &self,
        attachment_id: Uuid,
        org_id: Uuid,
    ) -> Result<PresignedUrl, AttachmentError>;
    
    /// Delete attachment
    pub async fn delete(
        &self,
        attachment_id: Uuid,
        org_id: Uuid,
    ) -> Result<(), AttachmentError>;
    
    /// List attachments for transaction
    pub async fn list_by_transaction(
        &self,
        transaction_id: Uuid,
        org_id: Uuid,
    ) -> Result<Vec<Attachment>, AttachmentError>;
}
```

### 3. Exchange Rate Fetcher

```rust
// backend/crates/core/src/currency/fetcher.rs

use reqwest::Client;

pub struct ExchangeRateFetcher {
    client: Client,
    base_url: String,
    source: RateSource,
}

#[derive(Debug, Clone)]
pub enum RateSource {
    Frankfurter { base_url: String },
    Mock,
    Manual,
}

pub struct FetchedRate {
    pub from_currency: String,
    pub to_currency: String,
    pub rate: Decimal,
    pub effective_date: NaiveDate,
    pub source: String,
}

pub struct BulkRateImport {
    pub rates: Vec<RateImportItem>,
}

pub struct RateImportItem {
    pub from_currency: String,
    pub to_currency: String,
    pub rate: Decimal,
    pub effective_date: NaiveDate,
}

impl ExchangeRateFetcher {
    /// Fetch latest rates from Frankfurter API
    pub async fn fetch_latest(
        &self,
        base_currency: &str,
        target_currencies: &[String],
    ) -> Result<Vec<FetchedRate>, FetcherError>;
    
    /// Fetch historical rates for a specific date
    pub async fn fetch_historical(
        &self,
        base_currency: &str,
        target_currencies: &[String],
        date: NaiveDate,
    ) -> Result<Vec<FetchedRate>, FetcherError>;
}

// Frankfurter API response structure
#[derive(Debug, Deserialize)]
struct FrankfurterResponse {
    amount: f64,
    base: String,
    date: String,
    rates: HashMap<String, f64>,
}
```

### 4. Dashboard Service

```rust
// backend/crates/core/src/dashboard/service.rs

pub struct DashboardService {
    ledger_repo: Arc<dyn LedgerRepository>,
    budget_repo: Arc<dyn BudgetRepository>,
    transaction_repo: Arc<dyn TransactionRepository>,
}

pub struct DashboardMetrics {
    pub period: Option<FiscalPeriodSummary>,
    pub cash_position: CashPosition,
    pub burn_rate: BurnRate,
    pub runway_days: i32,
    pub pending_approvals: PendingApprovals,
}

pub struct CashPosition {
    pub balance: Decimal,
    pub currency: String,
    pub change_from_last_period: Decimal,
    pub change_percent: Decimal,
}

pub struct BurnRate {
    pub daily: Decimal,
    pub monthly: Decimal,
}

pub struct PendingApprovals {
    pub count: i32,
    pub total_amount: Decimal,
}

pub struct CashFlowDataPoint {
    pub month: String,
    pub inflow: Decimal,
    pub outflow: Decimal,
}

pub struct ActivityItem {
    pub id: Uuid,
    pub activity_type: ActivityType,
    pub action: String,
    pub entity_type: EntityType,
    pub entity_id: Uuid,
    pub description: String,
    pub amount: Option<Decimal>,
    pub currency: Option<String>,
    pub user: Option<UserSummary>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum ActivityType {
    TransactionCreated,
    TransactionSubmitted,
    TransactionApproved,
    TransactionRejected,
    TransactionPosted,
    TransactionVoided,
    BudgetCreated,
    BudgetUpdated,
    BudgetLocked,
    UserInvited,
    UserRoleChanged,
}

impl DashboardService {
    /// Get dashboard metrics
    pub async fn get_metrics(
        &self,
        org_id: Uuid,
        period_id: Option<Uuid>,
    ) -> Result<DashboardMetrics, DashboardError>;
    
    /// Get cash flow data for charts
    pub async fn get_cash_flow(
        &self,
        org_id: Uuid,
        period_id: Option<Uuid>,
        months: u32,
    ) -> Result<Vec<CashFlowDataPoint>, DashboardError>;
    
    /// Get recent activity
    pub async fn get_recent_activity(
        &self,
        org_id: Uuid,
        limit: u32,
        activity_type: Option<ActivityType>,
        cursor: Option<String>,
    ) -> Result<(Vec<ActivityItem>, Option<String>), DashboardError>;
    
    /// Get budget vs actual summary
    pub async fn get_budget_vs_actual(
        &self,
        org_id: Uuid,
        budget_id: Option<Uuid>,
    ) -> Result<BudgetVsActualSummary, DashboardError>;
}
```

### 5. Rate Limiter Middleware

```rust
// backend/crates/api/src/middleware/rate_limit.rs

use tower_governor::{GovernorConfig, GovernorConfigBuilder, GovernorLayer};

pub struct RateLimitConfig {
    pub requests_per_second: u64,
    pub burst_size: u32,
}

pub fn create_rate_limit_layer(config: &RateLimitConfig) -> GovernorLayer {
    let governor_config = GovernorConfigBuilder::default()
        .per_second(config.requests_per_second)
        .burst_size(config.burst_size)
        .finish()
        .expect("Failed to build rate limit config");
    
    GovernorLayer::new(governor_config)
}

// Custom error handler for rate limit exceeded
pub async fn rate_limit_error_handler(
    err: GovernorError,
) -> impl IntoResponse {
    let retry_after = err.wait_time().as_secs();
    (
        StatusCode::TOO_MANY_REQUESTS,
        [("Retry-After", retry_after.to_string())],
        Json(ApiError {
            code: "RATE_LIMIT_EXCEEDED".to_string(),
            message: format!("Too many requests. Retry after {} seconds.", retry_after),
            details: None,
            request_id: None,
        }),
    )
}
```

### 6. Request Logger Middleware

```rust
// backend/crates/api/src/middleware/logging.rs

use tower_http::trace::{TraceLayer, DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse};
use tracing::Level;

pub fn create_trace_layer() -> TraceLayer<...> {
    TraceLayer::new_for_http()
        .make_span_with(|request: &Request<Body>| {
            let request_id = request
                .headers()
                .get("x-request-id")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("unknown");
            
            tracing::info_span!(
                "http_request",
                request_id = %request_id,
                method = %request.method(),
                uri = %request.uri(),
                user_id = tracing::field::Empty,
            )
        })
        .on_request(DefaultOnRequest::new().level(Level::INFO))
        .on_response(|response: &Response<_>, latency: Duration, span: &Span| {
            span.record("status", response.status().as_u16());
            span.record("latency_ms", latency.as_millis());
            tracing::info!("request completed");
        })
}
```

## Data Models

### Attachment Entity

```rust
// backend/crates/db/src/entities/attachments.rs

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "attachments")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub organization_id: Uuid,
    pub transaction_id: Option<Uuid>,
    pub attachment_type: AttachmentType,
    pub file_name: String,
    pub file_size: i64,
    pub mime_type: String,
    pub checksum_sha256: Option<String>,
    pub storage_provider: StorageProvider,
    pub storage_bucket: String,
    pub storage_key: String,
    pub storage_region: Option<String>,
    pub extracted_data: Option<Json>,
    pub ocr_processed_at: Option<DateTimeWithTimeZone>,
    pub uploaded_by: Uuid,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "attachment_type")]
pub enum AttachmentType {
    #[sea_orm(string_value = "receipt")]
    Receipt,
    #[sea_orm(string_value = "invoice")]
    Invoice,
    #[sea_orm(string_value = "contract")]
    Contract,
    #[sea_orm(string_value = "other")]
    Other,
}

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "storage_provider")]
pub enum StorageProvider {
    #[sea_orm(string_value = "azure_blob")]
    AzureBlob,
    #[sea_orm(string_value = "cloudflare_r2")]
    CloudflareR2,
    #[sea_orm(string_value = "local")]
    Local,
}
```

### API Request/Response Types

```rust
// backend/crates/api/src/routes/attachments.rs

#[derive(Debug, Deserialize)]
pub struct RequestUploadRequest {
    pub filename: String,
    pub content_type: String,
    pub file_size: u64,
}

#[derive(Debug, Serialize)]
pub struct RequestUploadResponse {
    pub attachment_id: Uuid,
    pub upload_url: String,
    pub upload_method: String,
    pub upload_headers: HashMap<String, String>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct AttachmentResponse {
    pub id: Uuid,
    pub transaction_id: Option<Uuid>,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    pub description: Option<String>,
    pub url: Option<String>,
    pub uploaded_by: UserSummary,
    pub uploaded_at: DateTime<Utc>,
}

// Exchange Rate Bulk Import
#[derive(Debug, Deserialize)]
pub struct BulkRateImportRequest {
    pub rates: Vec<RateImportItem>,
}

#[derive(Debug, Deserialize)]
pub struct RateImportItem {
    pub from_currency: String,
    pub to_currency: String,
    pub rate: String,
    pub effective_date: NaiveDate,
}

#[derive(Debug, Serialize)]
pub struct BulkRateImportResponse {
    pub imported_count: usize,
    pub updated_count: usize,
    pub errors: Vec<RateImportError>,
}
```

## Error Handling

### Consistent Error Response Format

```rust
// backend/crates/shared/src/error.rs

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: ApiErrorBody,
}

#[derive(Debug, Serialize)]
pub struct ApiErrorBody {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

// Error codes
pub const ERR_VALIDATION: &str = "VALIDATION_ERROR";
pub const ERR_NOT_FOUND: &str = "NOT_FOUND";
pub const ERR_UNAUTHORIZED: &str = "UNAUTHORIZED";
pub const ERR_FORBIDDEN: &str = "FORBIDDEN";
pub const ERR_RATE_LIMIT: &str = "RATE_LIMIT_EXCEEDED";
pub const ERR_STORAGE: &str = "STORAGE_ERROR";
pub const ERR_EXTERNAL_SERVICE: &str = "EXTERNAL_SERVICE_ERROR";
pub const ERR_INTERNAL: &str = "INTERNAL_ERROR";

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self.error.code.as_str() {
            ERR_VALIDATION => StatusCode::BAD_REQUEST,
            ERR_NOT_FOUND => StatusCode::NOT_FOUND,
            ERR_UNAUTHORIZED => StatusCode::UNAUTHORIZED,
            ERR_FORBIDDEN => StatusCode::FORBIDDEN,
            ERR_RATE_LIMIT => StatusCode::TOO_MANY_REQUESTS,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        
        (status, Json(self)).into_response()
    }
}
```



## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Presigned URL TTL Validity

*For any* presigned URL generated by the Storage_Service, the expiration time SHALL be within the configured TTL (15 minutes for upload, 1 hour for download) with a tolerance of ±5 seconds.

**Validates: Requirements 1.1, 1.3**

### Property 2: MIME Type Validation

*For any* file upload request, the Attachment_Service SHALL accept only files with MIME types in the allowed list (PDF, PNG, JPG, JPEG, GIF, WEBP, DOC, DOCX, XLS, XLSX) and reject all others with a validation error.

**Validates: Requirements 1.5**

### Property 3: File Size Validation

*For any* file upload request with size exceeding the configured limit (10MB default), the Attachment_Service SHALL reject the request with a clear error message containing the size limit.

**Validates: Requirements 1.6**

### Property 4: Storage Key Format

*For any* attachment created, the storage_key SHALL match the pattern `{org_id}/{transaction_id}/{attachment_id}/{filename}` where all UUIDs are valid and filename is sanitized.

**Validates: Requirements 1.8**

### Property 5: Bulk Rate Import Atomicity

*For any* bulk rate import request, if any rate in the batch fails validation, then no rates SHALL be inserted and the existing rates SHALL remain unchanged.

**Validates: Requirements 2.4**

### Property 6: Rate Upsert Behavior

*For any* exchange rate with the same (organization_id, from_currency, to_currency, effective_date) as an existing rate, the import SHALL update the existing rate rather than create a duplicate.

**Validates: Requirements 2.5**

### Property 7: External Service Failure Isolation

*For any* failure in the Frankfurter API, the existing exchange rates in the database SHALL remain unchanged and the error SHALL be returned to the caller.

**Validates: Requirements 2.3**

### Property 8: Toggle Status Idempotence

*For any* account or dimension value, toggling the status twice SHALL return the entity to its original state.

**Validates: Requirements 3.1, 3.3**

### Property 9: Deactivated Entity Usage Prevention

*For any* deactivated account, new ledger entries SHALL be rejected with a validation error. *For any* deactivated dimension value, new entry assignments SHALL be rejected with a validation error.

**Validates: Requirements 3.4, 3.5**

### Property 10: Cash Position Calculation

*For any* organization, the cash position SHALL equal the sum of current balances of all accounts with subtype 'cash' or 'bank'.

**Validates: Requirements 4.2**

### Property 11: Burn Rate Calculation

*For any* organization with expense transactions in the last 30 days, the daily burn rate SHALL equal the total expenses divided by 30.

**Validates: Requirements 4.3**

### Property 12: Runway Days Calculation

*For any* organization with positive burn rate, runway days SHALL equal cash position divided by daily burn rate, rounded down to the nearest integer.

**Validates: Requirements 4.4**

### Property 13: Cash Flow Aggregation

*For any* month in the cash flow data, the inflow SHALL equal the sum of all credit entries to cash/bank accounts, and outflow SHALL equal the sum of all debit entries from cash/bank accounts.

**Validates: Requirements 4.5**

### Property 14: Cursor-Based Pagination Consistency

*For any* paginated activity request, following the cursor to the next page SHALL return items that are strictly older than all items on the current page, with no duplicates or gaps.

**Validates: Requirements 4.6**

### Property 15: Error Response Consistency

*For any* API error response, the response body SHALL contain an `error` object with `code` (string) and `message` (string) fields.

**Validates: Requirements 5.1**

### Property 16: Auth Error Response

*For any* request with invalid JWT, the API SHALL return HTTP 401. *For any* request where user lacks required permission, the API SHALL return HTTP 403.

**Validates: Requirements 5.4, 5.5**

### Property 17: SQL Injection Prevention

*For any* query parameter or request body field containing SQL injection patterns (e.g., `'; DROP TABLE`, `OR 1=1`), the API SHALL either sanitize the input or reject the request, never executing the malicious SQL.

**Validates: Requirements 6.2**

### Property 18: Cross-Tenant Isolation

*For any* authenticated request, the API SHALL only return data belonging to the user's current organization. Attempting to access another organization's data SHALL return 404 or 403.

**Validates: Requirements 6.3**

### Property 19: Concurrent Transaction Balance Integrity

*For any* set of concurrent transactions on the same account, the final account balance SHALL equal the initial balance plus the sum of all transaction amounts, regardless of execution order.

**Validates: Requirements 6.5**

## Testing Strategy

### Property-Based Testing

The following properties will be tested using `proptest` crate with minimum 100 iterations per property:

| Property | Test File | Generator Strategy |
|----------|-----------|-------------------|
| 1 | `storage_props.rs` | Generate random upload/download requests |
| 2 | `attachment_props.rs` | Generate random MIME types (valid and invalid) |
| 3 | `attachment_props.rs` | Generate random file sizes around boundary |
| 4 | `attachment_props.rs` | Generate random org_id, transaction_id, filename |
| 5 | `exchange_rate_props.rs` | Generate batches with valid and invalid rates |
| 6 | `exchange_rate_props.rs` | Generate duplicate rate entries |
| 7 | `exchange_rate_props.rs` | Mock API failures and verify DB unchanged |
| 8 | `master_data_props.rs` | Generate toggle sequences |
| 9 | `master_data_props.rs` | Generate entries for deactivated entities |
| 10-13 | `dashboard_props.rs` | Generate ledger entries and verify calculations |
| 14 | `pagination_props.rs` | Generate activity items and verify cursor behavior |
| 15-16 | `error_props.rs` | Generate various error scenarios |
| 17 | `security_props.rs` | Generate SQL injection payloads |
| 18 | `rls_props.rs` | Generate cross-tenant access attempts |
| 19 | `concurrency_props.rs` | Generate concurrent transaction sets |

### Unit Tests

Unit tests will cover:
- Storage service configuration parsing
- MIME type validation logic
- File size validation logic
- Storage key generation
- Exchange rate parsing from Frankfurter response
- Dashboard calculation helpers
- Error response formatting
- JWT validation edge cases

### Integration Tests

Integration tests will cover:
- Full attachment upload/download flow with mock storage
- Exchange rate fetch and store flow
- Dashboard metrics with seeded data
- Rate limiting behavior
- End-to-end API error handling

### Load Testing

Using k6 for load testing:
- 100 concurrent users
- Target: p95 latency < 500ms
- Duration: 5 minutes sustained load
- Scenarios: mixed read/write operations

