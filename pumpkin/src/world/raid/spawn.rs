//! Wave spawning and spawn-position search.
//!
//! Ground truth: `/root/Vanilla/src/net/minecraft/world/entity/raid/Raid.java`
//! (`spawnGroup` at 456-491, `joinRaid` at 493-508, `findRandomSpawnPos` at
//! 571-589, `playSound` at 440-454).

use std::sync::Arc;

use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::chunk::ChunkHeightmapType;
use rand::{RngExt, rng};
use uuid::Uuid;

use crate::entity::EntityBase;
use crate::world::World;
use crate::world::natural_spawner::is_valid_empty_spawn_block;

use super::member::RaiderRegistry;
use super::state::{
    ALLOW_SPAWNING_WITHIN_VILLAGE_SECONDS_THRESHOLD, NUM_SPAWN_ATTEMPTS, VALID_RAID_RADIUS,
    VILLAGE_SEARCH_RADIUS,
};
use super::tick::{
    RaiderFacts, WorldFacts, grant_heroes_of_the_village, is_position_entity_ticking,
    update_raiders,
};
use super::wave::{BonusCap, RaiderType, bonus_cap, ravager_rider};
use super::{Raid, village};

/// One raider the wave wants to create.
#[derive(Clone, Copy, Debug)]
pub struct PlannedRaider {
    /// The entity type to create.
    pub entity_type: &'static EntityType,
    /// Whether this raider becomes the wave's patrol leader — vanilla sets the
    /// first raider for which `canBeLeader()` holds (`Raid.java:467-471`).
    pub is_leader: bool,
    /// When set, this raider is spawned riding the raider at the given index in
    /// the plan (`Raid.java:473-484`).
    pub rides_index: Option<usize>,
}

/// Vanilla `Raid.spawnGroup` composition step (`Raid.java:456-491`), without the
/// world.
///
/// Returns the raiders in vanilla's creation order: the `RaiderType.VALUES` outer
/// loop (`Raid.java:462`), the per-type count inner loop (`Raid.java:466`), and a
/// ravager's rider appended right after the ravager itself (`Raid.java:482`).
///
/// `bonus_draw` supplies vanilla's `random.nextInt(bonusSpawns + 1)`
/// (`Raid.java:673`) and, on Easy, the `random.nextInt(2)` that produces the cap
/// itself (`Raid.java:655`). Passing a deterministic closure makes the whole
/// composition testable.
#[must_use]
pub fn plan_wave(
    wave: i32,
    num_groups: i32,
    is_bonus_wave: bool,
    difficulty: pumpkin_util::Difficulty,
    mut bonus_draw: impl FnMut(i32) -> i32,
) -> Vec<PlannedRaider> {
    let mut plan: Vec<PlannedRaider> = Vec::new();
    let mut leader_set = false;

    for raider_type in RaiderType::VALUES {
        // Raid.java:464.
        let default = raider_type.default_num_spawns(wave, num_groups, is_bonus_wave);
        let bonus = match bonus_cap(raider_type, wave, difficulty, is_bonus_wave) {
            BonusCap::None => 0,
            // Raid.java:673 — `random.nextInt(bonusSpawns + 1)`.
            BonusCap::Fixed(cap) => bonus_draw(cap + 1),
            // Raid.java:655 — the cap is itself `random.nextInt(2)`, then 673 applies.
            BonusCap::EasyRandomTwo => {
                let cap = bonus_draw(2);
                if cap > 0 { bonus_draw(cap + 1) } else { 0 }
            }
        };
        let count = default + bonus;

        let mut ravagers_spawned = 0;
        for _ in 0..count {
            // Raid.java:467-471 — every raider type in a raid can be leader
            // (`PatrollingMonster.canBeLeader` returns true, `PatrollingMonster.java:67`;
            // vanilla `Witch` and `Ravager` do not override it).
            let is_leader = !leader_set;
            leader_set = true;
            let index = plan.len();
            plan.push(PlannedRaider {
                entity_type: raider_type.entity_type(),
                is_leader,
                rides_index: None,
            });

            // Raid.java:473-484 — only ravagers carry a rider.
            if raider_type != RaiderType::Ravager {
                continue;
            }
            let rider = ravager_rider(wave, ravagers_spawned);
            ravagers_spawned += 1;
            if let Some(rider) = rider {
                plan.push(PlannedRaider {
                    entity_type: rider,
                    is_leader: false,
                    rides_index: Some(index),
                });
            }
        }
    }

    plan
}

