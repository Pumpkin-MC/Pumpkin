use blocks::chest::ChestBlock;
use blocks::furnace::FurnaceBlock;
use properties::{
    age::Age,
    attachment::Attachment,
    axis::Axis,
    cardinal::{Down, East, North, South, Up, West},
    face::Face,
    half::Half,
    layers::Layers,
    open::Open,
    powered::Powered,
    signal_fire::SignalFire,
    slab_type::SlabType,
    stair_shape::StairShape,
    BlockPropertiesManager,
};

use crate::block::blocks::crafting_table::CraftingTableBlock;
use crate::block::blocks::jukebox::JukeboxBlock;
use crate::block::registry::BlockRegistry;
use properties::{facing::Facing, waterlog::Waterlogged};
use std::sync::Arc;

mod blocks;
pub mod properties;
pub mod pumpkin_block;
pub mod registry;

#[must_use]
pub fn default_registry() -> Arc<BlockRegistry> {
    let mut manager = BlockRegistry::default();

    manager.register(JukeboxBlock);
    manager.register(CraftingTableBlock);
    manager.register(FurnaceBlock);
    manager.register(ChestBlock);

    Arc::new(manager)
}

#[must_use]
pub fn default_block_properties_manager() -> Arc<BlockPropertiesManager> {
    let mut manager = BlockPropertiesManager::default();

    // This is the default state of the blocks
    manager.register(Age::Age0);
    manager.register(Attachment::Floor);
    manager.register(Axis::Y);
    manager.register(Down::False);
    manager.register(East::False);
    manager.register(Face::Floor);
    manager.register(Facing::North);
    manager.register(Half::Bottom);
    manager.register(Layers::Lay1);
    manager.register(North::False);
    manager.register(Open::False);
    manager.register(Powered::False);
    manager.register(SignalFire::False);
    manager.register(SlabType::Bottom);
    manager.register(South::False);
    manager.register(StairShape::Straight);
    manager.register(Up::False);
    manager.register(Waterlogged::False);
    manager.register(West::False);

    manager.build_properties_registry();

    Arc::new(manager)
}
