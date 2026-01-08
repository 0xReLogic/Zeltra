//! Password hashing with Argon2id.
//!
//! Uses the recommended Argon2id variant with secure defaults.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, PasswordVerifier, SaltString},
    Argon2, PasswordHash,
};
use thiserror::Error;

/// Errors that can occur during password operations.
#[derive(Debug, Error)]
pub enum PasswordError {
    /// Failed to hash password.
    #[error("failed to hash password: {0}")]
    HashError(String),

    /// Failed to verify password.
    #[error("failed to verify password: {0}")]
    VerifyError(String),

    /// Invalid password hash format.
    #[error("invalid password hash format")]
    InvalidHash,
}

/// Hashes a password using Argon2id.
///
/// # Arguments
///
/// * `password` - The plaintext password to hash.
///
/// # Returns
///
/// The hashed password as a PHC string format.
///
/// # Errors
///
/// Returns `PasswordError::HashError` if hashing fails.
///
/// # Example
///
/// ```
/// use zeltra_core::auth::hash_password;
///
/// let hash = hash_password("my_secure_password").unwrap();
/// assert!(hash.starts_with("$argon2id$"));
/// ```
pub fn hash_password(password: &str) -> Result<String, PasswordError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| PasswordError::HashError(e.to_string()))
}

/// Verifies a password against a hash.
///
/// # Arguments
///
/// * `password` - The plaintext password to verify.
/// * `hash` - The stored password hash (PHC string format).
///
/// # Returns
///
/// `true` if the password matches, `false` otherwise.
///
/// # Errors
///
/// Returns `PasswordError::InvalidHash` if the hash format is invalid.
/// Returns `PasswordError::VerifyError` if verification fails unexpectedly.
///
/// # Example
///
/// ```
/// use zeltra_core::auth::{hash_password, verify_password};
///
/// let hash = hash_password("my_password").unwrap();
/// assert!(verify_password("my_password", &hash).unwrap());
/// assert!(!verify_password("wrong_password", &hash).unwrap());
/// ```
pub fn verify_password(password: &str, hash: &str) -> Result<bool, PasswordError> {
    let parsed_hash = PasswordHash::new(hash).map_err(|_| PasswordError::InvalidHash)?;

    let argon2 = Argon2::default();

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(PasswordError::VerifyError(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password() {
        let password = "test_password_123!";
        let hash = hash_password(password).unwrap();

        // Hash should be in PHC format
        assert!(hash.starts_with("$argon2id$"));

        // Hash should be different from password
        assert_ne!(hash, password);
    }

    #[test]
    fn test_verify_correct_password() {
        let password = "correct_password";
        let hash = hash_password(password).unwrap();

        assert!(verify_password(password, &hash).unwrap());
    }

    #[test]
    fn test_verify_wrong_password() {
        let password = "correct_password";
        let hash = hash_password(password).unwrap();

        assert!(!verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_different_passwords_different_hashes() {
        let hash1 = hash_password("password1").unwrap();
        let hash2 = hash_password("password1").unwrap();

        // Same password should produce different hashes (due to random salt)
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_invalid_hash_format() {
        let result = verify_password("password", "invalid_hash");
        assert!(matches!(result, Err(PasswordError::InvalidHash)));
    }
}