/// Vanilla `Raid.findRandomSpawnPos` (`Raid.java:571-589`).
///
/// The ring radius shrinks as the countdown runs out:
/// `howFar = 0.22 * secondsRemaining - 0.24` (`Raid.java:573`).
///
/// # Approximation
///
/// Vanilla's final placement test is
/// `RAVAGER_SPAWN_PLACEMENT_TYPE.isSpawnPositionOk(level, pos, RAVAGER)`, falling back
/// to "snow below and air here" (`Raid.java:585`). Pumpkin's
/// `natural_spawner::rules::is_spawn_position_ok` is the port of that placement type, so
/// it is used directly; the snow fallback is reproduced verbatim.
#[must_use]
pub fn find_random_spawn_pos(
    world: &Arc<World>,
    center: BlockPos,
    raid_cooldown_ticks: i32,
    max_tries: i32,
) -> Option<BlockPos> {
    let seconds_remaining = raid_cooldown_ticks / 20;
    #[expect(
        clippy::cast_precision_loss,
        reason = "seconds_remaining is at most 15; vanilla does the same float cast"
    )]
    let how_far = 0.22f32 * seconds_remaining as f32 - 0.24f32;
    let start_angle = rng().random::<f32>() * std::f32::consts::TAU;

    for i in 0..max_tries {
        #[expect(
            clippy::cast_precision_loss,
            reason = "attempt index is bounded by max_tries (<= 20)"
        )]
        let angle = start_angle + std::f32::consts::PI * i as f32 / 8.0;
        // Raid.java:579-580.
        let spawn_x = center.0.x
            + floor_f32(angle.cos() * VILLAGE_SEARCH_RADIUS * how_far)
            + rng().random_range(0..3) * floor_f32(how_far);
        let spawn_z = center.0.z
            + floor_f32(angle.sin() * VILLAGE_SEARCH_RADIUS * how_far)
            + rng().random_range(0..3) * floor_f32(how_far);
        let spawn_y =
            world.get_heightmap_height(ChunkHeightmapType::WorldSurface, spawn_x, spawn_z);

        // Raid.java:581.
        if (spawn_y - center.0.y).abs() > VALID_RAID_RADIUS {
            continue;
        }
        let pos = BlockPos::new(spawn_x, spawn_y, spawn_z);

        // Raid.java:583 — refuse to spawn inside the village until the last seconds.
        if village::is_village(world, &pos)
            && seconds_remaining > ALLOW_SPAWNING_WITHIN_VILLAGE_SECONDS_THRESHOLD
        {
            continue;
        }
        // Raid.java:585 — the surrounding chunks must be present and ticking.
        if !has_chunks_around(world, pos) || !is_position_entity_ticking(world, pos) {
            continue;
        }
        if !is_placement_ok(world, pos) {
            continue;
        }
        return Some(pos);
    }

    None
}

/// Vanilla `Mth.floor` on an `f32`.
fn floor_f32(value: f32) -> i32 {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "matches vanilla Mth.floor, whose inputs here are small offsets"
    )]
    let floored = value.floor() as i32;
    floored
}

/// Vanilla `level.hasChunksAt(x-10, z-10, x+10, z+10)` (`Raid.java:584-585`).
fn has_chunks_around(world: &Arc<World>, pos: BlockPos) -> bool {
    const DELTA: i32 = 10;
    let min = BlockPos::new(pos.0.x - DELTA, pos.0.y, pos.0.z - DELTA).chunk_position();
    let max = BlockPos::new(pos.0.x + DELTA, pos.0.y, pos.0.z + DELTA).chunk_position();
    for cx in min.x..=max.x {
        for cz in min.y..=max.y {
            if !world
                .level
                .loaded_chunks
                .contains_key(&pumpkin_util::math::vector2::Vector2::new(cx, cz))
            {
                return false;
            }
        }
    }
    true
}

/// Vanilla `Raid.java:585` placement test, ravager placement plus the snow fallback.
fn is_placement_ok(world: &Arc<World>, pos: BlockPos) -> bool {
    if crate::world::natural_spawner::is_spawn_position_ok(world, &pos, &EntityType::RAVAGER) {
        return true;
    }
    // `level.getBlockState(pos.below()).is(Blocks.SNOW) && level.getBlockState(pos).isAir()`
    world.get_block(&pos.down()) == &pumpkin_data::Block::SNOW
        && world.get_block_state(&pos).is_air()
}

