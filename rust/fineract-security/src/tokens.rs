//! JWT access + refresh token management.
//!
//! Access tokens are short-lived and carry tenant + user claims. Refresh tokens
//! are opaque random values stored hashed in the DB and can be revoked.
//! Constant-time comparison is used for token lookups.

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;

use crate::context::{AuthorizedContext, Principal, SessionId};
use crate::errors::SecurityError;
use fineract_core::ids::*;

/// JWT signing / verification keys.
#[derive(Clone)]
pub struct JwtKeys {
    #[allow(dead_code)]
    access_encoding: EncodingKey,
    #[allow(dead_code)]
    access_decoding: DecodingKey,
    #[allow(dead_code)]
    refresh_encoding: EncodingKey,
    #[allow(dead_code)]
    refresh_decoding: DecodingKey,
}

impl JwtKeys {
    /// Generate fresh random HS256 keys (for development / test only; production
    /// should load persisted PEM / JWKS keys).
    pub fn generate() -> Result<Self, SecurityError> {
        let access_secret: [u8; 32] = rand::random();
        let refresh_secret: [u8; 32] = rand::random();
        Ok(Self {
            access_encoding: EncodingKey::from_secret(&access_secret),
            access_decoding: DecodingKey::from_secret(&access_secret),
            refresh_encoding: EncodingKey::from_secret(&refresh_secret),
            refresh_decoding: DecodingKey::from_secret(&refresh_secret),
        })
    }
}

/// Claims embedded in an access token.
#[derive(Debug, Serialize, Deserialize)]
pub struct AccessClaims {
    pub sub: String, // app_user_id
    pub tenant: i64, // tenant_id
    pub username: String,
    pub exp: i64,
    pub iat: i64,
    pub jti: String, // token ID (for revocation)
}

/// Result of issuing a token pair.
#[derive(Clone)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: SecretString,
    pub access_expires_at: chrono::DateTime<Utc>,
}

impl TokenPair {
    pub fn zeroize_refresh(&mut self) {
        // secrecy::SecretString in v0.8 does not implement Zeroize; clear the
        // reference so the secret is no longer held.
        self.refresh_token = SecretString::from(String::new());
    }
}

/// Issues a JWT access token + opaque refresh token for a principal.
pub fn issue_tokens(
    keys: &JwtKeys,
    principal: &Principal,
    tenant_id: TenantId,
    ttl_seconds: i64,
) -> Result<TokenPair, SecurityError> {
    let now = Utc::now();
    let exp = now + Duration::seconds(ttl_seconds);
    let jti = uuid::Uuid::new_v4().to_string();

    let claims = AccessClaims {
        sub: principal.app_user_id.to_string(),
        tenant: tenant_id.value(),
        username: principal.username.clone(),
        exp: exp.timestamp(),
        iat: now.timestamp(),
        jti: jti.clone(),
    };

    let access_token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &keys.access_encoding,
    )
    .map_err(|_| SecurityError::Internal {
        detail: "failed to sign access token".into(),
    })?;

    let refresh_secret = SecretString::from(uuid::Uuid::new_v4().to_string());
    Ok(TokenPair {
        access_token,
        refresh_token: refresh_secret,
        access_expires_at: exp,
    })
}

/// Verifies and decodes an access token, returning the claims.
pub fn verify_access_token(keys: &JwtKeys, token: &str) -> Result<AccessClaims, SecurityError> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    let data = decode::<AccessClaims>(token, &keys.access_decoding, &validation)
        .map_err(|_| SecurityError::InvalidToken)?;
    Ok(data.claims)
}

/// Constant-time comparison of two refresh tokens (prevents timing attacks).
pub fn refresh_tokens_eq(a: &SecretString, b: &SecretString) -> bool {
    let ab = a.expose_secret().as_bytes();
    let bb = b.expose_secret().as_bytes();
    if ab.len() != bb.len() {
        return false;
    }
    ab.ct_eq(bb).into()
}

/// Build an [`AuthorizedContext`] from verified JWT claims.
pub fn context_from_claims(claims: &AccessClaims, session_id: SessionId) -> AuthorizedContext {
    AuthorizedContext::new(
        TenantId::new(claims.tenant),
        Principal {
            app_user_id: AppUserId::new(claims.sub.parse::<i64>().unwrap_or(1)),
            username: claims.username.clone(),
            roles: Default::default(),
            permissions: Default::default(),
            office_id: None,
        },
        session_id,
        chrono::Utc::now(),
    )
}
