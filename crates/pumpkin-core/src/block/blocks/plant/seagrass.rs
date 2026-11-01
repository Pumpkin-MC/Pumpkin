use pumpkin_data::{
    Block, BlockDirection, BlockState, BlockStateId,
    block_properties::{DoubleBlockHalf, TallSeagrassLikeProperties, WaterLikeProperties},
    fluid::Fluid,
    tag::{self, Taggable},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{
    tick::TickPriority,
    world::{BlockAccessor, BlockFlags},
};

use crate::block::{
    BlockBehaviour, BonemealArgs, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    blocks::plant::PlantBlockBase,
};
#[pumpkin_block("minecraft:seagrass")]
pub struct SeaGrassBlock;
impl BlockBehaviour for SeaGrassBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if !<Self as PlantBlockBase>::can_place_at(self, args.world, args.position) {
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

    fn is_valid_bonemeal_target(&self, args: BonemealArgs<'_>) -> bool {
        args.world.get_block(&args.position.up()) == &Block::WATER
    }

    fn is_bonemeal_success(&self, _args: BonemealArgs<'_>) -> bool {
        true
    }

    fn perform_bonemeal(&self, args: BonemealArgs<'_>) {
        let lower_state = Block::TALL_SEAGRASS.default_state.id;
        let mut upper_props = TallSeagrassLikeProperties::from_state_id(lower_state);
        upper_props.half = DoubleBlockHalf::Upper;
        let above = args.position.up();
        args.world
            .set_block_state(args.position, lower_state, BlockFlags::NOTIFY_LISTENERS);
        args.world.set_block_state(
            &above,
            upper_props.to_state_id(&Block::TALL_SEAGRASS),
            BlockFlags::NOTIFY_LISTENERS,
        );
    }
}

impl PlantBlockBase for SeaGrassBlock {
    fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let (support_block, support_block_state) = block_accessor.get_block_and_state(pos);
        let (replacing_block, replacing_state) = block_accessor.get_block_and_state(&pos.up());
        if replacing_block == &Block::WATER {
            let water_props = WaterLikeProperties::from_state_id(replacing_state.id);
            if water_props.level != 0 && water_props.level != 8 {
                return false;
            }
        } else if replacing_block != &Block::SEAGRASS {
            return false;
        }
        supports_seagrass(support_block, support_block_state)
    }
}
#[must_use]
pub fn supports_seagrass(support_block: &Block, support_block_state: &BlockState) -> bool {
    support_block_state.is_side_solid(BlockDirection::Up)
        && !support_block.has_tag(&tag::Block::MINECRAFT_CANNOT_SUPPORT_SEAGRASS)
}
