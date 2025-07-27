use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::block_properties::{
    BlockProperties, EnumVariants, Integer0To3, NetherWartLikeProperties,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_world::BlockStateId;
use rand::Rng;

use crate::block::blocks::plant::PlantBlockBase;
use crate::block::blocks::plant::crop::CropBlockBase;
use crate::block::pumpkin_block::{
    CanPlaceAtArgs, GetStateForNeighborUpdateArgs, PumpkinBlock, RandomTickArgs,
};

type BeetrootProperties = NetherWartLikeProperties;

#[pumpkin_block("minecraft:beetroots")]
pub struct BeetrootBlock;

#[async_trait]
impl PumpkinBlock for BeetrootBlock {
    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        <Self as CropBlockBase>::can_plant_on_top(self, args.block_accessor, &args.position.down())
            .await
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

    async fn random_tick(&self, args: RandomTickArgs<'_>) {
        if rand::rng().random_range(0..2) != 0 {
            <Self as CropBlockBase>::random_tick(self, args.world, args.position).await;
        }
    }
}

impl PlantBlockBase for BeetrootBlock {}

impl CropBlockBase for BeetrootBlock {
    fn max_age(&self) -> i32 {
        3
    }

    fn get_age(&self, state: u16, block: &Block) -> i32 {
        let props = BeetrootProperties::from_state_id(state, block);
        i32::from(props.age.to_index())
    }

    fn state_with_age(&self, block: &Block, state: u16, age: i32) -> BlockStateId {
        let mut props = BeetrootProperties::from_state_id(state, block);
        props.age = Integer0To3::from_index(age as u16);
        props.to_state_id(block)
    }
}
