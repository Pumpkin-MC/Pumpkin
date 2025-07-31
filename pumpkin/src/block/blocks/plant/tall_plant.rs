use async_trait::async_trait;
use pumpkin_data::block_properties::{
    BlockProperties, DoubleBlockHalf, TallSeagrassLikeProperties,
};
use pumpkin_data::{Block, BlockDirection};
use pumpkin_world::BlockStateId;
use pumpkin_world::world::BlockFlags;

use crate::block::pumpkin_block::GetStateForNeighborUpdateArgs;
use crate::block::{
    blocks::plant::PlantBlockBase,
    pumpkin_block::{BlockMetadata, CanPlaceAtArgs, PlacedArgs, PumpkinBlock},
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

#[async_trait]
<<<<<<< HEAD
impl PumpkinBlock for TallPlantBlock {
    async fn placed(&self, args: PlacedArgs<'_>) {
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
    }

=======
impl BlockBehaviour for TallPlantBlock {
>>>>>>> master
    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        let upper_state = args
            .block_accessor
            .get_block_state(&args.position.up())
            .await;
        <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position).await
            && upper_state.is_air()
    }

    async fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let tall_plant_props = TallSeagrassLikeProperties::from_state_id(args.state_id, args.block);
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
        return Block::AIR.default_state.id;
    }
}

impl PlantBlockBase for TallPlantBlock {}
