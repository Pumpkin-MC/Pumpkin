use std::any::Any;
use std::sync::Arc;

use crate::block::registry::BlockActionResult;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use crate::world::World;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::gamemode::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

pub struct BoneMealItem;

impl BoneMealItem {
    pub fn grow_crop(stack: &mut ItemStack, world: &Arc<World>, pos: &BlockPos) -> bool {
        let (block, state) = world.get_block_and_state(pos);
        let Some(server) = world.server.upgrade() else {
            return false;
        };
        if server.block_registry.bone_meal(block, world, pos, state.id) {
            stack.decrement(1);
            if stack.item_count == 0 {
                stack.clear();
            }
            true
        } else {
            false
        }
    }
}

impl ItemMetadata for BoneMealItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::BONE_MEAL.id])
    }
}

impl ItemBehaviour for BoneMealItem {
    #[allow(clippy::too_many_lines)]
    fn use_on_block(
        &self,
        item: &mut ItemStack,
        player: &Player,
        location: BlockPos,
        _face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        _block: &Block,
        _server: &Server,
    ) -> BlockActionResult {
        let world = player.world();
        let original = (player.gamemode.load() == GameMode::Creative).then(|| item.clone());
        if Self::grow_crop(item, &world, &location) {
            if let Some(original) = original {
                *item = original;
            }
            world.emit_game_event(GameEvent::ItemInteractFinish.name(), player.position());
            world.sync_world_event(WorldEvent::ParticlesAndSoundPlantGrowth, location, 15);
            BlockActionResult::SuccessServer
        } else {
            BlockActionResult::Pass
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
