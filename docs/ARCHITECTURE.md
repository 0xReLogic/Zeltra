# Architecture

## System Overview

```
┌─────────────────┐     REST API      ┌─────────────────┐
│   Next.js 16    │◄────────────────►│   Rust (Axum)   │
│   (Frontend)    │                   │   (Backend)     │
└─────────────────┘                   └────────┬────────┘
                                               │
                                               │ SQLx
                                               ▼
                                      ┌─────────────────┐
                                      │   PostgreSQL    │
                                      └─────────────────┘
```

Architecture Pattern: Modular Monolith

Rationale: Simpler deployment, easier debugging, can be split into microservices later if needed.

## Tech Stack Decisions

### Backend (Rust 1.92+)

| Component | Choice | Rationale |
|-----------|--------|-----------|
| Framework | Axum 0.8.8 | Tokio ecosystem, ergonomic extractors, tower middleware, better DX than Actix |
| Database | SeaORM 2.0 | Async ORM, entity generation, migrations, new entity format, nested ActiveModel |
| Raw Queries | SQLx (optional) | For complex queries where ORM is overkill |
| Decimal | rust_decimal | Arbitrary precision, no floating-point errors for money |
| Serialization | Serde | Industry standard |
| Async Runtime | Tokio | De facto standard for async Rust |
| Error Handling | thiserror + anyhow | Type-safe errors for library, anyhow for application |

Note: SeaORM 2.0 dipilih (released Sep 2025) karena:
- New concise entity format - easier to write by hand
- Strongly-typed columns (bye-bye CamelCase)
- Nested ActiveModel - persist nested objects in one operation
- Smart Entity Loader - efficient multi-path relation queries
- Synchronous mode support for CLI tools
- Masih bisa fallback ke raw SQL kalau perlu

### Frontend (Next.js 16)

| Component | Choice | Rationale |
|-----------|--------|-----------|
| Framework | Next.js 16 | Turbopack default, improved caching APIs, stable RSC |
| Router | App Router | RSC support, layouts, streaming, PPR (Partial Prerendering) |
| Data Fetching | TanStack Query v5 | Caching, optimistic updates, offline persistence |
| Client State | Zustand | Lightweight, no boilerplate, works well with TanStack Query |
| UI Components | Shadcn/UI | Accessible, Radix primitives, copy-paste ownership |
| Styling | Tailwind CSS v4 | Utility-first, CSS-first config, faster builds |
| Validation | Zod | TypeScript-first, runtime validation, form integration |
| React | React 19 | useActionState, useOptimistic, React Compiler support |

### Authentication

| Deployment | Choice | Rationale |
|------------|--------|-----------|
| Both | Custom JWT + Session | Full control, self-hosted, database-backed sessions |
| Password Hashing | Argon2id | Memory-hard, recommended by OWASP |
| Token | JWT (short-lived) + Refresh Token | Stateless auth with revocation capability |

Decision: Custom auth implementation karena:
- Lucia Auth deprecated/maintenance mode
- Full control over session management
- Enterprise clients butuh custom auth flows (SSO, LDAP integration later)
- Database-backed sessions untuk audit trail

### Communication

Protocol: REST API with JSON

gRPC rejected untuk fase awal karena:
- Client hanya Next.js (browser), gRPC-web adds complexity
- REST sufficient untuk current requirements
- Bisa ditambahkan later untuk internal service communication

Alternative Consideration: TanStack Start
- Jika tim lebih prefer type-safe end-to-end tanpa RSC complexity
- Vite-based, faster dev builds
- Tapi Next.js 16 dipilih karena ecosystem maturity dan Vercel deployment simplicity

## Rust Project Structure

