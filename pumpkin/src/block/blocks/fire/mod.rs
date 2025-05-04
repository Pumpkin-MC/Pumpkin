use std::sync::Arc;

use pumpkin_data::Block;
use pumpkin_data::world::WorldEvent;
use pumpkin_util::math::position::BlockPos;
use soul_fire::SoulFireBlock;

use crate::world::World;

#[expect(clippy::module_inception)]
pub mod fire;
pub mod soul_fire;

pub struct FireBlockBase;

impl FireBlockBase {
    pub async fn get_fire_type(world: &World, pos: &BlockPos) -> Block {
        let (block, _block_state) = world.get_block_and_block_state(&pos.down()).await.unwrap();
        if SoulFireBlock::is_soul_base(&block) {
            return Block::SOUL_FIRE;
        }
        // TODO
        Block::FIRE
    }

    pub fn can_place_on(_block: &Block) -> bool {
        // TODO: make sure the block can be lit
        // block
        //     .is_tagged_with("minecraft:soul_fire_base_blocks")
        //     .unwrap()
        true
    }

    pub async fn can_place_at(world: &World, block_pos: &BlockPos) -> bool {
        let block_state = world.get_block_state(block_pos).await.unwrap();
        block_state.is_air()
            && Self::can_place_on(&world.get_block(&block_pos.down()).await.unwrap())
    }

    async fn broken(world: Arc<World>, block_pos: BlockPos) {
        world
            .sync_world_event(WorldEvent::FireExtinguished, block_pos, 0)
            .await;
    }
}
