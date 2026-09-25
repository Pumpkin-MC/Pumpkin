use pumpkin_data::BlockStateId;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{Block, BlockState};
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
            // Proberbly not the best solution, but works
            if args
                .entity
                .get_entity()
                .entity_type
                .resource_name
                .ends_with("_boat")
            {
                args.world
                    .break_block(args.position, None, BlockFlags::empty());
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
        let block = block_accessor.get_block(pos);
        let state = block_accessor.get_block_state(pos);
        let above_state = block_accessor.get_block_state(&pos.up());
        can_support_lily_pad(block, state, above_state)
    }
}

fn can_support_lily_pad(block: &Block, state: &BlockState, above_state: &BlockState) -> bool {
    let is_water = |state: &BlockState| {
        state.is_waterlogged()
            || Fluid::from_state_id(state.id)
                .is_some_and(|fluid| fluid.has_tag(&tag::Fluid::MINECRAFT_WATER))
    };

    (block.has_tag(&tag::Block::MINECRAFT_SUPPORTS_LILY_PAD) || is_water(state))
        && !is_water(above_state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::block_properties::WhiteWoolSlabLikeProperties;

    fn waterlogged_slab() -> &'static BlockState {
        let mut props = WhiteWoolSlabLikeProperties::default(&Block::OAK_SLAB);
        props.waterlogged = true;
        BlockState::from_id(props.to_state_id(&Block::OAK_SLAB))
    }

    #[test]
    fn water_and_waterlogged_blocks_support_lily_pads() {
        assert!(can_support_lily_pad(
            &Block::WATER,
            Block::WATER.default_state,
            Block::AIR.default_state
        ));
        assert!(can_support_lily_pad(
            &Block::OAK_SLAB,
            waterlogged_slab(),
            Block::AIR.default_state
        ));
        assert!(can_support_lily_pad(
            &Block::ICE,
            Block::ICE.default_state,
            Block::AIR.default_state
        ));
    }

    #[test]
    fn dry_blocks_do_not_support_lily_pads() {
        assert!(!can_support_lily_pad(
            &Block::STONE,
            Block::STONE.default_state,
            Block::AIR.default_state
        ));
        assert!(!can_support_lily_pad(
            &Block::OAK_SLAB,
            Block::OAK_SLAB.default_state,
            Block::AIR.default_state
        ));
    }

    #[test]
    fn water_at_the_pad_position_blocks_placement() {
        assert!(!can_support_lily_pad(
            &Block::WATER,
            Block::WATER.default_state,
            waterlogged_slab()
        ));
    }
}
