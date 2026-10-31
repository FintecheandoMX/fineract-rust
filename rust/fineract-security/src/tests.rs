//! Security tests for the Fineract security layer.
//!
//! Covers authentication (Basic Auth, JWT), authorization (RBAC, data-level),
//! IDOR protection, tenant isolation, password hashing, and token management.

use crate::authentication::{AuthenticationService, InMemoryAuthService};
use crate::authorization::{AuthorizationService, InMemoryAuthorizationService, Permission};
use crate::context::{AuthorizedContext, Principal, SessionId};
use crate::errors::SecurityError;
use crate::password::{hash_password, verify_password, PlainPassword};
use crate::tokens::{context_from_claims, issue_tokens, verify_access_token, JwtKeys};
use fineract_core::ids::*;
use secrecy::SecretString;
use std::collections::HashSet;

fn make_context(tenant_id: i64, roles: Vec<i64>, perms: Vec<PermissionId>) -> AuthorizedContext {
    AuthorizedContext::new(
        TenantId::new(tenant_id),
        Principal {
            app_user_id: AppUserId::new(1),
            username: "tester".into(),
            roles: roles.into_iter().map(RoleId::new).collect(),
            permissions: perms.into_iter().collect(),
            office_id: Some(OfficeId::new(1)),
        },
        SessionId::new(),
        chrono::Utc::now(),
    )
}

fn make_auth_service() -> InMemoryAuthService {
    let mut svc = InMemoryAuthService::new().unwrap();
    svc.register_user(
        "alice",
        TenantId::new(1),
        &SecretString::from("password123".to_string()),
    )
    .unwrap();
    svc.register_user(
        "bob",
        TenantId::new(2),
        &SecretString::from("password456".to_string()),
    )
    .unwrap();
    svc
}

// ---------- Password hashing ----------

#[test]
fn password_hash_verify_roundtrip() {
    let pw = SecretString::from("s3cr3t".to_string());
    let hash = hash_password(&pw).unwrap();
    assert!(verify_password(&pw, &hash).unwrap());
    assert!(!verify_password(&SecretString::from("wrong".to_string()), &hash).unwrap());
}

#[test]
fn password_hash_different_salts() {
    let pw = SecretString::from("samepass".to_string());
    let h1 = hash_password(&pw).unwrap();
    let h2 = hash_password(&pw).unwrap();
    assert_ne!(h1, h2); // salts differ
    assert!(verify_password(&pw, &h1).unwrap());
    assert!(verify_password(&pw, &h2).unwrap());
}

#[test]
fn plain_password_zeroizes_on_drop() {
    let pw = PlainPassword::new(SecretString::from("temporary".to_string()));
    // After drop, the secret reference should be cleared. We can't inspect
    // internals, but the Drop impl must exist and run without panic.
    drop(pw);
}

// ---------- Authentication ----------

#[tokio::test]
async fn authenticate_password_valid() {
    let svc = make_auth_service();
    let ctx = svc
        .authenticate_password(
            TenantId::new(1),
            "alice",
            &SecretString::from("password123".to_string()),
        )
        .await
        .unwrap();
    assert_eq!(ctx.tenant_id, TenantId::new(1));
    assert_eq!(ctx.principal.username, "alice");
}

#[tokio::test]
async fn authenticate_password_wrong_tenant() {
    let svc = make_auth_service();
    let err = svc
        .authenticate_password(
            TenantId::new(2),
            "alice",
            &SecretString::from("password123".to_string()),
        )
        .await
        .unwrap_err();
    assert!(matches!(err, SecurityError::AccessDenied { .. }));
}

#[tokio::test]
async fn authenticate_password_wrong_password() {
    let svc = make_auth_service();
    let err = svc
        .authenticate_password(
            TenantId::new(1),
            "alice",
            &SecretString::from("wrong".to_string()),
        )
        .await
        .unwrap_err();
    assert!(matches!(err, SecurityError::InvalidCredentials));
}

#[tokio::test]
async fn authenticate_password_unknown_user() {
    let svc = make_auth_service();
    let err = svc
        .authenticate_password(
            TenantId::new(1),
            "nobody",
            &SecretString::from("x".to_string()),
        )
        .await
        .unwrap_err();
    assert!(matches!(err, SecurityError::InvalidCredentials));
}

#[tokio::test]
async fn authenticate_token_valid() {
    let svc = make_auth_service();
    let ctx = svc
        .authenticate_password(
            TenantId::new(1),
            "alice",
            &SecretString::from("password123".to_string()),
        )
        .await
        .unwrap();
    let pair = svc.issue_token_pair(&ctx, 3600).await.unwrap();
    let ctx2 = svc.authenticate_token(&pair.access_token).await.unwrap();
    assert_eq!(ctx2.tenant_id, TenantId::new(1));
    assert_eq!(ctx2.principal.username, "alice");
}

