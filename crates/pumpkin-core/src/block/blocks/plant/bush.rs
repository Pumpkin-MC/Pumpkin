use pumpkin_data::{BlockDirection, BlockId, BlockStateId};
use pumpkin_world::world::BlockFlags;
use rand::seq::SliceRandom;

use crate::block::{
    BlockBehaviour, BlockMetadata, BonemealArgs, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    blocks::plant::PlantBlockBase,
};

pub struct BushBlock;

impl BlockMetadata for BushBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::BUSH, BlockId::FIREFLY_BUSH, BlockId::RED_SHRUB].into()
    }
}

impl BlockBehaviour for BushBlock {
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

    fn is_valid_bonemeal_target(&self, args: BonemealArgs<'_>) -> bool {
        BlockDirection::horizontal().into_iter().any(|dir| {
            let pos = args.position.offset(dir.to_offset());
            args.world.get_block_state(&pos).is_air()
                && <Self as PlantBlockBase>::can_place_at(self, args.world.as_ref(), &pos)
        })
    }

    fn is_bonemeal_success(&self, _args: BonemealArgs<'_>) -> bool {
        true
    }

    fn perform_bonemeal(&self, args: BonemealArgs<'_>) {
        let mut dirs = BlockDirection::horizontal();
        dirs.shuffle(&mut rand::rng());
        for dir in dirs {
            let pos = args.position.offset(dir.to_offset());
            if args.world.get_block_state(&pos).is_air()
                && <Self as PlantBlockBase>::can_place_at(self, args.world.as_ref(), &pos)
            {
                args.world.set_block_state(
                    &pos,
                    args.block.default_state.id,
                    BlockFlags::NOTIFY_ALL,
                );
                break;
            }
        }
    }
}

impl PlantBlockBase for BushBlock {}
