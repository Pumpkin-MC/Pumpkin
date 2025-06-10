use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::{Block, BlockState};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;

use crate::{block::pumpkin_block::PumpkinBlock, entity::EntityBase, server::Server, world::World};

#[pumpkin_block("minecraft:lily_pad")]
pub struct LilyPadBlock;

#[async_trait]
impl PumpkinBlock for LilyPadBlock {
    async fn on_entity_collision(
        &self,
        world: &Arc<World>,
        entity: &dyn EntityBase,
        pos: BlockPos,
        _block: Block,
        _state: BlockState,
        _server: &Server,
    ) {
        // Proberbly not the best solution, but works
        if entity
            .get_entity()
            .entity_type
            .resource_name
            .ends_with("_boat")
        {
            world.break_block(&pos, None, BlockFlags::empty()).await;
        }
    }
}
