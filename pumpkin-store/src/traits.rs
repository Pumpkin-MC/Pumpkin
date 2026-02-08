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

// ── Spatial Overlay (holograph pattern) ─────────────────────────────────

/// Number of u64 words in a [`SpatialOverlay`] (256 × 64 = 2^14 = 16384 bits).
pub const OVERLAY_WORDS: usize = 256;

/// Total bits in the overlay (2^14 = 16384).
pub const OVERLAY_BITS: usize = OVERLAY_WORDS * 64;

/// Bitpacked 2^14 Hamming vector for spatial activity overlay (ARCH-029).
///
/// Compresses an 8192^3 (2^39) spatial volume into 2^14 bits via spatial
/// hashing. Two independent tables XOR each other for activity detection:
///
/// ```text
/// Table A (ephemeral):  height + mob activity → changes per tick
/// Table B (static XY):  spatial map → changes on entity movement
///
/// XOR(A, B) = combined activity fingerprint
/// hamming_weight(XOR(A, B)) = total state changes
/// AVX-512: 32 ops to diff entire table (512 bits per lane)
/// ```
///
/// ## Holograph Pattern
///
/// From `AdaWorldAPI/holograph`: 8192^3 positions in 3D space mapped to
/// 2^14 bit Hamming vector. Entities bind/unbind bits by spatial hash of
/// their (x, y, z) position. Two tables XOR each other — ephemeral
/// (height + mob data, 16k) vs static (XY map, 16k).
///
/// ## Entity Slot Encoding (2 bits per slot)
///
/// ```text
/// 00 = inactive (no entity bound)
/// 01 = active (entity present, stable)
/// 10 = changed (entity state transition this tick)
/// 11 = anomalous (multi-goal switch or error)
/// ```
#[derive(Clone, PartialEq, Eq)]
pub struct SpatialOverlay {
    /// 256 × u64 = 16384 bits = 2^14 Hamming vector.
    bits: [u64; OVERLAY_WORDS],
}

impl SpatialOverlay {
    /// Empty overlay — all bits zero (no entities bound).
    pub const EMPTY: Self = Self {
        bits: [0u64; OVERLAY_WORDS],
    };

    /// Create an empty overlay.
    #[must_use]
    pub const fn new() -> Self {
        Self::EMPTY
    }

    /// Spatial hash: fold (x, y, z) into a bit index in `[0, 16384)`.
    ///
    /// Uses multiplicative hashing with golden-ratio-derived constants
    /// to distribute 3D coordinates across the 2^14 bit space.
    #[must_use]
    pub const fn spatial_hash(x: i32, y: i32, z: i32) -> usize {
        let hx = (x as u32).wrapping_mul(0x9E37_79B9);
        let hy = (y as u32).wrapping_mul(0x517C_C1B7);
        let hz = (z as u32).wrapping_mul(0x85EB_CA6B);
        let combined = hx ^ hy.wrapping_shl(5) ^ hz.wrapping_shl(10);
        (combined >> 18) as usize // top 14 bits → [0, 16384)
    }

    /// Set a bit at the given index.
    ///
    /// # Panics
    ///
    /// Panics if `bit >= 16384`.
    pub fn set_bit(&mut self, bit: usize) {
        assert!(bit < OVERLAY_BITS, "bit index out of range: {bit}");
        self.bits[bit / 64] |= 1u64 << (bit % 64);
    }

    /// Clear a bit at the given index.
    ///
    /// # Panics
    ///
    /// Panics if `bit >= 16384`.
    pub fn clear_bit(&mut self, bit: usize) {
        assert!(bit < OVERLAY_BITS, "bit index out of range: {bit}");
        self.bits[bit / 64] &= !(1u64 << (bit % 64));
    }

    /// Test whether a bit is set.
    ///
    /// # Panics
    ///
    /// Panics if `bit >= 16384`.
    #[must_use]
    pub const fn test_bit(&self, bit: usize) -> bool {
        assert!(bit < OVERLAY_BITS, "bit index out of range");
        (self.bits[bit / 64] >> (bit % 64)) & 1 == 1
    }

