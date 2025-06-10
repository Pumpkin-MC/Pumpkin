use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::block_properties::{
    BlockProperties, CactusLikeProperties, EnumVariants, Integer0To15,
};
use pumpkin_data::damage::DamageType;
use pumpkin_data::tag::Tagable;
use pumpkin_data::{Block, BlockDirection, BlockState};
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::chunk::TickPriority;
use pumpkin_world::world::{BlockAccessor, BlockFlags};

use crate::block::pumpkin_block::PumpkinBlock;
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;

#[pumpkin_block("minecraft:cactus")]
pub struct CactusBlock;

#[async_trait]
impl PumpkinBlock for CactusBlock {
    async fn on_scheduled_tick(&self, world: &Arc<World>, _block: &Block, pos: &BlockPos) {
        if !can_place_at(world.as_ref(), pos).await {
            world.break_block(pos, None, BlockFlags::empty()).await;
        }
    }

    async fn random_tick(&self, block: &Block, world: &Arc<World>, pos: &BlockPos) {
        if world.get_block_state(&pos.up()).await.is_air() {
            let state_id = world.get_block_state(pos).await.id;
            let age = CactusLikeProperties::from_state_id(state_id, block).age;
            if age == Integer0To15::L15 {
                world
                    .set_block_state(&pos.up(), state_id, BlockFlags::empty())
                    .await;
                let props = CactusLikeProperties {
                    age: Integer0To15::L0,
                };
                world
                    .set_block_state(pos, props.to_state_id(block), BlockFlags::empty())
                    .await;
            } else {
                let props = CactusLikeProperties {
                    age: Integer0To15::from_index(age.to_index() + 1),
                };
                world
                    .set_block_state(pos, props.to_state_id(block), BlockFlags::empty())
                    .await;
            }
        }
    }

    async fn on_entity_collision(
        &self,
        _world: &Arc<World>,
        entity: &dyn EntityBase,
        _pos: BlockPos,
        _block: Block,
        _state: BlockState,
        _server: &Server,
    ) {
        entity.damage(1.0, DamageType::CACTUS).await;
    }

    async fn get_state_for_neighbor_update(
        &self,
        world: &World,
        block: &Block,
        state: BlockStateId,
        pos: &BlockPos,
        _direction: BlockDirection,
        _neighbor_pos: &BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        if !can_place_at(world, pos).await {
            world
                .schedule_block_tick(block, *pos, 1, TickPriority::Normal)
                .await;
        }

        state
    }

    async fn can_place_at(
        &self,
        _server: Option<&Server>,
        _world: Option<&World>,
        block_accessor: &dyn BlockAccessor,
        _player: Option<&Player>,
        _block: &Block,
        block_pos: &BlockPos,
        _face: BlockDirection,
        _use_item_on: Option<&SUseItemOn>,
    ) -> bool {
        can_place_at(block_accessor, block_pos).await
    }
}

async fn can_place_at(world: &dyn BlockAccessor, block_pos: &BlockPos) -> bool {
    // TODO: use tags
    // Disallow to place any blocks nearby a cactus
    for direction in BlockDirection::horizontal() {
        let (block, state) = world
            .get_block_and_block_state(&block_pos.offset(direction.to_offset()))
            .await;
        if state.is_solid() || block == Block::LAVA {
            return false;
        }
    }
    let block = world.get_block(&block_pos.down()).await;
    // TODO: use tags
    (block == Block::CACTUS || block.is_tagged_with("minecraft:sand").unwrap())
        && !world.get_block_state(&block_pos.up()).await.is_liquid()
}
