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

//! Foundational types for the Fineract Rust migration.
//!
//! This crate contains the domain primitives that every other crate depends on:
//! strong typed identifiers, monetary amounts, date/time value objects, the
//! hierarchical error type, and the multi-tenancy / authorization context.
//!
//! Security: this crate is compiled with `#![forbid(unsafe_code)]`.

#![forbid(unsafe_code)]

pub mod error;
pub mod ids;
pub mod money;
pub mod tenant;

#[cfg(test)]
mod tests;
