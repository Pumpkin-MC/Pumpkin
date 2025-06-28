use std::sync::Arc;

use pumpkin_data::{
    Block,
    block_properties::{BlockProperties, EnumVariants, Integer0To7, WheatLikeProperties},
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::{BlockAccessor, BlockFlags};
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
        block == Block::FARMLAND
    }

    fn max_age(&self) -> i32 {
        7
    }

    async fn random_tick(&self, world: &Arc<World>, pos: &BlockPos) {
        let (block, state) = world.get_block_and_block_state(pos).await;
        let mut props = CropProperties::from_state_id(state.id, &block);
        if i32::from(props.age.to_index()) < self.max_age() {
            //TODO add moisture check
            let f = 5;
            if rand::rng().random_range(0..=(25 / f)) == 0 {
                props.age = Integer0To7::from_index(i32::from(props.age.to_index()) as u16 + 1);
                world
                    .set_block_state(pos, props.to_state_id(&block), BlockFlags::NOTIFY_NEIGHBORS)
                    .await;
            }
        }
    }

    //TODO add impl for light level
}
