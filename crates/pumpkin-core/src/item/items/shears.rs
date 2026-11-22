use std::any::Any;

use crate::block::registry::BlockActionResult;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::CaveVinesLikeProperties;
use pumpkin_data::block_properties::KelpLikeProperties;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{Block, BlockDirection, BlockStateId};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;

pub struct ShearsItem;

impl ItemMetadata for ShearsItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::SHEARS.id])
    }
}

impl ItemBehaviour for ShearsItem {
    fn use_on_block(
        &self,
        item: &mut ItemStack,
        player: &Player,
        location: BlockPos,
        _face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        block: &Block,
        _server: &Server,
    ) -> BlockActionResult {
        let world = player.world();
        let state_id = world.get_block_state_id(&location);

        if handle_growing_plant(item, player, &location, block, state_id) {
            BlockActionResult::Success
        } else {
            BlockActionResult::Pass
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn handle_growing_plant(
    item: &mut ItemStack,
    player: &Player,
    location: &BlockPos,
    block: &Block,
    state_id: BlockStateId,
) -> bool {
    let new_state_id = if KelpLikeProperties::handles_block_id(block.id) {
        let mut props = KelpLikeProperties::from_state_id(state_id);
        if props.age >= 25 {
            return false;
        }
        props.age = 25;
        props.to_state_id(block)
    } else if CaveVinesLikeProperties::handles_block_id(block.id) {
        let mut props = CaveVinesLikeProperties::from_state_id(state_id);
        if props.age >= 25 {
            return false;
        }
        props.age = 25;
        props.to_state_id(block)
    } else {
        return false;
    };

    let world = player.world();
    world.set_block_state(location, new_state_id, BlockFlags::NOTIFY_ALL);
    world.emit_game_event(GameEvent::BlockChange.name(), location.to_f64());
    world.play_sound(
        Sound::BlockGrowingPlantCrop,
        SoundCategory::Blocks,
        &location.to_f64(),
    );
    if player.gamemode.load() != pumpkin_util::GameMode::Creative {
        let _ = item.damage_item(1);
    }
    true
}
