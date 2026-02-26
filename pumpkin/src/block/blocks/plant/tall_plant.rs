use crate::block::PlacedArgs;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::block_properties::{
    BlockProperties, DoubleBlockHalf, TallSeagrassLikeProperties,
};
use pumpkin_world::BlockStateId;
use pumpkin_world::world::BlockFlags;

use crate::block::BlockFuture;
use crate::block::{
    BlockBehaviour, BlockMetadata, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    blocks::plant::PlantBlockBase,
};

pub struct TallPlantBlock;

impl BlockMetadata for TallPlantBlock {
    fn ids() -> Box<[u16]> {
        [
            Block::TALL_GRASS.id,
            Block::LARGE_FERN.id,
            Block::PITCHER_PLANT.id,
            // TallFlowerBlocks
            Block::SUNFLOWER.id,
            Block::LILAC.id,
            Block::PEONY.id,
            Block::ROSE_BUSH.id,
        ]
        .into()
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
            if Self::ids().contains(&other_block.id) {
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
    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let mut tall_plant_props =
                TallSeagrassLikeProperties::from_state_id(args.state_id, args.block);
            tall_plant_props.half = DoubleBlockHalf::Upper;
            args.world
                .set_block_state(
                    &args.position.offset(BlockDirection::Up.to_offset()),
                    tall_plant_props.to_state_id(args.block),
                    BlockFlags::NOTIFY_ALL | BlockFlags::SKIP_BLOCK_ADDED_CALLBACK,
                )
                .await;
        })
    }
}

impl PlantBlockBase for TallPlantBlock {}
