//! Caching layer over any [`GameDataStore`] backend.
//!
//! Wraps a delegate store and memoizes lookups in transparent [`CacheEntry`] DTOs.
//! Each entry records the lookup method, key, and the full record value — making
//! the cache inspectable for debugging and serialization.
//!
//! ## XOR Write-Through Guard (holograph pattern)
//!
//! Each [`CacheEntry`] carries an `xor_tag` computed at insertion time from the
//! `Cow` variant discriminant. On read-back, [`CacheEntry::verify_zero_copy`]
//! re-computes the tag and checks it matches. If an ephemeral variable switch
//! silently converted `Cow::Borrowed` to `Cow::Owned` (breaking zero-copy),
//! the XOR mismatch is detected immediately.
//!
//! This pattern is borrowed from `AdaWorldAPI/holograph` where it guards Arrow
//! IPC zero-copy buffers from accidental materialization.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use pumpkin_store::{CachedStore, StaticStore};
//!
//! let cached = CachedStore::new(StaticStore::new());
//! let block = cached.block_by_name("stone"); // delegate lookup + cache
//! let block = cached.block_by_name("stone"); // instant cache hit
//!
//! // Inspect cache contents
//! let snapshot = cached.snapshot();
//! println!("{} blocks cached, {} items cached", snapshot.blocks, snapshot.items);
//! ```
//!
//! No additional dependencies — just `std::collections::HashMap` with `RwLock`.

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::RwLock;

use serde::{Deserialize, Serialize};

use crate::error::StoreResult;
use crate::traits::{
    BlockRecord, EntityRecord, GameDataStore, ItemRecord, RecipeRecord, ZeroCopyGuard,
};

/// A transparent cache entry that records the lookup source and key.
///
/// Every cached value is wrapped in this DTO so the cache is inspectable:
/// you can see which method produced it, what key was used, and the full value.
///
/// ## XOR Write-Through Guard
///
/// The `xor_tag` field is computed at insertion time from the value's
/// [`ZeroCopyGuard::borrow_mask`]. On read-back, calling [`verify_zero_copy`]
/// re-computes the tag — if an ephemeral variable switch silently converted
/// a `Cow::Borrowed` to `Cow::Owned`, the mismatch is detected immediately.
///
/// [`verify_zero_copy`]: CacheEntry::verify_zero_copy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    /// The lookup method that produced this entry (e.g. `block_by_name`, `item_by_name`).
    pub method: Cow<'static, str>,
    /// The lookup key (e.g. "stone", `diamond_sword`, or a numeric ID as string).
    pub key: Cow<'static, str>,
    /// The cached value — the full record DTO from the delegate store.
    pub value: T,
    /// XOR tag computed at insertion: `borrow_mask ^ XOR_SENTINEL`.
    /// Used to detect zero-copy breach on read-back.
    pub xor_tag: u64,
}

impl<T: ZeroCopyGuard> CacheEntry<T> {
    /// Create a new cache entry with XOR tag computed from the value's borrow state.
    pub fn new(method: Cow<'static, str>, key: Cow<'static, str>, value: T) -> Self {
        let xor_tag = value.xor_tag();
        Self {
            method,
            key,
            value,
            xor_tag,
        }
    }

    /// Verify that zero-copy is intact: the value's current borrow state
    /// must match what was recorded at insertion time.
    ///
    /// Returns `true` if all `Cow::Borrowed` fields are still borrowed.
    /// Returns `false` if any field was silently switched to `Cow::Owned`.
    #[must_use]
    pub fn verify_zero_copy(&self) -> bool {
        self.value.verify_xor(self.xor_tag)
    }
}

/// Summary of current cache state — returned by [`CachedStore::snapshot`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheSnapshot {
    /// Number of cached block-by-id entries.
    pub blocks_by_id: usize,
    /// Number of cached block-by-name entries.
    pub blocks_by_name: usize,
    /// Number of cached block-by-state entries.
    pub blocks_by_state: usize,
    /// Number of cached item entries.
    pub items: usize,
    /// Number of cached entity entries.
    pub entities: usize,
    /// Total cached entries across all maps.
    pub total: usize,
}