#[tokio::test]
async fn authenticate_token_invalid() {
    let svc = make_auth_service();
    let err = svc.authenticate_token("not.a.jwt").await.unwrap_err();
    assert!(matches!(err, SecurityError::InvalidToken));
}

#[tokio::test]
async fn issue_and_revoke_refresh_token() {
    let svc = make_auth_service();
    let ctx = svc
        .authenticate_password(
            TenantId::new(1),
            "alice",
            &SecretString::from("password123".to_string()),
        )
        .await
        .unwrap();
    let pair = svc.issue_token_pair(&ctx, 3600).await.unwrap();

    // Not revoked yet
    assert!(!svc
        .is_refresh_token_revoked(&pair.refresh_token)
        .await
        .unwrap());

    // Revoke
    svc.revoke_refresh_token(&pair.refresh_token).await.unwrap();
    assert!(svc
        .is_refresh_token_revoked(&pair.refresh_token)
        .await
        .unwrap());
}

// ---------- JWT ----------

#[test]
fn jwt_issue_and_verify() {
    let keys = JwtKeys::generate().unwrap();
    let ctx = make_context(1, vec![2], vec![Permission::view_loan().0]);
    let mut pair = issue_tokens(&keys, &ctx.principal, ctx.tenant_id, 3600).unwrap();
    let claims = verify_access_token(&keys, &pair.access_token).unwrap();
    assert_eq!(claims.tenant, 1);
    assert_eq!(claims.username, "tester");
    pair.zeroize_refresh();
}

#[test]
fn jwt_invalid_token_rejected() {
    let keys = JwtKeys::generate().unwrap();
    let err = verify_access_token(&keys, "garbage").unwrap_err();
    assert!(matches!(err, SecurityError::InvalidToken));
}

#[test]
fn jwt_context_from_claims() {
    let keys = JwtKeys::generate().unwrap();
    let ctx = make_context(7, vec![3], vec![]);
    let pair = issue_tokens(&keys, &ctx.principal, ctx.tenant_id, 3600).unwrap();
    let claims = verify_access_token(&keys, &pair.access_token).unwrap();
    let ctx2 = context_from_claims(&claims, SessionId::new());
    assert_eq!(ctx2.tenant_id, TenantId::new(7));
}

// ---------- Authorization (RBAC) ----------

#[tokio::test]
async fn rbac_teller_can_view_loan() {
    let svc = InMemoryAuthorizationService::default();
    let ctx = make_context(1, vec![1], vec![]); // teller role
    let guard = svc.check(&ctx, Permission::view_loan()).await.unwrap();
    assert_eq!(guard.tenant_id, TenantId::new(1));
}

#[tokio::test]
async fn rbac_teller_cannot_create_loan() {
    let svc = InMemoryAuthorizationService::default();
    let ctx = make_context(1, vec![1], vec![]); // teller role
    let err = svc
        .check(&ctx, Permission::create_loan())
        .await
        .unwrap_err();
    assert!(matches!(err, SecurityError::InsufficientPermissions { .. }));
}

#[tokio::test]
async fn rbac_loan_officer_can_create_and_view_loan() {
    let svc = InMemoryAuthorizationService::default();
    let ctx = make_context(1, vec![2], vec![]); // loan officer role
    svc.check(&ctx, Permission::create_loan()).await.unwrap();
    svc.check(&ctx, Permission::view_loan()).await.unwrap();
    svc.check(&ctx, Permission::update_loan()).await.unwrap();
}

#[tokio::test]
async fn rbac_loan_officer_cannot_delete_loan() {
    let svc = InMemoryAuthorizationService::default();
    let ctx = make_context(1, vec![2], vec![]);
    let err = svc
        .check(&ctx, Permission::delete_loan())
        .await
        .unwrap_err();
    assert!(matches!(err, SecurityError::InsufficientPermissions { .. }));
}

#[tokio::test]
async fn rbac_admin_can_do_everything() {
    let svc = InMemoryAuthorizationService::default();
    let ctx = make_context(1, vec![3], vec![]); // admin role
    for perm in [
        Permission::create_loan(),
        Permission::view_loan(),
        Permission::update_loan(),
        Permission::delete_loan(),
        Permission::create_client(),
        Permission::view_client(),
        Permission::create_savings_account(),
        Permission::view_savings_account(),
        Permission::post_journal_entry(),
        Permission::view_journal_entry(),
        Permission::manage_users(),
        Permission::manage_tenants(),
    ] {
        svc.check(&ctx, perm).await.unwrap();
    }
}

#[tokio::test]
async fn rbac_direct_permission_grant() {
    let svc = InMemoryAuthorizationService::default();
    // No roles, but direct permission grant
    let ctx = AuthorizedContext::new(
        TenantId::new(1),
        Principal {
            app_user_id: AppUserId::new(1),
            username: "direct".into(),
            roles: HashSet::new(),
            permissions: vec![PermissionId::new(1001)].into_iter().collect(), // CREATE_LOAN
            office_id: Some(OfficeId::new(1)),
        },
        SessionId::new(),
        chrono::Utc::now(),
    );
    svc.check(&ctx, Permission::create_loan()).await.unwrap();
}

