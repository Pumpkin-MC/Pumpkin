//! Pillager patrols — vanilla `PatrolSpawner`.
//!
//! Ground truth: `/root/Vanilla/src/net/minecraft/world/level/levelgen/PatrolSpawner.java`
//! and `/root/Vanilla/src/net/minecraft/world/entity/monster/PatrollingMonster.java`.
//!
//! Driven from the world tick beside `PhantomSpawner`, matching vanilla, where both
//! live in the overworld's `customSpawners` list
//! (`MinecraftServer.java:460`) and are ticked by
//! `ServerLevel.tickCustomSpawners` (`ServerLevel.java:454-457`) from
//! `ServerChunkCache.tick` (`ServerChunkCache.java:386-387`).

use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};

use pumpkin_data::entity::EntityType;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::chunk::ChunkHeightmapType;
use rand::{RngExt, rng};
use uuid::Uuid;

use crate::entity::EntityBase;
use crate::entity::mob::MobEntity;
use crate::entity::mob::equipment::RegionalDifficulty;
use crate::entity::r#type::from_type;
use crate::world::World;
use crate::world::natural_spawner;

use super::village;

/// Vanilla `PatrolSpawner.tick` interval: `12000 + random.nextInt(1200)`
/// (`PatrolSpawner.java:38`).
pub const PATROL_INTERVAL_BASE: i32 = 12000;
/// Random span added to [`PATROL_INTERVAL_BASE`] (`PatrolSpawner.java:38`).
pub const PATROL_INTERVAL_SPAN: i32 = 1200;

/// Vanilla `random.nextInt(5) != 0` gate (`PatrolSpawner.java:42`).
pub const PATROL_SPAWN_CHANCE_DENOMINATOR: i32 = 5;

/// Vanilla `level.isCloseToVillage(player.blockPosition(), 2)` (`PatrolSpawner.java:53`).
pub const PATROL_VILLAGE_SECTION_DISTANCE: i32 = 2;

/// Vanilla offset: `(24 + random.nextInt(24)) * (random.nextBoolean() ? -1 : 1)`
/// (`PatrolSpawner.java:56-57`).
pub const PATROL_OFFSET_MIN: i32 = 24;
/// Random span of the patrol spawn offset (`PatrolSpawner.java:56-57`).
pub const PATROL_OFFSET_SPAN: i32 = 24;

/// Vanilla `int delta = 10` chunk-presence margin (`PatrolSpawner.java:59-60`).
pub const PATROL_CHUNK_MARGIN: i32 = 10;

/// Vanilla `PatrollingMonster.findPatrolTarget` offset range
/// (`PatrollingMonster.java:118`): `-500 + random.nextInt(1000)` on x and z.
pub const PATROL_TARGET_OFFSET_BASE: i32 = -500;
/// Span of the patrol-target offset (`PatrollingMonster.java:118`).
pub const PATROL_TARGET_OFFSET_SPAN: i32 = 1000;

/// Vanilla `PatrollingMonster.finalizeSpawn` natural-leader chance
/// (`PatrollingMonster.java:74`): `random.nextFloat() < 0.06f`, and only for spawn
/// reasons other than `PATROL`, `EVENT` and `STRUCTURE`.
pub const NATURAL_PATROL_LEADER_CHANCE: f32 = 0.06;

/// Vanilla `PatrolSpawner` (`PatrolSpawner.java:21-102`).
pub struct PatrolSpawner {
    /// Vanilla `PatrolSpawner.nextTick` (`PatrolSpawner.java:23`).
    next_tick: AtomicI32,
}

impl Default for PatrolSpawner {
    fn default() -> Self {
        Self {
            // Vanilla starts at 0, so the first eligible tick rolls immediately.
            next_tick: AtomicI32::new(0),
        }
    }
}

