/**
 * Budget types - re-exported from OpenAPI generated types
 */

import type {
  BudgetResponse,
  BudgetSummary,
  BudgetLineItemResponse,
  BudgetLineInput,
  CreateBudgetRequest as ApiCreateBudgetRequest,
  CreateBudgetLinesRequest,
  UpdateBudgetRequest,
  BudgetVsActualResponse,
} from './api-helpers'

// Re-export OpenAPI types
export type {
  BudgetResponse,
  BudgetSummary,
  BudgetLineItemResponse,
  BudgetLineInput,
  CreateBudgetLinesRequest,
  UpdateBudgetRequest,
  BudgetVsActualResponse,
}

// Budget type alias
export type Budget = BudgetResponse

// Budget with lines (from GET /budgets/{id})
export interface BudgetWithLines extends BudgetResponse {
  lines?: BudgetLineItemResponse[]
}

// Create budget request
export type CreateBudgetRequest = ApiCreateBudgetRequest

// Budget type enum
export type BudgetType = 'annual' | 'quarterly' | 'monthly' | 'project'

// Backend returns array directly
export type GetBudgetsResponse = BudgetResponse[]
