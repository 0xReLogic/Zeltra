/**
 * Budget types - re-exported from OpenAPI generated types
 */

import type {
  BudgetResponse,
  BudgetSummary,
  BudgetLineItemResponse,
  BudgetLineInput,
  BudgetLineResponse,
  CreateBudgetRequest as ApiCreateBudgetRequest,
  CreateBudgetLinesRequest,
  UpdateBudgetRequest,
  BudgetVsActualResponse,
  GetBudgetsResponse as ApiGetBudgetsResponse,
  GetBudgetLinesResponse as ApiGetBudgetLinesResponse,
} from './api-helpers'

// Re-export OpenAPI types
export type {
  BudgetResponse,
  BudgetSummary,
  BudgetLineItemResponse,
  BudgetLineInput,
  BudgetLineResponse,
  CreateBudgetLinesRequest,
  UpdateBudgetRequest,
  BudgetVsActualResponse,
}

// Budget type alias
export type Budget = BudgetResponse

// Budget with lines (from GET /budgets/{id})
export interface BudgetWithLines extends BudgetResponse {
  lines?: BudgetLineResponse[]
}

// Create budget request
export type CreateBudgetRequest = ApiCreateBudgetRequest

// Budget type enum
export type BudgetType = 'annual' | 'quarterly' | 'monthly' | 'project'

// Response wrappers from API
export type GetBudgetsResponse = ApiGetBudgetsResponse
export type GetBudgetLinesResponse = ApiGetBudgetLinesResponse
