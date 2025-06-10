use std::sync::Arc;

use pumpkin_data::Block;
use pumpkin_data::world::WorldEvent;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockAccessor;
use soul_fire::SoulFireBlock;

use crate::world::World;

#[expect(clippy::module_inception)]
pub mod fire;
pub mod soul_fire;

pub struct FireBlockBase;

impl FireBlockBase {
    pub async fn get_fire_type(world: &World, pos: &BlockPos) -> Block {
        let (block, _block_state) = world.get_block_and_block_state(&pos.down()).await;
        if SoulFireBlock::is_soul_base(&block) {
            return Block::SOUL_FIRE;
        }
        // TODO
        Block::FIRE
    }

    #[must_use]
    pub fn can_place_on(block: &Block) -> bool {
        let block = block.clone();

        // Make sure the block below is not a fire block or fluid block
        block != Block::SOUL_FIRE
            && block != Block::FIRE
            && block != Block::WATER
            && block != Block::LAVA
    }

    pub async fn can_place_at(block_accessor: &dyn BlockAccessor, block_pos: &BlockPos) -> bool {
        let block_state = block_accessor.get_block_state(block_pos).await;
        block_state.is_air()
            && Self::can_place_on(&block_accessor.get_block(&block_pos.down()).await)
    }

    async fn broken(world: Arc<World>, block_pos: BlockPos) {
        world
            .sync_world_event(WorldEvent::FireExtinguished, block_pos, 0)
            .await;
    }
}
