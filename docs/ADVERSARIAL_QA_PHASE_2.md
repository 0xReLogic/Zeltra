# Adversarial QA Analysis - Phase 2: Ledger Core + Multi-Currency

**Date:** 2026-01-09  
**Reviewer:** Adversarial QA Engineer / Chaos Tester / Financial Auditor  
**Scope:** Phase 2 - Ledger Core + API (Completed)  
**Status:** 229 tests passing, stress tests claim to pass

---

## Executive Summary

This document presents a brutally honest adversarial analysis of Zeltra's Phase 2 implementation. While the codebase shows competent engineering with proper use of `rust_decimal`, Banker's Rounding, and property-based tests, **several critical assumptions remain untested or are vulnerable under adversarial conditions**.

**Key Findings:**
1. **9 Critical Assumptions** identified with potential for silent financial loss
2. **14 Edge Cases** that could cause balance discrepancies
3. **6 Financial Abuse Scenarios** exploitable by sophisticated attackers
4. **5 High-Impact Failure Modes** that normal QA would miss

---

## 1. ASSUMPTION EXTRACTION

### 1.1 Numeric Range Assumptions

| ID | Assumption | Location | Risk Level |
|----|------------|----------|------------|
| A1.1 | `NUMERIC(19,4)` is sufficient for all monetary values | `DATABASE_SCHEMA.md` | **HIGH** |
| A1.2 | Exchange rates fit in `NUMERIC(19,10)` | `exchange_rates.rate` | MEDIUM |
| A1.3 | `account_version BIGINT` will never overflow | `ledger_entries` | LOW |
| A1.4 | No single transaction exceeds 999 trillion | Schema design | MEDIUM |

**Unstated assumption:** The system assumes organizations will never need to track values exceeding 10^15 with 4 decimal precision. Hyperinflation currencies (e.g., Venezuelan Bolivar, historical Zimbabwe Dollar) could violate this.

### 1.2 Precision and Rounding Assumptions

| ID | Assumption | Location | Risk Level |
|----|------------|----------|------------|
| A2.1 | 4 decimal places sufficient for functional amounts | `CurrencyService::convert()` | **CRITICAL** |
| A2.2 | 10 decimal places sufficient for exchange rates | `exchange_rates.rate` | **HIGH** |
| A2.3 | Banker's Rounding eliminates cumulative bias | `RoundingStrategy::MidpointNearestEven` | MEDIUM |
| A2.4 | Allocation always sums to original | `AllocationUtil` | **CRITICAL** |

**Critical gap:** The allocation algorithm in `allocation.rs:80-84` truncates to `u64` when calculating `extra_count`:
```rust
let extra_count = (remainder / unit)
    .round_dp_with_strategy(0, RoundingStrategy::ToZero)
    .to_u64()
    .unwrap_or(0);
```
If `remainder / unit` produces a value > `u64::MAX`, this silently returns 0.

### 1.3 Event Ordering Assumptions

| ID | Assumption | Location | Risk Level |
|----|------------|----------|------------|
| A3.1 | Entries within a transaction are processed sequentially | `LedgerService::validate_and_resolve()` | MEDIUM |
| A3.2 | `account_version` ordering is globally consistent | DB trigger | **CRITICAL** |
| A3.3 | Transaction commits are atomic across entries | `DEFERRABLE INITIALLY DEFERRED` | **HIGH** |
| A3.4 | Exchange rates are immutable once used | No enforcement | **HIGH** |

### 1.4 Idempotency Assumptions

| ID | Assumption | Location | Risk Level |
|----|------------|----------|------------|
| A4.1 | Same transaction cannot be posted twice | Status transitions | **CRITICAL** |
| A4.2 | Reference numbers are unique per org | `UNIQUE (organization_id, reference_number)` | MEDIUM |
| A4.3 | Balance updates are idempotent | DB trigger | **CRITICAL** |

**Gap:** No idempotency key on transaction creation. Network retries could create duplicate draft transactions.

### 1.5 Atomicity Assumptions

| ID | Assumption | Location | Risk Level |
|----|------------|----------|------------|
| A5.1 | All entries in a transaction commit together | SeaORM transaction | **CRITICAL** |
| A5.2 | Balance trigger and entry insert are atomic | PostgreSQL | MEDIUM |
| A5.3 | Account version increment cannot be skipped | `FOR UPDATE` lock | **HIGH** |

