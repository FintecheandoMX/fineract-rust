//! Client and group management domain module.
//!
//! Clients (individuals and groups), client hierarchy, and group management.

#![forbid(unsafe_code)]
#![allow(async_fn_in_trait)]

pub mod command;
pub mod domain;
pub mod repository;
pub mod service;

pub use command::*;
pub use domain::*;
pub use repository::*;
pub use service::*;
