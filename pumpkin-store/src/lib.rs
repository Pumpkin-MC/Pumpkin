//! # pumpkin-store — Pluggable Game Data Storage
//!
//! Abstracts game data access behind the [`GameDataStore`] trait with three tiers:
//!
//! ## Provider Tiers
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │              GameDataStore trait                             │
//! ├──────────────┬────────────────────┬─────────────────────────┤
//! │  StaticStore │   CachedStore      │     LanceStore          │
//! │  (default)   │   (cached)         │     (lance-store)       │
//! │  pumpkin-data│   Static+HashMap   │     hydrate from Static │
//! │  compile-time│   transparent DTOs │     Arrow zero-copy     │
//! │  zero-cost   │   O(1) hot path    │     lance 2.0 native    │
//! └──────────────┴────────────────────┴─────────────────────────┘
//! ```
//!
//! - **`StaticStore`** (default): Wraps `pumpkin-data` static arrays. Zero runtime cost,
//!   zero new dependencies. Developers see no difference from using pumpkin-data directly.
//!
//! - **`CachedStore`**: Wraps any store + adds `HashMap` memoization with transparent
//!   [`CacheEntry`] DTOs. Each entry records the lookup method, key, and value.
//!   No additional dependencies.
//!
//! - **`LanceStore`** (opt-in): Hydrates Lance tables FROM `StaticStore`, then serves
//!   zero-copy Arrow reads. Lance 2.0 provides native queries — no `DataFusion` sidecar.
//!   Enable with `--features lance-store`.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use pumpkin_store::{GameDataStore, open_default_store};
//!
//! let store = open_default_store();
//! let block = store.block_by_state_id(1);
//! let item = store.item_by_name("diamond_sword");
//! ```
//!
//! ## Cached Store
//!
//! ```rust,ignore
//! use pumpkin_store::{CachedStore, StaticStore};
//!
//! let cached = CachedStore::new(StaticStore::new());
//! let block = cached.block_by_name("stone"); // delegate + cache
//! let block = cached.block_by_name("stone"); // instant O(1) hit
//! let snap = cached.snapshot();               // inspect cache state
//! ```
//!
//! ## Lance Store (hydration from static)
//!
//! ```rust,ignore
//! use pumpkin_store::{StaticStore, LanceStore};
//!
//! let static_store = StaticStore::new();
//! let mut lance = LanceStore::open("./data/lance").await?;
//! lance.hydrate_from(&static_store).await?;  // one-time, zero-copy Arrow tables
//! ```
//!
//! ## Future: GEL (Graph Execution Language)
//!
//! The Lance backend is the substrate for a future Graph Execution Language layer
//! that compiles Java imports (e.g. Bukkit API calls) to graph operations over
//! Arrow columnar storage. See ARCH-020 and holograph (AdaWorldAPI/holograph).

mod error;
mod traits;

mod cached_store;

#[cfg(feature = "toml-store")]
mod static_store;

#[cfg(feature = "lance-store")]
mod lance_store;

pub use error::{StoreError, StoreResult};
pub use traits::{BlockRecord, EntityRecord, GameDataStore, ItemRecord, RecipeRecord};

pub use cached_store::{CacheEntry, CacheSnapshot, CachedStore};

#[cfg(feature = "toml-store")]
pub use static_store::StaticStore;

#[cfg(feature = "lance-store")]
pub use lance_store::LanceStore;

/// Store provider tier — selects which backend to use at runtime.
///
/// ```text
/// Static  → compile-time pumpkin-data, zero cost (default)
/// Cached  → Static + HashMap memoization, transparent DTOs
/// Lance   → hydrated from Static, Arrow zero-copy, lance 2.0 native queries
/// ```
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum StoreProvider {
    /// Use `StaticStore` directly — zero-cost, compile-time data.
    #[default]
    Static,
    /// Use `CachedStore<StaticStore>` — lazy `HashMap` cache over static data.
    Cached,
    /// Use `LanceStore` — Arrow columnar, hydrated from static data.
    Lance,
}

/// Open the default store based on enabled features.
///
/// Returns `StaticStore` wrapping pumpkin-data — zero-cost, always available.
/// For cached or Lance tiers, construct directly:
///
/// ```rust,ignore
/// // Cached tier
/// let store = CachedStore::new(open_default_store());
///
/// // Lance tier
/// let mut lance = LanceStore::open("./data/lance").await?;
/// lance.hydrate_from(&open_default_store()).await?;
/// ```
#[cfg(feature = "toml-store")]
#[must_use]
pub const fn open_default_store() -> StaticStore {
    StaticStore::new()
}
