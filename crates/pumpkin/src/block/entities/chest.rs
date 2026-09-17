use std::sync::{Arc, RwLock, atomic::AtomicBool};

use pumpkin_data::item_stack::ItemStack;
use pumpkin_util::math::position::BlockPos;

use crate::{
    block::viewer::ViewerCountTracker, impl_block_entity_for_chest, impl_chest_helper_methods,
    impl_clearable_for_chest, impl_inventory_for_chest, impl_viewer_count_listener_for_chest,
};

pub struct ChestBlockEntity {
    pub position: BlockPos,
    pub id: &'static str,
    pub components: super::components::BlockEntityComponents,
    pub items: RwLock<[ItemStack; Self::INVENTORY_SIZE]>,
    pub dirty: AtomicBool,
    pub comparator_dirty: AtomicBool,

    // Viewer
    viewers: ViewerCountTracker,
}

impl ChestBlockEntity {
    pub const INVENTORY_SIZE: usize = 27;
    pub const LID_ANIMATION_EVENT_TYPE: u8 = 1;
    pub const ID: &'static str = "minecraft:chest";
    pub const EMITS_REDSTONE: bool = false;

    /// Creates a copper chest with the distinct block-entity registry identifier.
    #[must_use]
    pub fn new_copper(position: BlockPos) -> Self {
        let mut chest = Self::new(position);
        chest.id = "minecraft:copper_chest";
        chest
    }
}

// Apply macros to generate trait implementations
impl_block_entity_for_chest!(ChestBlockEntity);
impl_inventory_for_chest!(ChestBlockEntity);
impl_clearable_for_chest!(ChestBlockEntity);
impl_viewer_count_listener_for_chest!(ChestBlockEntity);
impl_chest_helper_methods!(ChestBlockEntity);
