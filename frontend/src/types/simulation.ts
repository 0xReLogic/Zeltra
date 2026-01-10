export interface SimulationRequest {
  base_period_start: string
  projection_months: number
  revenue_growth_rate: string
  expense_growth_rate: string
  account_adjustments?: Record<string, string>
  dimension_filters?: string[]
}

export interface MonthlySummary {
  month: string
  period_name: string
  revenue: string
  expenses: string
  net_income: string
}

export interface AnnualSummary {
  total_projected_revenue: string
  total_projected_expenses: string
  projected_net_income: string
  net_profit_margin: string
}

export interface AccountProjection {
  period_name: string
  period_start: string
  period_end: string
  account_id: string
  account_code: string
  account_name: string
  account_type: string
  baseline_amount: string
  projected_amount: string
  change_percent: string
  revenue: string
  expenses: string
  net_income: string
  month: string
}

export interface SimulationResult {
  simulation_id: string
  parameters_hash: string
  cached: boolean
  projections: AccountProjection[]
  annual_summary: AnnualSummary
  monthly_summary: MonthlySummary[]
}