/// Vanilla `Raid.playSound` (`Raid.java:440-454`) — the raid horn.
///
/// Vanilla projects the sound onto a point 13 blocks from each player toward the
/// raid, so the horn appears to come from the raid's direction, and sends it to
/// every player within 64 blocks plus everyone already on the boss bar.
pub fn play_raid_horn(world: &Arc<World>, origin: BlockPos, bar_audience: &[Uuid]) {
    const DIST_AWAY: f64 = 13.0;
    const RANGE: f64 = 64.0;

    let raid_loc = origin.to_centered_f64();
    for player in world.players.load().iter() {
        let player_loc = player.position();
        let dx = raid_loc.x - player_loc.x;
        let dz = raid_loc.z - player_loc.z;
        let dist_between = dx.hypot(dz);
        let in_range = dist_between <= RANGE;
        let on_bar = bar_audience.contains(&player.gameprofile.id);
        if !in_range && !on_bar {
            continue;
        }
        // Vanilla divides by `distBtwn` unguarded; guard the exact-overlap case.
        let (x, z) = if dist_between > 0.0 {
            (
                player_loc.x + DIST_AWAY / dist_between * dx,
                player_loc.z + DIST_AWAY / dist_between * dz,
            )
        } else {
            (player_loc.x, player_loc.z)
        };
        let at = Vector3::new(x, player_loc.y, z);
        // Vanilla: volume 64.0, pitch 1.0, SoundSource.NEUTRAL.
        world.play_sound_fine(Sound::EventRaidHorn, SoundCategory::Neutral, &at, 64.0, 1.0);
    }
}

/// One raid's full tick — vanilla `Raid.tick` (`Raid.java:248-373`) end to end.
///
/// Phase 1 samples the world, phase 2 advances the state under the lock, phase 3
/// performs the async work the plan asked for. `Raids.tick` (`Raids.java:95`)
/// calls this for every live raid.
pub async fn tick_raid(world: &Arc<World>, raid: &Arc<Raid>) {
    // Phase 1 — sample the level, as vanilla's tick does inline.
    let facts = WorldFacts::sample(world, raid.center());

    // Phase 2 — advance every counter with the lock held, then drop it.
    let plan = raid.advance(&facts);

    // Phase 3 — the async follow-up.
    if !plan.remove_bossbar.is_empty() {
        super::registry::remove_bossbar_from(world, &plan.remove_bossbar, raid.bossbar_uuid).await;
    }

    // Raid.java:289-291 — resolve a spawn position for the upcoming wave. Vanilla
    // searches with maxTries = 8 from `getValidSpawnPos` (`Raid.java:380-386`).
    if plan.find_spawn_pos {
        let center = raid.center();
        let cooldown = raid.with(|inner| inner.state.raid_cooldown_ticks);
        let found = find_random_spawn_pos(world, center, cooldown, 8);
        raid.with(|inner| inner.wave_spawn_pos = found);
    }

    // Raid.java:303-305 — `updateRaiders` runs on the same once-a-second beat.
    if plan.refresh_players {
        refresh_bossbar_audience(world, raid).await;
        prune_raiders(world, raid).await;
    }

    // Raid.java:319-336 — spawn the scheduled waves, tracking failed attempts.
    if plan.waves_to_spawn > 0 {
        let mut sound_played = false;
        let mut attempts = 0;
        for _ in 0..plan.waves_to_spawn {
            let center = raid.center();
            let cooldown = raid.with(|inner| inner.state.raid_cooldown_ticks);
            // Raid.java:322 — the cached position first, else a 20-try search.
            let spawn_pos = raid
                .with(|inner| inner.wave_spawn_pos)
                .or_else(|| find_random_spawn_pos(world, center, cooldown, 20));

            if let Some(pos) = spawn_pos {
                if spawn_wave(world, raid, &world.raids.raiders, pos).await && !sound_played {
                    // Raid.java:326-329 — the horn plays once per tick.
                    play_raid_horn(world, pos, &raid.bossbar_players());
                    sound_played = true;
                }
            } else {
                attempts += 1;
            }
            // Raid.java:333-335.
            if attempts > NUM_SPAWN_ATTEMPTS {
                let players = raid.stop();
                super::registry::remove_bossbar_from(world, &players, raid.bossbar_uuid).await;
                break;
            }
        }
        // A wave changed the health denominator, so refresh the bar.
        push_health_progress(world, raid).await;
    }

    // Boss-bar pushes the plan asked for.
    let audience = raid.bossbar_players();
    if let Some(visible) = plan.visible
        && !visible
    {
        super::registry::remove_bossbar_from(world, &audience, raid.bossbar_uuid).await;
    }
    if let Some(title) = plan.title {
        let progress = raid.with(|inner| inner.last_progress);
        let bar = raid.make_bossbar(title, progress);
        for uuid in &audience {
            if let Some(player) = world.get_player_by_uuid(*uuid) {
                player.send_bossbar(&bar).await;
            }
        }
    } else if let Some(progress) = plan.progress {
        for uuid in &audience {
            if let Some(player) = world.get_player_by_uuid(*uuid) {
                player
                    .update_bossbar_health(&raid.bossbar_uuid, progress)
                    .await;
            }
        }
    }

    // Raid.java:341-353 — Hero of the Village on victory.
    if !plan.heroes.is_empty() {
        grant_heroes_of_the_village(world, &plan.heroes, plan.hero_amplifier).await;
    }
}

