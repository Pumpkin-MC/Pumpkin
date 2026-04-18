//! Storage abstractions for Pumpkin.
//!
//! Each persistent-data domain (level info, player data, chunks, POI, server
//! configs) has its own trait defined in this crate. Implementations are
//! provided by [`VanillaStorage`] (filesystem, vanilla-compatible layout) and
//! by [`MemoryStorage`] (ephemeral, format-agnostic).
//!
//! Domain traits are added in subsequent commits.

mod memory;
mod vanilla;

pub use memory::MemoryStorage;
pub use vanilla::VanillaStorage;
