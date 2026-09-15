//! Authentication and HTTP client utilities for Pumpkin.

#![deny(clippy::unwrap_used)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::panic))]

pub mod client;
pub mod jwt;

pub use client::{client, client_builder};
pub use p384;
