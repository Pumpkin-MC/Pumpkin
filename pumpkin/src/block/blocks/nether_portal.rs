use std::sync::Arc;

use crate::block::pumpkin_block::{
    GetStateForNeighborUpdateArgs, OnEntityCollisionArgs, PumpkinBlock,
};
use crate::entity::EntityBase;
use crate::world::World;
use crate::world::portal::nether::NetherPortal;
use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::block_properties::{Axis, BlockProperties, NetherPortalLikeProperties};
use pumpkin_data::entity::EntityType;
use pumpkin_macros::pumpkin_block;
use pumpkin_registry::VanillaDimensionType;
use pumpkin_util::GameMode;
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
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let axis = args.direction.to_axis();
        let is_horizontal = axis == Axis::X && axis == Axis::Z;
        let state_axis =
            NetherPortalLikeProperties::from_state_id(args.state_id, &Block::NETHER_PORTAL).axis;
        if is_horizontal
            || args.neighbor_state_id == args.state_id
            || NetherPortal::get_on_axis(args.world, args.location, state_axis)
                .await
                .is_some_and(|e| e.was_already_valid())
        {
            return args.state_id;
        }
        Block::AIR.default_state.id
    }

    async fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        let target_world = if args.world.dimension_type == VanillaDimensionType::TheNether {
            args.server
                .get_world_from_dimension(VanillaDimensionType::Overworld)
                .await
        } else {
            args.server
                .get_world_from_dimension(VanillaDimensionType::TheNether)
                .await
        };

        let portal_delay = Self::get_portal_time(args.world, args.entity).await;

        args.entity
            .get_entity()
            .try_use_portal(portal_delay, target_world, *args.location)
            .await;
    }
}
