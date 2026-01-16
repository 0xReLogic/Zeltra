# Requirements Document

## Introduction

This document specifies the requirements for the Dashboard Overview feature in Zeltra, a B2B expense and budgeting platform. The Dashboard provides a real-time financial overview including cash position, burn rate, runway metrics, cash flow visualization, recent activity feed, and budget vs actual comparison.

## Glossary

- **Dashboard**: The main overview page showing key financial metrics and activity
- **Cash_Position**: Current total cash balance across all asset accounts
- **Burn_Rate**: Rate of cash expenditure (daily and monthly)
- **Runway**: Number of days until cash runs out based on current burn rate
- **Cash_Flow_Chart**: Visual representation of inflows and outflows over time
- **Recent_Activity**: Feed of recent transactions and system events
- **Budget_vs_Actual**: Comparison of budgeted amounts vs actual spending
- **Metrics_Card**: UI component displaying a single KPI metric
- **Period**: Fiscal period for filtering dashboard data

## Requirements

### Requirement 1: Dashboard Metrics Display

**User Story:** As a finance manager, I want to see key financial metrics at a glance, so that I can quickly assess the company's financial health.

#### Acceptance Criteria

1. WHEN the Dashboard page loads, THE Dashboard SHALL display a Cash Position card showing current balance and percentage change from last month
2. WHEN the Dashboard page loads, THE Dashboard SHALL display a Monthly Burn Rate card showing monthly and daily burn amounts
3. WHEN the Dashboard page loads, THE Dashboard SHALL display a Runway card showing estimated days until cash depletion
4. WHEN the Dashboard page loads, THE Dashboard SHALL display a Pending Approvals card showing count and total value of pending transactions
5. IF the metrics API returns an error, THEN THE Dashboard SHALL display appropriate error state with retry option

### Requirement 2: Cash Flow Visualization

**User Story:** As a finance manager, I want to see cash flow trends over time, so that I can identify patterns in income and expenses.

#### Acceptance Criteria

1. WHEN the Dashboard page loads, THE Cash_Flow_Chart SHALL display inflows and outflows for the past 6 months by default
2. WHEN cash flow data is received, THE Cash_Flow_Chart SHALL correctly parse string amounts to numbers for chart rendering
3. WHEN the chart container is resized, THE Cash_Flow_Chart SHALL responsively adjust to the new dimensions
4. IF the cash flow API returns empty data, THEN THE Cash_Flow_Chart SHALL display an empty state message
5. WHEN hovering over chart data points, THE Cash_Flow_Chart SHALL display a tooltip with exact values

### Requirement 3: Recent Activity Feed

**User Story:** As a finance manager, I want to see recent financial activity, so that I can stay informed about transactions and changes.

#### Acceptance Criteria

1. WHEN the Dashboard page loads, THE Recent_Activity feed SHALL display the 10 most recent activities
2. WHEN an activity item is displayed, THE Recent_Activity feed SHALL show user name, action type, description, amount, and timestamp
3. WHEN an activity item is clicked, THE Dashboard SHALL navigate to the relevant entity detail page
4. WHEN activity type is 'approved', THE Recent_Activity feed SHALL display a green checkmark icon
5. WHEN activity type is 'rejected' or 'voided', THE Recent_Activity feed SHALL display a red X icon
6. IF the recent activity API returns an error, THEN THE Recent_Activity feed SHALL display appropriate error state

### Requirement 4: Budget vs Actual Widget

**User Story:** As a finance manager, I want to compare budgeted vs actual spending, so that I can track budget adherence.

#### Acceptance Criteria

1. WHEN the Dashboard page loads, THE Dashboard SHALL display a Budget vs Actual widget
2. WHEN budget data is available, THE Budget_vs_Actual widget SHALL show budget name, period, and overall status
3. WHEN budget data is available, THE Budget_vs_Actual widget SHALL display top budget line items with spent vs budgeted amounts
4. WHEN a line item exceeds budget, THE Budget_vs_Actual widget SHALL highlight it with a warning indicator
5. IF no active budget exists, THEN THE Budget_vs_Actual widget SHALL display an empty state with link to create budget

### Requirement 5: Type Alignment with Backend

**User Story:** As a developer, I want frontend types to match backend responses, so that data is correctly parsed and displayed.

#### Acceptance Criteria

1. THE Dashboard types SHALL match the OpenAPI schema definitions exactly
2. WHEN CashFlowDataPoint is received, THE Dashboard SHALL parse inflow, outflow, and net fields as strings (matching backend Decimal format)
3. WHEN ActivityItemResponse is received, THE Dashboard SHALL correctly map activity_type to type field
4. THE Dashboard query hooks SHALL use proper TypeScript types from api.generated.ts

### Requirement 6: OpenAPI Specification Alignment

**User Story:** As a developer, I want the OpenAPI spec to accurately reflect the API contract, so that generated types are correct.

#### Acceptance Criteria

1. THE OpenAPI spec SHALL define dashboard endpoint parameters with correct location (query vs path)
2. THE OpenAPI spec SHALL mark optional parameters as required: false
3. THE OpenAPI spec SHALL not use nullable array types like [string, 'null'] for optional params
4. WHEN OpenAPI spec is updated, THE frontend types SHALL be regenerated using openapi-typescript
