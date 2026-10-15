use pumpkin_data::block_properties::ScaffoldingLikeProperties;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::{Block, BlockDirection, BlockStateId};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockAccessor;

use crate::block::{BlockBehaviour, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnPlaceArgs};

#[pumpkin_block("minecraft:scaffolding")]
pub struct ScaffoldingBlock;

impl ScaffoldingBlock {
    #[must_use]
    pub fn get_distance(world: &dyn BlockAccessor, pos: &BlockPos) -> u8 {
        let below_pos = pos.down();
        let (below_block, below_state) = world.get_block_and_state(&below_pos);
        if below_block == &Block::SCAFFOLDING {
            return ScaffoldingLikeProperties::from_state_id(below_state.id).distance;
        } else if below_state.is_side_solid(BlockDirection::Up) && below_block.is_solid() {
            return 0;
        }

        let mut min_dist = 7u8;
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

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let distance = Self::get_distance(args.world, args.position);
        let mut props = ScaffoldingLikeProperties::default(args.block);
        props.distance = distance;
        props.bottom = Self::is_bottom(args.world, args.position, distance);
        props.waterlogged = args.replacing.water_source();
        props.to_state_id(args.block)
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
        let distance = Self::get_distance(args.world, args.position);
        if distance == 7 {
            return Block::AIR.default_state.id;
        }
        let mut new_props = props;
        new_props.distance = distance;
        new_props.bottom = Self::is_bottom(args.world, args.position, distance);
        new_props.to_state_id(args.block)
    }
}
