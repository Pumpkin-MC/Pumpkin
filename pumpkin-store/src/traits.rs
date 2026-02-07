use serde::{Deserialize, Serialize};

use crate::StoreResult;

/// Lightweight block record — serializable DTO decoupled from pumpkin-data internals.
///
/// This is what crosses the store boundary. Backends fill it from whatever
/// storage they use (static arrays, TOML files, Arrow batches, Lance tables).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockRecord {
    pub id: u16,
    pub name: &'static str,
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
    pub name: &'static str,
    pub max_stack_size: u8,
}

/// Lightweight entity type record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRecord {
    pub id: u16,
    pub name: &'static str,
    pub max_health: Option<f32>,
    pub is_mob: bool,
    pub width: f32,
    pub height: f32,
    pub fire_immune: bool,
}

/// Lightweight recipe record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeRecord {
    pub recipe_type: &'static str,
    pub group: &'static str,
    pub result_item: &'static str,
    pub result_count: u8,
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
}
