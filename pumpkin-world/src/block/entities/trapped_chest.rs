use std::sync::{Arc, atomic::AtomicBool};

use pumpkin_data::block_properties::BlockProperties;
use pumpkin_util::math::position::BlockPos;

use tokio::sync::Mutex;

use crate::{
    block::viewer::ViewerCountTracker, impl_block_entity_for_chest, impl_chest_helper_methods,
    impl_clearable_for_chest, impl_inventory_for_chest, impl_viewer_count_listener_for_chest,
    item::ItemStack,
};

pub struct TrappedChestBlockEntity {
    pub position: BlockPos,
    pub items: [Arc<Mutex<ItemStack>>; Self::INVENTORY_SIZE],
    pub dirty: AtomicBool,

    // Viewer
    viewers: ViewerCountTracker,
}

impl TrappedChestBlockEntity {
    pub const INVENTORY_SIZE: usize = 27;
    pub const LID_ANIMATION_EVENT_TYPE: u8 = 1;
    pub const ID: &'static str = "minecraft:trapped_chest";
    pub const EMITS_REDSTONE: bool = true;
}

// Apply macros to generate trait implementations
impl_block_entity_for_chest!(TrappedChestBlockEntity);
impl_inventory_for_chest!(TrappedChestBlockEntity);
impl_clearable_for_chest!(TrappedChestBlockEntity);
impl_viewer_count_listener_for_chest!(TrappedChestBlockEntity);
impl_chest_helper_methods!(TrappedChestBlockEntity);