    /// Bind an entity at spatial position — sets the hashed bit.
    pub fn bind(&mut self, x: i32, y: i32, z: i32) {
        self.set_bit(Self::spatial_hash(x, y, z));
    }

    /// Unbind an entity at spatial position — clears the hashed bit.
    pub fn unbind(&mut self, x: i32, y: i32, z: i32) {
        self.clear_bit(Self::spatial_hash(x, y, z));
    }

    /// XOR two tables — produces a diff showing which bits changed.
    ///
    /// Use two independent overlays (ephemeral + static) and XOR them
    /// for combined activity detection.
    #[must_use]
    pub fn xor_diff(&self, other: &Self) -> Self {
        let mut result = Self::EMPTY;
        for i in 0..OVERLAY_WORDS {
            result.bits[i] = self.bits[i] ^ other.bits[i];
        }
        result
    }

    /// Hamming weight — total number of set bits.
    ///
    /// On a diff overlay (from [`xor_diff`](Self::xor_diff)), this is
    /// the number of spatial buckets that changed between two snapshots.
    #[must_use]
    pub fn hamming_weight(&self) -> u32 {
        let mut count = 0u32;
        for word in &self.bits {
            count += word.count_ones();
        }
        count
    }

    /// Hamming distance between two overlays (popcount of their XOR).
    #[must_use]
    pub fn hamming_distance(&self, other: &Self) -> u32 {
        let mut count = 0u32;
        for i in 0..OVERLAY_WORDS {
            count += (self.bits[i] ^ other.bits[i]).count_ones();
        }
        count
    }

    /// Number of set bits (alias for [`hamming_weight`](Self::hamming_weight)).
    #[must_use]
    pub fn popcount(&self) -> u32 {
        self.hamming_weight()
    }

    /// Access the raw u64 word array (for SIMD / Arrow interop).
    #[must_use]
    pub const fn as_words(&self) -> &[u64; OVERLAY_WORDS] {
        &self.bits
    }

    /// Mutable access to the raw u64 word array.
    pub const fn as_words_mut(&mut self) -> &mut [u64; OVERLAY_WORDS] {
        &mut self.bits
    }
}

impl Default for SpatialOverlay {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for SpatialOverlay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpatialOverlay")
            .field("popcount", &self.popcount())
            .finish()
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

#[cfg(test)]
mod tests {
    use super::*;

    // ── MobGoalState tests ────────────────────────────────────────

    #[test]
    fn mob_goal_state_roundtrip() {
        let state = MobGoalState::new(1, 2, 3, 4, 0b1010, 0xABCD);
        assert_eq!(state.move_goal(), 1);
        assert_eq!(state.look_goal(), 2);
        assert_eq!(state.jump_goal(), 3);
        assert_eq!(state.target_goal(), 4);
        assert_eq!(state.disabled_controls(), 0b1010);
        assert_eq!(state.running_mask(), 0xABCD);
    }

    #[test]
    fn mob_goal_state_hamming() {
        let a = MobGoalState::new(1, 2, 0xF, 0xF, 0, 0);
        let b = MobGoalState::new(1, 3, 0xF, 0xF, 0, 0);
        // Only the LOOK slot changed (2→3), bits 4-7 differ
        let dist = a.hamming_distance(b);
        assert!(dist > 0, "different states must have nonzero hamming distance");
        assert!(dist <= 4, "single slot change should differ by ≤4 bits");
    }

    #[test]
    fn mob_goal_state_identical_is_zero() {
        let state = MobGoalState::new(5, 3, 2, 1, 0xF, 0xFFFF);
        assert_eq!(state.hamming_distance(state), 0);
    }

    // ── SpatialOverlay tests ──────────────────────────────────────

    #[test]
    fn overlay_empty_has_zero_popcount() {
        let overlay = SpatialOverlay::new();
        assert_eq!(overlay.popcount(), 0);
    }