### 1.6 Trust Assumptions

| ID | Assumption | Location | Risk Level |
|----|------------|----------|------------|
| A6.1 | Exchange rate lookup returns correct rates | `exchange_rate_lookup` callback | **HIGH** |
| A6.2 | Account validator is honest | `account_validator` callback | **HIGH** |
| A6.3 | Fiscal period status is accurate at commit time | TOCTOU window | **CRITICAL** |
| A6.4 | RLS context is set correctly per request | Middleware | **CRITICAL** |

### 1.7 Time Consistency Assumptions

| ID | Assumption | Location | Risk Level |
|----|------------|----------|------------|
| A7.1 | Transaction date is validated against fiscal period | Service layer | MEDIUM |
| A7.2 | Exchange rate effective_date matches transaction date | Lookup logic | **HIGH** |
| A7.3 | Server time is synchronized across nodes | Not addressed | **HIGH** |
| A7.4 | `created_at` timestamps are monotonic | `DEFAULT now()` | MEDIUM |

---

## 2. EDGE CASE GENERATION

### 2.1 Integer Overflow / Underflow Cases

| ID | Edge Case | Violated Assumption | Test Status |
|----|-----------|---------------------|-------------|
| E2.1.1 | Transaction with 10^16 in source_amount | A1.1 | **UNTESTED** |
| E2.1.2 | Exchange rate = 10^9 causing overflow on multiply | A1.4 | **UNTESTED** |
| E2.1.3 | Allocation of 1 cent among 10^20 recipients | A2.4 | **UNTESTED** |
| E2.1.4 | Running balance exceeds i64 after 2^63 entries | A1.3 | **UNTESTED** |

**Concrete test case for E2.1.3:**
```rust
// Test with allocation count that causes extra_count calculation to overflow
// When remainder / unit > u64::MAX, to_u64() returns 0 silently
let result = AllocationUtil::allocate_equal(
    Decimal::new(1, 2),  // $0.01
    1_000_000_000_000,   // 1 trillion recipients - realistic for large systems
    2
);
// Expected behavior: Should either:
//   1. Return Err indicating count too large, OR
//   2. Allocate correctly with sum == $0.01
// Actual risk: With extreme counts, may return allocations where sum != total
// due to truncation in extra_count calculation (line 80-84 in allocation.rs)
```

### 2.2 Floating-Point Precision Drift Cases

| ID | Edge Case | Violated Assumption | Test Status |
|----|-----------|---------------------|-------------|
| E2.2.1 | 1000 conversions at rate 1.000000001 | A2.3 | **UNTESTED** |
| E2.2.2 | Rate inversion: 1/3 * 3 != 1 | A2.2 | PARTIAL |
| E2.2.3 | JPY (0 decimals) → USD (2 decimals) → JPY round trip | A2.1 | **UNTESTED** |
| E2.2.4 | Sum of 1000 allocations vs single allocation | A2.4 | PARTIAL |

**Concrete test case for E2.2.3:**
```rust
// JPY 100 → USD → JPY may not equal 100
let jpy = dec!(100);
let jpy_usd_rate = dec!(0.0067);  // 1 JPY = 0.0067 USD
let usd_jpy_rate = dec!(149.25); // 1 USD = 149.25 JPY

let usd = CurrencyService::convert(jpy, jpy_usd_rate);  // 0.67 USD
let jpy_back = CurrencyService::convert(usd, usd_jpy_rate);  // 99.9975 → 100.00?
// Round trip may not preserve original
```

### 2.3 Rounding Bias Accumulation Cases

| ID | Edge Case | Violated Assumption | Test Status |
|----|-----------|---------------------|-------------|
| E2.3.1 | 10,000 transactions all rounding up | A2.3 | **UNTESTED** |
| E2.3.2 | Daily fee calculations over 10 years | A2.3 | **UNTESTED** |
| E2.3.3 | Allocation of odd pennies in large batches | A2.4 | **UNTESTED** |

### 2.4 Double Execution / Retry Cases

