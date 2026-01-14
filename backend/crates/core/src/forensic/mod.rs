use crate::reports::types::BalanceSheetReport;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Benford's Law (Advanced)
// ============================================================================

/// Benford analysis result.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct BenfordAnalysis {
    /// First digit distribution.
    pub first_digit_distribution: Vec<BenfordRecord>,
    /// Second digit distribution.
    pub second_digit_distribution: Vec<BenfordRecord>,
    /// Mean Absolute Deviation (MAD) Score.
    #[schema(example = 0.002)]
    pub mad_score: f64,
    /// MAD Verdict (Conform, Nonconform).
    #[schema(example = "Conform")]
    pub mad_verdict: String,
}

/// Single digit record for Benford.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct BenfordRecord {
    /// The digit (0-9).
    #[schema(example = 1)]
    pub digit: u8,
    /// Actual frequency percentage (0.0 - 100.0).
    #[schema(example = 30.1)]
    pub actual_percentage: f64,
    /// Expected frequency percentage.
    #[schema(example = 30.1)]
    pub expected_percentage: f64,
    /// Difference.
    pub difference: f64,
}

// ============================================================================
// Altman Z-Score
// ============================================================================

/// Altman Z-Score result.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AltmanZScoreResult {
    /// The calculated Z-Score.
    #[schema(example = 4.2)]
    pub score: f64,
    /// The zone interpretation.
    pub zone: AltmanZone,
    /// Detailed breakdown of the 5 factors.
    pub details: AltmanDetails,
}

/// Altman Z-Score Zone.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, utoipa::ToSchema)]
pub enum AltmanZone {
    /// Score > 2.99: Safe Zone.
    Safe,
    /// 1.81 < Score < 2.99: Grey Zone.
    Grey,
    /// Score < 1.81: Distress Zone.
    Distress,
}

/// Details of the 5 Altman Z-Score factors.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AltmanDetails {
    /// X1: Working Capital / Total Assets.
    pub x1_working_capital: f64,
    /// X2: Retained Earnings / Total Assets.
    pub x2_retained_earnings: f64,
    /// X3: EBIT / Total Assets.
    pub x3_ebit: f64,
    /// X4: Market Value of Equity / Total Liabilities.
    pub x4_equity: f64,
    /// X5: Sales / Total Assets.
    pub x5_sales: f64,
}

// ============================================================================
// Beneish M-Score
// ============================================================================

/// Beneish M-Score result.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct BeneishMScoreResult {
    /// The calculated M-Score.
    #[schema(example = -2.5)]
    pub score: f64,
    /// Manipulation Probability (Standard Normal CDF).
    #[schema(example = 0.05)]
    pub manipulation_probability: f64,
    /// Risk Level (Safe / Risk).
    #[schema(example = "Safe")]
    pub risk_level: String,
    /// Detailed breakdown of the 8 variables.
    pub details: BeneishDetails,
}

/// Details of the 8 Beneish M-Score variables.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct BeneishDetails {
    pub dsri: f64, // Days Sales in Receivables Index
    pub gmi: f64,  // Gross Margin Index
    pub aqi: f64,  // Asset Quality Index
    pub sgi: f64,  // Sales Growth Index
    pub depi: f64, // Depreciation Index
    pub sgai: f64, // SGA Index
    pub lvgi: f64, // Leverage Index
    pub tata: f64, // Total Accruals to Total Assets
}

// ============================================================================
// Service
// ============================================================================

/// Service for running forensic accounting checks.
pub struct ForensicService;

