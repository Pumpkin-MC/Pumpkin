use std::sync::Arc;

use crate::block::UseWithItemArgs;
use crate::block::entities::BlockEntity;
use crate::block::registry::BlockActionResult;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use crate::world::World;
use pumpkin_data::block_properties::{OakDoorLikeProperties, OakTrapdoorLikeProperties};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::Taggable;
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{Block, tag};
use pumpkin_data::{BlockDirection, BlockId, BlockStateId};
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

    let from_state_id = world.get_block_state_id(&location);
    let new_state_id = waxed_state_id(block, from_state_id, new_block);

    world.set_block_state(&location, new_state_id, BlockFlags::NOTIFY_ALL);
    world.sync_world_event(WorldEvent::ParticlesWaxOn, location, 0);
    true
}

fn waxed_state_id(
    from_block: &Block,
    from_state_id: BlockStateId,
    to_block: &Block,
) -> BlockStateId {
    if from_block.has_tag(&tag::Block::MINECRAFT_DOORS) {
        let door_props = OakDoorLikeProperties::from_state_id(from_state_id);
        let mut new_door_properties = OakDoorLikeProperties::default(to_block);
        new_door_properties.facing = door_props.facing;
        new_door_properties.open = door_props.open;
        new_door_properties.half = door_props.half;
        new_door_properties.hinge = door_props.hinge;
        new_door_properties.powered = door_props.powered;
        new_door_properties.to_state_id(to_block)
    } else if from_block.has_tag(&tag::Block::MINECRAFT_TRAPDOORS) {
        let trapdoor_props = OakTrapdoorLikeProperties::from_state_id(from_state_id);
        let mut new_trapdoor_properties = OakTrapdoorLikeProperties::default(to_block);
        new_trapdoor_properties.facing = trapdoor_props.facing;
        new_trapdoor_properties.half = trapdoor_props.half;
        new_trapdoor_properties.open = trapdoor_props.open;
        new_trapdoor_properties.powered = trapdoor_props.powered;
        new_trapdoor_properties.waterlogged = trapdoor_props.waterlogged;
        new_trapdoor_properties.to_state_id(to_block)
    } else {
        to_block.default_state.id
    }
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
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::block_properties::{Half, HorizontalFacing};

    #[test]
    fn waxing_preserves_trapdoor_properties() {
        for (from, to) in [
            (BlockId::COPPER_TRAPDOOR, BlockId::WAXED_COPPER_TRAPDOOR),
            (
                BlockId::EXPOSED_COPPER_TRAPDOOR,
                BlockId::WAXED_EXPOSED_COPPER_TRAPDOOR,
            ),
            (
                BlockId::WEATHERED_COPPER_TRAPDOOR,
                BlockId::WAXED_WEATHERED_COPPER_TRAPDOOR,
            ),
            (
                BlockId::OXIDIZED_COPPER_TRAPDOOR,
                BlockId::WAXED_OXIDIZED_COPPER_TRAPDOOR,
            ),
        ] {
            let from_block = from.to_block();
            let to_block = to.to_block();

            let mut props = OakTrapdoorLikeProperties::default(from_block);
            props.facing = HorizontalFacing::East;
            props.half = Half::Top;
            props.open = true;
            props.powered = true;
            props.waterlogged = true;
            let state_id = props.to_state_id(from_block);

            let waxed = waxed_state_id(from_block, state_id, to_block);
            assert_eq!(Block::from_state_id(waxed), to_block);

            let waxed_props = OakTrapdoorLikeProperties::from_state_id(waxed);
            assert_eq!(waxed_props.facing, HorizontalFacing::East);
            assert_eq!(waxed_props.half, Half::Top);
            assert!(waxed_props.open);
            assert!(waxed_props.powered);
            assert!(waxed_props.waterlogged);
        }
    }

    #[test]
    fn waxing_other_blocks_uses_default_state() {
        let waxed = waxed_state_id(
            &Block::COPPER_BLOCK,
            Block::COPPER_BLOCK.default_state.id,
            &Block::WAXED_COPPER_BLOCK,
        );
        assert_eq!(waxed, Block::WAXED_COPPER_BLOCK.default_state.id);
    }
}
