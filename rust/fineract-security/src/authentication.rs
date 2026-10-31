//! Authentication service: Basic Auth (argon2) and JWT.
//!
//! This trait-based service is the single entry point for verifying caller
//! identity. Implementations are free to back onto any user store.

use async_trait::async_trait;
use secrecy::{ExposeSecret, SecretString};
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use crate::context::{AuthorizedContext, Principal, SessionId};
use crate::errors::SecurityError;
use crate::tokens::TokenPair;
use crate::JwtKeys;
use fineract_core::ids::{AppUserId, TenantId};

/// Result of successful authentication.
#[derive(Clone)]
pub struct AuthenticationResult {
    pub context: AuthorizedContext,
    pub token_pair: Option<TokenPair>,
}

#[async_trait]
pub trait AuthenticationService: Send + Sync {
    /// Authenticate with username + password (Basic Auth flow). Returns a
    /// security context on success; no token pair is issued here (caller
    /// decides whether to issue JWT).
    async fn authenticate_password(
        &self,
        tenant_id: TenantId,
        username: &str,
        password: &SecretString,
    ) -> Result<AuthorizedContext, SecurityError>;

    /// Verify a JWT access token and rebuild the [`AuthorizedContext`].
    async fn authenticate_token(&self, token: &str) -> Result<AuthorizedContext, SecurityError>;

    /// Issue a token pair (access + refresh) for an already-authenticated
    /// principal.
    async fn issue_token_pair(
        &self,
        context: &AuthorizedContext,
        ttl_seconds: i64,
    ) -> Result<TokenPair, SecurityError>;

    /// Revoke a refresh token (logout).
    async fn revoke_refresh_token(&self, token: &SecretString) -> Result<(), SecurityError>;

    /// Check whether a refresh token has been revoked.
    async fn is_refresh_token_revoked(&self, token: &SecretString) -> Result<bool, SecurityError>;
}

/// In-memory authentication service for development / testing. Production
/// implementations should back onto a real user / credential store.
pub struct InMemoryAuthService {
    pub keys: JwtKeys,
    // username -> (tenant_id, argon2 hash)
    pub users: HashMap<String, (TenantId, String)>,
    // refresh token (argon2 hash) -> revoked flag
    pub revoked_refresh_tokens: Mutex<HashSet<String>>,
}

impl InMemoryAuthService {
    pub fn new() -> Result<Self, SecurityError> {
        Ok(Self {
            keys: JwtKeys::generate()?,
            users: HashMap::new(),
            revoked_refresh_tokens: Mutex::new(HashSet::new()),
        })
    }

    pub fn register_user(
        &mut self,
        username: &str,
        tenant_id: TenantId,
        password: &SecretString,
    ) -> Result<(), SecurityError> {
        let hash = crate::password::hash_password(password)?;
        self.users.insert(username.to_string(), (tenant_id, hash));
        Ok(())
    }
}

#[async_trait]
impl AuthenticationService for InMemoryAuthService {
    async fn authenticate_password(
        &self,
        tenant_id: TenantId,
        username: &str,
        password: &SecretString,
    ) -> Result<AuthorizedContext, SecurityError> {
        let (stored_tenant, hash) = self
            .users
            .get(username)
            .ok_or(SecurityError::InvalidCredentials)?;
        if *stored_tenant != tenant_id {
            return Err(SecurityError::AccessDenied {
                detail: "user does not belong to tenant".into(),
            });
        }
        let ok = crate::password::verify_password(password, hash)?;
        if !ok {
            return Err(SecurityError::InvalidCredentials);
        }
        Ok(AuthorizedContext::new(
            tenant_id,
            Principal {
                app_user_id: AppUserId::new(1), // placeholder; real impl resolves from user store
                username: username.to_string(),
                roles: Default::default(),
                permissions: Default::default(),
                office_id: None,
            },
            SessionId::new(),
            chrono::Utc::now(),
        ))
    }

    async fn authenticate_token(&self, token: &str) -> Result<AuthorizedContext, SecurityError> {
        let claims = crate::tokens::verify_access_token(&self.keys, token)?;
        let _tenant_id = TenantId::new(claims.tenant);
        Ok(crate::tokens::context_from_claims(
            &claims,
            SessionId::new(),
        ))
    }

    async fn issue_token_pair(
        &self,
        context: &AuthorizedContext,
        ttl_seconds: i64,
    ) -> Result<TokenPair, SecurityError> {
        let principal = crate::context::Principal {
            app_user_id: context.principal.app_user_id,
            username: context.principal.username.clone(),
            roles: context.principal.roles.clone(),
            permissions: context.principal.permissions.clone(),
            office_id: context.principal.office_id,
        };
        crate::tokens::issue_tokens(&self.keys, &principal, context.tenant_id, ttl_seconds)
    }

    async fn revoke_refresh_token(&self, token: &SecretString) -> Result<(), SecurityError> {
        self.revoked_refresh_tokens
            .lock()
            .unwrap()
            .insert(token.expose_secret().clone());
        Ok(())
    }

    async fn is_refresh_token_revoked(&self, token: &SecretString) -> Result<bool, SecurityError> {
        Ok(self
            .revoked_refresh_tokens
            .lock()
            .unwrap()
            .contains(token.expose_secret()))
    }
}