impl ForensicService {
    /// Perform Advanced Benford's Law analysis (1st and 2nd Digit + MAD).
    pub fn calculate_benford_law(amounts: Vec<Decimal>) -> BenfordAnalysis {
        let mut first_digit_counts: HashMap<u8, u64> = HashMap::new();
        let mut second_digit_counts: HashMap<u8, u64> = HashMap::new();
        let mut total_count = 0;

        for amount in amounts {
            let abs_amount = amount.abs();
            if abs_amount.is_zero() {
                continue;
            }

            let s = abs_amount.to_string(); // e.g. "120.50"
            // Filter digits only, ignoring '.'
            let digits: Vec<u32> = s
                .chars()
                .filter(char::is_ascii_digit)
                .filter_map(|c| c.to_digit(10))
                .collect();

            // Skip leading zeros if any (Decimal usually handles this, but strictly speaking)
            // But to_string on Decimal "0.5" gives "0.5", leading char is '0'.
            // Benford applies to significand. We should skip leading '0's.
            let significant_digits: Vec<u32> = digits.into_iter().skip_while(|&d| d == 0).collect();

            if significant_digits.is_empty() {
                continue;
            }

            // 1st Digit
            let d1 = significant_digits[0] as u8;
            *first_digit_counts.entry(d1).or_insert(0) += 1;

            // 2nd Digit (if exists)
            if significant_digits.len() >= 2 {
                let d2 = significant_digits[1] as u8;
                *second_digit_counts.entry(d2).or_insert(0) += 1;
            }

            total_count += 1;
        }

        if total_count == 0 {
            return BenfordAnalysis {
                first_digit_distribution: vec![],
                second_digit_distribution: vec![],
                mad_score: 0.0,
                mad_verdict: "Insufficient Data".to_string(),
            };
        }

        // --- 1st Digit Analysis ---
        let mut first_dist = Vec::new();
        for digit in 1..=9 {
            let count = *first_digit_counts.get(&digit).unwrap_or(&0);
            let actual = (count as f64 / f64::from(total_count)) * 100.0;
            let expected = (1.0 + 1.0 / f64::from(digit)).log10() * 100.0;

            first_dist.push(BenfordRecord {
                digit,
                actual_percentage: actual,
                expected_percentage: expected,
                difference: (actual - expected).abs(),
            });
        }

        // --- 2nd Digit Analysis ---
        // P(d2) = sum_{k=1}^9 log10(1 + 1/(10k + d2))
        let mut second_dist = Vec::new();
        for digit in 0..=9 {
            let count = *second_digit_counts.get(&digit).unwrap_or(&0);
            let actual = (count as f64 / f64::from(total_count)) * 100.0;

            let mut expected_prob = 0.0;
            for k in 1..=9 {
                let val = 1.0 + (1.0 / (10.0 * f64::from(k) + f64::from(digit)));
                expected_prob += val.log10();
            }
            let expected = expected_prob * 100.0;

            second_dist.push(BenfordRecord {
                digit,
                actual_percentage: actual,
                expected_percentage: expected,
                difference: (actual - expected).abs(),
            });
        }

        // --- MAD Calculation ---
        // Mean Absolute Deviation on 1st Digit
        let sum_abs_diff: f64 = first_dist.iter().map(|r| r.difference / 100.0).sum();
        let mad_score = sum_abs_diff / 9.0;

        // Critical values (Drake and Nigrini):
        // 0.000 - 0.006: Close Conformity
        // 0.006 - 0.012: Acceptable Conformity
        // 0.012 - 0.015: Marginally Acceptable
        // > 0.015: Nonconformity
        let mad_verdict = if mad_score <= 0.006 {
            "Close Conformity".to_string()
        } else if mad_score <= 0.012 {
            "Acceptable Conformity".to_string()
        } else if mad_score <= 0.015 {
            "Marginally Acceptable".to_string()
        } else {
            "Nonconformity".to_string()
        };

        BenfordAnalysis {
            first_digit_distribution: first_dist,
            second_digit_distribution: second_dist,
            mad_score,
            mad_verdict,
        }
    }

    /// Calculate Altman Z-Score.
    pub fn calculate_altman_z_score(
        _bs: &BalanceSheetReport, // Kept for API compatibility/future extraction
        total_assets: Decimal,
        working_capital: Decimal,
        retained_earnings: Decimal,
        ebit: Decimal,
        market_value_equity: Decimal,
        total_liabilities: Decimal,
        sales: Decimal,
    ) -> AltmanZScoreResult {
        let to_f64 = |d: Decimal| d.to_f64().unwrap_or(0.0);
        let t_assets = to_f64(total_assets);

        if t_assets == 0.0 {
            return AltmanZScoreResult {
                score: 0.0,
                zone: AltmanZone::Distress,
                details: AltmanDetails {
                    x1_working_capital: 0.0,
                    x2_retained_earnings: 0.0,
                    x3_ebit: 0.0,
                    x4_equity: 0.0,
                    x5_sales: 0.0,
                },
            };
        }

        let x1 = to_f64(working_capital) / t_assets;
        let x2 = to_f64(retained_earnings) / t_assets;
        let x3 = to_f64(ebit) / t_assets;

        let t_liabilities = to_f64(total_liabilities);
        let x4 = if t_liabilities == 0.0 {
            0.0
        } else {
            to_f64(market_value_equity) / t_liabilities
        };

        let x5 = to_f64(sales) / t_assets;

        // Z = 1.2X1 + 1.4X2 + 3.3X3 + 0.6X4 + 1.0X5
        let score = (1.2 * x1) + (1.4 * x2) + (3.3 * x3) + (0.6 * x4) + (1.0 * x5);

        let zone = if score > 2.99 {
            AltmanZone::Safe
        } else if score > 1.81 {
            AltmanZone::Grey
        } else {
            AltmanZone::Distress
        };

        AltmanZScoreResult {
            score,
            zone,
            details: AltmanDetails {
                x1_working_capital: x1,
                x2_retained_earnings: x2,
                x3_ebit: x3,
                x4_equity: x4,
                x5_sales: x5,
            },
        }
    }

