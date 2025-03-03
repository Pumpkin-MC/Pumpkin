use blocks::{chest::ChestBlock, furnace::FurnaceBlock, lever::LeverBlock, tnt::TNTBlock};
use properties::{
    BlockPropertiesManager,
    age::Age,
    attachment::Attachment,
    axis::Axis,
    cardinal::{Down, East, North, South, Up, West},
    face::Face,
    facing::Facing,
    half::Half,
    layers::Layers,
    open::Open,
    powered::Powered,
    signal_fire::SignalFire,
    slab_type::SlabType,
    stair_shape::StairShape,
    unstable::Unstable,
    waterlog::Waterlogged,
};
use pumpkin_data::entity::EntityType;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::{
    block::registry::{Block, State},
    item::ItemStack,
};
use rand::Rng;

use crate::block::registry::BlockRegistry;
use crate::entity::item::ItemEntity;
use crate::world::World;
use crate::{block::blocks::crafting_table::CraftingTableBlock, entity::player::Player};
use crate::{block::blocks::jukebox::JukeboxBlock, entity::experience_orb::ExperienceOrbEntity};
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
    manager.register(TNTBlock);
    manager.register(LeverBlock);

    Arc::new(manager)
}

pub async fn drop_loot(world: &Arc<World>, block: &Block, pos: &BlockPos, experience: bool) {
    if let Some(table) = &block.loot_table {
        let loot = table.get_loot();
        for item in loot {
            drop_stack(world, pos, item).await;
        }
    }

    if experience {
        if let Some(experience) = &block.experience {
            let amount = experience.experience.get();
            // TODO: Silk touch gives no exp
            if amount > 0 {
                ExperienceOrbEntity::spawn(world, pos.to_f64(), amount as u32).await;
            }
        }
    }
}

async fn drop_stack(world: &Arc<World>, pos: &BlockPos, stack: ItemStack) {
    let height = EntityType::ITEM.dimension[1] / 2.0;
    let pos = Vector3::new(
        f64::from(pos.0.x) + 0.5 + rand::thread_rng().gen_range(-0.25..0.25),
        f64::from(pos.0.y) + 0.5 + rand::thread_rng().gen_range(-0.25..0.25) - f64::from(height),
        f64::from(pos.0.z) + 0.5 + rand::thread_rng().gen_range(-0.25..0.25),
    );

    let entity = world.create_entity(pos, EntityType::ITEM);
    let item_entity = Arc::new(ItemEntity::new(
        entity,
        stack.item.id,
        u32::from(stack.item_count),
    ));
    world.spawn_entity(item_entity.clone()).await;
    item_entity.send_meta_packet().await;
}

pub async fn calc_block_breaking(player: &Player, state: &State, block_name: &str) -> f32 {
    let hardness = state.hardness;
    #[expect(clippy::float_cmp)]
    if hardness == -1.0 {
        // unbreakable
        return 0.0;
    }
    let i = if player.can_harvest(state, block_name).await {
        30
    } else {
        100
    };

    player.get_mining_speed(block_name).await / hardness / i as f32
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
    manager.register(Open::False());
    manager.register(Powered::False());
    manager.register(Unstable::False());
    manager.register(SignalFire::False());
    manager.register(SlabType::Bottom);
    manager.register(South::False);
    manager.register(StairShape::Straight);
    manager.register(Up::False);
    manager.register(Waterlogged::False());
    manager.register(West::False);

    manager.build_properties_registry();

    Arc::new(manager)
}
