use pumpkin_data::Block;
use pumpkin_data::block_properties::{
    BlockProperties, DoubleBlockHalf, TallSeagrassLikeProperties,
};
use pumpkin_world::BlockStateId;

use crate::block::BlockFuture;
use crate::block::{
    BlockBehaviour, BlockMetadata, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    blocks::plant::PlantBlockBase,
};

pub struct TallPlantBlock;

impl BlockMetadata for TallPlantBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        &[
            "tall_grass",
            "large_fern",
            "pitcher_plant",
            // TallFlowerBlocks
            "sunflower",
            "lilac",
            "peony",
            "rose_bush",
        ]
    }
}

impl BlockBehaviour for TallPlantBlock {
    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            let upper_state = args
                .block_accessor
                .get_block_state(&args.position.up())
                .await;
            <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position).await
                && upper_state.is_air()
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let tall_plant_props =
                TallSeagrassLikeProperties::from_state_id(args.state_id, args.block);
            let other_block_pos = match tall_plant_props.half {
                DoubleBlockHalf::Upper => args.position.down(),
                DoubleBlockHalf::Lower => args.position.up(),
            };
            let (other_block, other_state_id) =
                args.world.get_block_and_state_id(&other_block_pos).await;
            if self.ids().contains(&other_block.name) {
                let other_props =
                    TallSeagrassLikeProperties::from_state_id(other_state_id, other_block);
                let opposite_half = match tall_plant_props.half {
                    DoubleBlockHalf::Upper => DoubleBlockHalf::Lower,
                    DoubleBlockHalf::Lower => DoubleBlockHalf::Upper,
                };
                if other_props.half == opposite_half {
                    return args.state_id;
                }
            }
            Block::AIR.default_state.id
        })
    }
}

impl PlantBlockBase for TallPlantBlock {}
