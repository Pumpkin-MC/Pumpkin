use std::sync::{Arc, atomic::Ordering};

use pumpkin_data::{
    Block, BlockDirection, BlockState, BlockStateId, Rotation,
    block_properties::{Axis, HorizontalAxis, NetherPortalLikeProperties},
    dimension::Dimension,
    entity::EntityType,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::{Difficulty, GameMode, math::vector3::Vector3};
use rand::RngExt;
use uuid::Uuid;

use crate::{
    block::{
        BlockBehaviour, GetStateForNeighborUpdateArgs, OnEntityCollisionArgs, OnStateReplacedArgs,
        RandomTickArgs,
    },
    entity::{EntityBase, r#type::from_type},
    world::{World, portal::nether::NetherPortal},
};

#[pumpkin_block("minecraft:nether_portal")]
pub struct NetherPortalBlock;

impl NetherPortalBlock {
    /// Gets the portal delay time based on entity type and gamemode
    #[must_use]
    pub fn get_portal_time(world: &Arc<World>, entity: &dyn EntityBase) -> u32 {
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
    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let direction_axis = args.direction.to_axis();
        let state_axis = NetherPortalLikeProperties::from_state_id(args.state_id).axis;
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
                .is_some_and(|e| e.was_already_valid())
        {
            return args.state_id;
        }
        Block::AIR.default_state.id
    }

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        let level_info = args.world.level_info.load();
        let difficulty = level_info.difficulty;
        if !level_info.game_rules.spawn_mobs
            || difficulty == Difficulty::Peaceful
            || (args.world.dimension != Dimension::OVERWORLD
                && args.world.dimension != Dimension::OVERWORLD_CAVES)
        {
            return;
        }

        let difficulty_id = difficulty as u32;
        let roll = rand::rng().random_range(0..2000);
        if roll >= difficulty_id {
            return;
        }

        let player_close = args
            .world
            .get_closest_player(args.position.to_centered_f64(), 128.0)
            .is_some();
        if !player_close {
            return;
        }

        let mut bottom_pos = *args.position;
        while args.world.get_block(&bottom_pos) == &Block::NETHER_PORTAL {
            bottom_pos = bottom_pos.down();
        }

        if args
            .world
            .get_block_state(&bottom_pos)
            .is_side_solid(BlockDirection::Up)
        {
            let spawn_pos = Vector3::new(
                bottom_pos.0.x as f64 + 0.5,
                (bottom_pos.0.y + 1) as f64,
                bottom_pos.0.z as f64 + 0.5,
            );
            let mob = from_type(
                &EntityType::ZOMBIFIED_PIGLIN,
                spawn_pos,
                args.world,
                Uuid::new_v4(),
            );
            mob.get_entity()
                .portal_cooldown
                .store(300, Ordering::Relaxed);
            args.world.spawn_entity_non_save(mob);
        }
    }

    fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        let target_world =
            if args.world.dimension.minecraft_name == Dimension::THE_NETHER.minecraft_name {
                args.server.get_world_from_dimension(&Dimension::OVERWORLD)
            } else {
                args.server.get_world_from_dimension(&Dimension::THE_NETHER)
            };

        if Arc::ptr_eq(&target_world, args.world) {
            return;
        }

        tracing::debug!(
            "Nether portal collision at {:?}, targeting world {:?}",
            args.position,
            target_world.dimension.minecraft_name
        );
        args.entity
            .get_entity()
            .try_use_portal(target_world, *args.position);
    }

    fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        // Remove from POI storage when portal block is replaced
        let mut poi_storage = args
            .world
            .portal_poi
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        poi_storage.remove(args.position);
    }

    fn rotate(
        &self,
        block: &Block,
        state_id: BlockStateId,
        rotation: Rotation,
    ) -> &'static BlockState {
        match rotation {
            Rotation::Clockwise90 | Rotation::CounterClockwise90 => {
                let mut props = NetherPortalLikeProperties::from_state_id(state_id);
                props.axis = match props.axis {
                    HorizontalAxis::X => HorizontalAxis::Z,
                    HorizontalAxis::Z => HorizontalAxis::X,
                };
                let new_state_id = props.to_state_id(block);
                BlockState::from_id(new_state_id)
            }
            _ => BlockState::from_id(state_id),
        }
    }
}
