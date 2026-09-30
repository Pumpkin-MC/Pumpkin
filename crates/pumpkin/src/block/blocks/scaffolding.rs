use pumpkin_data::block_properties::ScaffoldingLikeProperties;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::{Block, BlockDirection, BlockStateId};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::{BlockAccessor, BlockFlags};

use crate::block::{
    BlockBehaviour, CanPlaceAtArgs, CanUpdateAtArgs, GetStateForNeighborUpdateArgs, OnPlaceArgs,
    OnScheduledTickArgs, PlacedArgs,
};
use crate::entity::falling::FallingEntity;

const TICK_DELAY: u8 = 1;
pub const STABILITY_MAX_DISTANCE: u8 = 7;

#[pumpkin_block("minecraft:scaffolding")]
pub struct ScaffoldingBlock;

impl ScaffoldingBlock {
    #[must_use]
    pub fn get_distance(world: &dyn BlockAccessor, pos: &BlockPos) -> u8 {
        let below_pos = pos.down();
        let (below_block, below_state) = world.get_block_and_state(&below_pos);
        let mut min_dist = 7u8;
        if below_block == &Block::SCAFFOLDING {
            min_dist = ScaffoldingLikeProperties::from_state_id(below_state.id).distance;
        } else if below_state.is_side_solid(BlockDirection::Up) {
            return 0;
        }

        for dir in BlockDirection::horizontal() {
            let neighbor_pos = pos.offset(dir.to_offset());
            let (neighbor_block, neighbor_state) = world.get_block_and_state(&neighbor_pos);
            if neighbor_block == &Block::SCAFFOLDING {
                let dist = ScaffoldingLikeProperties::from_state_id(neighbor_state.id).distance;
                min_dist = min_dist.min(dist.saturating_add(1));
                if min_dist == 1 {
                    break;
                }
            }
        }
        min_dist.min(7)
    }

    #[must_use]
    pub fn is_bottom(world: &dyn BlockAccessor, pos: &BlockPos, distance: u8) -> bool {
        distance > 0 && world.get_block(&pos.down()) != &Block::SCAFFOLDING
    }
}

impl BlockBehaviour for ScaffoldingBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        Self::get_distance(args.block_accessor, args.position) < 7
    }

    // `canBeReplaced`: true while holding scaffolding, the only time this is called.
    fn can_update_at(&self, _args: CanUpdateAtArgs<'_>) -> bool {
        true
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let distance = Self::get_distance(args.world, args.position);
        let mut props = ScaffoldingLikeProperties::default(args.block);
        props.distance = distance;
        props.bottom = Self::is_bottom(args.world, args.position, distance);
        props.waterlogged = args.replacing.water_source();
        props.to_state_id(args.block)
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        args.world.schedule_block_tick(
            args.block,
            *args.position,
            TICK_DELAY,
            TickPriority::Normal,
        );
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let props = ScaffoldingLikeProperties::from_state_id(args.state_id);
        if props.waterlogged {
            args.world.schedule_fluid_tick(
                &Fluid::WATER,
                *args.position,
                Fluid::WATER.flow_speed as u8,
                TickPriority::Normal,
            );
        }
        args.world.schedule_block_tick(
            args.block,
            *args.position,
            TICK_DELAY,
            TickPriority::Normal,
        );
        args.state_id
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let state_id = args.world.get_block_state_id(args.position);
        let props = ScaffoldingLikeProperties::from_state_id(state_id);
        let distance = Self::get_distance(args.world.as_ref(), args.position);
        let mut new_props = props;
        new_props.distance = distance;
        new_props.bottom = Self::is_bottom(args.world.as_ref(), args.position, distance);
        let new_state_id = new_props.to_state_id(args.block);

        if distance == STABILITY_MAX_DISTANCE {
            if props.distance == STABILITY_MAX_DISTANCE {
                FallingEntity::replace_spawn(args.world, *args.position, new_state_id);
            } else {
                args.world
                    .break_block(args.position, None, BlockFlags::NOTIFY_ALL);
            }
        } else if new_state_id != state_id {
            args.world
                .set_block_state(args.position, new_state_id, BlockFlags::NOTIFY_ALL);
        }
    }

    // `getCollisionShape` is empty for a placement context, so entities never block placing it.
    fn has_placement_collision(&self) -> bool {
        false
    }
}
