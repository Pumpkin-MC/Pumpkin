use async_trait::async_trait;
use pumpkin_data::tag;
use pumpkin_data::tag::Taggable;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{BlockStateId, world::BlockAccessor};

use crate::block::{
    blocks::plant::PlantBlockBase,
    pumpkin_block::{BlockMetadata, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, PumpkinBlock},
};

pub struct MushroomPlantBlock;

impl BlockMetadata for MushroomPlantBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        &["brown_mushroom", "red_mushroom"]
    }
}

#[async_trait]
impl PumpkinBlock for MushroomPlantBlock {
    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position).await
    }

    async fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        <Self as PlantBlockBase>::get_state_for_neighbor_update(
            self,
            args.world,
            args.position,
            args.state_id,
        )
        .await
    }
}

impl PlantBlockBase for MushroomPlantBlock {
    async fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let block = block_accessor.get_block(pos).await;
        block.is_tagged_with_by_tag(&tag::Block::MINECRAFT_MUSHROOM_GROW_BLOCK)
        // TODO: Check light level and isOpaqueFullCube
    }
}
