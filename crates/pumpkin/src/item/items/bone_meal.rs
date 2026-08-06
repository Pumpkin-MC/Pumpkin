use std::pin::Pin;

use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::Block;
use pumpkin_data::block_properties::{BlockProperties, WheatLikeProperties};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::Taggable;
use pumpkin_data::world::WorldEvent;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;

pub struct BoneMealItem;

impl ItemMetadata for BoneMealItem {
    fn ids() -> Box<[u16]> {
        [Item::BONE_MEAL.id].into()
    }
}

impl ItemBehaviour for BoneMealItem {
    fn use_on_block<'a>(
        &'a self,
        item: &'a mut ItemStack,
        player: &'a Player,
        location: BlockPos,
        _face: pumpkin_data::BlockDirection,
        _cursor_pos: Vector3<f32>,
        block: &'a Block,
        _server: &'a Server,
    ) -> Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let world = player.world();

            // Check if this block can be grown with bone meal
            // Vanilla: BoneMealItem.growCrop -> BonemealableBlock.isValidBonemealTarget
            if block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_CROPS) {
                // Get the current block state
                let state_id = world.get_block_state_id(&location);

                // Try to extract age from crop properties
                // Vanilla: CropBlock.isValidBonemealTarget -> !this.isMaxAge(state)
                let props = WheatLikeProperties::from_state_id(state_id, block);
                let current_age = i32::from(props.age);
                let max_age = 7; // CropBlock.MAX_AGE from vanilla source

                // Only apply bone meal if not at max age
                if current_age < max_age {
                    // Vanilla: CropBlock.performBonemeal -> this.growCrops
                    // growCrops advances age by: Mth.nextInt(level.getRandom(), 2, 5)
                    let age_increase = rand::rng().random_range(2..=5);
                    let new_age = std::cmp::min(current_age + age_increase, max_age) as u8;

                    // Set the new state
                    let mut new_props = WheatLikeProperties::from_state_id(state_id, block);
                    new_props.age = new_age;
                    let new_state_id = new_props.to_state_id(block);

                    world
                        .set_block_state(&location, new_state_id, BlockFlags::NOTIFY_ALL)
                        .await;

                    // Play the growth particles and sound
                    // Vanilla: BoneMealItem.useOn line 43-44: level.levelEvent(1505, pos, 15);
                    // the 15 is the particle-count data value for the client's green sparkle effect
                    world.sync_world_event(WorldEvent::ParticlesAndSoundPlantGrowth, location, 15);

                    // Consume the item
                    item.decrement_unless_creative(player.gamemode.load(), 1);
                }
            }
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