```
zeltra/
├── Cargo.toml              # Workspace root (GOD TIER config below)
├── rust-toolchain.toml     # Pin Rust version
├── .cargo/
│   └── config.toml         # Cargo settings (rustflags, etc)
│
├── crates/
│   ├── api/                # Axum HTTP handlers, routes
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── routes/
│   │       │   ├── mod.rs
│   │       │   ├── auth.rs
│   │       │   ├── accounts.rs
│   │       │   ├── transactions.rs
│   │       │   ├── budgets.rs
│   │       │   ├── reports.rs
│   │       │   └── simulation.rs
│   │       ├── middleware/
│   │       │   ├── mod.rs
│   │       │   ├── auth.rs
│   │       │   └── tenant.rs   # RLS context setter
│   │       └── extractors/
│   │           ├── mod.rs
│   │           └── claims.rs   # JWT claims extractor
│   │
│   ├── core/               # Business logic (ZERO external deps)
│   │   ├── Cargo.toml      # Only rust_decimal, chrono, thiserror
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── ledger/
│   │       │   ├── mod.rs
│   │       │   ├── entry.rs        # LedgerEntry domain type
│   │       │   ├── transaction.rs  # Transaction aggregate
│   │       │   ├── balance.rs      # Balance calculations
│   │       │   └── validation.rs   # Business rules
│   │       ├── currency/
│   │       │   ├── mod.rs
│   │       │   ├── exchange.rs     # Exchange rate logic
│   │       │   └── conversion.rs   # Multi-currency conversion
│   │       ├── dimension/
│   │       │   ├── mod.rs
│   │       │   └── filter.rs       # Dimensional filtering
│   │       ├── fiscal/
│   │       │   ├── mod.rs
│   │       │   └── period.rs       # Fiscal period rules
│   │       ├── simulation/
│   │       │   ├── mod.rs
│   │       │   ├── engine.rs       # Projection engine
│   │       │   └── scenario.rs     # What-if scenarios
│   │       └── budget/
│   │           ├── mod.rs
│   │           └── variance.rs     # Budget vs actual
│   │
│   ├── db/                 # SeaORM entities, migrations
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── entities/       # Generated by sea-orm-cli
│   │       │   ├── mod.rs
│   │       │   ├── prelude.rs
│   │       │   ├── user.rs
│   │       │   ├── organization.rs
│   │       │   ├── account.rs
│   │       │   ├── transaction.rs
│   │       │   ├── ledger_entry.rs
│   │       │   └── ...
│   │       ├── repositories/   # Query abstractions
│   │       │   ├── mod.rs
│   │       │   ├── ledger.rs
│   │       │   ├── account.rs
│   │       │   └── report.rs
│   │       └── migration/      # sea-orm-migration
│   │           ├── mod.rs
│   │           └── m20260107_000001_initial.rs
│   │
│   └── shared/             # Common types, errors
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── types/
│           │   ├── mod.rs
│           │   ├── money.rs        # Decimal wrapper with currency
│           │   ├── pagination.rs   # PageRequest, PageResponse
│           │   └── id.rs           # Typed IDs (UserId, OrgId, etc)
│           ├── error.rs            # AppError enum
│           └── config.rs           # Config struct
│
├── bins/
│   └── server/             # Main binary
│       ├── Cargo.toml
│       └── src/
│           └── main.rs
│
├── tests/                  # Integration tests
│   ├── common/
│   │   └── mod.rs          # Test helpers, fixtures
│   ├── ledger_tests.rs
│   ├── api_tests.rs
│   └── stress_tests.rs
│
└── config/
    ├── default.toml
    ├── development.toml
    └── production.toml
```

### Workspace Cargo.toml (GOD TIER)