impl PatrolSpawner {
    /// Vanilla `PatrolSpawner.tick(level, spawnEnemies)` (`PatrolSpawner.java:26-79`).
    ///
    /// `spawn_enemies` is the same flag `PhantomSpawner` receives from the world
    /// tick, i.e. vanilla `ServerChunkCache.spawnEnemies`.
    pub async fn tick(&self, world: &Arc<World>, spawn_enemies: bool) {
        // PatrolSpawner.java:27-32.
        if !spawn_enemies {
            return;
        }
        if !world.level_info.load().game_rules.spawn_patrols {
            return;
        }

        // PatrolSpawner.java:34-38 — decrement, then re-arm.
        let remaining = self.next_tick.fetch_sub(1, Ordering::Relaxed) - 1;
        if remaining > 0 {
            return;
        }
        let interval = PATROL_INTERVAL_BASE + rng().random_range(0..PATROL_INTERVAL_SPAN);
        // Vanilla does `nextTick += ...` on a value that has just gone <= 0.
        self.next_tick.fetch_add(interval, Ordering::Relaxed);

        // PatrolSpawner.java:39-41 — `level.isBrightOutside()`.
        if !is_bright_outside(world).await {
            return;
        }
        // PatrolSpawner.java:42-44.
        if rng().random_range(0..PATROL_SPAWN_CHANCE_DENOMINATOR) != 0 {
            return;
        }

        // PatrolSpawner.java:45-52 — pick one random non-spectator player.
        let players = world.players.load();
        if players.is_empty() {
            return;
        }
        let index = rng().random_range(0..players.len());
        let Some(player) = players.get(index).cloned() else {
            return;
        };
        drop(players);
        if player.is_spectator() {
            return;
        }

        let player_pos = player.get_entity().block_pos.load();
        // PatrolSpawner.java:53-55 — never spawn a patrol on top of a village.
        if village::is_close_to_village(world, &player_pos, PATROL_VILLAGE_SECTION_DISTANCE) {
            return;
        }

        // PatrolSpawner.java:56-58.
        let offset_x = patrol_offset();
        let offset_z = patrol_offset();
        let mut spawn_pos = BlockPos::new(
            player_pos.0.x + offset_x,
            player_pos.0.y,
            player_pos.0.z + offset_z,
        );

        // PatrolSpawner.java:59-62 — the 10-block chunk-presence margin.
        if !chunks_present_around(world, &spawn_pos, PATROL_CHUNK_MARGIN) {
            return;
        }

        // PatrolSpawner.java:63-65 is the `CAN_PILLAGER_PATROL_SPAWN` environment
        // attribute. Pumpkin has no environment-attribute system, and the two
        // vanilla sources that switch it off are the mushroom-fields biome
        // (`OverworldBiomes.java:217`) and the `EARLY_GAME` timeline which keeps
        // patrols off for the first 120000 ticks (`Timelines.java:63`). Neither is
        // ported, so the gate is skipped rather than guessed at.

        // PatrolSpawner.java:66 — group size from the regional difficulty.
        let group_size = patrol_group_size(world, &spawn_pos);

        for i in 0..group_size {
            // PatrolSpawner.java:68 — snap to the surface each iteration.
            let surface_y = world.get_heightmap_height(
                ChunkHeightmapType::MotionBlockingNoLeaves,
                spawn_pos.0.x,
                spawn_pos.0.z,
            );
            spawn_pos = BlockPos::new(spawn_pos.0.x, surface_y, spawn_pos.0.z);

            // PatrolSpawner.java:69-75 — the leader must succeed or the group aborts.
            let is_leader = i == 0;
            let spawned = spawn_patrol_member(world, &spawn_pos, is_leader).await;
            if is_leader && !spawned {
                break;
            }

            // PatrolSpawner.java:76-77 — scatter the next member.
            spawn_pos = BlockPos::new(
                spawn_pos.0.x + rng().random_range(0..5) - rng().random_range(0..5),
                spawn_pos.0.y,
                spawn_pos.0.z + rng().random_range(0..5) - rng().random_range(0..5),
            );
        }
    }
}

/// Vanilla `(24 + random.nextInt(24)) * (random.nextBoolean() ? -1 : 1)`
/// (`PatrolSpawner.java:56-57`).
fn patrol_offset() -> i32 {
    let magnitude = PATROL_OFFSET_MIN + rng().random_range(0..PATROL_OFFSET_SPAN);
    if rng().random::<bool>() {
        -magnitude
    } else {
        magnitude
    }
}

/// Vanilla `ceil(getCurrentDifficultyAt(pos).getEffectiveDifficulty()) + 1`
/// (`PatrolSpawner.java:66`).
fn patrol_group_size(world: &Arc<World>, pos: &BlockPos) -> i32 {
    let difficulty = RegionalDifficulty::at(world, pos.to_f64());
    #[expect(
        clippy::cast_possible_truncation,
        reason = "effective difficulty is bounded by 0..~6; ceil keeps it in i32"
    )]
    let ceiled = difficulty.effective_difficulty.ceil() as i32;
    ceiled + 1
}

/// Vanilla `Level.isBrightOutside` — the inverse of `isDarkOutside`, i.e. daytime.
///
/// Pumpkin tracks the same quantity as `World::sky_darken` (vanilla
/// `ambientDarkness`), refreshed each environment tick in
/// `pumpkin/src/world/tick.rs`. Daylight means a darken value below the
/// monster-spawn threshold, which is how the phantom spawner reads the same field.
async fn is_bright_outside(world: &Arc<World>) -> bool {
    let time = world.level_time.lock().await.time_of_day.rem_euclid(24000);
    // Vanilla `Level.isBrightOutside()` is `!isDarkOutside()`, which for the
    // overworld clock is the day half of the cycle.
    (0..12000).contains(&time)
}