    #[test]
    fn overlay_set_and_test_bit() {
        let mut overlay = SpatialOverlay::new();
        overlay.set_bit(0);
        overlay.set_bit(100);
        overlay.set_bit(16383);
        assert!(overlay.test_bit(0));
        assert!(overlay.test_bit(100));
        assert!(overlay.test_bit(16383));
        assert!(!overlay.test_bit(1));
        assert_eq!(overlay.popcount(), 3);
    }

    #[test]
    fn overlay_clear_bit() {
        let mut overlay = SpatialOverlay::new();
        overlay.set_bit(42);
        assert!(overlay.test_bit(42));
        overlay.clear_bit(42);
        assert!(!overlay.test_bit(42));
        assert_eq!(overlay.popcount(), 0);
    }

    #[test]
    fn overlay_bind_unbind() {
        let mut overlay = SpatialOverlay::new();
        overlay.bind(100, 64, -200);
        assert!(overlay.popcount() > 0);

        let hash = SpatialOverlay::spatial_hash(100, 64, -200);
        assert!(overlay.test_bit(hash));

        overlay.unbind(100, 64, -200);
        assert!(!overlay.test_bit(hash));
    }

    #[test]
    fn overlay_xor_diff_detects_changes() {
        let mut a = SpatialOverlay::new();
        let mut b = SpatialOverlay::new();

        // Bind same position in both — no diff
        a.bind(10, 20, 30);
        b.bind(10, 20, 30);
        let diff = a.xor_diff(&b);
        assert_eq!(diff.popcount(), 0, "identical binds should produce zero diff");

        // Bind extra position in b — diff shows it
        b.bind(40, 50, 60);
        let diff = a.xor_diff(&b);
        assert!(diff.popcount() > 0, "different binds should produce nonzero diff");
    }

    #[test]
    fn overlay_hamming_distance() {
        let mut a = SpatialOverlay::new();
        let mut b = SpatialOverlay::new();

        a.bind(0, 0, 0);
        b.bind(1000, 1000, 1000);

        let dist = a.hamming_distance(&b);
        // Two different positions bound → at least 2 bits differ (could be more if hash collision)
        assert!(dist >= 1);
    }

    #[test]
    fn overlay_two_tables_xor() {
        // Ephemeral table: mob activity
        let mut ephemeral = SpatialOverlay::new();
        ephemeral.bind(100, 64, 200);
        ephemeral.bind(200, 70, 300);

        // Static table: XY spatial map
        let mut static_xy = SpatialOverlay::new();
        static_xy.bind(100, 64, 200); // same entity, present in both

        // XOR reveals what's ephemeral-only (activity that moved)
        let diff = ephemeral.xor_diff(&static_xy);
        assert!(
            diff.popcount() > 0,
            "ephemeral has extra bind, XOR should be nonzero"
        );
    }

    #[test]
    fn overlay_constants_correct() {
        assert_eq!(OVERLAY_WORDS, 256);
        assert_eq!(OVERLAY_BITS, 16384);
        assert_eq!(OVERLAY_BITS, 1 << 14);
    }

    #[test]
    fn overlay_spatial_hash_in_range() {
        // Various coordinates should all hash to valid range
        for &(x, y, z) in &[
            (0, 0, 0),
            (8191, 8191, 8191),
            (-8192, -8192, -8192),
            (i32::MAX, i32::MIN, 0),
            (100, -64, 320),
        ] {
            let hash = SpatialOverlay::spatial_hash(x, y, z);
            assert!(
                hash < OVERLAY_BITS,
                "hash({x},{y},{z}) = {hash} out of range"
            );
        }
    }

    #[test]
    fn overlay_as_words_roundtrip() {
        let mut overlay = SpatialOverlay::new();
        overlay.bind(42, 42, 42);
        let words = overlay.as_words();
        assert_eq!(words.len(), 256);

        // At least one word should be nonzero
        assert!(words.iter().any(|&w| w != 0));
    }

    #[test]
    #[should_panic(expected = "bit index out of range")]
    fn overlay_set_bit_panics_out_of_range() {
        let mut overlay = SpatialOverlay::new();
        overlay.set_bit(16384); // exactly out of range
    }
}