```toml
[workspace]
resolver = "2"
members = ["crates/*", "bins/*"]

[workspace.package]
version = "0.1.0"
authors = ["Zeltra Team"]
edition = "2024"
license = "MIT OR Apache-2.0"
rust-version = "1.92"

# ============================================================================
# CENTRALIZED DEPENDENCIES - Single source of truth for versions
# ============================================================================
[workspace.dependencies]
# === Web Framework ===
axum = { version = "0.8.8", features = ["macros", "ws", "multipart"] }
axum-extra = { version = "0.10", features = ["typed-header", "cookie"] }
tower = { version = "0.5", features = ["full"] }
tower-http = { version = "0.6", features = [
    "fs", "cors", "compression-gzip", "trace", 
    "timeout", "limit", "request-id", "sensitive-headers"
]}
hyper = { version = "1.6", features = ["full"] }
hyper-util = "0.1"

# === Async Runtime ===
tokio = { version = "1.49", features = ["full"] }
tokio-util = { version = "0.7", features = ["io"] }
futures = "0.3"

# === Database (SeaORM 2.0 - Released Sep 2025) ===
sea-orm = { version = "2.0", features = [
    "sqlx-postgres", 
    "runtime-tokio-rustls",
    "macros",
    "with-chrono",
    "with-json",
    "with-uuid"
]}
sea-orm-migration = "2.0"
sqlx = { version = "0.8", features = [
    "runtime-tokio-rustls",
    "postgres",
    "uuid",
    "chrono",
    "json"
]}

# === Serialization ===
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_with = "3.12"

# === Money & Time (CRITICAL - no floats!) ===
rust_decimal = { version = "1.37", features = ["serde", "serde-with-str"] }
chrono = { version = "0.4", features = ["serde"] }

# === Validation ===
validator = { version = "0.20", features = ["derive"] }
garde = { version = "0.22", features = ["derive", "email", "url"] }

# === Error Handling ===
thiserror = "2.0"
anyhow = "1.0"

# === Logging & Tracing ===
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json", "fmt"] }

# === Security & Auth ===
argon2 = "0.5"
jsonwebtoken = "9.3"
uuid = { version = "1.11", features = ["v4", "v7", "serde"] }

# === Configuration ===
config = { version = "0.15", features = ["toml"] }
dotenvy = "0.15"

# === HTTP Client (for exchange rates, etc) ===
reqwest = { version = "0.12", default-features = false, features = [
    "rustls-tls", "json", "gzip"
]}

# === Parallel Processing (for simulation) ===
rayon = "1.10"

# === Utilities ===
bytes = "1.9"
once_cell = "1.20"
dashmap = "6.1"
pin-project = "1.1"

# ============================================================================
# DEV DEPENDENCIES
# ============================================================================
[workspace.dependencies.fake]
version = "3.0"
features = ["derive"]

[workspace.dependencies.mockall]
version = "0.13"

[workspace.dependencies.rstest]
version = "0.24"

[workspace.dependencies.testcontainers]
version = "0.24"
features = ["postgres"]

[workspace.dependencies.criterion]
version = "0.6"
features = ["html_reports", "async_tokio"]

# ============================================================================
# WORKSPACE LINTS - Consistent across all crates
# ============================================================================
[workspace.lints.rust]
unsafe_code = "deny"
missing_docs = "warn"

[workspace.lints.clippy]
all = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }
# Allow these (too noisy)
module_name_repetitions = "allow"
must_use_candidate = "allow"
missing_errors_doc = "allow"
# CRITICAL for financial app
float_arithmetic = "deny"
float_cmp = "deny"
float_cmp_const = "deny"

# ============================================================================
# BUILD PROFILES
# ============================================================================
[profile.dev]
opt-level = 0

[profile.dev.package."*"]
opt-level = 3  # Optimize deps in dev for faster runtime

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
strip = true
panic = "abort"

[profile.release-debug]
inherits = "release"
strip = false
debug = true
```

### Individual Crate Cargo.toml Examples

**crates/core/Cargo.toml** (ZERO external deps except essentials):
```toml
[package]
name = "zeltra-core"
version.workspace = true
edition.workspace = true
rust-version.workspace = true

[dependencies]
rust_decimal = { workspace = true }
chrono = { workspace = true }
thiserror = { workspace = true }
serde = { workspace = true }

# NO database deps, NO web deps - pure business logic

[lints]
workspace = true
```