/// Vanilla `Raid.updatePlayers` (`Raid.java:203-214`): add players who entered the
/// raid, remove those who left.
async fn refresh_bossbar_audience(world: &Arc<World>, raid: &Arc<Raid>) {
    let current = super::registry::raid_bossbar_audience(world, raid);
    let (added, removed) = raid.with(|inner| {
        let added: Vec<Uuid> = current
            .iter()
            .filter(|uuid| !inner.bossbar_players.contains(uuid))
            .copied()
            .collect();
        let removed: Vec<Uuid> = inner
            .bossbar_players
            .iter()
            .filter(|uuid| !current.contains(uuid))
            .copied()
            .collect();
        inner.bossbar_players = current.clone();
        (added, removed)
    });

    if !added.is_empty() {
        let (title, progress) = raid.with(|inner| (inner.last_title, inner.last_progress));
        let bar = raid.make_bossbar(title, progress);
        for uuid in added {
            if let Some(player) = world.get_player_by_uuid(uuid) {
                player.send_bossbar(&bar).await;
            }
        }
    }
    super::registry::remove_bossbar_from(world, &removed, raid.bossbar_uuid).await;
}

/// Vanilla `Raid.updateRaiders` (`Raid.java:411-438`), against the live world.
async fn prune_raiders(world: &Arc<World>, raid: &Arc<Raid>) {
    let center = raid.center();
    let members = raid.all_raiders();
    if members.is_empty() {
        return;
    }

    let registry = &world.raids.raiders;
    let mut facts = Vec::with_capacity(members.len());
    for uuid in members {
        let membership = registry.get(uuid).unwrap_or_default();
        let entity = world.get_entity_by_uuid(uuid);
        let (gone, distance_sq, tick_count, health, in_village, no_action_time) = match &entity {
            // Raid.java:423 — absent from the level lookup counts as gone.
            None => (true, 0.0, 0, 0.0, false, 0),
            Some(entity) => {
                let base = entity.get_entity();
                let pos = base.block_pos.load();
                let health = entity
                    .get_living_entity()
                    .map_or(0.0, |living| living.health.load());
                // GAP: vanilla reads `raider.getNoActionTime()` (`Raid.java:426`),
                // which lives on `MobEntity` (`no_action_time`, backed by
                // `despawn_counter`). `EntityBase` exposes no generic accessor for
                // it — `get_living_entity` and `get_player` are the only downcasts
                // in the trait — and enumerating every raider concrete type here
                // would duplicate the `from_type` table. Reporting 0 means the
                // `no_action_time > MAX_NO_ACTION_TIME` arm never fires, so raiders
                // are never pruned for idling *outside* the village; they are still
                // pruned when removed, in the wrong dimension, or beyond
                // `RAID_REMOVAL_THRESHOLD_SQR` (`Raid.java:418`), which are the
                // paths that actually end stuck raids. Adding a `get_mob_entity`
                // accessor to `EntityBase` would close this.
                (
                    base.is_removed(),
                    pos.to_centered_f64()
                        .squared_distance_to_vec(&center.to_centered_f64()),
                    base.age.load(std::sync::atomic::Ordering::Relaxed),
                    health,
                    village::is_village(world, &pos),
                    0,
                )
            }
        };

        facts.push(RaiderFacts {
            uuid,
            wave: membership.wave,
            health,
            gone,
            distance_sq_to_center: distance_sq,
            tick_count,
            in_village,
            no_action_time,
            ticks_outside_raid: membership.ticks_outside_raid,
            is_patrol_leader: membership.patrol_leader,
        });
    }

    let outcome = update_raiders(center, &facts);
    for uuid in outcome.increment_outside {
        registry.update(uuid, |member| member.ticks_outside_raid += 1);
    }
    for wave in outcome.clear_leader_waves {
        raid.remove_leader(wave);
    }
    for (uuid, wave, health, remove_health) in outcome.drop {
        raid.remove_from_raid(uuid, wave, health, remove_health);
        registry.clear_raid(uuid);
    }
}