/// A caching wrapper around any [`GameDataStore`].
///
/// Thread-safe via `RwLock<HashMap>`. Cache is populated lazily on first
/// access per key. The delegate store is called exactly once per unique key.
/// Each entry is a transparent [`CacheEntry`] DTO with method/key metadata.
pub struct CachedStore<S: GameDataStore> {
    delegate: S,
    blocks_by_id: RwLock<HashMap<u16, CacheEntry<BlockRecord>>>,
    blocks_by_name: RwLock<HashMap<String, CacheEntry<BlockRecord>>>,
    blocks_by_state: RwLock<HashMap<u16, CacheEntry<BlockRecord>>>,
    items_by_name: RwLock<HashMap<String, CacheEntry<ItemRecord>>>,
    entities_by_name: RwLock<HashMap<String, CacheEntry<EntityRecord>>>,
}

impl<S: GameDataStore> CachedStore<S> {
    /// Create a new cached store wrapping the given delegate.
    pub fn new(delegate: S) -> Self {
        Self {
            delegate,
            blocks_by_id: RwLock::new(HashMap::new()),
            blocks_by_name: RwLock::new(HashMap::new()),
            blocks_by_state: RwLock::new(HashMap::new()),
            items_by_name: RwLock::new(HashMap::new()),
            entities_by_name: RwLock::new(HashMap::new()),
        }
    }

    /// Return a reference to the underlying delegate store.
    pub const fn delegate(&self) -> &S {
        &self.delegate
    }

    /// Clear all cached entries. Next lookup will hit the delegate again.
    pub fn invalidate(&self) {
        self.blocks_by_id.write().unwrap().clear();
        self.blocks_by_name.write().unwrap().clear();
        self.blocks_by_state.write().unwrap().clear();
        self.items_by_name.write().unwrap().clear();
        self.entities_by_name.write().unwrap().clear();
    }

    /// Return a snapshot of the current cache size across all maps.
    pub fn snapshot(&self) -> CacheSnapshot {
        let blocks_by_id = self.blocks_by_id.read().unwrap().len();
        let blocks_by_name = self.blocks_by_name.read().unwrap().len();
        let blocks_by_state = self.blocks_by_state.read().unwrap().len();
        let items = self.items_by_name.read().unwrap().len();
        let entities = self.entities_by_name.read().unwrap().len();
        CacheSnapshot {
            blocks_by_id,
            blocks_by_name,
            blocks_by_state,
            items,
            entities,
            total: blocks_by_id + blocks_by_name + blocks_by_state + items + entities,
        }
    }
}

impl<S: GameDataStore> GameDataStore for CachedStore<S> {
    fn block_by_id(&self, id: u16) -> StoreResult<BlockRecord> {
        // Check cache first (read lock) + XOR verify
        if let Some(entry) = self.blocks_by_id.read().unwrap().get(&id) {
            debug_assert!(entry.verify_zero_copy(), "XOR zero-copy breach: block_by_id({id})");
            return Ok(entry.value.clone());
        }
        // Miss — delegate, then cache with XOR tag (write lock)
        let record = self.delegate.block_by_id(id)?;
        let entry = CacheEntry::new(
            Cow::Borrowed("block_by_id"),
            Cow::Owned(id.to_string()),
            record.clone(),
        );
        self.blocks_by_id.write().unwrap().insert(id, entry);
        Ok(record)
    }

