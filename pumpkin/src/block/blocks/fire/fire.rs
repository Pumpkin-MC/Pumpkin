use pumpkin_data::block_properties::HorizontalAxis;
use pumpkin_data::entity::EntityType;
use pumpkin_registry::DimensionType;
use pumpkin_world::world::BlockAccessor;
use rand::Rng;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use async_trait::async_trait;
use pumpkin_data::{Block, BlockDirection, BlockState};
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::chunk::TickPriority;

use crate::block::pumpkin_block::PumpkinBlock;
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;
use crate::world::portal::nether::NetherPortal;

use super::FireBlockBase;

#[pumpkin_block("minecraft:fire")]
pub struct FireBlock;

impl FireBlock {
    #[must_use]
    pub fn get_fire_tick_delay() -> i32 {
        30 + rand::thread_rng().gen_range(0..10)
    }
}

#[async_trait]
impl PumpkinBlock for FireBlock {
    async fn placed(
        &self,
        world: &Arc<World>,
        block: &Block,
        state_id: BlockStateId,
        pos: &BlockPos,
        old_state_id: BlockStateId,
        _notify: bool,
    ) {
        if old_state_id == state_id {
            // Already a fire
            return;
        }

        let dimension = world.dimension_type;
        // First lets check if we are in OverWorld or Nether, its not possible to place an Nether portal in other dimensions in Vanilla
        if dimension == DimensionType::Overworld || dimension == DimensionType::TheNether {
            if let Some(portal) = NetherPortal::get_new_portal(world, pos, HorizontalAxis::X).await
            {
                portal.create(world).await;
                return;
            }
        }

        world
            .schedule_block_tick(
                block,
                *pos,
                Self::get_fire_tick_delay() as u16,
                TickPriority::Normal,
            )
            .await;
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
        let base_entity = entity.get_entity();
        if !base_entity.entity_type.fire_immune {
            let ticks = base_entity.fire_ticks.load(Ordering::Relaxed);
            if ticks < 0 {
                base_entity.fire_ticks.store(ticks + 1, Ordering::Relaxed);
            } else if base_entity.entity_type == EntityType::PLAYER {
                let rnd_ticks = rand::thread_rng().gen_range(1..3);
                base_entity
                    .fire_ticks
                    .store(ticks + rnd_ticks, Ordering::Relaxed);
            }
            if base_entity.fire_ticks.load(Ordering::Relaxed) >= 0 {
                base_entity.set_on_fire_for(8.0);
            }
        }
    }

    async fn get_state_for_neighbor_update(
        &self,
        world: &World,
        _block: &Block,
        state_id: BlockStateId,
        block_pos: &BlockPos,
        _direction: BlockDirection,
        _neighbor_pos: &BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        if !FireBlockBase::can_place_on(&world.get_block(&block_pos.down()).await) {
            return Block::AIR.default_state_id;
        }

        state_id
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
        let state = block_accessor.get_block_state(block_pos).await;
        // TODO: add more
        state.is_side_solid(BlockDirection::Up)
    }

    async fn broken(
        &self,
        _block: &Block,
        _player: &Arc<Player>,
        block_pos: BlockPos,
        _server: &Server,
        world: Arc<World>,
        _state: BlockState,
    ) {
        FireBlockBase::broken(world, block_pos).await;
    }
}
