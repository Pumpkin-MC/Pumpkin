use std::sync::Arc;

use pumpkin_data::{
    Block,
    block_properties::{BlockProperties, EnumVariants, Integer0To7, WheatLikeProperties},
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{
    BlockStateId,
    world::{BlockAccessor, BlockFlags},
};
use rand::Rng;

use crate::{block::blocks::plant::PlantBlockBase, world::World};

type CropProperties = WheatLikeProperties;

pub mod beetroot;
pub mod carrot;
pub mod potatoes;
pub mod torch_flower;
pub mod wheat;

trait CropBlockBase: PlantBlockBase {
    async fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let block = block_accessor.get_block(pos).await;
        block == &Block::FARMLAND
    }

    fn max_age(&self) -> i32 {
        7
    }

    fn get_age(&self, state: u16, block: &Block) -> i32 {
        let props = CropProperties::from_state_id(state, block);
        i32::from(props.age.to_index())
    }

    fn state_with_age(&self, block: &Block, state: u16, age: i32) -> BlockStateId {
        let mut props = CropProperties::from_state_id(state, block);
        props.age = Integer0To7::from_index(age as u16);
        props.to_state_id(block)
    }

    async fn random_tick(&self, world: &Arc<World>, pos: &BlockPos) {
        let (block, state) = world.get_block_and_state_id(pos).await;
        let age = self.get_age(state, block);
        if age < self.max_age() {
            //TODO add moisture check
            let f = 5;
            if rand::rng().random_range(0..=(25 / f)) == 0 {
                world
                    .set_block_state(
                        pos,
                        self.state_with_age(block, state, age + 1),
                        BlockFlags::NOTIFY_NEIGHBORS,
                    )
                    .await;
            }
        }
    }

    //TODO add impl for light level
}
