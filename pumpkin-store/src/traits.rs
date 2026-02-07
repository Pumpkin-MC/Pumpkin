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

/// Bitpacked mob goal state for Hamming XOR overlay (ARCH-027).
///
/// Encodes a mob's entire goal selector state in a single `u64`:
/// ```text
/// Bits  0-3:  active MOVE goal index (0xF = none)
/// Bits  4-7:  active LOOK goal index
/// Bits  8-11: active JUMP goal index
/// Bits 12-15: active TARGET goal index
/// Bits 16-19: disabled controls mask (4 bits)
/// Bits 20-35: running mask (16 bits, 1 per PrioritizedGoal)
/// Bits 36-63: reserved (priority encoding, cooldown state)
/// ```
///
/// The Hamming XOR overlay computes state transitions:
/// ```text
/// prev_state XOR curr_state = diff_bits
/// hamming_weight(diff_bits) = number of goal state changes
/// ```
///
/// - `hamming_distance == 0` → stable (no goals changed)
/// - `hamming_distance == 1` → single goal switch (normal tick)
/// - `hamming_distance > 4`  → anomalous multi-goal switch
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MobGoalState(pub u64);

impl MobGoalState {
    /// No goals active, no controls disabled.
    pub const EMPTY: Self = Self(0x0000_FFFF_FFFF_FFFFu64.reverse_bits());

    /// Create a new goal state with explicit control slot indices.
    ///
    /// Pass `0xF` for a control slot with no active goal.
    #[must_use]
    pub const fn new(
        move_goal: u8,
        look_goal: u8,
        jump_goal: u8,
        target_goal: u8,
        disabled_controls: u8,
        running_mask: u16,
    ) -> Self {
        let bits = (move_goal as u64 & 0xF)
            | ((look_goal as u64 & 0xF) << 4)
            | ((jump_goal as u64 & 0xF) << 8)
            | ((target_goal as u64 & 0xF) << 12)
            | ((disabled_controls as u64 & 0xF) << 16)
            | ((running_mask as u64) << 20);
        Self(bits)
    }

    /// XOR two states — the result encodes which bits changed.
    #[must_use]
    pub const fn xor_diff(self, other: Self) -> u64 {
        self.0 ^ other.0
    }

    /// Hamming distance between two states (number of changed bits).
    #[must_use]
    pub const fn hamming_distance(self, other: Self) -> u32 {
        self.xor_diff(other).count_ones()
    }

    /// Extract the active goal index for the MOVE control slot.
    #[must_use]
    pub const fn move_goal(self) -> u8 {
        (self.0 & 0xF) as u8
    }

    /// Extract the active goal index for the LOOK control slot.
    #[must_use]
    pub const fn look_goal(self) -> u8 {
        ((self.0 >> 4) & 0xF) as u8
    }

    /// Extract the active goal index for the JUMP control slot.
    #[must_use]
    pub const fn jump_goal(self) -> u8 {
        ((self.0 >> 8) & 0xF) as u8
    }

    /// Extract the active goal index for the TARGET control slot.
    #[must_use]
    pub const fn target_goal(self) -> u8 {
        ((self.0 >> 12) & 0xF) as u8
    }

    /// Extract the disabled controls mask.
    #[must_use]
    pub const fn disabled_controls(self) -> u8 {
        ((self.0 >> 16) & 0xF) as u8
    }

    /// Extract the running mask (1 bit per goal, up to 16 goals).
    #[must_use]
    pub const fn running_mask(self) -> u16 {
        ((self.0 >> 20) & 0xFFFF) as u16
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
