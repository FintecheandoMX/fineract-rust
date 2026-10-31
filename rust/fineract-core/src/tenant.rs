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

//! Multi-tenancy context.
//!
//! Every data access and state-changing operation must hold an explicit
//! [`TenantContext`]. It is impossible to construct a repository or domain
//! service without one, which makes tenant leakage a compile-time problem
//! rather than a runtime hope.

use crate::ids::TenantId;

/// The tenant that owns the current request / unit of work.
///
/// Carried through the call stack as an explicit parameter (or via an
/// extractor at the API boundary) and never read from ambient / global state
/// that a caller could forget to set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TenantContext {
    pub tenant_id: TenantId,
}

impl TenantContext {
    pub fn new(tenant_id: TenantId) -> Self {
        Self { tenant_id }
    }
}
