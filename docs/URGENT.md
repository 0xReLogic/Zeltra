# 🚨 URGENT: Tier Enforcement Gaps

> **Last Updated**: 2026-01-14
> **Status**: BUSINESS_MODEL.md updated ✅ | DB values match ✅ | Backend enforcement needs work ⚠️

---

## ✅ DB VALUES MATCH BUSINESS_MODEL.md

| Feature        | Starter |  Growth   | Enterprise | Status |
| :------------- | :-----: | :-------: | :--------: | :----: |
| Users          |   50    |    200    | Unlimited  |   ✅   |
| Transactions   |  1,000  |  10,000   | Unlimited  |   ✅   |
| Storage        |  5 GB   |   50 GB   |   500 GB   |   ✅   |
| Dimensions     |    2    | Unlimited | Unlimited  |   ✅   |
| Budgets        |    3    | Unlimited | Unlimited  |   ✅   |
| Multi-Currency |   ❌    |    ✅     |     ✅     |   ✅   |
| Simulation     |   ❌    |    ❌     |     ✅     |   ✅   |
| Accruals       |   ❌    |    ❌     |     ✅     |   ✅   |
| Intercompany   |   ❌    |    ❌     |     ✅     |   ✅   |

---

## ✅ BACKEND ENFORCED (Working correctly)

| Feature                      | Route              | Check Function                               |
| :--------------------------- | :----------------- | :------------------------------------------- |
| `max_users`                  | `organizations.rs` | `check_tier_user_limit()`                    |
| `max_transactions_per_month` | `transactions.rs`  | `check_monthly_transaction_limit()`          |
| `max_dimensions`             | `dimensions.rs`    | `check_dimension_limit()`                    |
| `has_multi_currency`         | `sentinel.rs:220`  | `check_tier_feature("has_multi_currency")`   |
| `has_auto_accruals`          | `sentinel.rs:348`  | `check_tier_feature("has_auto_accruals")`    |
| `has_intercompany_hub`       | `sentinel.rs:586`  | `check_tier_feature("has_intercompany_hub")` |

---

## ❌ NOT ENFORCED (Feature exists, no tier check!)

| Feature        | Route            | DB Column               | Required Fix                               |
| :------------- | :--------------- | :---------------------- | :----------------------------------------- |
| **Simulation** | `simulation.rs`  | `has_simulation`        | Add `check_tier_feature("has_simulation")` |
| **Budgets**    | `budgets.rs`     | `max_budgets`           | Add budget count check before create       |
| **Storage**    | `attachments.rs` | `attachment_storage_gb` | Add storage quota check before upload      |

---

## 🔧 ACTION ITEMS (Before Launch)

1. [x] Add `has_simulation` tier check in `simulation.rs` ✅ DONE
2. [x] Add `max_budgets` limit check in `budgets.rs` ✅ DONE
3. [x] Add `attachment_storage_gb` quota check in `attachments.rs` ✅ DONE
