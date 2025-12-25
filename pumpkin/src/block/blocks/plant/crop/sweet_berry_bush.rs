use pumpkin_data::{
    Block,
    block_properties::{BlockProperties, EnumVariants, Integer0To3, NetherWartLikeProperties},
    item::Item,
    tag::Taggable,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{
    item::ItemStack,
    world::{BlockAccessor, BlockFlags},
};
use rand::Rng;

use crate::block::{
    BlockBehaviour, BlockFuture, CanPlaceAtArgs, RandomTickArgs,
    blocks::plant::{PlantBlockBase, crop::CropBlockBase},
    registry::BlockActionResult,
};

#[pumpkin_block("minecraft:sweet_berry_bush")]
pub struct SweetBerryBushBlock;

impl BlockBehaviour for SweetBerryBushBlock {
    fn normal_use<'a>(
        &'a self,
        args: crate::block::NormalUseArgs<'a>,
    ) -> BlockFuture<'a, crate::block::registry::BlockActionResult> {
        Box::pin(async move {
            let state_id = args.world.get_block_state_id(args.position).await;
            let mut props = NetherWartLikeProperties::from_state_id(state_id, args.block);
            match props.age {
                Integer0To3::L2 | Integer0To3::L3 => {
                    let index = props.age.to_index() as u8;
                    props.age = pumpkin_data::block_properties::Integer0To3::L1;
                    let count: u8 = rand::rng().random_range((index - 1)..=(index));
                    for _ in 0..count {
                        args.world
                            .drop_stack(
                                args.position,
                                ItemStack::new(1, &Item::SWEET_BERRIES), //
                            )
                            .await;
                    }
                    args.world
                        .set_block_state(
                            args.position,
                            props.to_state_id(&Block::SWEET_BERRY_BUSH),
                            BlockFlags::NOTIFY_ALL,
                        )
                        .await;
                    BlockActionResult::SuccessServer
                }
                _ => BlockActionResult::Pass,
            }
        })
    }

    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position).await
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: crate::block::GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, pumpkin_world::BlockStateId> {
        Box::pin(async move {
            <Self as PlantBlockBase>::get_state_for_neighbor_update(
                self,
                args.world,
                args.position,
                args.state_id,
            )
            .await
        })
    }

    fn random_tick<'a>(&'a self, args: RandomTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            <Self as CropBlockBase>::random_tick(self, args.world, args.position).await;
        })
    }
}

impl PlantBlockBase for SweetBerryBushBlock {
    async fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let block = block_accessor.get_block(pos).await;
        block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_DIRT)
    }

    async fn get_state_for_neighbor_update(
        &self,
        block_accessor: &dyn pumpkin_world::world::BlockAccessor,
        block_pos: &pumpkin_util::math::position::BlockPos,
        block_state: pumpkin_world::BlockStateId,
    ) -> pumpkin_world::BlockStateId {
        if !<Self as PlantBlockBase>::can_place_at(self, block_accessor, block_pos).await {
            return pumpkin_data::Block::AIR.default_state.id;
        }
        block_state
    }

    async fn can_place_at(
        &self,
        block_accessor: &dyn pumpkin_world::world::BlockAccessor,
        block_pos: &pumpkin_util::math::position::BlockPos,
    ) -> bool {
        <Self as PlantBlockBase>::can_plant_on_top(self, block_accessor, &block_pos.down()).await
    }
}

impl CropBlockBase for SweetBerryBushBlock {
    async fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        <Self as PlantBlockBase>::can_plant_on_top(self, block_accessor, pos).await
    }

    fn max_age(&self) -> i32 {
        3
    }

    fn get_age(&self, state: u16, block: &pumpkin_data::Block) -> i32 {
        let props = NetherWartLikeProperties::from_state_id(state, block);
        i32::from(props.age.to_index())
    }

    fn state_with_age(
        &self,
        block: &pumpkin_data::Block,
        state: u16,
        age: i32,
    ) -> pumpkin_world::BlockStateId {
        let mut props = NetherWartLikeProperties::from_state_id(state, block);
        props.age = pumpkin_data::block_properties::Integer0To3::from_index(age as u16);
        props.to_state_id(block)
    }

    async fn random_tick(&self, world: &std::sync::Arc<crate::world::World>, pos: &BlockPos) {
        let (block, state) = world.get_block_and_state_id(pos).await;
        let age = self.get_age(state, block);
        if age < self.max_age() {
            let state_above = world.get_block_state(&pos.up()).await;

            if state_above.is_full_cube() || state_above.is_solid() {
                return;
            }
            if rand::rng().random_range(0..=25) == 0 {
                world
                    .set_block_state(
                        pos,
                        self.state_with_age(block, state, age + 1),
                        pumpkin_world::world::BlockFlags::NOTIFY_NEIGHBORS,
                    )
                    .await;
            }
        }
    }
}