    /// Calculate Beneish M-Score.
    /// This requires current year (t) and previous year (t-1) values.
    /// Uses 8-variable model.
    pub fn calculate_beneish_m_score(
        // Receivables
        receivables_t: Decimal,
        receivables_t1: Decimal,
        // Sales
        sales_t: Decimal,
        sales_t1: Decimal,
        // COGS
        cogs_t: Decimal,
        cogs_t1: Decimal,
        // Total Assets
        assets_t: Decimal,
        assets_t1: Decimal,
        // PPE (Property, Plant, Equipment)
        ppe_t: Decimal,
        ppe_t1: Decimal,
        // Depreciation
        dep_t: Decimal,
        dep_t1: Decimal,
        // SGA Expense
        sga_t: Decimal,
        sga_t1: Decimal,
        // Net Income
        ni_t: Decimal,
        // Cash from Operations
        cfo_t: Decimal,
        // Long Term Debt & Current Liabilities
        ltd_t: Decimal,
        cl_t: Decimal,
        ltd_t1: Decimal,
        cl_t1: Decimal,
    ) -> BeneishMScoreResult {
        let to_f64 = |d: Decimal| d.to_f64().unwrap_or(0.0);

        // Prevent div by zero helpers
        let div = |n: f64, d: f64| if d.abs() < 0.0001 { 0.0 } else { n / d };

        // 1. DSRI (Days Sales in Receivables Index)
        // (Rec_t / Sales_t) / (Rec_t1 / Sales_t1)
        let dsri = div(
            div(to_f64(receivables_t), to_f64(sales_t)),
            div(to_f64(receivables_t1), to_f64(sales_t1)),
        );

        // 2. GMI (Gross Margin Index)
        // [(Sales_t1 - COGS_t1) / Sales_t1] / [(Sales_t - COGS_t) / Sales_t]
        // Note: GMI > 1 means margin deteriorated (bad sign)
        let gm_t1 = div(to_f64(sales_t1 - cogs_t1), to_f64(sales_t1));
        let gm_t = div(to_f64(sales_t - cogs_t), to_f64(sales_t));
        let gmi = div(gm_t1, gm_t);

        // 3. AQI (Asset Quality Index)
        // (1 - (CA_t + PPE_t)/Assets_t) / (1 - (CA_t1 + PPE_t1)/Assets_t1)
        // Actually simplified: Non-Current Assets approx.
        // Let's use the formula: [1 - (CurrentAssets + PPE) / TotalAssets]
        // BUT we don't have Current Assets arg here distinctly?
        // Let's assume passed PPE is net.
        // Formula often: AQI = (1 - (CurrentAssets + PPE)/TotalAssets)_t / ...
        // We will approximate AQ measure as: (TotalAssets - PPE) / TotalAssets if we lack CurrentAssets breakdown here.
        // Wait, normally AQI measures increase in "soft" assets (intangibles).
        // Let's assume (TotalAssets - PPE) represents everything else.
        let aqi_t = div(to_f64(assets_t - ppe_t), to_f64(assets_t)); // Soft assets ratio t
        let aqi_t1 = div(to_f64(assets_t1 - ppe_t1), to_f64(assets_t1));
        let aqi = div(aqi_t, aqi_t1);

        // 4. SGI (Sales Growth Index)
        // Sales_t / Sales_t1
        let sgi = div(to_f64(sales_t), to_f64(sales_t1));

        // 5. DEPI (Depreciation Index)
        // (Dep_t1 / (Dep_t1 + PPE_t1)) / (Dep_t / (Dep_t + PPE_t))
        // Rate of depreciation. If rate slows down (DEPI > 1), might be extending useful life to boost income.
        let rate_t1 = div(to_f64(dep_t1), to_f64(dep_t1 + ppe_t1));
        let rate_t = div(to_f64(dep_t), to_f64(dep_t + ppe_t));
        let depi = div(rate_t1, rate_t);

        // 6. SGAI (SGA Index)
        // (SGA_t / Sales_t) / (SGA_t1 / Sales_t1)
        let sgai = div(
            div(to_f64(sga_t), to_f64(sales_t)),
            div(to_f64(sga_t1), to_f64(sales_t1)),
        );

        // 7. LVGI (Leverage Index)
        // [(LTD_t + CL_t) / Assets_t] / [(LTD_t1 + CL_t1) / Assets_t1]
        let lev_t = div(to_f64(ltd_t + cl_t), to_f64(assets_t));
        let lev_t1 = div(to_f64(ltd_t1 + cl_t1), to_f64(assets_t1));
        let lvgi = div(lev_t, lev_t1);

        // 8. TATA (Total Accruals to Total Assets)
        // (Net Income - CashFromOps) / Total Assets
        let tata = div(to_f64(ni_t - cfo_t), to_f64(assets_t));

        // M-Score Formula (8 variables)
        let m_score = -4.84 + 0.920 * dsri + 0.528 * gmi + 0.404 * aqi + 0.892 * sgi + 0.115 * depi
            - 0.172 * sgai
            + 4.679 * tata
            - 0.327 * lvgi;

        // Manipulation Probability
        // Using Standard Normal CDF (approximate or use 'statrs', but for MVP we use a simple approximation)
        // sigmoid-ish? No, it's cumulative normal distribution.
        // Approx: 1 / (1 + exp(-M)) is logistic. Not exactly CDF.
        // Simple error function approximation for CDF:
        // CDF(x) = 0.5 * (1 + erf(x / sqrt(2)))
        // We probably don't have 'erf' in std.
        // Let's use a simple logistic approximation for now as "Risk Probability" or skip precise prob.
        // Or implement simple approximation of NormCDF.
        let prob = norm_cdf_approx(m_score);

        let risk_level = if m_score > -1.78 {
            "Possible Manipulation".to_string()
        } else {
            "Safe".to_string()
        };

        BeneishMScoreResult {
            score: m_score,
            manipulation_probability: prob,
            risk_level,
            details: BeneishDetails {
                dsri,
                gmi,
                aqi,
                sgi,
                depi,
                sgai,
                lvgi,
                tata,
            },
        }
    }
}

