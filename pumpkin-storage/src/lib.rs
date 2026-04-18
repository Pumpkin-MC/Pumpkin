//! Storage abstractions for Pumpkin.
//!
//! Each persistent-data domain (level info, player data, chunks, POI, server
//! configs) has its own trait defined in this crate. Implementations are
//! provided by the [`VanillaStorage`] struct (filesystem, vanilla-compatible
//! layout) and by [`MemoryStorage`] (ephemeral, format-agnostic).
//!
//! Domain traits and backends are added in subsequent commits.