    fn block_by_name(&self, name: &str) -> StoreResult<BlockRecord> {
        if let Some(entry) = self.blocks_by_name.read().unwrap().get(name) {
            debug_assert!(entry.verify_zero_copy(), "XOR zero-copy breach: block_by_name({name})");
            return Ok(entry.value.clone());
        }
        let record = self.delegate.block_by_name(name)?;
        let entry = CacheEntry::new(
            Cow::Borrowed("block_by_name"),
            Cow::Owned(name.to_string()),
            record.clone(),
        );
        self.blocks_by_name
            .write()
            .unwrap()
            .insert(name.to_string(), entry);
        Ok(record)
    }

    fn block_by_state_id(&self, state_id: u16) -> StoreResult<BlockRecord> {
        if let Some(entry) = self.blocks_by_state.read().unwrap().get(&state_id) {
            debug_assert!(
                entry.verify_zero_copy(),
                "XOR zero-copy breach: block_by_state_id({state_id})"
            );
            return Ok(entry.value.clone());
        }
        let record = self.delegate.block_by_state_id(state_id)?;
        let entry = CacheEntry::new(
            Cow::Borrowed("block_by_state_id"),
            Cow::Owned(state_id.to_string()),
            record.clone(),
        );
        self.blocks_by_state
            .write()
            .unwrap()
            .insert(state_id, entry);
        Ok(record)
    }

    fn block_count(&self) -> usize {
        self.delegate.block_count()
    }

    fn item_by_name(&self, name: &str) -> StoreResult<ItemRecord> {
        if let Some(entry) = self.items_by_name.read().unwrap().get(name) {
            debug_assert!(entry.verify_zero_copy(), "XOR zero-copy breach: item_by_name({name})");
            return Ok(entry.value.clone());
        }
        let record = self.delegate.item_by_name(name)?;
        let entry = CacheEntry::new(
            Cow::Borrowed("item_by_name"),
            Cow::Owned(name.to_string()),
            record.clone(),
        );
        self.items_by_name
            .write()
            .unwrap()
            .insert(name.to_string(), entry);
        Ok(record)
    }

    fn item_count(&self) -> usize {
        self.delegate.item_count()
    }

    fn entity_by_name(&self, name: &str) -> StoreResult<EntityRecord> {
        if let Some(entry) = self.entities_by_name.read().unwrap().get(name) {
            debug_assert!(
                entry.verify_zero_copy(),
                "XOR zero-copy breach: entity_by_name({name})"
            );
            return Ok(entry.value.clone());
        }
        let record = self.delegate.entity_by_name(name)?;
        let entry = CacheEntry::new(
            Cow::Borrowed("entity_by_name"),
            Cow::Owned(name.to_string()),
            record.clone(),
        );
        self.entities_by_name
            .write()
            .unwrap()
            .insert(name.to_string(), entry);
        Ok(record)
    }

    fn entity_count(&self) -> usize {
        self.delegate.entity_count()
    }

    fn recipes_for_output(&self, item_name: &str) -> StoreResult<Vec<RecipeRecord>> {
        // Recipes are not cached — they return Vec and are typically
        // queried infrequently. Delegate directly.
        self.delegate.recipes_for_output(item_name)
    }

    fn recipe_count(&self) -> usize {
        self.delegate.recipe_count()
    }
}

#[cfg(test)]
#[cfg(feature = "toml-store")]
mod tests {
    use super::*;
    use crate::StaticStore;

    #[test]
    fn cached_lookup_returns_same_as_delegate() {
        let cached = CachedStore::new(StaticStore::new());
        let direct = StaticStore::new();

        let cached_block = cached.block_by_name("stone").unwrap();
        let direct_block = direct.block_by_name("stone").unwrap();

        assert_eq!(cached_block.id, direct_block.id);
        assert_eq!(cached_block.name, direct_block.name);
        assert_eq!(cached_block.hardness, direct_block.hardness);
    }

    #[test]
    fn second_lookup_hits_cache() {
        let cached = CachedStore::new(StaticStore::new());

        // First lookup — cache miss, delegates to StaticStore
        let first = cached.block_by_name("stone").unwrap();
        // Second lookup — cache hit
        let second = cached.block_by_name("stone").unwrap();

        assert_eq!(first.id, second.id);
        assert_eq!(first.name, second.name);
    }

