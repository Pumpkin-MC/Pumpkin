use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::StoreResult;

/// Lightweight block record — serializable DTO decoupled from pumpkin-data internals.
///
/// This is what crosses the store boundary. Backends fill it from whatever
/// storage they use (static arrays, TOML files, Arrow batches, Lance tables).
///
/// Uses `Cow<'static, str>` for zero-copy from pumpkin-data statics AND
/// owned strings from TOML registry files (`.claude/registry/blocks.toml`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockRecord {
    pub id: u16,
    pub name: Cow<'static, str>,
    pub hardness: f32,
    pub blast_resistance: f32,
    pub is_air: bool,
    pub is_solid: bool,
    pub luminance: u8,
    pub item_id: u16,
    pub default_state_id: u16,
    pub state_count: u16,
}

/// Lightweight item record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemRecord {
    pub id: u16,
    pub name: Cow<'static, str>,
    pub max_stack_size: u8,
}

/// Lightweight entity type record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRecord {
    pub id: u16,
    pub name: Cow<'static, str>,
    pub max_health: Option<f32>,
    pub is_mob: bool,
    pub width: f32,
    pub height: f32,
    pub fire_immune: bool,
}

/// Lightweight recipe record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeRecord {
    pub recipe_type: Cow<'static, str>,
    pub group: Cow<'static, str>,
    pub result_item: Cow<'static, str>,
    pub result_count: u8,
}

/// XOR sentinel for zero-copy write-through guard (holograph pattern).
///
/// Used by [`CacheEntry`](crate::CacheEntry) to detect if a `Cow::Borrowed` field
/// was silently converted to `Cow::Owned` during ephemeral variable switching.
pub const XOR_SENTINEL: u64 = 0xDEAD_BEEF_CAFE_BABE;

/// Reports which `Cow` fields are currently `Borrowed` (zero-copy).
///
/// Each bit in the returned mask corresponds to a `Cow` field:
/// - bit 0 = first `Cow` field (typically `name`)
/// - bit 1 = second `Cow` field, etc.
///
/// Used by the XOR write-through guard in [`CacheEntry`](crate::CacheEntry):
/// at insertion, `borrow_mask() ^ XOR_SENTINEL` is stored. On read-back,
/// re-computing `borrow_mask() ^ XOR_SENTINEL` must produce the same tag.
/// If a `Cow::Borrowed` silently flipped to `Cow::Owned`, the tags diverge.
pub trait ZeroCopyGuard {
    /// Bitmask of Borrowed `Cow` fields. Bit i = 1 if field i is `Cow::Borrowed`.
    fn borrow_mask(&self) -> u64;

    /// Compute the XOR tag for zero-copy verification.
    fn xor_tag(&self) -> u64 {
        self.borrow_mask() ^ XOR_SENTINEL
    }

    /// Verify that the current borrow state matches a previously computed tag.
    /// Returns `true` if zero-copy is intact, `false` if breached.
    fn verify_xor(&self, tag: u64) -> bool {
        self.xor_tag() == tag
    }
}

impl ZeroCopyGuard for BlockRecord {
    fn borrow_mask(&self) -> u64 {
        u64::from(matches!(self.name, Cow::Borrowed(_)))
    }
}

impl ZeroCopyGuard for ItemRecord {
    fn borrow_mask(&self) -> u64 {
        u64::from(matches!(self.name, Cow::Borrowed(_)))
    }
}

impl ZeroCopyGuard for EntityRecord {
    fn borrow_mask(&self) -> u64 {
        u64::from(matches!(self.name, Cow::Borrowed(_)))
    }
}

impl ZeroCopyGuard for RecipeRecord {
    fn borrow_mask(&self) -> u64 {
        let b0 = u64::from(matches!(self.recipe_type, Cow::Borrowed(_)));
        let b1 = u64::from(matches!(self.group, Cow::Borrowed(_)));
        let b2 = u64::from(matches!(self.result_item, Cow::Borrowed(_)));
        b0 | (b1 << 1) | (b2 << 2)
    }
}