/// Vanilla `level.hasChunksAt(x - 10, z - 10, x + 10, z + 10)`
/// (`PatrolSpawner.java:60`).
fn chunks_present_around(world: &Arc<World>, pos: &BlockPos, margin: i32) -> bool {
    let min_chunk_x = (pos.0.x - margin) >> 4;
    let max_chunk_x = (pos.0.x + margin) >> 4;
    let min_chunk_z = (pos.0.z - margin) >> 4;
    let max_chunk_z = (pos.0.z + margin) >> 4;
    for chunk_x in min_chunk_x..=max_chunk_x {
        for chunk_z in min_chunk_z..=max_chunk_z {
            let chunk = pumpkin_util::math::vector2::Vector2::new(chunk_x, chunk_z);
            if !world.level.loaded_chunks.contains_key(&chunk) {
                return false;
            }
        }
    }
    true
}

/// Vanilla `PatrolSpawner.spawnPatrolMember` (`PatrolSpawner.java:81-101`).
///
/// Returns whether a pillager was actually placed.
async fn spawn_patrol_member(world: &Arc<World>, pos: &BlockPos, is_leader: bool) -> bool {
    // PatrolSpawner.java:82-85 — `NaturalSpawner.isValidEmptySpawnBlock`.
    let state = world.get_block_state(pos);
    if !natural_spawner::is_valid_empty_spawn_block(state, &EntityType::PILLAGER) {
        return false;
    }
    // PatrolSpawner.java:86-88 — `checkPatrollingMonsterSpawnRules`.
    if !MobEntity::check_patrolling_monster_spawn_rules(world, pos) {
        return false;
    }

    let entity = from_type(&EntityType::PILLAGER, pos.to_f64(), world, Uuid::new_v4());
    let uuid = entity.get_entity().entity_uuid;

    // PatrolSpawner.java:91-94 — the leader gets a patrol target; all members are
    // marked patrolling because `finalizeSpawn` sees `EntitySpawnReason.PATROL`
    // (`PatrollingMonster.java:79-81`).
    world.raids.raiders.update(uuid, |member| {
        member.patrolling = true;
        if is_leader {
            // `setPatrolLeader(true)` (`PatrollingMonster.java:112-115`).
            member.patrol_leader = true;
            // `findPatrolTarget()` (`PatrollingMonster.java:117-120`).
            member.patrol_target = Some(find_patrol_target(pos));
        }
    });

    // PatrolSpawner.java:97 — `addFreshEntityWithPassengers`.
    world.spawn_entity(entity).await;
    true
}

/// Vanilla `PatrollingMonster.findPatrolTarget` (`PatrollingMonster.java:117-120`).
#[must_use]
pub fn find_patrol_target(pos: &BlockPos) -> BlockPos {
    BlockPos::new(
        pos.0.x + PATROL_TARGET_OFFSET_BASE + rng().random_range(0..PATROL_TARGET_OFFSET_SPAN),
        pos.0.y,
        pos.0.z + PATROL_TARGET_OFFSET_BASE + rng().random_range(0..PATROL_TARGET_OFFSET_SPAN),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patrol_offset_stays_in_the_vanilla_ring() {
        for _ in 0..1000 {
            let offset = patrol_offset();
            let magnitude = offset.abs();
            assert!(
                (PATROL_OFFSET_MIN..PATROL_OFFSET_MIN + PATROL_OFFSET_SPAN).contains(&magnitude),
                "offset {offset} outside 24..48"
            );
        }
    }

    #[test]
    fn patrol_offset_takes_both_signs() {
        let mut saw_negative = false;
        let mut saw_positive = false;
        for _ in 0..1000 {
            if patrol_offset() < 0 {
                saw_negative = true;
            } else {
                saw_positive = true;
            }
        }
        assert!(saw_negative && saw_positive);
    }

    #[test]
    fn patrol_target_stays_within_the_vanilla_span() {
        let origin = BlockPos::new(0, 64, 0);
        for _ in 0..1000 {
            let target = find_patrol_target(&origin);
            assert!((-500..500).contains(&target.0.x));
            assert!((-500..500).contains(&target.0.z));
            // findPatrolTarget only offsets x and z.
            assert_eq!(target.0.y, 64);
        }
    }

    #[test]
    fn spawner_re_arms_within_the_vanilla_interval() {
        let spawner = PatrolSpawner::default();
        // Default 0 means the very next tick is eligible.
        assert_eq!(spawner.next_tick.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn constants_match_vanilla() {
        // PatrolSpawner.java:38, 42, 53, 56, 59.
        assert_eq!(PATROL_INTERVAL_BASE, 12000);
        assert_eq!(PATROL_INTERVAL_SPAN, 1200);
        assert_eq!(PATROL_SPAWN_CHANCE_DENOMINATOR, 5);
        assert_eq!(PATROL_VILLAGE_SECTION_DISTANCE, 2);
        assert_eq!(PATROL_OFFSET_MIN, 24);
        assert_eq!(PATROL_OFFSET_SPAN, 24);
        assert_eq!(PATROL_CHUNK_MARGIN, 10);
        // PatrollingMonster.java:74, 118.
        assert!((NATURAL_PATROL_LEADER_CHANCE - 0.06).abs() < f32::EPSILON);
        assert_eq!(PATROL_TARGET_OFFSET_BASE, -500);
        assert_eq!(PATROL_TARGET_OFFSET_SPAN, 1000);
    }
}
