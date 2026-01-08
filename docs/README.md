# Zeltra Documentation

High-Performance B2B Expense & Budgeting Engine built with Rust + Next.js.

## Quick Links

- [Architecture](./ARCHITECTURE.md) - System design, tech stack, folder structure
- [Database Schema](./DATABASE_SCHEMA.md) - Multi-tenancy, double-entry ledger, constraints
- [API Specification](./API_SPEC.md) - Endpoints, request/response contracts
- [Core Features](./FEATURES.md) - Ledger service, simulation engine
- [Business Model](./BUSINESS_MODEL.md) - Pricing tiers, target market
- [Roadmap](./ROADMAP.md) - Development phases, timeline

## Core Principles

1. Speed through Rust - Zero-cost abstractions, memory safety, fearless concurrency
2. Data Integrity through Double-Entry - Every transaction balanced, audit-ready
3. Enterprise-Ready - Self-hosted capable, strict data isolation

## Tech Stack Summary

| Layer | Technology |
|-------|------------|
| Backend | Rust 1.92+ (Axum 0.8.8, SeaORM 1.1, Tokio 1.49) |
| Frontend | Next.js 16 (App Router, RSC, Server Actions, Turbopack) |
| Database | PostgreSQL 16 |
| Styling | Tailwind CSS v4, Shadcn/UI |
| State | TanStack Query v5, Zustand |
| Validation | Zod (Frontend), Rust types (Backend) |
