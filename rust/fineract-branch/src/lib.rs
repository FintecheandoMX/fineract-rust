//! Branch (office) management domain module.
//!
//! Branches (offices) form the organisational hierarchy in Fineract. Each
//! branch belongs to a tenant and may have a parent branch, forming a tree.
//! All data access is tenant-scoped and authorization-checked by construction.

#![forbid(unsafe_code)]
#![allow(async_fn_in_trait)]

mod command;
mod domain;
mod repository;
mod service;

pub use command::*;
pub use domain::*;
pub use repository::*;
pub use service::*;