/// Vanilla `Raid.updateBossbar` (`Raid.java:510-512`) — recompute from live health.
async fn push_health_progress(world: &Arc<World>, raid: &Arc<Raid>) {
    let mut health = 0.0f32;
    for uuid in raid.all_raiders() {
        if let Some(entity) = world.get_entity_by_uuid(uuid)
            && let Some(living) = entity.get_living_entity()
        {
            health += living.health.load();
        }
    }
    let progress = raid.with(|inner| {
        let progress = inner.state.health_progress(health);
        inner.last_progress = progress;
        progress
    });
    for uuid in raid.bossbar_players() {
        if let Some(player) = world.get_player_by_uuid(uuid) {
            player
                .update_bossbar_health(&raid.bossbar_uuid, progress)
                .await;
        }
    }
}

/// Vanilla `Raid.spawnGroup` + `joinRaid` (`Raid.java:456-508`), executed against
/// the world.
///
/// Returns `true` when the wave was created. The caller owns the attempt budget
/// (`Raid.java:331-335`).
///
/// # Approximation
///
/// Vanilla calls `raider.applyRaidBuffs(level, groupNumber, false)`
/// (`Raid.java:503`), which is where wave-scaled enchanted crossbows, witch
/// potions and ravager buffs come from. Pumpkin's mob types do not implement
/// `applyRaidBuffs`, and inventing the buff tables would violate the parity rule,
/// so no buffs are applied. The raid's `enchant_odds` (`Raid.getEnchantOdds`) is
/// ported and exposed on [`Raid`] ready for whoever adds them.
pub async fn spawn_wave(
    world: &Arc<World>,
    raid: &Arc<Raid>,
    raiders: &RaiderRegistry,
    pos: BlockPos,
) -> bool {
    // Raid.java:458-461 — the wave number is the next group, and totalHealth resets.
    let (wave, num_groups, is_bonus, difficulty) = raid.with(|inner| {
        let raiders_alive = inner.total_raiders_alive();
        inner.state.total_health = 0.0;
        (
            inner.state.groups_spawned + 1,
            inner.state.num_groups,
            inner.state.should_spawn_bonus_group(raiders_alive),
            inner.state.status,
        )
    });
    let _ = difficulty;
    let world_difficulty = world.level_info.load().difficulty;

    let plan = plan_wave(wave, num_groups, is_bonus, world_difficulty, |bound| {
        if bound <= 1 {
            0
        } else {
            rng().random_range(0..bound)
        }
    });
    if plan.is_empty() {
        return false;
    }

    // Spawn in plan order so a rider's mount already exists.
    let mut spawned: Vec<Option<Arc<dyn EntityBase>>> = Vec::with_capacity(plan.len());
    for planned in &plan {
        let spawn_at = Vector3::new(
            f64::from(pos.0.x) + 0.5,
            f64::from(pos.0.y) + 1.0,
            f64::from(pos.0.z) + 0.5,
        );
        let entity =
            crate::entity::r#type::from_type(planned.entity_type, spawn_at, world, Uuid::new_v4());
        let uuid = entity.get_entity().entity_uuid;
        let health = entity
            .get_living_entity()
            .map_or(0.0, |living| living.health.load());

        // Raid.java:494-499 — joinRaid bookkeeping.
        raid.add_wave_mob(wave, uuid, health, true);
        raiders.update(uuid, |membership| {
            membership.raid_id = Some(raid.id);
            membership.wave = wave;
            membership.can_join_raid = true;
            membership.ticks_outside_raid = 0;
            if planned.is_leader {
                membership.patrol_leader = true;
                membership.patrolling = true;
            }
        });
        if planned.is_leader {
            raid.set_leader(wave, uuid);
        }

        world.spawn_entity(entity.clone()).await;
        spawned.push(Some(entity));
    }

    // Raid.java:483-484 — riders mount after both entities exist.
    for (index, planned) in plan.iter().enumerate() {
        let Some(mount_index) = planned.rides_index else {
            continue;
        };
        let (Some(Some(rider)), Some(Some(mount))) = (
            spawned.get(index).cloned(),
            spawned.get(mount_index).cloned(),
        ) else {
            continue;
        };
        mount.get_entity().add_passenger(mount.clone(), rider).await;
    }

    // Raid.java:487-490.
    raid.with(|inner| {
        inner.wave_spawn_pos = None;
        inner.state.groups_spawned += 1;
        inner.state.started = true;
    });
    true
}

