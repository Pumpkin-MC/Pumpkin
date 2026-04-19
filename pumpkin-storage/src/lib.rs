//! Storage abstractions for Pumpkin.
//!
//! Each persistent-data domain (level info, player data, chunks, POI, server
//! configs) has its own trait defined in this crate. Implementations are
//! provided by [`VanillaStorage`] (filesystem, vanilla-compatible layout) and
//! by [`MemoryStorage`] (ephemeral, format-agnostic).
//!
//! Additional domain traits are added in subsequent commits.

pub mod banlist;
pub mod error;
pub mod level_info;
pub mod player_data;
pub mod user_cache;

mod memory;
mod null;
mod vanilla;

pub use error::StorageError;
pub use memory::MemoryStorage;
pub use null::NullStorage;
pub use vanilla::VanillaStorage;

#[cfg(test)]
mod conformance_tests;
