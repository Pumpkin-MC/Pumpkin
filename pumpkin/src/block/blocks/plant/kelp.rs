use crate::block::blocks::plant::PlantBlockBase;
use crate::block::{BlockBehaviour, BlockFuture, BlockMetadata, CanPlaceAtArgs};
use pumpkin_data::Block;

pub struct KelpBlock;

impl BlockMetadata for KelpBlock {
    fn ids() -> Box<[u16]> {
        [Block::KELP.id, Block::KELP_PLANT.id].into()
    }
}

impl BlockBehaviour for KelpBlock {
    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            // Determine support block
            let support_pos = args.position.down();
            let support_block = args.block_accessor.get_block(&support_pos).await;

            // If placing the base kelp block, allow placement on water or on other kelp segments.
            if args.block.id == Block::KELP.id {
                support_block == &Block::WATER
                    || support_block == &Block::KELP
                    || support_block == &Block::KELP_PLANT
            } else {
                support_block == &Block::KELP || support_block == &Block::KELP_PLANT
            }
        })
    }

    // TODO: proper kelp placement (fix `can_place_at`) and break behavior (including supporting blocks)
}

impl PlantBlockBase for KelpBlock {}
