// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements. See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership. The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. See the License for the
// specific language governing permissions and limitations
// under the License.

//! Hierarchical domain error type.
//!
//! Errors are categorized so the API layer can map them to appropriate HTTP
//! status codes without leaking internal details to clients.

use thiserror::Error;

/// Domain-level error. All variants carry enough context for logging and
/// observability, but never secrets, raw SQL, or stack traces.
#[derive(Debug, Error)]
pub enum DomainError {
    #[error("not found: {entity} {id}")]
    NotFound { entity: String, id: String },

    #[error("already exists: {entity} {id}")]
    AlreadyExists { entity: String, id: String },

    #[error("invalid identifier for {entity}: value {value}")]
    InvalidIdentifier { entity: String, value: i64 },

    #[error("validation failed: {field} — {reason}")]
    Validation { field: String, reason: String },

    #[error("business rule violated: {rule}")]
    BusinessRule { rule: String },

    #[error("insufficient funds for account {account}")]
    InsufficientFunds { account: String },

    #[error("conflict: {detail}")]
    Conflict { detail: String },

    #[error("unauthorized: {detail}")]
    Unauthorized { detail: String },

    #[error("forbidden: {detail}")]
    Forbidden { detail: String },

    #[error("tenant context missing")]
    TenantMissing,

    #[error("tenant isolation violation")]
    TenantIsolationViolation,

    #[error("idempotency key already processed with a different payload")]
    IdempotencyConflict,

    #[error("internal: {detail}")]
    Internal { detail: String },
}
