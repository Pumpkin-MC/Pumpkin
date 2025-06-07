use std::sync::Arc;

use crate::block::pumpkin_block::PumpkinBlock;
use crate::entity::EntityBase;
use crate::server::Server;
use crate::world::World;
use crate::world::portal::nether::NetherPortal;
use async_trait::async_trait;
use pumpkin_data::block_properties::{Axis, BlockProperties, NetherPortalLikeProperties};
use pumpkin_data::entity::EntityType;
use pumpkin_data::{Block, BlockDirection, BlockState};
use pumpkin_macros::pumpkin_block;
use pumpkin_registry::DimensionType;
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;

#[pumpkin_block("minecraft:nether_portal")]
pub struct NetherPortalBlock;

impl NetherPortalBlock {
    /// Gets the portal delay time based on entity type and gamemode
    async fn get_portal_time(world: &Arc<World>, entity: &dyn EntityBase) -> u32 {
        let entity_type = entity.get_entity().entity_type;

        match entity_type {
            EntityType::PLAYER => (world.get_player_by_id(entity.get_entity().entity_id).await)
                .map_or(80, |player| match player.gamemode.load() {
                    GameMode::Creative => 0,
                    _ => 80,
                }),
            _ => 0,
        }
    }
}

#[async_trait]
impl PumpkinBlock for NetherPortalBlock {
    async fn get_state_for_neighbor_update(
        &self,
        world: &World,
        _block: &Block,
        state: BlockStateId,
        pos: &BlockPos,
        direction: BlockDirection,
        _neighbor_pos: &BlockPos,
        neighbor_state: BlockStateId,
    ) -> BlockStateId {
        let axis = direction.to_axis();
        let is_horizontal = axis == Axis::X && axis == Axis::Z;
        let state_axis =
            NetherPortalLikeProperties::from_state_id(state, &Block::NETHER_PORTAL).axis;
        if is_horizontal
            || neighbor_state == state
            || NetherPortal::get_on_axis(world, pos, state_axis)
                .await
                .is_some_and(|e| e.was_already_valid())
        {
            return state;
        }
        Block::AIR.default_state_id
    }

    async fn on_entity_collision(
        &self,
        world: &Arc<World>,
        entity: &dyn EntityBase,
        pos: BlockPos,
        _block: Block,
        _state: BlockState,
        server: &Server,
    ) {
        let target_world = if world.dimension_type == DimensionType::TheNether {
            server
                .get_world_from_dimension(DimensionType::Overworld)
                .await
        } else {
            server
                .get_world_from_dimension(DimensionType::TheNether)
                .await
        };

        let portal_delay = Self::get_portal_time(world, entity).await;

        entity
            .get_entity()
            .try_use_portal(portal_delay, target_world, pos)
            .await;
    }
}
