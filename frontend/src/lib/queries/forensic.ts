import { useQuery } from '@tanstack/react-query'
import { apiClient } from '../api/client'

// Query keys for cache management
const FORENSIC_KEYS = {
  all: ['forensic'] as const,
  reconciliation: () => [...FORENSIC_KEYS.all, 'reconciliation'] as const,
  benford: () => [...FORENSIC_KEYS.all, 'benford'] as const,
  healthScore: () => [...FORENSIC_KEYS.all, 'health-score'] as const,
}

// Response types
interface AccountDiscrepancy {
  account_id: string
  account_code: string
  account_name: string
  stored_balance: string
  calculated_balance: string
  difference: string
  status: 'Matched' | 'WithinTolerance' | 'Discrepancy'
}

interface BenfordRecord {
  digit: number
  actual_percentage: number
  expected_percentage: number
  difference: number
}

interface BenfordResponse {
  distribution_1st_digit: BenfordRecord[]
  distribution_2nd_digit: BenfordRecord[]
  mad_score: number
  mad_verdict: string
}

interface AltmanDetails {
  x1_working_capital: number
  x2_retained_earnings: number
  x3_ebit: number
  x4_equity: number
  x5_sales: number
}

interface BeneishDetails {
  dsri: number
  gmi: number
  aqi: number
  sgi: number
  depi: number
  sgai: number
  lvgi: number
  tata: number
}

interface HealthScoreResponse {
  z_score: number
  z_zone: string
  z_details: AltmanDetails
  m_score: number
  m_risk_level: string
  m_prob: number
  m_details: BeneishDetails
}

interface ReconciliationResponse {
  organization_id: string
  run_at: string
  total_accounts: number
  matched_count: number
  within_tolerance_count: number
  discrepancy_count: number
  is_clean: boolean
  accounts: AccountDiscrepancy[]
}

/**
 * GET /organizations/{org_id}/forensic/reconciliation
 * Run balance reconciliation (Enterprise only)
 */
export function useReconciliation() {
  return useQuery({
    queryKey: FORENSIC_KEYS.reconciliation(),
    queryFn: () => apiClient<ReconciliationResponse>('/forensic/reconciliation'),
  })
}

/**
 * GET /organizations/{org_id}/forensic/benford
 * Run Benford's Law analysis (Enterprise only)
 */
export function useBenford() {
  return useQuery({
    queryKey: FORENSIC_KEYS.benford(),
    queryFn: () => apiClient<BenfordResponse>('/forensic/benford'),
  })
}

/**
 * GET /organizations/{org_id}/forensic/health-score
 * Run Financial Health checks (Enterprise only)
 */
export function useHealthScore() {
  return useQuery({
    queryKey: FORENSIC_KEYS.healthScore(),
    queryFn: () => apiClient<HealthScoreResponse>('/forensic/health-score'),
  })
}

export type { ReconciliationResponse, AccountDiscrepancy, BenfordResponse, BenfordRecord, HealthScoreResponse }