**crates/api/Cargo.toml**:
```toml
[package]
name = "zeltra-api"
version.workspace = true
edition.workspace = true

[dependencies]
# Internal crates
zeltra-core = { path = "../core" }
zeltra-db = { path = "../db" }
zeltra-shared = { path = "../shared" }

# Web
axum = { workspace = true }
axum-extra = { workspace = true }
tower = { workspace = true }
tower-http = { workspace = true }
tokio = { workspace = true }

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }

# Auth
jsonwebtoken = { workspace = true }
argon2 = { workspace = true }

# Utils
tracing = { workspace = true }
uuid = { workspace = true }

[dev-dependencies]
rstest = { workspace = true }
fake = { workspace = true }

[lints]
workspace = true
```

**crates/db/Cargo.toml**:
```toml
[package]
name = "zeltra-db"
version.workspace = true
edition.workspace = true

[dependencies]
zeltra-shared = { path = "../shared" }

sea-orm = { workspace = true }
sea-orm-migration = { workspace = true }
sqlx = { workspace = true }

tokio = { workspace = true }
chrono = { workspace = true }
uuid = { workspace = true }
rust_decimal = { workspace = true }
serde = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }

[dev-dependencies]
testcontainers = { workspace = true }

[lints]
workspace = true
```

### rust-toolchain.toml

```toml
[toolchain]
channel = "1.92"
components = ["rustfmt", "clippy", "rust-analyzer"]
```

### .cargo/config.toml

```toml
[build]
rustflags = [
    "-D", "warnings",
    "-D", "clippy::float_arithmetic",
]

[target.x86_64-unknown-linux-gnu]
rustflags = ["-C", "target-cpu=native"]

[alias]
t = "test"
c = "clippy"
b = "build --release"
```

### Dependency Flow

```
┌─────────────────────────────────────────────────────────────┐
│                        bins/server                          │
│                      (main.rs entry)                        │
└─────────────────────────────┬───────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                        crates/api                           │
│              (Axum routes, middleware, extractors)          │
└──────────┬──────────────────┬──────────────────┬────────────┘
           │                  │                  │
           ▼                  ▼                  ▼
┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐
│   crates/core    │  │    crates/db     │  │  crates/shared   │
│  (Business Logic)│  │ (SeaORM, Repos)  │  │ (Types, Errors)  │
│   ZERO web deps  │  │                  │  │                  │
└────────┬─────────┘  └────────┬─────────┘  └────────┬─────────┘
         │                     │                     │
         └─────────────────────┼─────────────────────┘
                               │
                               ▼
                    ┌──────────────────┐
                    │  crates/shared   │
                    │  (Money, IDs)    │
                    └──────────────────┘
```

Workspace vs Monolith Decision: **Workspace**

Rationale:
- Faster incremental compilation (only recompile changed crates)
- Clear dependency boundaries enforced by Cargo
- `core` crate has ZERO web/db deps (pure business logic, easy to test)
- Clippy lints centralized - `float_arithmetic = "deny"` across all crates
- Version pinning in one place - no dependency hell

## Next.js Project Structure

```
web/
├── package.json
├── next.config.ts
├── tailwind.config.ts      # Tailwind v4 CSS-first config
├── tsconfig.json
│
├── src/
│   ├── app/
│   │   ├── layout.tsx
│   │   ├── page.tsx
│   │   ├── (auth)/
│   │   │   ├── login/
│   │   │   │   └── page.tsx
│   │   │   └── register/
│   │   │       └── page.tsx
│   │   │
│   │   └── (dashboard)/
│   │       ├── layout.tsx
│   │       ├── page.tsx
│   │       ├── accounts/
│   │       │   └── page.tsx
│   │       ├── transactions/
│   │       │   ├── page.tsx
│   │       │   └── [id]/
│   │       │       └── page.tsx
│   │       ├── budgets/
│   │       │   └── page.tsx
│   │       ├── simulation/
│   │       │   └── page.tsx
│   │       └── approvals/
│   │           └── page.tsx
│   │
│   ├── components/
│   │   ├── ui/             # Shadcn components (Radix primitives)
│   │   ├── forms/
│   │   ├── charts/
│   │   └── layouts/
│   │
│   ├── lib/
│   │   ├── api/            # API client, typed fetch wrappers
│   │   ├── queries/        # TanStack Query hooks
│   │   ├── stores/         # Zustand stores (client state only)
│   │   ├── utils/
│   │   └── validations/    # Zod schemas (shared with API types)
│   │
│   ├── actions/            # Server Actions
│   │   ├── auth.ts
│   │   └── transactions.ts
│   │
│   └── types/
│       └── index.ts        # Shared TypeScript types
│
└── public/
```