    #[test]
    fn cache_entry_has_transparent_metadata() {
        let cached = CachedStore::new(StaticStore::new());

        // Populate cache
        let _ = cached.block_by_name("stone").unwrap();

        // Inspect the cache entry directly
        let cache = cached.blocks_by_name.read().unwrap();
        let entry = cache.get("stone").unwrap();
        assert_eq!(entry.method, "block_by_name");
        assert_eq!(entry.key, "stone");
        assert_eq!(entry.value.name, "stone");
    }

    #[test]
    fn cache_entry_serializes_to_json() {
        let cached = CachedStore::new(StaticStore::new());
        let _ = cached.block_by_name("stone").unwrap();

        let cache = cached.blocks_by_name.read().unwrap();
        let entry = cache.get("stone").unwrap();
        let json = serde_json::to_string(entry).unwrap();
        assert!(json.contains("block_by_name"));
        assert!(json.contains("stone"));
    }

    #[test]
    fn invalidate_clears_cache() {
        let cached = CachedStore::new(StaticStore::new());

        // Populate cache
        let _ = cached.block_by_name("stone").unwrap();
        assert_eq!(cached.snapshot().blocks_by_name, 1);

        // Invalidate
        cached.invalidate();
        assert_eq!(cached.snapshot().total, 0);

        // Re-lookup works
        let block = cached.block_by_name("stone").unwrap();
        assert_eq!(block.name, "stone");
    }

    #[test]
    fn snapshot_tracks_counts() {
        let cached = CachedStore::new(StaticStore::new());

        assert_eq!(cached.snapshot().total, 0);

        let _ = cached.block_by_name("stone").unwrap();
        let _ = cached.block_by_id(0).unwrap();
        let _ = cached.item_by_name("diamond_sword").unwrap();
        let _ = cached.entity_by_name("zombie").unwrap();

        let snap = cached.snapshot();
        assert_eq!(snap.blocks_by_name, 1);
        assert_eq!(snap.blocks_by_id, 1);
        assert_eq!(snap.items, 1);
        assert_eq!(snap.entities, 1);
        assert_eq!(snap.total, 4);
    }

    #[test]
    fn cached_item_lookup() {
        let cached = CachedStore::new(StaticStore::new());
        let item = cached.item_by_name("diamond_sword").unwrap();
        assert_eq!(item.name, "diamond_sword");

        // Second call from cache
        let item2 = cached.item_by_name("diamond_sword").unwrap();
        assert_eq!(item.id, item2.id);
    }

    #[test]
    fn cached_entity_lookup() {
        let cached = CachedStore::new(StaticStore::new());
        let entity = cached.entity_by_name("zombie").unwrap();
        assert_eq!(entity.name, "zombie");
        assert!(entity.is_mob);
    }

    #[test]
    fn cached_block_by_id() {
        let cached = CachedStore::new(StaticStore::new());
        let block = cached.block_by_id(0).unwrap();
        assert_eq!(block.name, "air");

        // Cache hit
        let block2 = cached.block_by_id(0).unwrap();
        assert_eq!(block.id, block2.id);
    }

    #[test]
    fn cached_block_by_state_id() {
        let cached = CachedStore::new(StaticStore::new());
        let block = cached.block_by_state_id(0).unwrap();
        assert_eq!(block.name, "air");
    }

    #[test]
    fn delegate_accessor() {
        let cached = CachedStore::new(StaticStore::new());
        let block = cached.delegate().block_by_name("stone").unwrap();
        assert_eq!(block.name, "stone");
    }

    #[test]
    fn cached_as_trait_object() {
        let cached: Box<dyn GameDataStore> =
            Box::new(CachedStore::new(StaticStore::new()));
        let block = cached.block_by_name("stone").unwrap();
        assert_eq!(block.name, "stone");
    }