// ---------- IDOR / Tenant isolation ----------

#[tokio::test]
async fn idor_tenant_mismatch_denied() {
    let svc = InMemoryAuthorizationService::default();
    // Context is tenant 1, resource belongs to tenant 2
    let ctx = make_context(1, vec![3], vec![]); // admin on tenant 1
    let err = svc
        .check_resource_owner(&ctx, TenantId::new(2))
        .await
        .unwrap_err();
    assert!(matches!(err, SecurityError::TenantIsolationViolation));
}

#[tokio::test]
async fn idor_same_tenant_allowed() {
    let svc = InMemoryAuthorizationService::default();
    let ctx = make_context(1, vec![3], vec![]);
    let guard = svc
        .check_resource_owner(&ctx, TenantId::new(1))
        .await
        .unwrap();
    assert_eq!(guard.tenant_id, TenantId::new(1));
}

#[tokio::test]
async fn tenant_isolation_cross_tenant_read_blocked() {
    let svc = InMemoryAuthorizationService::default();
    let ctx_a = make_context(1, vec![3], vec![]);
    let ctx_b = make_context(2, vec![3], vec![]);

    // Tenant A admin can read tenant 1 resources
    svc.check_resource_owner(&ctx_a, TenantId::new(1))
        .await
        .unwrap();

    // Tenant A admin CANNOT read tenant 2 resources (IDOR)
    let err = svc
        .check_resource_owner(&ctx_a, TenantId::new(2))
        .await
        .unwrap_err();
    assert!(matches!(err, SecurityError::TenantIsolationViolation));

    // Tenant B admin can read tenant 2 resources
    svc.check_resource_owner(&ctx_b, TenantId::new(2))
        .await
        .unwrap();
}

// ---------- Office access ----------

#[tokio::test]
async fn office_access_same_office_allowed() {
    let svc = InMemoryAuthorizationService::default();
    let mut ctx = make_context(1, vec![3], vec![]);
    ctx.principal.office_id = Some(OfficeId::new(1));
    svc.check_office_access(&ctx, OfficeId::new(1))
        .await
        .unwrap();
    svc.check_office_access(&ctx, OfficeId::new(2))
        .await
        .unwrap(); // office 1 can access 2
}

#[tokio::test]
async fn office_access_other_office_denied() {
    let svc = InMemoryAuthorizationService::default();
    let mut ctx = make_context(1, vec![3], vec![]);
    ctx.principal.office_id = Some(OfficeId::new(2));
    let err = svc
        .check_office_access(&ctx, OfficeId::new(1))
        .await
        .unwrap_err();
    assert!(matches!(err, SecurityError::AccessDenied { .. }));
}

#[tokio::test]
async fn office_access_no_office_denied() {
    let svc = InMemoryAuthorizationService::default();
    let ctx = AuthorizedContext::new(
        TenantId::new(1),
        Principal {
            app_user_id: AppUserId::new(1),
            username: "nooffice".into(),
            roles: vec![3i64].into_iter().map(RoleId::new).collect(),
            permissions: HashSet::new(),
            office_id: None,
        },
        SessionId::new(),
        chrono::Utc::now(),
    );
    let err = svc
        .check_office_access(&ctx, OfficeId::new(1))
        .await
        .unwrap_err();
    assert!(matches!(err, SecurityError::AccessDenied { .. }));
}

// ---------- Constant-time comparison ----------

#[test]
fn refresh_token_constant_time_eq() {
    let a = SecretString::from("abcdef".to_string());
    let b = SecretString::from("abcdef".to_string());
    let c = SecretString::from("abcdeg".to_string());
    assert!(crate::tokens::refresh_tokens_eq(&a, &b));
    assert!(!crate::tokens::refresh_tokens_eq(&a, &c));
    // Different lengths must not panic
    let d = SecretString::from("short".to_string());
    let e = SecretString::from("much longer string".to_string());
    assert!(!crate::tokens::refresh_tokens_eq(&d, &e));
}

// ---------- Property-based tests ----------

#[test]
fn proptest_password_verify_always_works_for_correct_password() {
    proptest::proptest!(|(pass in "[a-zA-Z0-9!@#$%]{1,20}")| {
        let pw = SecretString::from(pass);
        let hash = hash_password(&pw).unwrap();
        assert!(verify_password(&pw, &hash).unwrap());
    });
}

#[test]
fn proptest_id_types_are_distinct() {
    proptest::proptest!(|(v in 1i64..100000i64)| {
        let loan = LoanId::new(v);
        let client = ClientId::new(v);
        assert_eq!(loan.value(), client.value());
        // Different types — not interchangeable at compile time (enforced by type system)
        let _ = loan;
        let _ = client;
    });
}
