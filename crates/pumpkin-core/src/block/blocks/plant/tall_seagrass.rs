use pumpkin_data::{
    Block, BlockDirection, BlockId, BlockStateId,
    block_properties::{Axis, DoubleBlockHalf, TallSeagrassLikeProperties, WaterLikeProperties},
    fluid::Fluid,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{
    tick::TickPriority,
    world::{BlockAccessor, BlockFlags},
};

use crate::block::{
    BlockBehaviour, BrokenArgs, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, PlacedArgs,
    blocks::plant::{PlantBlockBase, seagrass::supports_seagrass},
};
#[pumpkin_block("minecraft:tall_seagrass")]
pub struct TallSeaGrassBlock;

impl TallSeaGrassBlock {
    #[must_use]
    pub fn is_full_water(block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let (block, state) = block_accessor.get_block_and_state(pos);
        if block == &Block::WATER {
            let level = WaterLikeProperties::from_state_id(state.id).level;
            return level == 0 || level == 8;
        }
        state.id.is_waterlogged()
            || matches!(
                block.id,
                BlockId::KELP
                    | BlockId::KELP_PLANT
                    | BlockId::SEAGRASS
                    | BlockId::TALL_SEAGRASS
                    | BlockId::BUBBLE_COLUMN
            )
    }

    fn can_survive(
        block_accessor: &dyn BlockAccessor,
        pos: &BlockPos,
        state_id: BlockStateId,
    ) -> bool {
        let props = TallSeagrassLikeProperties::from_state_id(state_id);
        if props.half == DoubleBlockHalf::Upper {
            let (below_block, below_state) = block_accessor.get_block_and_state(&pos.down());
            return below_block == &Block::TALL_SEAGRASS
                && TallSeagrassLikeProperties::from_state_id(below_state.id).half
                    == DoubleBlockHalf::Lower;
        }
        if !Self::is_full_water(block_accessor, pos) {
            return false;
        }
        let (support_block, support_block_state) = block_accessor.get_block_and_state(&pos.down());
        supports_seagrass(support_block, support_block_state)
    }
}

impl BlockBehaviour for TallSeaGrassBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        let props = TallSeagrassLikeProperties::from_state_id(args.state.id);
        if props.half == DoubleBlockHalf::Upper {
            return Self::can_survive(args.block_accessor, args.position, args.state.id);
        }
        if args.player.is_some() {
            let up_pos = args.position.up();
            if let Some(world) = args.world
                && up_pos.0.y > world.get_top_y()
            {
                return false;
            }
            if !args.block_accessor.get_block_state(&up_pos).replaceable()
                || !Self::is_full_water(args.block_accessor, &up_pos)
            {
                return false;
            }
        }
        Self::can_survive(args.block_accessor, args.position, args.state.id)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let props = TallSeagrassLikeProperties::from_state_id(args.state_id);
        if args.direction.to_axis() == Axis::Y
            && (props.half == DoubleBlockHalf::Lower) == (args.direction == BlockDirection::Up)
        {
            let neighbor_block = args.world.get_block(args.neighbor_position);
            if neighbor_block != &Block::TALL_SEAGRASS
                || TallSeagrassLikeProperties::from_state_id(args.neighbor_state_id).half
                    == props.half
            {
                return Block::AIR.default_state.id;
            }
        }

        if props.half == DoubleBlockHalf::Lower
            && args.direction == BlockDirection::Down
            && !Self::can_survive(args.world, args.position, args.state_id)
        {
            return Block::AIR.default_state.id;
        }

        if !Self::can_survive(args.world, args.position, args.state_id) {
            return Block::AIR.default_state.id;
        }

        args.world.schedule_fluid_tick(
            &Fluid::WATER,
            *args.position,
            Fluid::WATER.flow_speed as u8,
            TickPriority::Normal,
        );
        args.state_id
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        let mut props = TallSeagrassLikeProperties::from_state_id(args.state_id);
        if props.half != DoubleBlockHalf::Lower {
            return;
        }
        props.half = DoubleBlockHalf::Upper;
        args.world.set_block_state(
            &args.position.up(),
            props.to_state_id(args.block),
            BlockFlags::NOTIFY_ALL | BlockFlags::SKIP_BLOCK_ADDED_CALLBACK,
        );
    }

    fn broken(&self, args: BrokenArgs<'_>) {
        let props = TallSeagrassLikeProperties::from_state_id(args.state.id);
        let other_block_pos = match props.half {
            DoubleBlockHalf::Upper => args.position.down(),
            DoubleBlockHalf::Lower => args.position.up(),
        };
        let (other_block, other_state_id) = args.world.get_block_and_state_id(&other_block_pos);
        if other_block == &Block::TALL_SEAGRASS {
            let other_props = TallSeagrassLikeProperties::from_state_id(other_state_id);
            if other_props.half != props.half {
                args.world.break_block(
                    &other_block_pos,
                    None,
                    BlockFlags::SKIP_DROPS
                        | BlockFlags::SKIP_BLOCK_ADDED_CALLBACK
                        | BlockFlags::NOTIFY_ALL,
                );
            }
        }
    }
}

impl PlantBlockBase for TallSeaGrassBlock {
    fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let (support_block, support_block_state) = block_accessor.get_block_and_state(pos);
        supports_seagrass(support_block, support_block_state)
    }
}
