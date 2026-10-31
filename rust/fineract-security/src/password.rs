//! Password hashing and verification using argon2id.
//!
//! Passwords are always handled via [`secrecy::SecretString`] and cleared from
//! memory with [`zeroize::Zeroize`] after use.

use argon2::password_hash::{PasswordHash, SaltString};
use argon2::{Algorithm, Argon2, Params, PasswordHasher, PasswordVerifier, Version};
use rand::rngs::OsRng;
use secrecy::{ExposeSecret, SecretString};
use zeroize::Zeroize;

use crate::errors::SecurityError;

/// Cost parameter for argon2id. 15 is a strong baseline for interactive logins.
const ARGON2_COST: u32 = 15;
const ARGON2_MEMORY: u32 = 65536; // 64 MiB
const ARGON2_PARALLELISM: u32 = 4;

fn argon2_instance() -> Result<Argon2<'static>, SecurityError> {
    let params =
        Params::new(ARGON2_MEMORY, ARGON2_COST, ARGON2_PARALLELISM, Some(32)).map_err(|_| {
            SecurityError::Internal {
                detail: "invalid argon2 parameters".into(),
            }
        })?;
    Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
}

/// Hash a password into an argon2id encoded string (PHC format).
///
/// The returned string includes the salt and parameters and can be stored as-is.
pub fn hash_password(password: &SecretString) -> Result<String, SecurityError> {
    let salt = SaltString::generate(&mut OsRng);
    let encoded_hash = argon2_instance()?
        .hash_password(password.expose_secret().as_bytes(), &salt)
        .map_err(|_| SecurityError::Internal {
            detail: "password hashing failed".into(),
        })?;
    Ok(encoded_hash.to_string())
}

/// Verify a password against an argon2id encoded hash.
pub fn verify_password(password: &SecretString, encoded_hash: &str) -> Result<bool, SecurityError> {
    let parsed_hash = PasswordHash::new(encoded_hash).map_err(|_| SecurityError::Internal {
        detail: "invalid stored password hash".into(),
    })?;
    let ok = argon2_instance()?
        .verify_password(password.expose_secret().as_bytes(), &parsed_hash)
        .is_ok();
    Ok(ok)
}

/// A zeroizing wrapper around a plaintext password during authentication.
///
/// `secrecy::SecretString` in v0.8 does not implement `zeroize::Zeroize`, so
/// we clear the reference manually on drop. The secret is never logged or
/// displayed.
pub struct PlainPassword(pub SecretString);

impl PlainPassword {
    pub fn new(password: SecretString) -> Self {
        Self(password)
    }
}

impl Zeroize for PlainPassword {
    fn zeroize(&mut self) {
        // secrecy::SecretString in v0.8 does not implement Zeroize; clear the
        // reference by replacing with an empty secret.
        self.0 = SecretString::from(String::new());
    }
}

impl Drop for PlainPassword {
    fn drop(&mut self) {
        self.zeroize();
    }
}
