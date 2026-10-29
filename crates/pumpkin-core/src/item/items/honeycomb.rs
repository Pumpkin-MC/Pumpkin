use std::sync::Arc;

use crate::block::UseWithItemArgs;
use crate::block::blocks::weathering_copper::with_properties_of;
use crate::block::entities::BlockEntity;
use crate::block::registry::BlockActionResult;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use crate::world::World;
use pumpkin_data::block_properties::{
    ChestLikeProperties, ChestType, DoubleBlockHalf, OakDoorLikeProperties,
};
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
    let Some(replacement) = get_waxed_equivalent(block.id) else {
        return false;
    };
    let new_block = replacement.to_block();
    let current_state_id = world.get_block_state_id(&location);
    let new_state_id = with_properties_of(block, current_state_id, new_block);

    if block.has_tag(&tag::Block::MINECRAFT_DOORS) {
        let door_props = OakDoorLikeProperties::from_state_id(current_state_id);
        let other_half_pos = match door_props.half {
            DoubleBlockHalf::Lower => location.up(),
            DoubleBlockHalf::Upper => location.down(),
        };
        let (other_block, other_state_id) = world.get_block_and_state_id(&other_half_pos);
        if other_block == block {
            let other_new_state_id = with_properties_of(other_block, other_state_id, new_block);
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
            if let Some(neighbor_waxed) = get_waxed_equivalent(neighbor_block.id) {
                let neighbor_new_block = neighbor_waxed.to_block();
                let neighbor_new_state =
                    with_properties_of(neighbor_block, neighbor_state_id, neighbor_new_block);
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

const fn get_waxed_equivalent(id: BlockId) -> Option<BlockId> {
    match id {
        BlockId::OXIDIZED_COPPER => Some(BlockId::WAXED_OXIDIZED_COPPER),
        BlockId::WEATHERED_COPPER => Some(BlockId::WAXED_WEATHERED_COPPER),
        BlockId::EXPOSED_COPPER => Some(BlockId::WAXED_EXPOSED_COPPER),
        BlockId::COPPER_BLOCK => Some(BlockId::WAXED_COPPER_BLOCK),
        BlockId::OXIDIZED_CHISELED_COPPER => Some(BlockId::WAXED_OXIDIZED_CHISELED_COPPER),
        BlockId::WEATHERED_CHISELED_COPPER => Some(BlockId::WAXED_WEATHERED_CHISELED_COPPER),
        BlockId::EXPOSED_CHISELED_COPPER => Some(BlockId::WAXED_EXPOSED_CHISELED_COPPER),
        BlockId::CHISELED_COPPER => Some(BlockId::WAXED_CHISELED_COPPER),
        BlockId::OXIDIZED_COPPER_GRATE => Some(BlockId::WAXED_OXIDIZED_COPPER_GRATE),
        BlockId::WEATHERED_COPPER_GRATE => Some(BlockId::WAXED_WEATHERED_COPPER_GRATE),
        BlockId::EXPOSED_COPPER_GRATE => Some(BlockId::WAXED_EXPOSED_COPPER_GRATE),
        BlockId::COPPER_GRATE => Some(BlockId::WAXED_COPPER_GRATE),
        BlockId::OXIDIZED_CUT_COPPER => Some(BlockId::WAXED_OXIDIZED_CUT_COPPER),
        BlockId::WEATHERED_CUT_COPPER => Some(BlockId::WAXED_WEATHERED_CUT_COPPER),
        BlockId::EXPOSED_CUT_COPPER => Some(BlockId::WAXED_EXPOSED_CUT_COPPER),
        BlockId::CUT_COPPER => Some(BlockId::WAXED_CUT_COPPER),
        BlockId::OXIDIZED_CUT_COPPER_STAIRS => Some(BlockId::WAXED_OXIDIZED_CUT_COPPER_STAIRS),
        BlockId::WEATHERED_CUT_COPPER_STAIRS => Some(BlockId::WAXED_WEATHERED_CUT_COPPER_STAIRS),
        BlockId::EXPOSED_CUT_COPPER_STAIRS => Some(BlockId::WAXED_EXPOSED_CUT_COPPER_STAIRS),
        BlockId::CUT_COPPER_STAIRS => Some(BlockId::WAXED_CUT_COPPER_STAIRS),
        BlockId::OXIDIZED_CUT_COPPER_SLAB => Some(BlockId::WAXED_OXIDIZED_CUT_COPPER_SLAB),
        BlockId::WEATHERED_CUT_COPPER_SLAB => Some(BlockId::WAXED_WEATHERED_CUT_COPPER_SLAB),
        BlockId::EXPOSED_CUT_COPPER_SLAB => Some(BlockId::WAXED_EXPOSED_CUT_COPPER_SLAB),
        BlockId::CUT_COPPER_SLAB => Some(BlockId::WAXED_CUT_COPPER_SLAB),
        BlockId::OXIDIZED_COPPER_BULB => Some(BlockId::WAXED_OXIDIZED_COPPER_BULB),
        BlockId::WEATHERED_COPPER_BULB => Some(BlockId::WAXED_WEATHERED_COPPER_BULB),
        BlockId::EXPOSED_COPPER_BULB => Some(BlockId::WAXED_EXPOSED_COPPER_BULB),
        BlockId::COPPER_BULB => Some(BlockId::WAXED_COPPER_BULB),
        BlockId::OXIDIZED_COPPER_DOOR => Some(BlockId::WAXED_OXIDIZED_COPPER_DOOR),
        BlockId::WEATHERED_COPPER_DOOR => Some(BlockId::WAXED_WEATHERED_COPPER_DOOR),
        BlockId::EXPOSED_COPPER_DOOR => Some(BlockId::WAXED_EXPOSED_COPPER_DOOR),
        BlockId::COPPER_DOOR => Some(BlockId::WAXED_COPPER_DOOR),
        BlockId::OXIDIZED_COPPER_TRAPDOOR => Some(BlockId::WAXED_OXIDIZED_COPPER_TRAPDOOR),
        BlockId::WEATHERED_COPPER_TRAPDOOR => Some(BlockId::WAXED_WEATHERED_COPPER_TRAPDOOR),
        BlockId::EXPOSED_COPPER_TRAPDOOR => Some(BlockId::WAXED_EXPOSED_COPPER_TRAPDOOR),
        BlockId::COPPER_TRAPDOOR => Some(BlockId::WAXED_COPPER_TRAPDOOR),
        BlockId::OXIDIZED_COPPER_BARS => Some(BlockId::WAXED_OXIDIZED_COPPER_BARS),
        BlockId::WEATHERED_COPPER_BARS => Some(BlockId::WAXED_WEATHERED_COPPER_BARS),
        BlockId::EXPOSED_COPPER_BARS => Some(BlockId::WAXED_EXPOSED_COPPER_BARS),
        BlockId::COPPER_BARS => Some(BlockId::WAXED_COPPER_BARS),
        BlockId::OXIDIZED_COPPER_CHAIN => Some(BlockId::WAXED_OXIDIZED_COPPER_CHAIN),
        BlockId::WEATHERED_COPPER_CHAIN => Some(BlockId::WAXED_WEATHERED_COPPER_CHAIN),
        BlockId::EXPOSED_COPPER_CHAIN => Some(BlockId::WAXED_EXPOSED_COPPER_CHAIN),
        BlockId::COPPER_CHAIN => Some(BlockId::WAXED_COPPER_CHAIN),
        BlockId::OXIDIZED_COPPER_LANTERN => Some(BlockId::WAXED_OXIDIZED_COPPER_LANTERN),
        BlockId::WEATHERED_COPPER_LANTERN => Some(BlockId::WAXED_WEATHERED_COPPER_LANTERN),
        BlockId::EXPOSED_COPPER_LANTERN => Some(BlockId::WAXED_EXPOSED_COPPER_LANTERN),
        BlockId::COPPER_LANTERN => Some(BlockId::WAXED_COPPER_LANTERN),
        BlockId::OXIDIZED_COPPER_CHEST => Some(BlockId::WAXED_OXIDIZED_COPPER_CHEST),
        BlockId::WEATHERED_COPPER_CHEST => Some(BlockId::WAXED_WEATHERED_COPPER_CHEST),
        BlockId::EXPOSED_COPPER_CHEST => Some(BlockId::WAXED_EXPOSED_COPPER_CHEST),
        BlockId::COPPER_CHEST => Some(BlockId::WAXED_COPPER_CHEST),
        BlockId::OXIDIZED_COPPER_GOLEM_STATUE => Some(BlockId::WAXED_OXIDIZED_COPPER_GOLEM_STATUE),
        BlockId::WEATHERED_COPPER_GOLEM_STATUE => {
            Some(BlockId::WAXED_WEATHERED_COPPER_GOLEM_STATUE)
        }
        BlockId::EXPOSED_COPPER_GOLEM_STATUE => Some(BlockId::WAXED_EXPOSED_COPPER_GOLEM_STATUE),
        BlockId::COPPER_GOLEM_STATUE => Some(BlockId::WAXED_COPPER_GOLEM_STATUE),
        BlockId::OXIDIZED_LIGHTNING_ROD => Some(BlockId::WAXED_OXIDIZED_LIGHTNING_ROD),
        BlockId::WEATHERED_LIGHTNING_ROD => Some(BlockId::WAXED_WEATHERED_LIGHTNING_ROD),
        BlockId::EXPOSED_LIGHTNING_ROD => Some(BlockId::WAXED_EXPOSED_LIGHTNING_ROD),
        BlockId::LIGHTNING_ROD => Some(BlockId::WAXED_LIGHTNING_ROD),
        _ => None,
    }
}
