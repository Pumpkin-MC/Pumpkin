//! # pumpkin-store — Pluggable Game Data Storage
//!
//! Abstracts game data access behind the [`GameDataStore`] trait.
//!
//! ## Single-Flip Separation of Concerns
//!
//! [`StoreProvider`] is the **single control point** for selecting behavior.
//! Community default is `StoreProvider::Static` — zero cost, zero extended features.
//! One flip to `Cached` or `Lance` activates extended capabilities. Everything
//! flows through `StoreProvider::open()` → `Box<dyn GameDataStore>`.
//!
//! ```text
//! StoreProvider::open()            ← single flip point
//!      │
//!      ├── Static  (default)       ← community path: pumpkin-data only
//!      │   Zero cost, no cache, no XOR, no overlays.
//!      │   Identical to using pumpkin-data directly.
//!      │
//!      ├── Cached                  ← extended: HashMap + XOR zero-copy guard
//!      │   Transparent CacheEntry DTOs, debug_assert XOR verification.
//!      │
//!      └── Lance                   ← extended: Arrow columnar, zero-copy
//!          Hydrated from Static, lance 2.0 native queries.
//! ```
//!
//! ## Core API (all tiers)
//!
//! ```rust,ignore
//! use pumpkin_store::{StoreProvider, GameDataStore};
//!
//! let store = StoreProvider::default().open(); // Static — community default
//! let block = store.block_by_name("stone")?;
//! let item  = store.item_by_name("diamond_sword")?;
//! ```
//!
//! ## Extended API (Cached/Lance tiers)
//!
//! The [`overlay`] module contains spatial types used by extended tiers.
//! These are always compiled (zero deps) but only active when the provider
//! is flipped away from `Static`.
//!
//! ```rust,ignore
//! use pumpkin_store::StoreProvider;
//! use pumpkin_store::overlay::{SpatialOverlay, MobGoalState};
//!
//! let store = StoreProvider::Cached.open(); // single flip → extended
//! let mut world_map = SpatialOverlay::new();
//! world_map.bind(100, 64, -200);
//! ```

mod error;
mod traits;

mod cached_store;

#[cfg(feature = "toml-store")]
mod static_store;

#[cfg(feature = "lance-store")]
mod lance_store;

pub use error::{StoreError, StoreResult};

// ── Core API (community path) ─────────────────────────────────────────
// These types are used by all tiers including Static (community default).
pub use traits::{BlockRecord, EntityRecord, GameDataStore, GameMappingRecord, ItemRecord, RecipeRecord};

// ── Extended API (Cached/Lance tiers) ──────────────────────────────────
// Spatial overlays, XOR guards, and goal state encoding. Always compiled
// (zero deps) but only _active_ when StoreProvider is flipped from Static.
// Community default path never touches these.
pub mod overlay {
    pub use crate::traits::{
        MobGoalState, SpatialOverlay, ZeroCopyGuard, OVERLAY_BITS, OVERLAY_WORDS, XOR_SENTINEL,
    };
}

pub use cached_store::{CacheEntry, CacheSnapshot, CachedStore};

#[cfg(feature = "toml-store")]
pub use static_store::StaticStore;

#[cfg(feature = "lance-store")]
pub use lance_store::LanceStore;

/// Store provider tier — meta-switch for transparent backend routing.
///
/// Works like a NAT: callers get `Box<dyn GameDataStore>` from [`open`](Self::open)
/// and call methods without knowing which backend handles them. The provider
/// transparently routes all commands to the selected tier:
///
/// ```text
/// StoreProvider::open()
///      │
///      ├── Static  → StaticStore (pumpkin-data, compile-time, zero cost)
///      ├── Cached  → CachedStore<StaticStore> (HashMap memoization + XOR guard)
///      └── Lance   → LanceStore (Arrow zero-copy, lance 2.0 native queries)
/// ```
///
/// All tiers implement the same `GameDataStore` trait — including additive methods
/// like `game_mappings()` that don't conflict with existing block/item/entity/recipe
/// lookups. Static returns empty for relationship queries; higher tiers populate them.
///
/// # Examples
///
/// ```rust,ignore
/// use pumpkin_store::StoreProvider;
///
/// // Default: compile-time static data, zero cost
/// let store = StoreProvider::default().open();
/// let block = store.block_by_name("stone")?;
///
/// // Cached: HashMap memoization + XOR zero-copy guard
/// let store = StoreProvider::Cached.open();
/// let block = store.block_by_name("stone")?; // delegate + cache
/// let block = store.block_by_name("stone")?; // instant O(1) hit
///
/// // Lance: async construction (use LanceStore::open() directly)
/// let mut lance = LanceStore::open("./data/lance").await?;
/// lance.hydrate_from(&*StoreProvider::default().open()).await?;
/// ```
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum StoreProvider {
    /// `StaticStore` — zero-cost, compile-time pumpkin-data lookups.
    #[default]
    Static,
    /// `CachedStore<StaticStore>` — lazy `HashMap` + XOR write-through guard.
    Cached,
    /// `LanceStore` — Arrow columnar, hydrated from Static. Requires async.
    Lance,
}

