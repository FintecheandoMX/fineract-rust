//! Working capital loan management domain module.
//! Revolving working capital loans with dynamic drawdown.

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
