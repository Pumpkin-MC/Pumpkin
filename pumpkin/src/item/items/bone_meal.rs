use std::pin::Pin;

use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::item::Item;
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::item::ItemStack;
use pumpkin_world::world::BlockFlags;

pub struct BoneMealItem;

impl ItemMetadata for BoneMealItem {
    fn ids() -> Box<[u16]> {
        [Item::BONE_MEAL.id].into()
    }
}

/// Check if a block is a crop that can grow (wheat, carrots, potatoes, beetroot).
fn is_growable_crop(block: &Block) -> bool {
    block == &Block::WHEAT
        || block == &Block::CARROTS
        || block == &Block::POTATOES
        || block == &Block::BEETROOTS
        || block == &Block::MELON_STEM
        || block == &Block::PUMPKIN_STEM
        || block == &Block::TORCHFLOWER_CROP
}

/// Check if a block can be bone-mealed to produce growth or decoration.
fn is_fertilizable(block: &Block) -> bool {
    is_growable_crop(block)
        || block == &Block::GRASS_BLOCK
        || block == &Block::MOSS_BLOCK
        || block == &Block::PALE_MOSS_BLOCK
        || block == &Block::COCOA
        || block == &Block::SWEET_BERRY_BUSH
        || block == &Block::CAVE_VINES
        || block == &Block::CAVE_VINES_PLANT
        || block == &Block::KELP
        || block == &Block::KELP_PLANT
        || block == &Block::BAMBOO
        || block == &Block::BAMBOO_SAPLING
        || block == &Block::SEA_PICKLE
        || block == &Block::SMALL_DRIPLEAF
        || block == &Block::BIG_DRIPLEAF
        || block == &Block::PINK_PETALS
}

impl ItemBehaviour for BoneMealItem {
    fn use_on_block<'a>(
        &'a self,
        item: &'a mut ItemStack,
        player: &'a Player,
        location: BlockPos,
        _face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        block: &'a Block,
        _server: &'a Server,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            if !is_fertilizable(block) {
                return;
            }

            let world = player.world();

            // For crops, advance growth by 2-5 stages (random)
            // This is simplified — vanilla has complex per-crop growth logic
            if is_growable_crop(block) {
                // Get current state and advance to the next few stages
                let current_state = world.get_block_state(&location).await;
                // Find the block's last state (fully grown) by checking property "age"
                // The max age differs per crop: wheat=7, carrots=7, potatoes=7, beetroots=3
                let max_state_offset = match block {
                    b if b == &Block::BEETROOTS => 3,
                    b if b == &Block::TORCHFLOWER_CROP => 2,
                    _ => 7, // wheat, carrots, potatoes, melon_stem, pumpkin_stem
                };

                let base_state = block.default_state.id;
                let current_age = current_state.id.saturating_sub(base_state);
                if current_age >= max_state_offset {
                    // Already fully grown
                    return;
                }

                // Advance by 2-5 stages (simplified: advance by 3)
                let new_age = (current_age + 3).min(max_state_offset);
                let new_state_id = base_state + new_age;

                world
                    .set_block_state(&location, new_state_id, BlockFlags::NOTIFY_ALL)
                    .await;
            }

            // Show bone meal particles (WorldEvent 1505 = BONE_MEAL_USE)
            world
                .sync_world_event(WorldEvent::BoneMealUsed, location, 0)
                .await;

            // Consume item
            let gamemode = player.gamemode.load();
            item.decrement_unless_creative(gamemode, 1);
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
