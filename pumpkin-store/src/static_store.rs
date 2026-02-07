//! Default backend: wraps pumpkin-data's compile-time generated static arrays.
//!
//! Zero runtime cost — all lookups delegate to `Block::from_*`, `Item::from_*`, etc.
//! No heap allocation, no I/O, no new dependencies beyond pumpkin-data.

use std::borrow::Cow;

use pumpkin_data::Block;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;

use crate::error::{StoreError, StoreResult};
use crate::traits::{BlockRecord, EntityRecord, GameDataStore, ItemRecord, RecipeRecord};

/// Default store backed by pumpkin-data's static arrays.
///
/// This is a zero-cost wrapper — all methods are thin delegations to
/// `Block::from_id()`, `Item::from_registry_key()`, etc.
/// No heap allocation, no I/O.
pub struct StaticStore;

impl StaticStore {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for StaticStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a pumpkin-data `Block` to our portable `BlockRecord` DTO.
fn block_to_record(b: &'static Block) -> BlockRecord {
    BlockRecord {
        id: b.id,
        name: Cow::Borrowed(b.name),
        hardness: b.hardness,
        blast_resistance: b.blast_resistance,
        is_air: b.is_air(),
        is_solid: b.is_solid(),
        luminance: b.default_state.luminance,
        item_id: b.item_id,
        default_state_id: b.default_state.id,
        state_count: b.states.len() as u16,
    }
}

/// Convert a pumpkin-data `Item` to our portable `ItemRecord` DTO.
fn item_to_record(item: &'static Item) -> ItemRecord {
    use pumpkin_data::data_component::DataComponent;
    use pumpkin_data::data_component_impl::MaxStackSizeImpl;

    let max_stack = item
        .components
        .iter()
        .find_map(|(comp, imp)| {
            if matches!(comp, DataComponent::MaxStackSize) {
                imp.as_any()
                    .downcast_ref::<MaxStackSizeImpl>()
                    .map(|m| m.size)
            } else {
                None
            }
        })
        .unwrap_or(64);

    ItemRecord {
        id: item.id,
        name: Cow::Borrowed(item.registry_key),
        max_stack_size: max_stack,
    }
}

/// Convert a pumpkin-data `EntityType` to our portable `EntityRecord` DTO.
const fn entity_to_record(e: &'static EntityType) -> EntityRecord {
    EntityRecord {
        id: e.id,
        name: Cow::Borrowed(e.resource_name),
        max_health: e.max_health,
        is_mob: e.mob,
        width: e.dimension[0],
        height: e.dimension[1],
        fire_immune: e.fire_immune,
    }
}

impl GameDataStore for StaticStore {
    fn block_by_id(&self, id: u16) -> StoreResult<BlockRecord> {
        let b = Block::from_id(id);
        // from_id returns AIR for out-of-range, check if the ID actually matches
        if b.id != id && id != 0 {
            return Err(StoreError::BlockNotFound(format!("id={id}")));
        }
        Ok(block_to_record(b))
    }

    fn block_by_name(&self, name: &str) -> StoreResult<BlockRecord> {
        Block::from_name(name)
            .map(block_to_record)
            .ok_or_else(|| StoreError::BlockNotFound(name.to_string()))
    }

    fn block_by_state_id(&self, state_id: u16) -> StoreResult<BlockRecord> {
        let b = Block::from_state_id(state_id);
        Ok(block_to_record(b))
    }

    fn block_count(&self) -> usize {
        // pumpkin-data has 1166 block types
        1166
    }

    fn item_by_name(&self, name: &str) -> StoreResult<ItemRecord> {
        let key = name.strip_prefix("minecraft:").unwrap_or(name);
        Item::from_registry_key(key)
            .map(item_to_record)
            .ok_or_else(|| StoreError::ItemNotFound(name.to_string()))
    }

    fn item_count(&self) -> usize {
        // Items are dense from 0..max_id; use a const from pumpkin-data
        // There are ~1400 items in 1.21.11
        1470
    }

