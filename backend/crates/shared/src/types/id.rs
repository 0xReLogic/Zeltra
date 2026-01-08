//! Typed IDs for type-safe entity references.
//!
//! Using typed IDs prevents accidentally passing a `UserId` where an `OrgId` is expected.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Macro to generate typed ID wrappers.
macro_rules! typed_id {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            /// Creates a new random ID using UUID v7 (time-ordered).
            #[must_use]
            pub fn new() -> Self {
                Self(Uuid::now_v7())
            }

            /// Creates an ID from an existing UUID.
            #[must_use]
            pub const fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }

            /// Returns the inner UUID.
            #[must_use]
            pub const fn into_inner(self) -> Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl std::str::FromStr for $name {
            type Err = uuid::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(Uuid::parse_str(s)?))
            }
        }
    };
}

typed_id!(UserId, "Unique identifier for a user.");
typed_id!(OrganizationId, "Unique identifier for an organization.");
typed_id!(
    AccountId,
    "Unique identifier for a chart of accounts entry."
);
typed_id!(TransactionId, "Unique identifier for a transaction.");
typed_id!(LedgerEntryId, "Unique identifier for a ledger entry.");
typed_id!(BudgetId, "Unique identifier for a budget.");
typed_id!(FiscalYearId, "Unique identifier for a fiscal year.");
typed_id!(FiscalPeriodId, "Unique identifier for a fiscal period.");
typed_id!(DimensionTypeId, "Unique identifier for a dimension type.");
typed_id!(DimensionValueId, "Unique identifier for a dimension value.");
typed_id!(ApprovalRuleId, "Unique identifier for an approval rule.");
typed_id!(SessionId, "Unique identifier for a user session.");
