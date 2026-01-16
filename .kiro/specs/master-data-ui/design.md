# Design: Master Data UI

## Architecture

### Page Structure
```
/dashboard/master-data/           → Hub page with navigation cards
/dashboard/master-data/fiscal-periods/  → Fiscal years & periods management
/dashboard/master-data/dimensions/      → Dimension types & values management
/dashboard/master-data/exchange-rates/  → Exchange rate management
```

### Component Hierarchy

#### Hub Page
- `MasterDataPage` - Main hub with navigation cards
  - Card: Chart of Accounts → links to `/dashboard/accounts`
  - Card: Fiscal Periods → links to `/dashboard/master-data/fiscal-periods`
  - Card: Dimensions → links to `/dashboard/master-data/dimensions`
  - Card: Exchange Rates → links to `/dashboard/master-data/exchange-rates`

#### Fiscal Periods Page
- `FiscalPeriodsPage` - Main page component
  - `CreateFiscalYearDialog` - Dialog for creating new fiscal year
  - `FiscalYearsTable` - Expandable table showing years and periods
    - Period status dropdown (Open/Soft Close/Close)

#### Dimensions Page
- `DimensionsPage` - Main page with tabs
  - `DimensionTypeDialog` - Dialog for creating dimension types
  - `DimensionValues` - Tab content showing values for each type
    - `CreateValueDialog` - Dialog for creating dimension values

#### Exchange Rates Page
- `ExchangeRatesPage` - Main page component
  - `AddRateDialog` - Dialog for manual rate entry
  - `BulkImportDialog` - Dialog for CSV bulk import
  - `RateHistoryTable` - Table showing rate history

## API Integration

### Query Hooks (TanStack Query)
```typescript
// Fiscal
useFiscalYears()           → GET /fiscal-years
useCreateFiscalYear()      → POST /fiscal-years
useUpdatePeriodStatus()    → PATCH /fiscal-periods/{id}/status

// Dimensions
useDimensions()            → GET /dimensions
useCreateDimensionType()   → POST /dimensions/types
useCreateDimensionValue()  → POST /dimensions/values

// Exchange Rates
useExchangeRates()         → GET /exchange-rates
useCreateExchangeRate()    → POST /exchange-rates
useBulkImportRates()       → POST /exchange-rates/bulk
useFetchLiveRates()        → POST /exchange-rates/fetch
```

### Type Alignment

#### Period Status
- Frontend display: `'Open' | 'SoftClose' | 'Closed'`
- Backend API: `'open' | 'soft_close' | 'closed'`
- Converter: `toBackendStatus()` in `types/fiscal.ts`

## UI Components Used

### From shadcn/ui
- Card, CardHeader, CardContent, CardTitle, CardDescription
- Table, TableHeader, TableBody, TableRow, TableCell
- Dialog, DialogContent, DialogHeader, DialogTitle
- Button, Input, Label, Checkbox
- Badge (with custom color variants)
- DropdownMenu, DropdownMenuContent, DropdownMenuItem
- Tabs, TabsList, TabsTrigger, TabsContent
- Select, SelectContent, SelectItem
- Form (react-hook-form integration)

### Icons (Lucide)
- CalendarRange, Layers, RefreshCw, BookOpen
- Plus, Loader2, ChevronDown, ChevronRight
- Lock, Unlock, Archive, MoreHorizontal

## State Management

### Loading States
- Use `Loader2` spinner component with `animate-spin`
- Center in container with `flex items-center justify-center`

### Error Handling
- Display toast notifications via `sonner`
- Show API error messages to user
- Graceful fallback for empty data

### Cache Invalidation
- Invalidate related queries on mutation success
- Use query key patterns for targeted invalidation