    /// Critical: Verify that `StaticStore` → `CachedStore` chain preserves
    /// `Cow::Borrowed` (zero-copy). Cache cloning must NOT break into Owned.
    #[test]
    fn zero_copy_preserved_through_cache() {
        let cached = CachedStore::new(StaticStore::new());

        // First lookup (cache miss → delegate → cache)
        let block1 = cached.block_by_name("stone").unwrap();
        assert!(
            matches!(block1.name, Cow::Borrowed(_)),
            "first lookup must return Cow::Borrowed"
        );

        // Second lookup (cache hit → cloned from cache)
        let block2 = cached.block_by_name("stone").unwrap();
        assert!(
            matches!(block2.name, Cow::Borrowed(_)),
            "cache hit must return Cow::Borrowed, not heap-allocated Owned"
        );

        // Item through cache
        let item = cached.item_by_name("diamond_sword").unwrap();
        assert!(matches!(item.name, Cow::Borrowed(_)));

        let item2 = cached.item_by_name("diamond_sword").unwrap();
        assert!(
            matches!(item2.name, Cow::Borrowed(_)),
            "cached item must stay Cow::Borrowed"
        );

        // Entity through cache
        let entity = cached.entity_by_name("zombie").unwrap();
        assert!(matches!(entity.name, Cow::Borrowed(_)));

        let entity2 = cached.entity_by_name("zombie").unwrap();
        assert!(
            matches!(entity2.name, Cow::Borrowed(_)),
            "cached entity must stay Cow::Borrowed"
        );
    }

    /// XOR write-through guard: verify tag is set and verifies correctly
    /// for entries cached from `StaticStore` (all fields `Cow::Borrowed`).
    #[test]
    fn xor_guard_passes_for_borrowed() {
        use crate::traits::XOR_SENTINEL;

        let cached = CachedStore::new(StaticStore::new());
        let _ = cached.block_by_name("stone").unwrap();

        let cache = cached.blocks_by_name.read().unwrap();
        let entry = cache.get("stone").unwrap();

        // Tag should be: borrow_mask(1) ^ SENTINEL
        assert_eq!(entry.xor_tag, 1 ^ XOR_SENTINEL);
        assert!(entry.verify_zero_copy());
    }

    /// XOR write-through guard: verify that a manually corrupted entry
    /// (simulating `Cow::Borrowed` → `Cow::Owned` flip) fails verification.
    #[test]
    fn xor_guard_detects_owned_breach() {
        use crate::traits::XOR_SENTINEL;

        let store = StaticStore::new();
        let record = store.block_by_name("stone").unwrap();

        // Create entry with Borrowed name — tag records borrow_mask = 1
        let entry = CacheEntry::new(
            Cow::Borrowed("block_by_name"),
            Cow::Borrowed("stone"),
            record,
        );
        assert!(entry.verify_zero_copy());
        assert_eq!(entry.xor_tag, 1 ^ XOR_SENTINEL);

        // Simulate zero-copy breach: force name to Owned.
        // We intentionally clone here to create an independent copy to mutate.
        #[allow(clippy::redundant_clone)]
        let mut breached = entry.clone();
        breached.value.name = Cow::Owned("stone".to_string());

        // XOR tag was computed with borrow_mask=1, but now borrow_mask=0
        // So verify_zero_copy must return false
        assert!(
            !breached.verify_zero_copy(),
            "XOR guard must detect Borrowed→Owned flip"
        );
    }

    /// XOR guard with entity record through cache.
    #[test]
    fn xor_guard_entity_through_cache() {
        let cached = CachedStore::new(StaticStore::new());
        let _ = cached.entity_by_name("zombie").unwrap();

        let cache = cached.entities_by_name.read().unwrap();
        let entry = cache.get("zombie").unwrap();
        assert!(entry.verify_zero_copy());
    }
}
