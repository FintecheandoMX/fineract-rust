# Fineract Rust Migration — Security Threat Model

**Scope:** IDOR, SQL injection, XSS, and multi-tenant isolation in the Rust
rewrite of Apache Fineract. **Status:** Phase 0 foundation.

---

## 1. Threat model assumptions

- Primary database: PostgreSQL (one pool per tenant / schema-based isolation).
- All data access is parameterized via `sqlx`; no dynamic SQL.
- All identifiers are strong newtypes (`LoanId`, `ClientId`, `OfficeId`,
  `TenantId`, …), never raw `i64` across module boundaries.
- Every request carries an explicit `AuthorizedContext` (tenant + authenticated
  user + roles/permissions). There is no ambient/ThreadLocal authority.
- `#![forbid(unsafe_code)]` on all crates; any future `unsafe` requires a
  documented SAFETY justification and audit trail.
- Secrets use `secrecy` + `zeroize`; comparisons use `subtle` (constant time);
  randomness uses `rand` CSPRNG.

---

## 2. IDOR (Insecure Direct Object Reference)

### Threat
An attacker supplies a resource ID (loan, client, account, payment) belonging to
another client/office/tenant and reads or mutates it.

### Countermeasures (by construction)
- Typed IDs: a `LoanId` cannot be passed where a `ClientId` is expected.
- `AuthorizedContext` is an explicit parameter on every domain service and
  repository; it cannot be forgotten.
- The `AuthorizationService` checks ownership/office/tenant/role before any
  data access, at the domain service layer (not only the API layer).
- Client-supplied IDs are validated at the boundary
  (`fineract-validation::validate_id`) and rejected if non-positive or
  malformed; never coerced.

### Required tests (every data-touching phase)
- IDOR: access resource with a swapped/guessed ID from another client/tenant
  → must be denied.
- Negative: unauthenticated call → denied.

---

## 3. SQL Injection

### Threat
Attacker-controlled input injected into a SQL query to read/modify data or
bypass checks.

### Countermeasures (by construction)
- **Zero string concatenation or `format!` for SQL.** Only parameterized
  `sqlx` queries (`query`, `query_as`, `query!`, `query_as!`).
- `sqlx` compile-time-checked macros where possible (Phase 4+).
- Typed ID newtypes encode values as `i64` parameters — never interpolated.
- Database users run with least privilege; statement timeouts enabled.

### Required tests
- Injection attempts: special characters, quotes, semicolons, `UNION SELECT`
  in string fields → must be treated as literal data, never executed.

---

## 4. Cross-Site Scripting (XSS)

### Threat
Attacker-controlled content rendered/executed in a client context.

### Countermeasures (by construction)
- Prefer structured JSON over HTML.
- Any HTML output passes through a strict sanitizer (`ammonia` / `html-escape`).
- Strict Content-Security-Policy, `X-Content-Type-Options`, `X-Frame-Options`,
  `Referrer-Policy`, `Permissions-Policy`, `HSTS` headers on every response
  (tower-http middleware, Phase 7).
- Never reflect raw user input into response headers.

---

## 5. Multi-tenant isolation

### Threat
Tenant A's data leaked to Tenant B (via query, cache, shared connection pool,
or event).

### Countermeasures (by construction)
- `TenantContext` (tenant_id) is explicit and mandatory; no ambient tenant.
- Tenant-aware connection pools (one per tenant) or schema-based isolation.
- Every query filters by `tenant_id` by default at the repository layer.
- Cross-tenant access prohibited even for "super user" paths without explicit,
  audited elevation.
- Tenant identifier never trusted from client headers without verification.

### Required tests
- Tenant isolation positive + negative (Tenant A must never see Tenant B's rows,
  including via guessed IDs).

---

## 6. Additional mandatory controls

| Control | Implementation |
|---|---|
| Authentication | Basic Auth, OAuth2/JWT, 2FA hooks (Phase 2) |
| Authorization | RBAC + data-level permissions, `AuthorizationService` |
| Passwords | `argon2` hashing, `secrecy`/`zeroize` |
| Tokens | `jsonwebtoken` + `subtle` constant-time comparison |
| Secrets | `secrecy` (`SecretString`), never logged/displayed |
| Random values | `rand` CSPRNG |
| Error handling | `thiserror`; no `unwrap`/`expect` in production; no internal details leaked |
| Logging | `tracing` structured, secrets/PII redacted |
| Supply chain | `cargo audit` + `cargo deny` on every CI |

---

## 7. Known dependency exception (documented)

**RUSTSEC-2023-0071 — rsa 0.9.10 "Marvin Attack" (medium, no fix available).**

- `rsa` is a **lockfile-only orphan**: `cargo tree -i rsa` returns no
  dependents in this project's build graph.
- This project is **PostgreSQL-only**; `rsa` is reachable only through the
  unused `sqlx-mysql` backend and is **never compiled or linked**.
- `cargo audit` is run with `--ignore RUSTSEC-2023-0071` and this decision is
  recorded here. Revisit if MySQL support is added or a fixed `rsa` ships.

---

## 8. Required security tests (every relevant phase)

- IDOR attempts (swapped IDs across client/office/tenant)
- Authorization matrix (roles × actions, allowed vs denied)
- Tenant isolation violations (negative tests)
- Invalid/malicious input (malformed JSON, extra fields, oversized payloads)
- Injection attempts (special characters, SQL/JavaScript payloads)
- Privilege escalation attempts

Tests must fail loudly if any security property is broken.
