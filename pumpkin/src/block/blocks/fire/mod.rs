use std::sync::Arc;

use crate::block::pumpkin_block::PumpkinBlock;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;
use crate::world::portal::NetherPortal;
use async_trait::async_trait;
use pumpkin_data::block_properties::HorizontalAxis;
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{Block, BlockState};
use pumpkin_registry::DimensionType;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::BlockDirection;
use soul_fire::SoulFireBlock;

#[expect(clippy::module_inception)]
pub mod fire;
pub mod soul_fire;

pub struct FireBlockBase;

impl FireBlockBase {
    pub async fn get_state(world: &World, pos: &BlockPos) -> Block {
        let (block, _block_state) = world.get_block_and_block_state(&pos.down()).await.unwrap();
        if SoulFireBlock::is_soul_base(&block) {
            return Block::SOUL_FIRE;
        }
        // TODO
        Block::FIRE
    }
}

#[async_trait]
impl PumpkinBlock for FireBlockBase {
    async fn placed(
        &self,
        world: &Arc<World>,
        _block: &Block,
        state_id: BlockStateId,
        pos: &BlockPos,
        old_state_id: BlockStateId,
        _notify: bool,
    ) {
        if old_state_id == state_id {
            return;
        }
        let dimension = world.dimension_type;
        // First lets check if we are in OverWorld or Nether, its not possible to place an Nether portal in other dimensions in Vanilla
        if dimension == DimensionType::Overworld || dimension == DimensionType::TheNether {
            if let Some(portal) = NetherPortal::get_new_portal(world, pos, HorizontalAxis::X).await
            {
                portal.create(world).await;
            }
        }
    }

    async fn can_place_at(
        &self,
        world: &World,
        block_pos: &BlockPos,
        face: BlockDirection,
    ) -> bool {
        let block_state = world.get_block_state(block_pos).await.unwrap();

        if !block_state.is_air() {
            return false;
        }
        let block = Self::get_state(world, block_pos).await;

        if let Some(block) = world.block_registry.get_pumpkin_block(&block) {
            return block.can_place_at(world, block_pos, face).await;
        }
        return false;
    }

    async fn broken(
        &self,
        _block: &Block,
        _player: &Player,
        position: BlockPos,
        _server: &Server,
        world: Arc<World>,
        _state: BlockState,
    ) {
        world
            .sync_world_event(WorldEvent::FireExtinguished, position, 0)
            .await;
    }
}
