use pumpkin_data::{
    Block, BlockDirection, BlockStateId,
    block_properties::{BlockProperties, OakLeavesLikeProperties},
    tag,
    tag::Taggable,
};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{
    tick::TickPriority,
    world::{BlockAccessor, BlockFlags},
};

use crate::block::{
    BlockBehaviour, GetStateForNeighborUpdateArgs, OnPlaceArgs, OnScheduledTickArgs, RandomTickArgs,
};

pub const DECAY_DISTANCE: u8 = 7;

#[pumpkin_block_from_tag("minecraft:leaves")]
pub struct LeavesBlock;

#[must_use]
pub fn get_distance_at(block: &Block, state_id: BlockStateId) -> u8 {
    if block.has_tag(&tag::Block::MINECRAFT_PREVENTS_NEARBY_LEAF_DECAY) {
        0
    } else if block.has_tag(&tag::Block::MINECRAFT_LEAVES) {
        OakLeavesLikeProperties::from_state_id(state_id, block).distance
    } else {
        DECAY_DISTANCE
    }
}

#[must_use]
pub fn update_distance(
    world: &dyn BlockAccessor,
    pos: &BlockPos,
    mut props: OakLeavesLikeProperties,
) -> OakLeavesLikeProperties {
    let mut min_distance = DECAY_DISTANCE;
    for direction in BlockDirection::all() {
        let neighbor_pos = pos.offset(direction.to_offset());
        let (neighbor_block, neighbor_state) = world.get_block_and_state(&neighbor_pos);
        let dist = get_distance_at(neighbor_block, neighbor_state.id).saturating_add(1);
        min_distance = min_distance.min(dist);
        if min_distance == 1 {
            break;
        }
    }
    props.distance = min_distance.min(DECAY_DISTANCE);
    props
}

impl BlockBehaviour for LeavesBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props =
            OakLeavesLikeProperties::from_state_id(args.block.default_state.id, args.block);
        props.persistent = true;
        props.waterlogged = args.replacing.water_source();
        props = update_distance(args.world, args.position, props);
        props.to_state_id(args.block)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let current_props = OakLeavesLikeProperties::from_state_id(args.state_id, args.block);
        if current_props.waterlogged {
            args.world.schedule_fluid_tick(
                &pumpkin_data::fluid::Fluid::WATER,
                *args.position,
                pumpkin_data::fluid::Fluid::WATER.flow_speed as u8,
                TickPriority::Normal,
            );
        }

        let neighbor_block = args.world.get_block(args.neighbor_position);
        let distance_from_neighbor =
            get_distance_at(neighbor_block, args.neighbor_state_id).saturating_add(1);

        if distance_from_neighbor != 1 || current_props.distance != distance_from_neighbor {
            args.world
                .schedule_block_tick(args.block, *args.position, 1, TickPriority::Normal);
        }

        args.state_id
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let state_id = args.world.get_block_state_id(args.position);
        let props = OakLeavesLikeProperties::from_state_id(state_id, args.block);
        let updated_props = update_distance(&**args.world, args.position, props);
        let new_state_id = updated_props.to_state_id(args.block);
        if new_state_id != state_id {
            args.world
                .set_block_state(args.position, new_state_id, BlockFlags::NOTIFY_ALL);
        }
    }

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        let state_id = args.world.get_block_state_id(args.position);
        let props = OakLeavesLikeProperties::from_state_id(state_id, args.block);
        if !props.persistent && props.distance == DECAY_DISTANCE {
            args.world
                .break_block(args.position, None, BlockFlags::empty());
        }
    }
}