impl StoreProvider {
    /// Open a store with this provider tier.
    ///
    /// Returns a `Box<dyn GameDataStore>` that transparently routes all commands
    /// to the selected backend — the caller doesn't know or care which tier handles it.
    ///
    /// # Panics
    ///
    /// Panics if `Lance` is selected — Lance requires async construction.
    /// Use `LanceStore::open()` directly for the Lance tier.
    #[cfg(feature = "toml-store")]
    #[must_use]
    pub fn open(self) -> Box<dyn GameDataStore> {
        match self {
            Self::Static => Box::new(StaticStore::new()),
            Self::Cached => Box::new(CachedStore::new(StaticStore::new())),
            Self::Lance => {
                panic!("Lance tier requires async — use LanceStore::open() directly")
            }
        }
    }
}

/// Open the default store based on enabled features.
///
/// Returns `StaticStore` wrapping pumpkin-data — zero-cost, always available.
/// For tier selection via meta-switch, use [`StoreProvider::open`] instead.
///
/// ```rust,ignore
/// // Direct construction
/// let store = open_default_store();
///
/// // Meta-switch (equivalent, returns Box<dyn GameDataStore>)
/// let store = StoreProvider::default().open();
/// ```
#[cfg(feature = "toml-store")]
#[must_use]
pub const fn open_default_store() -> StaticStore {
    StaticStore::new()
}

#[cfg(test)]
#[cfg(feature = "toml-store")]
mod tests {
    use super::*;

    #[test]
    fn provider_static_routes_block_lookup() {
        let store = StoreProvider::Static.open();
        let block = store.block_by_name("stone").unwrap();
        assert_eq!(block.name, "stone");
    }

    #[test]
    fn provider_cached_routes_block_lookup() {
        let store = StoreProvider::Cached.open();
        let block = store.block_by_name("stone").unwrap();
        assert_eq!(block.name, "stone");

        // Second call hits cache — same result, transparent to caller
        let block2 = store.block_by_name("stone").unwrap();
        assert_eq!(block.id, block2.id);
    }

    #[test]
    fn provider_default_is_static() {
        assert_eq!(StoreProvider::default(), StoreProvider::Static);
    }

    #[test]
    #[should_panic(expected = "Lance tier requires async")]
    fn provider_lance_panics_sync() {
        let _store = StoreProvider::Lance.open();
    }

    #[test]
    fn provider_additive_methods_available() {
        // Additive methods (game_mappings) are available on all tiers
        // without conflicting with existing block/item/entity lookups.
        let store = StoreProvider::Static.open();
        let mappings = store.game_mappings("biome", "plains").unwrap();
        assert!(mappings.is_empty(), "Static tier returns empty game_mappings");

        let count = store.game_mapping_count();
        assert_eq!(count, 0);

        // Same methods accessible through Cached tier
        let cached = StoreProvider::Cached.open();
        let mappings = cached.game_mappings("biome", "plains").unwrap();
        assert!(mappings.is_empty());
    }

    #[test]
    fn provider_routes_item_and_entity() {
        let store = StoreProvider::Cached.open();
        let item = store.item_by_name("diamond_sword").unwrap();
        assert_eq!(item.name, "diamond_sword");

        let entity = store.entity_by_name("zombie").unwrap();
        assert_eq!(entity.name, "zombie");
        assert!(entity.is_mob);
    }
}