| ID | Edge Case | Violated Assumption | Test Status |
|----|-----------|---------------------|-------------|
| E2.4.1 | HTTP timeout → client retry → duplicate entry | A4.1 | **UNTESTED** |
| E2.4.2 | DB connection drop during commit | A5.1 | **UNTESTED** |
| E2.4.3 | Balance trigger fires twice for same entry | A4.3 | **UNTESTED** |
| E2.4.4 | Concurrent POST with same reference_number | A4.2 | PARTIAL |

### 2.5 Partial Commit Cases

| ID | Edge Case | Violated Assumption | Test Status |
|----|-----------|---------------------|-------------|
| E2.5.1 | 3 of 5 entries inserted before DB crash | A5.1 | **UNTESTED** |
| E2.5.2 | Transaction header saved but no entries | A5.1 | **UNTESTED** |
| E2.5.3 | Entry dimensions saved but entry fails | A5.2 | **UNTESTED** |

### 2.6 Race Condition Cases

| ID | Edge Case | Violated Assumption | Test Status |
|----|-----------|---------------------|-------------|
| E2.6.1 | Two transactions posting to same account simultaneously | A5.3 | TESTED ✓ |
| E2.6.2 | Fiscal period closed between validation and commit | A6.3 | **UNTESTED** |
| E2.6.3 | Account deactivated between validation and commit | A6.2 | **UNTESTED** |
| E2.6.4 | Exchange rate updated between lookup and use | A6.1 | **UNTESTED** |

### 2.7 State Desynchronization Cases

| ID | Edge Case | Violated Assumption | Test Status |
|----|-----------|---------------------|-------------|
| E2.7.1 | Balance tracker disagrees with sum of entries | A3.2 | TESTED ✓ |
| E2.7.2 | Account version gap (1, 2, 4 - missing 3) | A3.2 | **UNTESTED** |
| E2.7.3 | Previous_balance doesn't match prior current_balance | A3.2 | TESTED ✓ |

### 2.8 Clock Skew / Timestamp Cases

| ID | Edge Case | Violated Assumption | Test Status |
|----|-----------|---------------------|-------------|
| E2.8.1 | Transaction date in future fiscal period | A7.1 | **UNTESTED** |
| E2.8.2 | Exchange rate lookup with date 1 day off | A7.2 | **UNTESTED** |
| E2.8.3 | `created_at` older than parent transaction | A7.4 | **UNTESTED** |
| E2.8.4 | DST transition causing duplicate/missing hour | A7.3 | **UNTESTED** |

### 2.9 Replayed Valid Request Cases

| ID | Edge Case | Violated Assumption | Test Status |
|----|-----------|---------------------|-------------|
| E2.9.1 | Old valid JWT used after session revoked | Trust | **UNTESTED** |
| E2.9.2 | Transaction replay with same idempotency key | A4.1 | **UNTESTED** |
| E2.9.3 | Rate-limited request succeeds on retry | Trust | **UNTESTED** |

---

## 3. FINANCIAL ABUSE SCENARIOS

### 3.1 Balance Divergence Without Detection

**Scenario: Silent Rounding Theft**

An attacker creates thousands of small transactions designed to exploit rounding:

```
Attack Vector:
1. Create 10,000 transactions of $0.005 each
2. Each rounds to $0.00 or $0.01 depending on Banker's Rounding
3. Systematically choose amounts that always round UP for attacker
4. Over time, attacker gains cents that don't exist on credit side

Impact: $50-100 per 10,000 transactions in free money
Detection Difficulty: HIGH - each transaction balances individually

Why QA Misses It:
- Unit tests check individual transactions balance
- No test for aggregate rounding bias over time
- No reconciliation of theoretical vs actual totals
```

### 3.2 Reconciliation Passes But Totals Wrong

**Scenario: Exchange Rate Manipulation**

```
Attack Vector:
1. Admin/accountant has access to exchange_rates table
2. Creates transaction at favorable rate A
3. Updates exchange rate retroactively to rate B
4. Historical queries show different totals than what was posted
5. Reconciliation uses current rates, not transaction rates

Impact: Arbitrary financial manipulation
Detection Difficulty: VERY HIGH - all current data looks correct

Why QA Misses It:
- No immutability enforcement on exchange_rates
- No audit log of rate changes
- functional_amount is stored, but source_amount + rate could be recalculated
```