/// Cross-entity relationship mapping record (ARCH-027).
///
/// Stores relationships between game objects that don't fit in entity tables:
/// biome→spawn rules, structure→loot tables, block→item drops, mob→goal states.
///
/// Used by the `game_mapping` Lance table and queryable via
/// `GameDataStore::game_mappings()`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameMappingRecord {
    /// Source category (e.g. "biome", "structure", "block", `mob_goal`).
    pub source_type: Cow<'static, str>,
    /// Source key within category (e.g. "plains", `desert_pyramid`, "stone").
    pub source_key: Cow<'static, str>,
    /// Target category (e.g. `entity_spawn`, `loot_table`, `item_drop`, `goal_state`).
    pub target_type: Cow<'static, str>,
    /// Target key (e.g. "zombie", `chest_loot_desert`, "cobblestone").
    pub target_key: Cow<'static, str>,
    /// Optional weight/priority for weighted relationships (spawn weights, etc.).
    pub weight: Option<f32>,
}

impl ZeroCopyGuard for GameMappingRecord {
    fn borrow_mask(&self) -> u64 {
        let b0 = u64::from(matches!(self.source_type, Cow::Borrowed(_)));
        let b1 = u64::from(matches!(self.source_key, Cow::Borrowed(_)));
        let b2 = u64::from(matches!(self.target_type, Cow::Borrowed(_)));
        let b3 = u64::from(matches!(self.target_key, Cow::Borrowed(_)));
        b0 | (b1 << 1) | (b2 << 2) | (b3 << 3)
    }
}

/// Core trait for game data access.
///
/// Backends implement this to provide block, item, entity, and recipe lookups.
/// The trait uses `&'static` string references for zero-copy compatibility with
/// pumpkin-data's compile-time generated data.
///
/// # Backends
///
/// - [`StaticStore`](crate::StaticStore) — wraps pumpkin-data (default, zero-cost)
/// - [`LanceStore`](crate::LanceStore) — Arrow columnar + `DataFusion` SQL (opt-in)
pub trait GameDataStore: Send + Sync {
    // ── Blocks ──────────────────────────────────────────────────

    /// Look up a block by its numeric ID.
    fn block_by_id(&self, id: u16) -> StoreResult<BlockRecord>;

    /// Look up a block by registry name (e.g. "stone", "minecraft:stone").
    fn block_by_name(&self, name: &str) -> StoreResult<BlockRecord>;

    /// Look up a block by one of its state IDs.
    fn block_by_state_id(&self, state_id: u16) -> StoreResult<BlockRecord>;

    /// Total number of blocks in the registry.
    fn block_count(&self) -> usize;

    // ── Items ───────────────────────────────────────────────────

    /// Look up an item by registry name.
    fn item_by_name(&self, name: &str) -> StoreResult<ItemRecord>;

    /// Total number of items in the registry.
    fn item_count(&self) -> usize;

    // ── Entities ────────────────────────────────────────────────

    /// Look up an entity type by registry name.
    fn entity_by_name(&self, name: &str) -> StoreResult<EntityRecord>;

    /// Total number of entity types.
    fn entity_count(&self) -> usize;

    // ── Recipes ─────────────────────────────────────────────────

    /// Get all recipes that produce the given item name.
    fn recipes_for_output(&self, item_name: &str) -> StoreResult<Vec<RecipeRecord>>;

    /// Total number of recipes.
    fn recipe_count(&self) -> usize;

    // ── Game Mappings ────────────────────────────────────────────

    /// Query cross-entity relationship mappings by source type and key.
    ///
    /// Returns all mappings from the given source (e.g. all spawn rules
    /// for a biome, all goals for a mob type, all drops for a block).
    fn game_mappings(
        &self,
        source_type: &str,
        source_key: &str,
    ) -> StoreResult<Vec<GameMappingRecord>>;

    /// Total number of game mapping entries.
    fn game_mapping_count(&self) -> usize;
}
