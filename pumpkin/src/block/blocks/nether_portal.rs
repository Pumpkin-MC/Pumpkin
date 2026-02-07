use std::sync::Arc;

use crate::block::{
    BlockBehaviour, BlockFuture, GetStateForNeighborUpdateArgs, OnEntityCollisionArgs,
    OnStateReplacedArgs,
};
use crate::entity::EntityBase;
use crate::world::World;
use crate::world::portal::nether::NetherPortal;
use pumpkin_data::Block;
use pumpkin_data::block_properties::{
    Axis, BlockProperties, HorizontalAxis, NetherPortalLikeProperties,
};
use pumpkin_data::dimension::Dimension;
use pumpkin_data::entity::EntityType;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::GameMode;
use pumpkin_world::BlockStateId;

#[pumpkin_block("minecraft:nether_portal")]
pub struct NetherPortalBlock;

impl NetherPortalBlock {
    /// Gets the portal delay time based on entity type and gamemode
    fn get_portal_time(world: &Arc<World>, entity: &dyn EntityBase) -> u32 {
        let entity_type = entity.get_entity().entity_type;
        let level_info = world.level_info.load();
        match entity_type.id {
            id if id == EntityType::PLAYER.id => (world
                .get_player_by_id(entity.get_entity().entity_id))
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
            let direction_axis = args.direction.to_axis();
            let state_axis =
                NetherPortalLikeProperties::from_state_id(args.state_id, &Block::NETHER_PORTAL)
                    .axis;
            // Convert HorizontalAxis to Axis for comparison
            let state_axis_full: Axis = match state_axis {
                HorizontalAxis::X => Axis::X,
                HorizontalAxis::Z => Axis::Z,
            };
            // Vanilla logic: keep portal if direction is horizontal AND different from portal axis
            let is_horizontal_and_different =
                args.direction.is_horizontal() && direction_axis != state_axis_full;
            if is_horizontal_and_different
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
            let target_world = if args.world.dimension == Dimension::THE_NETHER {
                args.server.get_world_from_dimension(&Dimension::OVERWORLD)
            } else {
                args.server.get_world_from_dimension(&Dimension::THE_NETHER)
            };

            let portal_delay = Self::get_portal_time(args.world, args.entity);

            args.entity
                .get_entity()
                .try_use_portal(portal_delay, target_world, *args.position)
                .await;
        })
    }

    fn on_state_replaced<'a>(&'a self, args: OnStateReplacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            // Remove from POI storage when portal block is replaced
            let mut poi_storage = args.world.portal_poi.lock().await;
            poi_storage.remove(args.position);
        })
    }
}