### 3.3 Fee/Spread Gaming

**Scenario: Allocation Remainder Theft**

```
Attack Vector:
1. Create expense splits that always assign extra cents to controlled account
2. AllocationUtil gives extra units to first N items
3. Attacker ensures their account is always in first N positions
4. Over thousands of allocations, cents accumulate

Code Vulnerability (allocation.rs:86-88):
// Build result: first N items get the extra unit
(0..count)
    .map(|i| if i < extra_count { base + unit } else { base })
    .collect()

Impact: Systematic bias toward first recipients
Detection Difficulty: HIGH - allocation sum always equals total

Why QA Misses It:
- Tests verify sum invariant, not distribution fairness
- No randomization of extra unit assignment
- No test for positional bias
```

### 3.4 Funds Stuck Scenario

**Scenario: Fiscal Period Race Condition**

```
Attack Vector:
1. Start transaction creation at 23:59:59 on period end
2. Validation passes (period OPEN)
3. Admin closes fiscal period at 00:00:01
4. Commit fails due to closed period
5. Draft transaction stuck - can't post, can't delete (references closed period)

Impact: Funds in limbo, manual intervention required
Detection Difficulty: MEDIUM - visible in pending transactions

Why QA Misses It:
- Tests don't simulate concurrent period closure
- No chaos testing of timing edge cases
- No stress test for period boundaries
```

### 3.5 Systematic Favoritism

**Scenario: Deterministic Rounding Bias**

```
Attack Vector:
Banker's Rounding is deterministic:
- 2.5 → 2 (down to even)
- 3.5 → 4 (up to even)

Create transactions that systematically hit midpoints:
- Revenue: always amounts ending in .x5 where x is odd (rounds UP)
- Expenses: always amounts ending in .x5 where x is even (rounds DOWN)

Impact: Systematic inflation of profit margin by 0.5% on midpoint values
Detection Difficulty: VERY HIGH - each transaction is "correct"

Why QA Misses It:
- Tests verify rounding is deterministic (good)
- No test for systematic exploitation of determinism
```

### 3.6 Global Invariant Failure

**Scenario: Multi-Currency Total Mismatch**

```
Attack Vector:
1. Organization base currency: USD
2. Create EUR transaction, converted at rate R1
3. Create GBP transaction, converted at rate R2
4. Both transactions balance in functional_amount
5. However: SUM(functional_amount) across all entries might not equal 0

Mechanism:
- Entry 1: EUR 100 → USD 110.00 (debit)
- Entry 2: EUR 100 → USD 110.00 (credit)
  Transaction balances ✓

- Entry 3: EUR 100 → USD 109.9999 (different rounding path)
- Entry 4: EUR 100 → USD 110.00
  Transaction balances ✓

Global: USD 329.9999 debit, USD 330.00 credit
Difference: -$0.0001

Impact: Trial balance shows non-zero difference
Detection Difficulty: HIGH - only visible at aggregate level

Why QA Misses It:
- Tests verify per-transaction balance
- No test for cross-transaction aggregate balance
- No global invariant: SUM(all debits) = SUM(all credits)
```

---

## 4. FAILURE IMPACT ANALYSIS

### 4.1 CRITICAL: Allocation Overflow (E2.1.3)

**Exact Failure Scenario:**
```rust
// 1 trillion recipients - realistic for global user base or IoT devices
AllocationUtil::allocate_equal(dec!(0.01), 1_000_000_000_000, 2)
```

**Production Occurrence:**
- Mass payment to large user base
- Referral bonus distribution
- Dividend allocation to shareholders

**Financial Impact:**
- Best case: Panic, transaction fails
- Worst case: Silent truncation, funds disappear
- Potential loss: Entire allocation amount

**Why Normal QA Misses It:**
- Property tests use `1..100` for count
- No test with astronomically large counts
- No integration test with production-scale data

### 4.2 CRITICAL: TOCTOU Race on Fiscal Period (E2.6.2)

**Exact Failure Scenario:**
```
T0: validate_fiscal_period() returns OPEN
T1: Admin clicks "Close Period" 
T2: DB trigger closes period
T3: transaction.commit() executes
T4: Balance updates applied to CLOSED period
```

