use std::sync::{Arc, atomic::AtomicBool};

use pumpkin_data::block_properties::BlockProperties;
use pumpkin_util::math::position::BlockPos;

use tokio::sync::Mutex;

use crate::{
    block::viewer::ViewerCountTracker, impl_block_entity_for_chest, impl_chest_helper_methods,
    impl_clearable_for_chest, impl_inventory_for_chest, impl_viewer_count_listener_for_chest,
    item::ItemStack,
};

pub struct ChestBlockEntity {
    pub position: BlockPos,
    pub items: [Arc<Mutex<ItemStack>>; Self::INVENTORY_SIZE],
    pub dirty: AtomicBool,

    // Viewer
    viewers: ViewerCountTracker,
}

impl ChestBlockEntity {
    pub const INVENTORY_SIZE: usize = 27;
    pub const LID_ANIMATION_EVENT_TYPE: u8 = 1;
    pub const ID: &'static str = "minecraft:chest";
    pub const EMITS_REDSTONE: bool = false;
}

// Apply macros to generate trait implementations
impl_block_entity_for_chest!(ChestBlockEntity);
impl_inventory_for_chest!(ChestBlockEntity);
impl_clearable_for_chest!(ChestBlockEntity);
impl_viewer_count_listener_for_chest!(ChestBlockEntity);
impl_chest_helper_methods!(ChestBlockEntity);
