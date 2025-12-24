use std::sync::Arc;

use crate::block::{
    BlockBehaviour, BlockFuture, GetStateForNeighborUpdateArgs, OnEntityCollisionArgs,
};
use crate::entity::EntityBase;
use crate::world::World;
use crate::world::portal::nether::NetherPortal;
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
        let level_info = world.level_info.read().await;
        match entity_type.id {
            id if id == EntityType::PLAYER.id => (world
                .get_player_by_id(entity.get_entity().entity_id)
                .await)
                .map_or(80, |player| match player.gamemode.load() {
                    GameMode::Creative => {
                        level_info.game_rules.players_nether_portal_creative_delay as u32
                    }
                    _ => level_info.game_rules.players_nether_portal_default_delay as u32,
                }),
            _ => 0,
        }
    }
}

impl BlockBehaviour for NetherPortalBlock {
    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let axis = args.direction.to_axis();
            let is_horizontal = axis == Axis::X && axis == Axis::Z;
            let state_axis =
                NetherPortalLikeProperties::from_state_id(args.state_id, &Block::NETHER_PORTAL)
                    .axis;
            if is_horizontal
                || args.neighbor_state_id == args.state_id
                || NetherPortal::get_on_axis(args.world, args.position, state_axis)
                    .await
                    .is_some_and(|e| e.was_already_valid())
            {
                return args.state_id;
            }
            Block::AIR.default_state.id
        })
    }

    fn on_entity_collision<'a>(&'a self, args: OnEntityCollisionArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
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
                .try_use_portal(portal_delay, target_world, *args.position)
                .await;
        })
    }
}