## Monorepo Structure (Combined)

```
zeltra/
├── apps/
│   └── web/                # Next.js frontend
│
├── packages/
│   └── rust-backend/       # Rust workspace (structure above)
│
├── docker/
│   ├── Dockerfile.api
│   ├── Dockerfile.web
│   └── docker-compose.yml
│
├── .github/
│   └── workflows/
│       └── ci.yml
│
└── README.md
```

## Infrastructure

### Cloud Credits (Start Date: 07-01-2026)

| Provider | Credit | Duration | Expiry | Usage |
|----------|--------|----------|--------|-------|
| Heroku | $336 | 2 years | 07-01-2028 | Postgres Database |
| DigitalOcean | $200 | 1 year | 07-01-2027 | Rust API Compute |
| Azure | $100 | 1 year | 07-01-2027 | Enterprise Demo |

### Production Stack

```
┌─────────────────────────────────────────────────────────────┐
│                        CLOUDFLARE                           │
│                   (DNS, CDN, DDoS Protection)               │
└─────────────────┬───────────────────────┬───────────────────┘
                  │                       │
                  ▼                       ▼
┌─────────────────────────┐   ┌─────────────────────────┐
│        VERCEL           │   │     DIGITALOCEAN        │
│    (Next.js Frontend)   │   │    (Rust API Server)    │
│                         │   │                         │
│  - App Router + RSC     │   │  - Docker Container     │
│  - Edge Functions       │   │  - Droplet $12-24/mo    │
│  - Free Tier OK         │   │  - Auto-scaling later   │
└─────────────────────────┘   └───────────┬─────────────┘
                                          │
                  ┌───────────────────────┼───────────────────────┐
                  │                       │                       │
                  ▼                       ▼                       ▼
┌─────────────────────────┐   ┌─────────────────────────┐   ┌─────────────────────────┐
│    HEROKU POSTGRES      │   │    CLOUDFLARE R2        │   │      SUPABASE           │
│      (Database)         │   │  (File Attachments)     │   │    (Alternative DB)     │
│                         │   │                         │   │                         │
│  - Managed Postgres     │   │  - S3-compatible        │   │  - Free 500MB           │
│  - Auto backups         │   │  - Zero egress fees     │   │  - Managed Postgres     │
│  - $336 credit          │   │  - Cheap storage        │   │  - Built-in Auth (skip) │
└─────────────────────────┘   └─────────────────────────┘   └─────────────────────────┘
```

### Environment Strategy

| Environment | Frontend | Backend | Database | Storage |
|-------------|----------|---------|----------|---------|
| Development | localhost:3000 | localhost:8080 | Docker Postgres | Local filesystem |
| Staging | Vercel Preview | DO Droplet | Heroku Postgres (dev) | Cloudflare R2 |
| Production | Vercel | DO Droplet | Heroku Postgres | Cloudflare R2 |
| Enterprise | Client infra | Client infra | Client Postgres | Client storage |

### Cost Projection (Post-Credits)

| Service | Monthly Cost | Notes |
|---------|--------------|-------|
| Vercel | $0-20 | Free tier generous, Pro if needed |
| DigitalOcean | $12-24 | Basic Droplet, scale as needed |
| Heroku Postgres | $9-50 | Mini to Standard-0 |
| Cloudflare R2 | ~$5 | $0.015/GB storage, zero egress |
| Cloudflare | $0 | Free tier sufficient |
| **Total** | **$26-99/mo** | After credits expire |