**Production Occurrence:**
- Month-end close procedures
- Concurrent transaction entry during close
- Automated batch processing during close

**Financial Impact:**
- Posted transactions in closed period (audit failure)
- Incorrect period totals
- Compliance violations (SOX, GAAP)

**Why Normal QA Misses It:**
- Tests don't simulate concurrent admin actions
- Integration tests run sequentially
- No chaos engineering for timing attacks

### 4.3 HIGH: Exchange Rate Retroactive Manipulation (3.2)

**Exact Failure Scenario:**
```sql
-- Posted transaction used rate 1.10
UPDATE exchange_rates SET rate = 1.05 
WHERE from_currency = 'EUR' AND effective_date = '2026-01-15';

-- Historical report now shows different conversion
-- But posted functional_amount is unchanged
```

**Production Occurrence:**
- "Correcting" historical rate errors
- Malicious insider manipulation
- Import of new rate data overwriting existing

**Financial Impact:**
- Audit trail inconsistency
- Regulatory compliance failure
- Potential fraud facilitation

**Why Normal QA Misses It:**
- Exchange rates have no immutability constraint
- No audit log for rate changes
- Tests don't verify historical consistency

### 4.4 HIGH: Duplicate Transaction via Retry (E2.4.1)

**Exact Failure Scenario:**
```
T0: Client sends POST /transactions
T1: Server processes, creates transaction
T2: Server begins response
T3: Network timeout (client doesn't receive 201)
T4: Client retries POST /transactions
T5: Server creates DUPLICATE transaction
```

**Production Occurrence:**
- Flaky network connections
- Mobile app in poor coverage
- Load balancer timeout during peak

**Financial Impact:**
- Double-posted expenses
- Duplicate revenue recognition
- Account balance inflation

**Why Normal QA Misses It:**
- No idempotency key implementation
- Tests use reliable local connections
- No chaos testing of network failures

### 4.5 MEDIUM: Running Balance Desync (E2.7.2)

**Exact Failure Scenario:**
```
account_version sequence: [1, 2, 3, 4, 4, 5]  -- duplicate 4!
previous_balance[5] = current_balance[4]  -- which 4?
```

**Production Occurrence:**
- Database recovery from backup
- Manual intervention during incident
- Bug in concurrent update handling

**Financial Impact:**
- Point-in-time balance queries return wrong values
- Historical reports are inconsistent
- Audit trail breaks

**Why Normal QA Misses It:**
- Tests assume version sequence is always correct
- No test for version gaps or duplicates
- Stress tests don't verify version integrity

---

## 5. DETECTION STRATEGY

### 5.1 For Allocation Overflow (E2.1.3)

**Invariant:** `allocate_equal(total, count, dp).iter().sum() == total.round_dp(dp)`

**Test Type:** Property-based test with extreme values
```rust
#[test]
fn prop_allocation_handles_extreme_counts() {
    // Test with counts that stress the allocation algorithm
    // These values probe different failure modes:
    // - 10B: Large but potentially feasible in production
    // - 100B: Tests i64 boundary conditions  
    // - 1T: Tests extra_count truncation behavior
    for count in [10_000_000_000, 100_000_000_000, 1_000_000_000_000] {
        let result = std::panic::catch_unwind(|| {
            AllocationUtil::allocate_equal(dec!(100), count, 2)
        });
        
        match result {
            Ok(allocations) => {
                // If it doesn't panic, sum MUST equal total
                let sum: Decimal = allocations.iter().copied().sum();
                assert_eq!(sum, dec!(100), 
                    "Allocation sum mismatch for count={}", count);
            }
            Err(_) => {
                // Panic is acceptable for extreme counts
                // Better to fail loudly than silently lose money
            }
        }
    }
}
```

**Production Metrics:**
- Alert: `allocation_count > 1_000_000`
- Log: All allocation operations with count, total, and result sum
- Reconciliation: Daily check that SUM(allocations) = SUM(originals)

### 5.2 For TOCTOU Race (E2.6.2)

**Invariant:** `fiscal_period.status = CLOSED ⟹ transaction.status ≠ posted`

