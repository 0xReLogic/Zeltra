# Requirements: Master Data UI

## Overview
Master Data module provides configuration and base data settings for the accounting system, including Chart of Accounts, Fiscal Periods, Dimensions, and Exchange Rates.

## Functional Requirements

### REQ-1: Master Data Hub Page
- **REQ-1.1**: Display hub page at `/dashboard/master-data` with navigation cards
- **REQ-1.2**: Show 4 main sections: Chart of Accounts, Fiscal Periods, Dimensions, Exchange Rates
- **REQ-1.3**: Each card should link to respective detail page
- **REQ-1.4**: Cards should have icons, titles, descriptions, and "Open setting" action

### REQ-2: Fiscal Periods Management
- **REQ-2.1**: Display fiscal years with expandable periods at `/dashboard/master-data/fiscal-periods`
- **REQ-2.2**: Allow creating new fiscal year with auto-generated monthly periods
- **REQ-2.3**: Support optional adjustment period (Period 13)
- **REQ-2.4**: Allow changing period status: Open, Soft Close, Closed
- **REQ-2.5**: Enforce sequential closing (earlier periods must be closed first)
- **REQ-2.6**: Show loading spinner during data fetch

### REQ-3: Dimensions Management
- **REQ-3.1**: Display dimension types as tabs at `/dashboard/master-data/dimensions`
- **REQ-3.2**: Allow creating new dimension types (code, name, description)
- **REQ-3.3**: Allow creating dimension values within each type
- **REQ-3.4**: Support tier gating for dimension limits (Starter: 2, Growth: 10, Enterprise: 100)
- **REQ-3.5**: Show dimension values in table with Code, Name, Description columns

### REQ-4: Exchange Rates Management
- **REQ-4.1**: Display exchange rate history at `/dashboard/master-data/exchange-rates`
- **REQ-4.2**: Allow manual rate entry with From/To currency, Rate, Date
- **REQ-4.3**: Support bulk import of rates
- **REQ-4.4**: Support syncing live rates from external API (Frankfurter)
- **REQ-4.5**: Show rate history sorted by date (newest first)

## Non-Functional Requirements

### REQ-5: UI/UX Standards
- **REQ-5.1**: Follow shadcn/ui design patterns
- **REQ-5.2**: Use Loader2 spinner for loading states
- **REQ-5.3**: Show toast notifications for success/error
- **REQ-5.4**: Support keyboard navigation (Escape to close dialogs)
- **REQ-5.5**: Responsive design for mobile viewports

### REQ-6: Type Safety
- **REQ-6.1**: All types must match OpenAPI spec exactly
- **REQ-6.2**: Use proper type converters for API communication
- **REQ-6.3**: No TypeScript errors in production build
