use pumpkin_data::BlockStateId;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::{BlockAccessor, BlockFlags};

use crate::block::{GetStateForNeighborUpdateArgs, blocks::plant::PlantBlockBase};

use crate::block::{BlockBehaviour, CanPlaceAtArgs, OnEntityCollisionArgs};

#[pumpkin_block("minecraft:lily_pad")]
pub struct LilyPadBlock;

impl BlockBehaviour for LilyPadBlock {
    fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        {
            let name = args.entity.get_entity().entity_type.resource_name;
            if name.ends_with("_boat") || name.ends_with("_raft") {
                args.world
                    .break_block(args.position, None, BlockFlags::NOTIFY_ALL);
            }
        }
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        <Self as PlantBlockBase>::get_state_for_neighbor_update(
            self,
            args.world,
            args.position,
            args.state_id,
        )
    }
}

impl PlantBlockBase for LilyPadBlock {
    fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let (block, state) = block_accessor.get_block_and_state(pos);
        let has_fluid = pumpkin_data::fluid::Fluid::from_state_id(state.id)
            .is_some_and(|f| f.has_tag(&tag::Fluid::MINECRAFT_SUPPORTS_LILY_PAD))
            || state.is_waterlogged();
        let above_state = block_accessor.get_block_state(&pos.up());
        let above_fluid_empty = pumpkin_data::fluid::Fluid::from_state_id(above_state.id).is_none()
            && !above_state.is_waterlogged();
        (has_fluid || block.has_tag(&tag::Block::MINECRAFT_SUPPORTS_LILY_PAD))
            && above_fluid_empty
    }
}
