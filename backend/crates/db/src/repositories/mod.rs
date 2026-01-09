//! Repository abstractions for data access.
//!
//! Repositories provide a clean interface for database operations,
//! hiding the `SeaORM` implementation details from the rest of the application.

pub mod account;
pub mod dimension;
pub mod email_verification;
pub mod exchange_rate;
pub mod fiscal;
pub mod organization;
pub mod session;
pub mod subscription;
pub mod user;

pub use account::{
    AccountError, AccountFilter, AccountRepository, AccountWithBalance, CreateAccountInput,
    UpdateAccountInput,
};
pub use dimension::{
    CreateDimensionTypeInput, CreateDimensionValueInput, DimensionError, DimensionRepository,
    DimensionTypeFilter, DimensionValueFilter, UpdateDimensionTypeInput, UpdateDimensionValueInput,
};
pub use email_verification::EmailVerificationRepository;
pub use exchange_rate::{
    CreateExchangeRateInput, ExchangeRateError, ExchangeRateLookup, ExchangeRateRepository,
    RateLookupMethod,
};
pub use fiscal::{CreateFiscalYearInput, FiscalError, FiscalRepository, FiscalYearWithPeriods};
pub use organization::OrganizationRepository;
pub use session::SessionRepository;
pub use subscription::{Feature, LimitCheckResult, ResourceLimit, SubscriptionRepository};
pub use user::UserRepository;
