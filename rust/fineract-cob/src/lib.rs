//! Close-of-business / scheduled job engine.
//!
//! Phase 6 implements a reliable, restartable, partitioned job framework
//! built on Tokio. All jobs are tenant-scoped and authorization-checked,
//! enforcing multi-tenancy isolation even for scheduled batch operations.

#![forbid(unsafe_code)]
#![allow(async_fn_in_trait)]

pub mod command;
mod domain;
mod engine;
mod repository;
mod scheduler;
mod service;
#[cfg(test)]
mod tests;
mod worker;

pub use command::*;
pub use domain::*;
pub use engine::*;
pub use repository::*;
pub use scheduler::*;
pub use service::*;
pub use worker::*;
