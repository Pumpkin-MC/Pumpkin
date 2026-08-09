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
pub async fn find_safe_world_spawn(world: &World, suggestion: BlockPos) -> Vector3<f64> {
    let adventure_mode = if let Some(server) = world.server.upgrade() {
        server.defaultgamemode.lock().await.gamemode == GameMode::Adventure
    } else {
        false
    };
    if adventure_mode {
        load_spawn_chunk(world, suggestion).await;
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

    load_spawn_chunk(world, suggestion).await;
    fixup_spawn_height(world, suggestion)
}

async fn load_spawn_chunk(world: &World, suggestion: BlockPos) {
    world
        .level
        .get_or_fetch_chunk(
            Vector2::new(suggestion.0.x >> 4, suggestion.0.z >> 4),
            |_| (),
        )
        .await;
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

    // ChunkGenerator.getSpawnHeight: noise generators return 64, while flat
    // generators return minY plus the number of configured layers.
    let top_y = if world.dimension.has_ceiling {
        match world.level.world_gen.as_ref() {
            pumpkin_world::generation::generator::WorldGenerator::Noise(_) => 64,
            pumpkin_world::generation::generator::WorldGenerator::Flat(generator) => {
                min_y
                    + generator
                        .layers
                        .iter()
                        .map(|layer| layer.height)
                        .sum::<i32>()
            }
        }
    } else {
        world.get_heightmap_height(ChunkHeightmapType::MotionBlocking, x, z)
    };
    if top_y < min_y {
        return None;
    }

    // Vanilla rejects columns where a body of water sits over dry terrain: the
    // WORLD_SURFACE height (ignores fluids) is at or below the MOTION_BLOCKING
    // top (which counts water as blocking) but above the true OCEAN_FLOOR.
    let surface = world.get_heightmap_height(ChunkHeightmapType::WorldSurface, x, z);
    let ocean_floor = world.get_heightmap_height(ChunkHeightmapType::OceanFloor, x, z);
    if is_ocean_covered_column(surface, top_y, ocean_floor) {
        return None;
    }

    let mut y = top_y + 1;
    while y >= min_y {
        let pos = BlockPos(Vector3::new(x, y, z));
        let state = world.get_block_state(&pos);
        if Fluid::from_state_id(state.id).is_some() {
            break;
        }
        if has_full_upward_face(&state) {
            return Some(pos.up());
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

/// Port of vanilla's `Block.isFaceFull(shape, UP)` for the collision shapes
/// exposed by Pumpkin. The face may be covered by more than one shape.
fn has_full_upward_face(state: &pumpkin_data::BlockState) -> bool {
    let shapes: Vec<_> = state
        .get_block_collision_shapes()
        .filter(|shape| shape.min.y <= 1.0 && shape.max.y >= 1.0)
        .collect();
    if shapes.is_empty() {
        return false;
    }

    let mut x_edges = vec![0.0, 1.0];
    let mut z_edges = vec![0.0, 1.0];
    for shape in &shapes {
        x_edges.push(shape.min.x.clamp(0.0, 1.0));
        x_edges.push(shape.max.x.clamp(0.0, 1.0));
        z_edges.push(shape.min.z.clamp(0.0, 1.0));
        z_edges.push(shape.max.z.clamp(0.0, 1.0));
    }
    x_edges.sort_by(f64::total_cmp);
    x_edges.dedup();
    z_edges.sort_by(f64::total_cmp);
    z_edges.dedup();

    x_edges.windows(2).all(|x| {
        z_edges.windows(2).all(|z| {
            let x = (x[0] + x[1]) * 0.5;
            let z = (z[0] + z[1]) * 0.5;
            shapes.iter().any(|shape| {
                shape.min.x <= x && x <= shape.max.x && shape.min.z <= z && z <= shape.max.z
            })
        })
    })
}

/// Port of vanilla's `noCollisionNoLiquid`: a standing player's bounding box
/// has no block, fluid, or entity collisions.
fn no_collision_no_liquid(world: &World, pos: &BlockPos) -> bool {
    let bb = BoundingBox::new_from_pos(
        f64::from(pos.0.x) + 0.5,
        f64::from(pos.0.y),
        f64::from(pos.0.z) + 0.5,
        &PLAYER_DIMENSIONS,
    );
    if has_block_collision(world, bb) || has_fluid_collision(world, bb) {
        return false;
    }

    // CollisionGetter.noCollision(null, box, true) queries an epsilon-expanded
    // box, then tests the actual entity boxes. Items and projectiles are not
    // included because they cannot be collided with by a null source.
    let query_box = bb.expand(1.0e-7, 1.0e-7, 1.0e-7);
    !world.get_all_at_box(&query_box).iter().any(|entity| {
        let entity = entity.as_ref();
        !entity.is_spectator()
            && entity.get_entity().is_alive()
            && entity.can_be_collided_with()
            && entity.get_entity().bounding_box.load().intersects(&bb)
    })
}

fn has_block_collision(world: &World, bounding_box: BoundingBox) -> bool {
    BlockPos::iterate(bounding_box.min_block_pos(), bounding_box.max_block_pos()).any(|pos| {
        let state = world.get_block_state(&pos);
        !state.is_air()
            && state
                .get_block_collision_shapes()
                .any(|shape| shape.at_pos(pos).intersects(&bounding_box))
    })
}

fn has_fluid_collision(world: &World, bounding_box: BoundingBox) -> bool {
    BlockPos::iterate(bounding_box.min_block_pos(), bounding_box.max_block_pos()).any(|pos| {
        let (fluid, state) = world.get_fluid_and_fluid_state(&pos);
        if fluid.id == Fluid::EMPTY.id {
            return false;
        }

        let fluid_min_y = f64::from(pos.0.y);
        let fluid_max_y = fluid_min_y + world.get_fluid_height(&pos, fluid, state);
        fluid_max_y > bounding_box.min.y && fluid_min_y < bounding_box.max.y
    })
}

/// Vanilla `Vec3.atBottomCenterOf`.
fn at_bottom_center_of(pos: BlockPos) -> Vector3<f64> {
    Vector3::new(
        f64::from(pos.0.x) + 0.5,
        f64::from(pos.0.y),
        f64::from(pos.0.z) + 0.5,
    )
}

/// Vanilla's `getLevelRespawnPos` ocean-column rejection: `surface <= topY &&
/// surface > oceanFloor`. True when a body of water sits over dry-ish
/// terrain, e.g. shallow water or ice over land.
const fn is_ocean_covered_column(surface: i32, top_y: i32, ocean_floor: i32) -> bool {
    surface <= top_y && surface > ocean_floor
}

#[cfg(test)]
mod tests {
    use super::is_ocean_covered_column;

    #[test]
    fn rejects_water_over_dry_terrain() {
        // WORLD_SURFACE sits at the water's surface (== MOTION_BLOCKING top),
        // but well above the true OCEAN_FLOOR: a deep column of water.
        assert!(is_ocean_covered_column(70, 70, 60));
    }

    #[test]
    fn accepts_dry_land() {
        // No water: WORLD_SURFACE, MOTION_BLOCKING top, and OCEAN_FLOOR all
        // agree on the same solid ground height.
        assert!(!is_ocean_covered_column(70, 70, 70));
    }

    #[test]
    fn accepts_surface_above_motion_blocking_top() {
        // topY came from a cave-world spawn height below the true surface;
        // vanilla only rejects when surface <= topY.
        assert!(!is_ocean_covered_column(80, 70, 60));
    }

    #[test]
    fn accepts_when_ocean_floor_matches_surface() {
        // No fluid above the floor at all: surface == ocean_floor.
        assert!(!is_ocean_covered_column(70, 75, 70));
    }
}