    fn entity_by_name(&self, name: &str) -> StoreResult<EntityRecord> {
        EntityType::from_name(name)
            .map(entity_to_record)
            .ok_or_else(|| StoreError::EntityNotFound(name.to_string()))
    }

    fn entity_count(&self) -> usize {
        149
    }

    fn recipes_for_output(&self, _item_name: &str) -> StoreResult<Vec<RecipeRecord>> {
        // Recipe iteration requires scanning all recipes — defer to a future
        // implementation that indexes recipes by output item.
        // For now return empty; the lance-store backend will do SQL queries.
        Ok(Vec::new())
    }

    fn recipe_count(&self) -> usize {
        // ~1470 recipes in 1.21.11
        1470
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_stone_by_name() {
        let store = StaticStore::new();
        let block = store.block_by_name("stone").unwrap();
        assert_eq!(block.name, "stone");
        assert!(!block.is_air);
        assert!(block.is_solid);
    }

    #[test]
    fn lookup_air_by_id() {
        let store = StaticStore::new();
        let block = store.block_by_id(0).unwrap();
        assert_eq!(block.name, "air");
        assert!(block.is_air);
    }

    #[test]
    fn lookup_block_by_state_id() {
        let store = StaticStore::new();
        // State ID 0 is air
        let block = store.block_by_state_id(0).unwrap();
        assert_eq!(block.name, "air");
    }

    #[test]
    fn lookup_diamond_sword() {
        let store = StaticStore::new();
        let item = store.item_by_name("diamond_sword").unwrap();
        assert_eq!(item.name, "diamond_sword");
    }

    #[test]
    fn lookup_zombie() {
        let store = StaticStore::new();
        let entity = store.entity_by_name("zombie").unwrap();
        assert_eq!(entity.name, "zombie");
        assert!(entity.is_mob);
    }

    #[test]
    fn item_not_found() {
        let store = StaticStore::new();
        assert!(store.item_by_name("nonexistent_item_xyz").is_err());
    }

    #[test]
    fn entity_not_found() {
        let store = StaticStore::new();
        assert!(store.entity_by_name("nonexistent_entity_xyz").is_err());
    }

    #[test]
    fn block_count_positive() {
        let store = StaticStore::new();
        assert!(store.block_count() > 1000);
    }

    #[test]
    fn trait_object_works() {
        let store: Box<dyn GameDataStore> = Box::new(StaticStore::new());
        let block = store.block_by_name("stone").unwrap();
        assert_eq!(block.name, "stone");
    }

    /// Verify that `StaticStore` produces `Cow::Borrowed` (zero-copy pointers
    /// into pumpkin-data statics), and that cloning preserves this invariant.
    /// If this test fails, something is forcing heap allocation on the read path.
    #[test]
    fn zero_copy_invariant_preserved() {
        let store = StaticStore::new();

        // Block name must be Borrowed (pointer to static "stone")
        let block = store.block_by_name("stone").unwrap();
        assert!(
            matches!(block.name, Cow::Borrowed(_)),
            "block.name must be Cow::Borrowed, got Owned"
        );

        // Clone must preserve Borrowed — NOT heap-allocate.
        // We intentionally clone here to test the invariant.
        #[allow(clippy::redundant_clone)]
        let cloned = block.clone();
        assert!(
            matches!(cloned.name, Cow::Borrowed(_)),
            "cloned block.name must stay Cow::Borrowed"
        );

        // Item name
        let item = store.item_by_name("diamond_sword").unwrap();
        assert!(
            matches!(item.name, Cow::Borrowed(_)),
            "item.name must be Cow::Borrowed"
        );

        // Entity name
        let entity = store.entity_by_name("zombie").unwrap();
        assert!(
            matches!(entity.name, Cow::Borrowed(_)),
            "entity.name must be Cow::Borrowed"
        );

        // Cloned entity must still be Borrowed.
        #[allow(clippy::redundant_clone)]
        let cloned_entity = entity.clone();
        assert!(
            matches!(cloned_entity.name, Cow::Borrowed(_)),
            "cloned entity.name must stay Cow::Borrowed"
        );
    }
}