/// Whether a block state permits a raider to stand here — used by the placement
/// fallback and re-exported so callers do not need the natural-spawner path.
#[must_use]
pub fn is_empty_spawn_block(
    state: &pumpkin_data::BlockState,
    entity_type: &'static EntityType,
) -> bool {
    is_valid_empty_spawn_block(state, entity_type)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_util::Difficulty;

    /// A draw that always returns 0 — vanilla's "unlucky" path, so every count is
    /// the table's default with no bonus.
    fn no_bonus(_bound: i32) -> i32 {
        0
    }

    /// A draw that always returns `bound - 1`, the maximum `nextInt` can produce.
    fn max_bonus(bound: i32) -> i32 {
        (bound - 1).max(0)
    }

    fn counts(plan: &[PlannedRaider], entity_type: &'static EntityType) -> usize {
        plan.iter()
            .filter(|raider| raider.entity_type == entity_type)
            .count()
    }

    #[test]
    fn wave_one_normal_is_four_pillagers() {
        let plan = plan_wave(1, 5, false, Difficulty::Normal, no_bonus);
        assert_eq!(plan.len(), 4);
        assert_eq!(counts(&plan, &EntityType::PILLAGER), 4);
    }

    #[test]
    fn exactly_one_leader_per_wave() {
        for wave in 1..=7 {
            let plan = plan_wave(wave, 7, false, Difficulty::Hard, no_bonus);
            if plan.is_empty() {
                continue;
            }
            let leaders = plan.iter().filter(|r| r.is_leader).count();
            assert_eq!(leaders, 1, "wave {wave} must have exactly one leader");
            // Vanilla sets the leader on the first created raider.
            assert!(plan[0].is_leader);
        }
    }

    #[test]
    fn riders_never_become_the_leader() {
        // Wave 5 normal spawns a ravager with a pillager rider.
        let plan = plan_wave(5, 5, false, Difficulty::Normal, no_bonus);
        for raider in &plan {
            if raider.rides_index.is_some() {
                assert!(!raider.is_leader);
            }
        }
    }

    #[test]
    fn wave_five_normal_pairs_a_pillager_onto_the_ravager() {
        let plan = plan_wave(5, 5, false, Difficulty::Normal, no_bonus);
        let ravager_index = plan
            .iter()
            .position(|r| r.entity_type == &EntityType::RAVAGER)
            .expect("wave 5 has a ravager");
        let rider = plan
            .iter()
            .find(|r| r.rides_index == Some(ravager_index))
            .expect("the ravager carries a rider");
        assert_eq!(rider.entity_type, &EntityType::PILLAGER);
    }

    #[test]
    fn wave_seven_hard_puts_an_evoker_on_the_first_ravager() {
        let plan = plan_wave(7, 7, false, Difficulty::Hard, no_bonus);
        let ravager_indices: Vec<usize> = plan
            .iter()
            .enumerate()
            .filter(|(_, r)| r.entity_type == &EntityType::RAVAGER)
            .map(|(i, _)| i)
            .collect();
        // Wave 7 ravager column is 2 (Raid.java:740).
        assert_eq!(ravager_indices.len(), 2);
        let first_rider = plan
            .iter()
            .find(|r| r.rides_index == Some(ravager_indices[0]))
            .expect("first ravager has a rider");
        assert_eq!(first_rider.entity_type, &EntityType::EVOKER);
        let second_rider = plan
            .iter()
            .find(|r| r.rides_index == Some(ravager_indices[1]))
            .expect("second ravager has a rider");
        assert_eq!(second_rider.entity_type, &EntityType::VINDICATOR);
    }

    #[test]
    fn creation_order_follows_raider_type_ordinals() {
        // Wave 5 hard: vindicator 4, evoker 1, pillager 4, witch 0, ravager 1.
        let plan = plan_wave(5, 7, false, Difficulty::Hard, no_bonus);
        // Ignore riders, which are interleaved after their mount.
        let order: Vec<&'static str> = plan
            .iter()
            .filter(|r| r.rides_index.is_none())
            .map(|r| r.entity_type.resource_name)
            .collect();
        let first_pillager = order.iter().position(|n| *n == "pillager").unwrap();
        let first_vindicator = order.iter().position(|n| *n == "vindicator").unwrap();
        let first_evoker = order.iter().position(|n| *n == "evoker").unwrap();
        assert!(first_vindicator < first_evoker);
        assert!(first_evoker < first_pillager);
    }

    #[test]
    fn wave_four_normal_brings_the_witches() {
        // Witch column at wave 4 is 3 (Raid.java:739), and the witch bonus is
        // suppressed at wave 4 (Raid.java:646).
        let plan = plan_wave(4, 5, false, Difficulty::Normal, max_bonus);
        assert_eq!(counts(&plan, &EntityType::WITCH), 3);
    }

    #[test]
    fn hard_mode_bonus_adds_up_to_two_vindicators_and_pillagers() {
        let base = plan_wave(3, 7, false, Difficulty::Hard, no_bonus);
        let boosted = plan_wave(3, 7, false, Difficulty::Hard, max_bonus);
        // Cap 2 -> nextInt(3) max 2 extra each for vindicator and pillager.
        assert_eq!(
            counts(&boosted, &EntityType::VINDICATOR),
            counts(&base, &EntityType::VINDICATOR) + 2
        );
        assert_eq!(
            counts(&boosted, &EntityType::PILLAGER),
            counts(&base, &EntityType::PILLAGER) + 2
        );
    }

    #[test]
    fn easy_bonus_is_a_nested_draw_capped_at_one() {
        let base = plan_wave(3, 3, false, Difficulty::Easy, no_bonus);
        let boosted = plan_wave(3, 3, false, Difficulty::Easy, max_bonus);
        // nextInt(2) -> 1, then nextInt(2) -> 1: exactly one extra.
        assert_eq!(
            counts(&boosted, &EntityType::PILLAGER),
            counts(&base, &EntityType::PILLAGER) + 1
        );
    }

    #[test]
    fn evoker_count_never_moves_with_the_bonus_draw() {
        for wave in 1..=7 {
            let base = plan_wave(wave, 7, false, Difficulty::Hard, no_bonus);
            let boosted = plan_wave(wave, 7, false, Difficulty::Hard, max_bonus);
            assert_eq!(
                counts(&base, &EntityType::EVOKER),
                counts(&boosted, &EntityType::EVOKER),
                "wave {wave} evoker count must be table-only"
            );
        }
    }

    #[test]
    fn bonus_wave_uses_the_final_column_and_can_add_a_ravager() {
        // Hard bonus wave: default columns are index 7, and the ravager bonus
        // applies only on a bonus wave (Raid.java:666).
        let plan = plan_wave(99, 7, true, Difficulty::Hard, max_bonus);
        // Wave-7 ravager column is 2, plus up to 1 bonus.
        assert_eq!(counts(&plan, &EntityType::RAVAGER), 3);
        // Wave-7 vindicator column is 5, plus up to 2 bonus.
        assert_eq!(counts(&plan, &EntityType::VINDICATOR), 7);
    }

    #[test]
    fn easy_bonus_wave_gets_no_extra_ravager() {
        let plan = plan_wave(99, 3, true, Difficulty::Easy, max_bonus);
        // Easy numGroups 3 -> ravager column index 3 is 1, and no bonus on Easy.
        assert_eq!(counts(&plan, &EntityType::RAVAGER), 1);
    }

    #[test]
    fn peaceful_style_empty_wave_plans_nothing() {
        // Wave 0 is the unused padding column: every type is 0.
        let plan = plan_wave(0, 5, false, Difficulty::Normal, no_bonus);
        assert!(plan.is_empty());
    }
}