**Test Type:** Chaos / Race condition test
```rust
#[tokio::test]
async fn test_fiscal_period_close_race() {
    // Start transaction in one task
    let tx_task = tokio::spawn(async {
        create_and_post_transaction(...).await
    });
    
    // Close period in another task
    let close_task = tokio::spawn(async {
        close_fiscal_period(...).await  
    });
    
    // Both complete - verify no transactions posted to closed period
}
```

**Production Metrics:**
- Alert: Transaction `posted_at` > fiscal_period `closed_at`
- Log: All period status changes with timestamp
- Query: Weekly audit for transactions in closed periods

### 5.3 For Exchange Rate Manipulation (3.2)

**Invariant:** `exchange_rates` rows are immutable after first use

**Test Type:** Integration test with audit verification
```rust
#[test]
fn test_exchange_rate_immutability() {
    // Create transaction using rate
    // Attempt to update rate
    // Verify update fails or creates new version
}
```

**Production Metrics:**
- Alert: Any UPDATE on `exchange_rates` table
- Log: Trigger that logs all exchange_rate changes
- Reconciliation: Compare `functional_amount` with `source_amount * rate`

### 5.4 For Duplicate Transactions (E2.4.1)

**Invariant:** Each idempotency_key produces at most one transaction

**Test Type:** Integration test with retry simulation
```rust
#[tokio::test]
async fn test_idempotent_transaction_creation() {
    let idempotency_key = Uuid::new_v4();
    
    // First request
    let resp1 = create_transaction(idempotency_key).await;
    
    // Simulated retry
    let resp2 = create_transaction(idempotency_key).await;
    
    // Both should return same transaction ID
    assert_eq!(resp1.id, resp2.id);
}
```

**Production Metrics:**
- Alert: Same reference_number submitted within 5 minutes
- Log: All transaction creation attempts with idempotency key
- Query: Daily check for potential duplicates (same org, amount, date, accounts)

### 5.5 For Running Balance Desync (E2.7.2)

**Invariant:** 
- `account_version` is strictly increasing with no gaps
- `previous_balance[N] == current_balance[N-1]`

**Test Type:** Integrity verification after stress test
```rust
#[test]
fn verify_running_balance_integrity() {
    // After stress test, verify for each account:
    let entries = get_entries_by_account_ordered_by_version(account_id);
    
    for window in entries.windows(2) {
        assert_eq!(window[0].current_balance, window[1].previous_balance);
        assert_eq!(window[0].account_version + 1, window[1].account_version);
    }
}
```

**Production Metrics:**
- Alert: `MAX(account_version) != COUNT(entries)` for any account
- Log: All balance updates with before/after values
- Query: Nightly integrity check on running balances

---

## 6. RECOMMENDATIONS (For Reference Only)

> Note: Per instructions, fixes are NOT proposed. These are detection priorities.

1. **IMMEDIATE:** Add property test for allocation with extreme counts
2. **IMMEDIATE:** Add global invariant test: SUM(all debits) = SUM(all credits)  
3. **HIGH:** Implement idempotency key for transaction creation
4. **HIGH:** Add chaos test for fiscal period close race
5. **MEDIUM:** Add exchange rate immutability enforcement
6. **MEDIUM:** Add reconciliation job comparing calculated vs stored balances

---

## 7. CONCLUSION

Phase 2 demonstrates solid engineering fundamentals:
- ✓ Proper use of `rust_decimal` 
- ✓ Banker's Rounding implementation
- ✓ Property-based tests for core invariants
- ✓ Concurrent transaction stress testing
- ✓ Running balance consistency tests

However, the system has **blind spots typical of non-adversarial testing**:
- ✗ No extreme value testing
- ✗ No TOCTOU race testing  
- ✗ No global invariant verification
- ✗ No idempotency protection
- ✗ No immutability enforcement on critical data
- ✗ No chaos engineering for failure modes

**Bottom Line:** The ledger will likely work correctly under normal conditions. Under adversarial conditions, stress, or component failures, there are multiple paths to silent financial loss.

The engineers believe the core logic is "mostly correct." **This analysis suggests the core logic IS mostly correct, but the boundary conditions and failure modes are not adequately protected.**

---

*This report was prepared assuming all invariants can be violated under stress, inputs are adversarial but valid, and time/order/retries behave badly. Rust memory safety does NOT guarantee financial correctness.*
