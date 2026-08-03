use std::sync::Arc;

use pumpkin_data::{
    Block,
    BlockDirection::{East, North, South, West},
    BlockStateId,
    block_properties::{BlockProperties, FarmlandLikeProperties, WheatLikeProperties},
};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use pumpkin_world::world::{BlockAccessor, BlockFlags};
use rand::RngExt;

use crate::{
    block::blocks::plant::PlantBlockBase, plugin::api::events::block::block_grow::BlockGrowEvent,
    world::World,
};

type CropProperties = WheatLikeProperties;
type FarmlandProperties = FarmlandLikeProperties;

pub mod beetroot;
pub mod carrot;
pub mod gourds;
pub mod nether_wart;
pub mod potatoes;
pub mod sweet_berry_bush;
pub mod torch_flower;
pub mod wheat;

trait CropBlockBase: PlantBlockBase {
    fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let block = block_accessor.get_block(pos);
        block == &Block::FARMLAND
    }

    fn max_age(&self) -> i32 {
        7
    }

    fn get_age(&self, state: BlockStateId, block: &Block) -> i32 {
        let props = CropProperties::from_state_id(state, block);
        i32::from(props.age)
    }

    fn state_with_age(&self, block: &Block, state: BlockStateId, age: i32) -> BlockStateId {
        let mut props = CropProperties::from_state_id(state, block);
        props.age = age as u8;
        props.to_state_id(block)
    }

    async fn random_tick(&self, world: &Arc<World>, pos: &BlockPos) {
        let (block, state) = world.get_block_and_state_id(pos);
        let age = self.get_age(state, block);
        if age < self.max_age() {
            let f = get_available_moisture(world, pos, block).await;
            if rand::rng().random_range(0..=(25.0 / f).floor() as i64) == 0 {
                let mut new_state_id = self.state_with_age(block, state, age + 1);
                if let Some(server) = world.server.upgrade() {
                    let event = BlockGrowEvent::new(
                        world.clone(),
                        block,
                        state,
                        Block::from_state_id(new_state_id),
                        new_state_id,
                        *pos,
                    );
                    let event = server.plugin_manager.fire(event).await;
                    if event.cancelled {
                        return;
                    }
                    new_state_id = event.new_state_id;
                }
                world
                    .set_block_state(pos, new_state_id, BlockFlags::NOTIFY_NEIGHBORS)
                    .await;
            }
        }
    }

    //TODO add impl for light level

    /// Age gained per bone meal application. Vanilla `CropBlock.getBonemealAgeIncrease`
    /// returns a uniform `randInt(2..=5)`; beetroot overrides this to `1`.
    fn bonemeal_age_increase(&self) -> i32 {
        rand::rng().random_range(2..=5)
    }

    /// Whether bone meal may be applied. Mirrors vanilla `CropBlock::isValidBonemealTarget`:
    /// valid until the crop is fully grown.
    fn can_bonemeal(&self, state: BlockStateId, block: &Block) -> bool {
        self.get_age(state, block) < self.max_age()
    }

    /// Advance the crop by `bonemeal_age_increase`, clamped to `max_age`. Fires `BlockGrowEvent`
    /// for parity with `random_tick`, honouring cancellation and `new_state_id` rewrites.
    async fn grow_from_bonemeal(&self, world: &Arc<World>, pos: &BlockPos) {
        let (block, state) = world.get_block_and_state_id(pos);
        let age = self.get_age(state, block);
        let new_age = (age + self.bonemeal_age_increase()).min(self.max_age());
        let mut new_state_id = self.state_with_age(block, state, new_age);
        if let Some(server) = world.server.upgrade() {
            let event = BlockGrowEvent::new(
                world.clone(),
                block,
                state,
                Block::from_state_id(new_state_id),
                new_state_id,
                *pos,
            );
            let event = server.plugin_manager.fire(event).await;
            if event.cancelled {
                return;
            }
            new_state_id = event.new_state_id;
        }
        world
            .set_block_state(pos, new_state_id, BlockFlags::NOTIFY_ALL)
            .await;
    }
}

pub async fn get_available_moisture(world: &Arc<World>, pos: &BlockPos, block: &Block) -> f32 {
    let mut moisture = 1.0;
    let down_pos = pos.down();

    for dx in -1..=1 {
        for dz in -1..=1 {
            let mut local_moisture = 0.0;

            let (block, block_state) =
                world.get_block_and_state_id(&down_pos.offset(Vector3 { x: dx, y: 0, z: dz }));
            if block == &Block::FARMLAND {
                local_moisture = 1.0;
                let props = FarmlandProperties::from_state_id(block_state, block);
                if props.moisture != 0 {
                    local_moisture = 3.0;
                }
            }

            if dx != 0 || dz != 0 {
                local_moisture /= 4.0;
            }

            moisture += local_moisture;
        }
    }

    let north = pos.offset(North.to_offset());
    let south = pos.offset(South.to_offset());
    let west = pos.offset(West.to_offset());
    let east = pos.offset(East.to_offset());
    let horizontal = world.get_block(&west) == block || world.get_block(&east) == block;
    let vertical = world.get_block(&north) == block || world.get_block(&south) == block;
    if (horizontal && vertical)
        || world.get_block(&west.offset(North.to_offset())) == block
        || world.get_block(&east.offset(North.to_offset())) == block
        || world.get_block(&east.offset(South.to_offset())) == block
        || world.get_block(&west.offset(South.to_offset())) == block
    {
        moisture /= 2.0;
    }

    moisture
}

#[cfg(test)]
mod tests {
    use super::CropBlockBase;
    use super::beetroot::BeetrootBlock;
    use crate::block::blocks::plant::PlantBlockBase;

    struct DefaultCrop;
    impl PlantBlockBase for DefaultCrop {}
    impl CropBlockBase for DefaultCrop {}

    #[test]
    fn default_crop_bonemeal_increase_is_2_to_5() {
        let crop = DefaultCrop;
        for _ in 0..1000 {
            let inc = crop.bonemeal_age_increase();
            assert!(
                (2..=5).contains(&inc),
                "increase out of vanilla range: {inc}"
            );
        }
    }

    #[test]
    fn beetroot_bonemeal_increase_is_1() {
        assert_eq!(BeetrootBlock.bonemeal_age_increase(), 1);
    }
}