/// Simple approximation of Standard Normal CDF.
fn norm_cdf_approx(x: f64) -> f64 {
    // Constants for approximation
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x_abs = x.abs() / 2.0_f64.sqrt();

    let t = 1.0 / (1.0 + p * x_abs);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x_abs * x_abs).exp();

    0.5 * (1.0 + sign * y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_benford_mad_calculation() {
        // Test case where digit 1 has 100% frequency
        // Exp 1: 30.1%. Diff: 69.9.
        // Others: Exp ~8-17%. Diff: same.
        // MAD should be high.
        let amounts = vec![dec!(100), dec!(100), dec!(100)];
        let result = ForensicService::calculate_benford_law(amounts);

        assert_eq!(result.first_digit_distribution[0].actual_percentage, 100.0);
        assert!(result.mad_score > 0.015);
        assert_eq!(result.mad_verdict, "Nonconformity");
    }

    #[test]
    fn test_beneish_m_score_safe() {
        // Create a scenario with stable ratios (Index ~ 1.0)
        // M = -4.84 + sum(coeffs * 1.0)
        // Coeff sum approx: 0.9+0.5+0.4+0.9+0.1-0.17+4.6-0.3 = ~7
        // Wait, TATA should be near 0 (Accruals/Assets).
        // If everything is 1.0 except TATA=0.
        // Coeffs sans TATA: 0.92+0.528+0.404+0.892+0.115-0.172-0.327 = 2.36
        // M = -4.84 + 2.36 = -2.48
        // -2.48 < -1.78 -> Safe.

        let d1 = dec!(1000);
        let res = ForensicService::calculate_beneish_m_score(
            d1, d1, // Rec
            d1, d1, // Sales
            d1, d1, // COGS
            d1, d1, // Assets
            d1, d1, // PPE
            d1, d1, // Dep
            d1, d1, // SGA
            d1, // NI
            d1, // CFO (NI=CFO -> TATA=0)
            d1, d1, d1, d1, // Debt
        );

        assert!(res.score < -1.78);
        assert_eq!(res.risk_level, "Safe");
    }

    // ========================================================================
    // Property-Based Tests (Math Hardening)
    // ========================================================================

    proptest! {
        #[test]
        fn prop_benford_analysis_never_panics(
            amounts in proptest::collection::vec(
                any::<f64>().prop_map(|f| Decimal::from_f64_retain(f).unwrap_or_default()),
                0..100
            )
        ) {
            let _res = ForensicService::calculate_benford_law(amounts);
        }

        #[test]
        fn prop_beneish_check_division_safety(
            v in any::<f64>()
        ) {
             let d = Decimal::from_f64_retain(v).unwrap_or_default();
             let _res = ForensicService::calculate_beneish_m_score(
                 d, d, d, d, d, d, d, d, d, d, d, d, d, d, d, d, d, d, d, d
             );
        }
    }
}
