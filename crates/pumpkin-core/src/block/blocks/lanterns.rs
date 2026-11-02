use crate::block::blocks::weathering_copper::change_over_time;
use crate::block::{
    BlockBehaviour, BlockIsReplacing, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnPlaceArgs,
    PathComputationType, RandomTickArgs, can_support_center,
};
use crate::entity::EntityBase;
use pumpkin_data::block_properties::{Facing, LanternLikeProperties};
use pumpkin_data::fluid::Fluid;
use pumpkin_data::{Block, BlockDirection, BlockState, BlockStateId};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockAccessor;

#[pumpkin_block_from_tag("minecraft:lanterns")]
pub struct LanternBlock;

impl BlockBehaviour for LanternBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = LanternLikeProperties::default(args.block);
        props.r#waterlogged = args.replacing.water_source();

        let mut directions = args.player.get_entity().get_entity_facing_order();
        if args.replacing == BlockIsReplacing::None {
            let face = args.direction.to_facing();
            if let Some(i) = directions.iter().position(|&d| d == face)
                && i > 0
            {
                directions.copy_within(0..i, 1);
                directions[0] = face;
            }
        }

        for dir in directions {
            if dir == Facing::Up || dir == Facing::Down {
                let hanging = dir == Facing::Up;
                if can_survive(args.world, args.position, hanging) {
                    props.r#hanging = hanging;
                    return props.to_state_id(args.block);
                }
            }
        }

        Block::AIR.default_state.id
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        if args.player.is_none() {
            let props = LanternLikeProperties::from_state_id(args.state.id);
            return can_survive(args.block_accessor, args.position, props.hanging);
        }
        can_survive(args.block_accessor, args.position, false)
            || can_survive(args.block_accessor, args.position, true)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let props = LanternLikeProperties::from_state_id(args.state_id);
        if props.r#waterlogged {
            args.world.schedule_fluid_tick(
                &Fluid::WATER,
                *args.position,
                Fluid::WATER.flow_speed as u8,
                TickPriority::Normal,
            );
        }
        let support_dir = if props.r#hanging {
            BlockDirection::Up
        } else {
            BlockDirection::Down
        };
        if support_dir == args.direction && !can_survive(args.world, args.position, props.r#hanging)
        {
            return Block::AIR.default_state.id;
        }
        args.state_id
    }

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        change_over_time(args.world, args.position, args.block);
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}

fn can_survive(block_accessor: &dyn BlockAccessor, position: &BlockPos, hanging: bool) -> bool {
    let direction = if hanging {
        BlockDirection::Up
    } else {
        BlockDirection::Down
    };
    can_support_center(
        block_accessor,
        &position.offset(direction.to_offset()),
        direction.opposite(),
    )
}
