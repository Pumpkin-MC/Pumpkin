use std::sync::Arc;

use crate::block::UseWithItemArgs;
use crate::block::entities::BlockEntity;
use crate::block::registry::BlockActionResult;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use crate::world::World;
use pumpkin_data::block_properties::{
    ChestLikeProperties, ChestType, DoubleBlockHalf, OakDoorLikeProperties,
};
use pumpkin_data::block_transformer::get_waxed;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::Taggable;
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{Block, tag};
use pumpkin_data::{BlockDirection, BlockId};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;

pub struct HoneyCombItem;

impl ItemMetadata for HoneyCombItem {
    fn ids() -> Box<[u16]> {
        [Item::HONEYCOMB.id].into()
    }
}

impl ItemBehaviour for HoneyCombItem {
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
        if try_wax_block(&world, location, block) {
            item.decrement_unless_creative(player.gamemode.load(), 1);
            BlockActionResult::Success
        } else {
            BlockActionResult::Pass
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Waxes the block at `location` if it has a waxed equivalent, emitting the wax
/// particles and sound on success.
pub(crate) fn try_wax_block(world: &Arc<World>, location: BlockPos, block: &Block) -> bool {
    let Some(replacement) = get_waxed(block.id) else {
        return false;
    };
    let new_block = replacement.to_block();
    let current_state_id = world.get_block_state_id(&location);
    let new_state_id = new_block.with_properties_of(block, current_state_id);

    if block.has_tag(&tag::Block::MINECRAFT_DOORS) {
        let door_props = OakDoorLikeProperties::from_state_id(current_state_id);
        let other_half_pos = match door_props.half {
            DoubleBlockHalf::Lower => location.up(),
            DoubleBlockHalf::Upper => location.down(),
        };
        let (other_block, other_state_id) = world.get_block_and_state_id(&other_half_pos);
        if other_block == block {
            let other_new_state_id = new_block.with_properties_of(other_block, other_state_id);
            world.set_block_state(
                &other_half_pos,
                other_new_state_id,
                BlockFlags::NOTIFY_LISTENERS | BlockFlags::FORCE_STATE,
            );
        }
    } else if matches!(
        block.id,
        BlockId::COPPER_CHEST
            | BlockId::EXPOSED_COPPER_CHEST
            | BlockId::WEATHERED_COPPER_CHEST
            | BlockId::OXIDIZED_COPPER_CHEST
    ) {
        let chest_props = ChestLikeProperties::from_state_id(current_state_id);
        if chest_props.r#type != ChestType::Single {
            let neighbor_dir = match chest_props.r#type {
                ChestType::Left => chest_props.facing.rotate_clockwise(),
                ChestType::Right => chest_props.facing.rotate_counter_clockwise(),
                ChestType::Single => chest_props.facing,
            };
            let neighbor_pos = location.offset(neighbor_dir.to_offset());
            let (neighbor_block, neighbor_state_id) = world.get_block_and_state_id(&neighbor_pos);
            if let Some(neighbor_waxed) = get_waxed(neighbor_block.id) {
                let neighbor_new_block = neighbor_waxed.to_block();
                let neighbor_new_state =
                    neighbor_new_block.with_properties_of(neighbor_block, neighbor_state_id);
                world.set_block_state(
                    &neighbor_pos,
                    neighbor_new_state,
                    BlockFlags::NOTIFY_LISTENERS,
                );
                world.emit_game_event(GameEvent::BlockChange.name(), neighbor_pos.to_f64());
                world.sync_world_event(WorldEvent::ParticlesWaxOn, neighbor_pos, 0);
                world.play_sound(
                    Sound::ItemHoneycombWaxOn,
                    SoundCategory::Blocks,
                    &location.to_f64(),
                );
            }
        }
    }

    world.set_block_state(&location, new_state_id, BlockFlags::NOTIFY_ALL);
    world.emit_game_event(GameEvent::BlockChange.name(), location.to_f64());
    world.sync_world_event(WorldEvent::ParticlesWaxOn, location, 0);
    world.play_sound(
        Sound::ItemHoneycombWaxOn,
        SoundCategory::Blocks,
        &location.to_f64(),
    );
    true
}

impl HoneyCombItem {
    pub fn apply_to_sign(
        &self,
        args: &UseWithItemArgs<'_>,
        block_entity: &Arc<dyn BlockEntity>,
        sign_entity: &crate::block::entities::sign::SignEntityRef<'_>,
    ) -> BlockActionResult {
        if sign_entity.is_waxed() {
            return BlockActionResult::PassToDefaultBlockAction;
        }
        sign_entity.set_waxed(true);

        args.world.update_block_entity(block_entity);
        args.world
            .sync_world_event(WorldEvent::ParticlesWaxOn, *args.position, 0);
        args.world.play_sound(
            Sound::ItemHoneycombWaxOn,
            SoundCategory::Blocks,
            &args.position.to_f64(),
        );

        BlockActionResult::Success
    }
}
