//! # oxysf-core
//!
//! The engine behind oxy-sf, a fast native reimplementation of the commonly
//! used parts of the Salesforce CLI (`sf`).
//!
//! This crate provides authentication flows (OAuth web-server with PKCE, JWT
//! bearer, refresh-token), credential storage, an authenticated [`Connection`],
//! REST helpers, configuration resolution, output rendering, and threading
//! utilities. It contains no async/await: networking uses `reqwest::blocking`.
//!
//! oxy-sf is an independent project and is not affiliated with or endorsed by
//! Salesforce.
//!
//! [`Connection`]: connection::Connection

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod api;
pub mod auth;
pub mod config;
pub mod connection;
pub mod context;
pub mod error;
pub mod output;
pub mod threading;

pub use connection::Connection;
pub use context::CommandContext;
pub use error::{Result, SfError};
pub use output::{Envelope, OutputMode};
