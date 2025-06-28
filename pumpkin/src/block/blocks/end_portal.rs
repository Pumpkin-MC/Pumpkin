use std::sync::Arc;

use crate::block::pumpkin_block::PumpkinBlock;
use crate::entity::EntityBase;
use crate::server::Server;
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::{Block, BlockState};
use pumpkin_macros::pumpkin_block;
use pumpkin_registry::VanillaDimensionType;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::entities::end_portal::EndPortalBlockEntity;

#[pumpkin_block("minecraft:end_portal")]
pub struct EndPortalBlock;

#[async_trait]
impl PumpkinBlock for EndPortalBlock {
    async fn on_entity_collision(
        &self,
        world: &Arc<World>,
        entity: &dyn EntityBase,
        pos: BlockPos,
        _block: Block,
        _state: BlockState,
        server: &Server,
    ) {
        let world = if world.dimension_type == VanillaDimensionType::TheEnd {
            server
                .get_world_from_dimension(VanillaDimensionType::Overworld)
                .await
        } else {
            server
                .get_world_from_dimension(VanillaDimensionType::TheEnd)
                .await
        };
        entity.get_entity().try_use_portal(0, world, pos).await;
    }

    async fn placed(
        &self,
        world: &Arc<World>,
        _block: &Block,
        _state_id: u16,
        _pos: &BlockPos,
        _old_state_id: u16,
        _notify: bool,
    ) {
        world
            .add_block_entity(Arc::new(EndPortalBlockEntity::new(*_pos)))
            .await;
    }

    async fn on_state_replaced(
        &self,
        world: &Arc<World>,
        _block: &Block,
        location: BlockPos,
        _old_state_id: u16,
        _moved: bool,
    ) {
        world.remove_block_entity(&location).await;
    }
}
