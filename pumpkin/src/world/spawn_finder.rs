use std::sync::Arc;

use pumpkin_data::fluid::Fluid;
use pumpkin_util::GameMode;
use pumpkin_util::math::{
    boundingbox::{BoundingBox, EntityDimensions},
    position::BlockPos,
    vector2::Vector2,
    vector3::Vector3,
};
use pumpkin_world::chunk::ChunkHeightmapType;
use rand::RngExt;

use crate::world::World;

/// Vanilla `PlayerSpawnFinder.PLAYER_DIMENSIONS`.
const PLAYER_DIMENSIONS: EntityDimensions = EntityDimensions::new(0.6, 1.8, 1.62);

/// Vanilla `PlayerSpawnFinder.ABSOLUTE_MAX_ATTEMPTS`.
const ABSOLUTE_MAX_ATTEMPTS: i64 = 1024;

/// Port of vanilla's `PlayerSpawnFinder.findSpawn`.
///
/// Searches a coprime-offset spiral of candidates around `suggestion` within
/// the `respawn_radius` game rule (clamped to the world border distance),
/// returning the first position with a solid, unobstructed floor. Falls back
/// to a vertical walk from `suggestion` (`fixupSpawnHeight`) if no candidate
/// succeeds, matching vanilla's own fallback and its Adventure-mode shortcut.
pub async fn find_safe_world_spawn(world: &Arc<World>, suggestion: BlockPos) -> Vector3<f64> {
    let adventure_mode = if let Some(server) = world.server.upgrade() {
        server.defaultgamemode.lock().await.gamemode == GameMode::Adventure
    } else {
        false
    };
    if adventure_mode {
        return fixup_spawn_height(world, suggestion);
    }

    let respawn_radius = world.level_info.load().game_rules.respawn_radius;
    let mut radius = i32::try_from(respawn_radius.max(0)).unwrap_or(i32::MAX);

    let dist_to_border = {
        let border = world.worldborder.lock().await;
        border
            .distance_to_border(f64::from(suggestion.0.x), f64::from(suggestion.0.z))
            .floor() as i32
    };
    if dist_to_border < radius {
        radius = dist_to_border;
    }
    if dist_to_border <= 1 {
        radius = 1;
    }

    let square_side = i64::from(radius) * 2 + 1;
    let candidate_count = ABSOLUTE_MAX_ATTEMPTS.min(square_side * square_side);
    let coprime = get_coprime(candidate_count);
    let offset = rand::rng().random_range(0..candidate_count.max(1));

    // Vanilla schedules candidates asynchronously via chunk-loading tickets; this
    // uses a plain sequential loop, awaiting the chunk fetch inline instead.
    for candidate_index in 0..candidate_count {
        let value = (offset + coprime * candidate_index) % candidate_count;
        let delta_x = value % square_side;
        let delta_z = value / square_side;
        let target_x = suggestion.0.x + i32::try_from(delta_x).unwrap_or(0) - radius;
        let target_z = suggestion.0.z + i32::try_from(delta_z).unwrap_or(0) - radius;

        let chunk_pos = Vector2::new(target_x >> 4, target_z >> 4);
        world.level.get_or_fetch_chunk(chunk_pos, |_| ()).await;

        if let Some(spawn_pos) = get_level_respawn_pos(world, target_x, target_z)
            && no_collision_no_liquid(world, &spawn_pos)
        {
            return at_bottom_center_of(spawn_pos);
        }
    }

    fixup_spawn_height(world, suggestion)
}

/// Vanilla `PlayerSpawnFinder.getCoprime`.
const fn get_coprime(possible_origins: i64) -> i64 {
    if possible_origins <= 16 {
        possible_origins - 1
    } else {
        17
    }
}

/// Port of vanilla's `getLevelRespawnPos`. Returns the floor block a player
/// could stand on at this column, or `None` if the column is unsuitable
/// (below the world, or a "ravine-like" surface/ocean-floor mismatch).
fn get_level_respawn_pos(world: &World, x: i32, z: i32) -> Option<BlockPos> {
    let min_y = world.dimension.min_y;

    // No per-generator getSpawnHeight; approximated as min_y + 32 (vanilla's
    // default Nether spawn height), clamped to the dimension's top.
    let top_y = if world.dimension.has_ceiling {
        (min_y + 32).min(min_y + world.dimension.height - 1)
    } else {
        world.get_heightmap_height(ChunkHeightmapType::MotionBlocking, x, z)
    };
    if top_y < min_y {
        return None;
    }

    // Vanilla also rejects shallow-water/ice-plateau columns via the OCEAN_FLOOR
    // heightmap; Pumpkin has no OceanFloor variant, so that filter is skipped.
    let mut y = top_y + 1;
    while y >= min_y {
        let pos = BlockPos(Vector3::new(x, y, z));
        let state = world.get_block_state(&pos);
        if Fluid::from_state_id(state.id).is_some() {
            break;
        }
        if state.is_solid() {
            let above = pos.up();
            let above_state = world.get_block_state(&above);
            if !above_state.is_solid() && Fluid::from_state_id(above_state.id).is_none() {
                return Some(above);
            }
        }
        y -= 1;
    }

    None
}

/// Port of vanilla's `fixupSpawnHeight`: walks up from `spawn_pos` until clear
/// of any collision/liquid, then back down until landing just above a solid
/// floor.
fn fixup_spawn_height(world: &World, spawn_pos: BlockPos) -> Vector3<f64> {
    let min_y = world.dimension.min_y;
    let max_y = min_y + world.dimension.height;

    let mut pos = spawn_pos;
    while pos.0.y < max_y && !no_collision_no_liquid(world, &pos) {
        pos = pos.up();
    }
    pos = pos.down();
    while pos.0.y > min_y && no_collision_no_liquid(world, &pos) {
        pos = pos.down();
    }
    pos = pos.up();

    at_bottom_center_of(pos)
}

/// Port of vanilla's `noCollisionNoLiquid`: the block at `pos` isn't a fluid,
/// and a standing player's bounding box placed there has no collisions.
fn no_collision_no_liquid(world: &World, pos: &BlockPos) -> bool {
    let state = world.get_block_state(pos);
    if Fluid::from_state_id(state.id).is_some() {
        return false;
    }
    let bb = BoundingBox::new_from_pos(
        f64::from(pos.0.x) + 0.5,
        f64::from(pos.0.y),
        f64::from(pos.0.z) + 0.5,
        &PLAYER_DIMENSIONS,
    );
    world.is_space_empty(bb)
}

/// Vanilla `Vec3.atBottomCenterOf`.
fn at_bottom_center_of(pos: BlockPos) -> Vector3<f64> {
    Vector3::new(
        f64::from(pos.0.x) + 0.5,
        f64::from(pos.0.y),
        f64::from(pos.0.z) + 0.5,
    )
}
