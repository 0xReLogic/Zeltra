//! Repository abstractions for data access.
//!
//! Repositories provide a clean interface for database operations,
//! hiding the `SeaORM` implementation details from the rest of the application.

pub mod organization;
pub mod session;
pub mod subscription;
pub mod user;

pub use organization::OrganizationRepository;
pub use session::SessionRepository;
pub use subscription::{Feature, LimitCheckResult, ResourceLimit, SubscriptionRepository};
pub use user::UserRepository;
