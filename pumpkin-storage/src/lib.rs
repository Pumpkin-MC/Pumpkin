//! Storage abstractions for Pumpkin.
//!
//! Each persistent-data domain (world info, player data, chunks, POI, server
//! configs) gets its own trait defined in this crate. Implementations are
//! provided by [`VanillaStorage`] (filesystem, vanilla-compatible layout) and
//! by [`MemoryStorage`] (ephemeral, format-agnostic).
//!
//! Native `async fn` in traits is not dyn-compatible, so each domain trait
//! returns [`BoxFuture`] instead of declaring `async fn` directly.

use std::pin::Pin;

pub mod error;
pub mod world_info;

mod memory;
mod vanilla;

pub use error::StorageError;
pub use memory::MemoryStorage;
pub use vanilla::VanillaStorage;

#[cfg(test)]
mod conformance_tests;

/// Boxed `Send` future returned from dyn-compatible storage trait methods.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
