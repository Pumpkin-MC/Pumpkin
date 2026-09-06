use pumpkin_data::block_properties::{DoubleBlockHalf, TallSeagrassLikeProperties};
use pumpkin_data::{Block, BlockId, BlockStateId};
use pumpkin_world::world::BlockFlags;

use crate::block::{
    BlockBehaviour, BlockMetadata, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    blocks::plant::PlantBlockBase,
};

pub struct ShortPlantBlock;

impl BlockMetadata for ShortPlantBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::SHORT_GRASS, BlockId::FERN].into()
    }
}

impl BlockBehaviour for ShortPlantBlock {
    fn is_valid_bonemeal_target(&self, args: crate::block::BonemealArgs<'_>) -> bool {
        let above = args.position.up();
        args.world.is_in_height_limit(above.0.y)
            && args.world.is_loaded(&above)
            && args.world.get_block_state(&above).is_air()
    }

    fn perform_bonemeal(&self, args: crate::block::BonemealArgs<'_>) {
        {
            let grown = if args.block == &Block::FERN {
                &Block::LARGE_FERN
            } else {
                &Block::TALL_GRASS
            };
            let lower = grown.default_state.id;
            args.world
                .set_block_state(args.position, lower, BlockFlags::NOTIFY_LISTENERS);
            let mut props = TallSeagrassLikeProperties::from_state_id(lower);
            props.half = DoubleBlockHalf::Upper;
            args.world.set_block_state(
                &args.position.up(),
                props.to_state_id(grown),
                BlockFlags::NOTIFY_LISTENERS,
            );
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

impl PlantBlockBase for ShortPlantBlock {}
