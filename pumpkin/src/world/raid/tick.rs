//! The world-facing half of vanilla `Raid.tick` (`Raid.java:248-373`).
//!
//! Ground truth: `/root/Vanilla/src/net/minecraft/world/entity/raid/Raid.java`.
//!
//! # Shape
//!
//! Vanilla's `tick` freely mixes state mutation with level queries. Pumpkin cannot:
//! [`Raid`]'s interior sits behind a `std::sync::Mutex`, which must not be held
//! across an `await`. Each tick therefore runs in phases:
//!
//! 1. Read the world facts vanilla consults (`hasChunkAt`, `isVillage`, difficulty).
//! 2. Take the lock, advance every counter, and record a [`TickPlan`] of the async
//!    work that fell out. Drop the lock.
//! 3. Execute the plan: spawn waves, push boss-bar packets, grant Hero of the Village.
//!
//! The ordering inside step 2 follows `Raid.java:252-355` statement for statement.

use std::sync::Arc;

use pumpkin_data::effect::StatusEffect;
use pumpkin_data::potion::Effect;
use pumpkin_util::Difficulty;
use pumpkin_util::math::get_section_cord;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use uuid::Uuid;

use crate::entity::EntityBase;
use crate::entity::player::statistics::{CustomStatistic, StatisticCategory};
use crate::world::World;

use super::state::{DEFAULT_PRE_RAID_TICKS, MAX_NO_ACTION_TIME, OUTSIDE_RAID_BOUNDS_TIMEOUT};
use super::state::{
    HERO_OF_THE_VILLAGE_DURATION, LOW_MOB_THRESHOLD, MAX_CELEBRATION_TICKS, NUM_SPAWN_ATTEMPTS,
    POST_RAID_TICK_LIMIT, RAID_REMOVAL_THRESHOLD_SQR, RAID_TIMEOUT_TICKS,
    SECTION_RADIUS_FOR_FINDING_NEW_VILLAGE_CENTER,
};
use super::{BarTitle, Raid, RaidInner, RaidStatus, village};

/// Async follow-up work produced by one synchronous state advance.
#[derive(Default)]
pub struct TickPlan {
    /// Number of waves to spawn this tick (vanilla loops `while shouldSpawnGroup()`,
    /// `Raid.java:321-336`).
    pub waves_to_spawn: i32,
    /// Cached wave spawn position, when one was already resolved.
    pub wave_spawn_pos: Option<BlockPos>,
    /// Vanilla `updatePlayers` is due (`Raid.java:293`, `304`, `363`).
    pub refresh_players: bool,
    /// Boss-bar caption to push, when it changed.
    pub title: Option<BarTitle>,
    /// Boss-bar progress to push, when it changed.
    pub progress: Option<f32>,
    /// Boss-bar visibility to push, when it changed (`Raid.java:260`, `364`).
    pub visible: Option<bool>,
    /// Players whose boss bar must be torn down (`stop` → `removeAllPlayers`).
    pub remove_bossbar: Vec<Uuid>,
    /// Raiders to drop from the raid, with the health to deduct
    /// (`updateRaiders`, `Raid.java:433-437`).
    pub drop_raiders: Vec<(Uuid, i32, f32, bool)>,
    /// Victory just landed: grant Hero of the Village to these players
    /// (`Raid.java:341-353`).
    pub heroes: Vec<Uuid>,
    /// Amplifier for the Hero of the Village effect: `raidOmenLevel - 1`
    /// (`Raid.java:347`).
    pub hero_amplifier: i32,
    /// A new spawn position must be searched for (`Raid.java:289-291`).
    pub find_spawn_pos: bool,
}

/// Everything vanilla's `tick` reads from the level, sampled once up front.
pub struct WorldFacts {
    /// Vanilla `level.hasChunkAt(this.center)` (`Raid.java:254`).
    pub center_chunk_loaded: bool,
    /// Vanilla `level.getDifficulty()` (`Raid.java:255`).
    pub difficulty: Difficulty,
    /// Vanilla `level.isVillage(this.center)` (`Raid.java:265`).
    pub center_is_village: bool,
    /// Nearest village section centre within
    /// `SECTION_RADIUS_FOR_FINDING_NEW_VILLAGE_CENTER`, and whether it is a village —
    /// vanilla `moveRaidCenterToNearbyVillageSection` (`Raid.java:375-378`).
    pub relocated_center: Option<BlockPos>,
    /// Whether the relocated centre is itself a village (the second
    /// `isVillage` check at `Raid.java:268`).
    pub relocated_is_village: bool,
}

impl WorldFacts {
    /// Samples the level exactly where vanilla's `tick` would.
    #[must_use]
    pub fn sample(world: &Arc<World>, center: BlockPos) -> Self {
        let center_chunk_loaded = is_chunk_loaded(world, center);
        let difficulty = world.level_info.load().difficulty;
        let center_is_village = village::is_village(world, &center);

        // Vanilla only relocates when the centre stopped being a village.
        let relocated_center = if center_is_village {
            None
        } else {
            nearest_village_section_center(world, center)
        };
        let relocated_is_village = relocated_center
            .as_ref()
            .is_some_and(|pos| village::is_village(world, pos));

        Self {
            center_chunk_loaded,
            difficulty,
            center_is_village,
            relocated_center,
            relocated_is_village,
        }
    }
}

/// Whether the chunk containing `pos` is loaded — vanilla `Level.hasChunkAt`.
fn is_chunk_loaded(world: &Arc<World>, pos: BlockPos) -> bool {
    world
        .level
        .loaded_chunks
        .contains_key(&pos.chunk_position())
}

/// Whether `pos` sits in a chunk Pumpkin currently ticks entities in.
///
/// Vanilla's `isPositionEntityTicking` (`Raid.java:286`, `585`) asks the chunk map
/// whether the position is inside the entity-ticking (simulation) distance.
/// `World::active_chunks` is precisely that set — see `pumpkin/src/world/tick.rs`,
/// which gates entity ticking on it.
#[must_use]
pub fn is_position_entity_ticking(world: &Arc<World>, pos: BlockPos) -> bool {
    let chunk = Vector2::new(get_section_cord(pos.0.x), get_section_cord(pos.0.z));
    world.active_chunks.load().contains(&chunk)
}

/// Vanilla `Raid.moveRaidCenterToNearbyVillageSection` (`Raid.java:375-378`).
///
/// Scans the 5×5×5 cube of sections around the centre
/// (`SectionPos.cube(SectionPos.of(center), 2)`), keeps those that are villages,
/// and picks the section centre closest to the current raid centre.
fn nearest_village_section_center(world: &Arc<World>, center: BlockPos) -> Option<BlockPos> {
    let radius = SECTION_RADIUS_FOR_FINDING_NEW_VILLAGE_CENTER;
    let (sx, sy, sz) = (center.0.x >> 4, center.0.y >> 4, center.0.z >> 4);
    let mut best: Option<(BlockPos, i32)> = None;

    for dx in -radius..=radius {
        for dy in -radius..=radius {
            for dz in -radius..=radius {
                // Vanilla `SectionPos.center()`: section origin + 8 on each axis.
                let candidate = BlockPos::new(
                    ((sx + dx) << 4) + 8,
                    ((sy + dy) << 4) + 8,
                    ((sz + dz) << 4) + 8,
                );
                if !village::is_village(world, &candidate) {
                    continue;
                }
                let distance = candidate.squared_distance(&center);
                if best.is_none_or(|(_, best_distance)| distance < best_distance) {
                    best = Some((candidate, distance));
                }
            }
        }
    }

    best.map(|(pos, _)| pos)
}

/// Per-raider facts `updateRaiders` needs (`Raid.java:411-438`).
pub struct RaiderFacts {
    /// Entity `Uuid`.
    pub uuid: Uuid,
    /// The wave the raider belongs to.
    pub wave: i32,
    /// Current health, for the `totalHealth` deduction.
    pub health: f32,
    /// Vanilla `raider.isRemoved()` or a dimension mismatch, or missing from the
    /// world lookup (`Raid.java:418`, `423`).
    pub gone: bool,
    /// Squared distance from the raid centre (`Raid.java:418`).
    pub distance_sq_to_center: f64,
    /// Vanilla `raider.tickCount` (`Raid.java:422`).
    pub tick_count: i32,
    /// Vanilla `level.isVillage(raiderPos)` (`Raid.java:426`).
    pub in_village: bool,
    /// Vanilla `raider.getNoActionTime()` (`Raid.java:426`).
    pub no_action_time: i32,
    /// Vanilla `raider.getTicksOutsideRaid()` (`Raid.java:427-429`).
    pub ticks_outside_raid: i32,
    /// Whether the raider is the wave's patrol leader (`Raid.java:435`).
    pub is_patrol_leader: bool,
}

/// Outcome of the `updateRaiders` pass: who leaves, and whose leader slot clears.
pub struct UpdateRaidersOutcome {
    /// `(uuid, wave, health, remove_from_total_health)`.
    pub drop: Vec<(Uuid, i32, f32, bool)>,
    /// Waves whose leader slot must be cleared (`Raid.java:435-437`).
    pub clear_leader_waves: Vec<i32>,
    /// Raiders whose `ticksOutsideRaid` counter must be incremented
    /// (`Raid.java:426-428`).
    pub increment_outside: Vec<Uuid>,
}

/// Vanilla `Raid.updateRaiders` (`Raid.java:411-438`), as a pure reduction.
///
/// Vanilla mutates `ticksOutsideRaid` in place mid-loop; here the increments are
/// returned so the caller applies them to the membership table. The increment and
/// the `>= 30` test therefore both use the pre-increment value, exactly as vanilla
/// does — vanilla increments the field and then compares the *getter*, which by then
/// returns the incremented value, so the returned list carries the same +1 and the
/// comparison below is made against the updated number.
#[must_use]
pub fn update_raiders(center: BlockPos, raiders: &[RaiderFacts]) -> UpdateRaidersOutcome {
    let mut outcome = UpdateRaidersOutcome {
        drop: Vec::new(),
        clear_leader_waves: Vec::new(),
        increment_outside: Vec::new(),
    };

    for raider in raiders {
        // Raid.java:418 — removed, wrong dimension, or beyond 112 blocks.
        if raider.gone || raider.distance_sq_to_center >= RAID_REMOVAL_THRESHOLD_SQR {
            push_drop(&mut outcome, raider);
            continue;
        }
        // Raid.java:422 — the rest of the checks only apply after 600 ticks.
        if raider.tick_count <= 600 {
            continue;
        }

        // Raid.java:426-428.
        let mut ticks_outside = raider.ticks_outside_raid;
        if !raider.in_village && raider.no_action_time > MAX_NO_ACTION_TIME {
            ticks_outside += 1;
            outcome.increment_outside.push(raider.uuid);
        }
        // Raid.java:429-430.
        if ticks_outside >= OUTSIDE_RAID_BOUNDS_TIMEOUT {
            push_drop(&mut outcome, raider);
        }
    }

    let _ = center;
    outcome
}

fn push_drop(outcome: &mut UpdateRaidersOutcome, raider: &RaiderFacts) {
    // Raid.java:434 — updateRaiders always removes the health.
    outcome
        .drop
        .push((raider.uuid, raider.wave, raider.health, true));
    if raider.is_patrol_leader {
        outcome.clear_leader_waves.push(raider.wave);
    }
}

impl Raid {
    /// Vanilla `Raid.tick` (`Raid.java:248-373`) — the synchronous state advance.
    ///
    /// Returns the async work the caller must then perform. `raiders_alive` is read
    /// once at the top, matching vanilla's single `int raidersAlive` read
    /// (`Raid.java:280`) which the rest of the method reuses even after spawning.
    pub fn advance(&self, facts: &WorldFacts) -> TickPlan {
        self.with(|inner| Self::advance_locked(inner, facts))
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one-to-one port of Raid.tick; splitting it would obscure the vanilla ordering"
    )]
    fn advance_locked(inner: &mut RaidInner, facts: &WorldFacts) -> TickPlan {
        let mut plan = TickPlan::default();

        // Raid.java:249-251.
        if inner.state.is_stopped() {
            return plan;
        }

        if inner.state.status == RaidStatus::Ongoing {
            // Raid.java:252-264.
            let was_active = inner.state.active;
            inner.state.active = facts.center_chunk_loaded;

            if facts.difficulty == Difficulty::Peaceful {
                stop_into(inner, &mut plan);
                return plan;
            }
            if was_active != inner.state.active {
                inner.visible = inner.state.active;
                plan.visible = Some(inner.state.active);
            }
            if !inner.state.active {
                return plan;
            }

            // Raid.java:265-274 — relocate, then give up if still not a village.
            let mut is_village = facts.center_is_village;
            if !is_village {
                if let Some(new_center) = facts.relocated_center {
                    inner.set_center(new_center);
                    is_village = facts.relocated_is_village;
                }
            }
            if !is_village {
                if inner.state.groups_spawned > 0 {
                    inner.state.status = RaidStatus::Loss;
                } else {
                    stop_into(inner, &mut plan);
                    return plan;
                }
            }

            // Raid.java:275-279.
            inner.state.ticks_active += 1;
            if inner.state.ticks_active >= RAID_TIMEOUT_TICKS {
                stop_into(inner, &mut plan);
                return plan;
            }

            // Raid.java:280 — read once, reused below even after a wave spawns.
            let raiders_alive = inner.total_raiders_alive();

            // Raid.java:281-302 — the between-wave countdown.
            if raiders_alive == 0 && inner.state.has_more_waves() {
                if inner.state.raid_cooldown_ticks > 0 {
                    // Raid.java:284-291.
                    let has_cached = inner.wave_spawn_pos.is_some();
                    plan.find_spawn_pos = !has_cached && inner.state.raid_cooldown_ticks % 5 == 0;

                    if inner.state.raid_cooldown_ticks == DEFAULT_PRE_RAID_TICKS
                        || inner.state.raid_cooldown_ticks % 20 == 0
                    {
                        plan.refresh_players = true;
                    }
                    inner.state.raid_cooldown_ticks -= 1;
                    set_progress(inner, &mut plan, inner.state.cooldown_progress());
                } else if inner.state.raid_cooldown_ticks == 0 && inner.state.groups_spawned > 0 {
                    // Raid.java:297-301 — restart the countdown and bail for this tick.
                    inner.state.raid_cooldown_ticks = DEFAULT_PRE_RAID_TICKS;
                    set_title(inner, &mut plan, BarTitle::Raid);
                    return plan;
                }
            }

            // Raid.java:303-315 — the once-a-second refresh.
            if inner.state.ticks_active % 20 == 0 {
                plan.refresh_players = true;
                let title = if raiders_alive > 0 && raiders_alive <= LOW_MOB_THRESHOLD {
                    BarTitle::RaidersRemaining(raiders_alive)
                } else {
                    BarTitle::Raid
                };
                set_title(inner, &mut plan, title);
            }

            // Raid.java:319-336 — spawn as many groups as the state allows. The
            // attempt/`stop` budget lives with the caller, which is the side that
            // learns whether a spawn position could be found.
            let mut waves = 0;
            let mut probe = inner.state.clone();
            while probe.should_spawn_group(raiders_alive) && waves <= NUM_SPAWN_ATTEMPTS {
                probe.groups_spawned += 1;
                waves += 1;
            }
            plan.waves_to_spawn = waves;
            plan.wave_spawn_pos = inner.wave_spawn_pos;

            // Raid.java:337-354 — post-raid grace, then victory.
            if inner.state.started && !inner.state.has_more_waves() && raiders_alive == 0 {
                if inner.state.post_raid_ticks < POST_RAID_TICK_LIMIT {
                    inner.state.post_raid_ticks += 1;
                } else {
                    inner.state.status = RaidStatus::Victory;
                    plan.heroes = inner.heroes_of_the_village.iter().copied().collect();
                    plan.hero_amplifier = inner.state.raid_omen_level - 1;
                }
            }
        } else if inner.state.is_over() {
            // Raid.java:356-371 — the celebration window.
            inner.state.celebration_ticks += 1;
            if inner.state.celebration_ticks >= MAX_CELEBRATION_TICKS {
                stop_into(inner, &mut plan);
                return plan;
            }
            if inner.state.celebration_ticks % 20 == 0 {
                plan.refresh_players = true;
                if !inner.visible {
                    inner.visible = true;
                    plan.visible = Some(true);
                }
                if inner.state.is_victory() {
                    set_progress(inner, &mut plan, 0.0);
                    set_title(inner, &mut plan, BarTitle::Victory);
                } else {
                    set_title(inner, &mut plan, BarTitle::Defeat);
                }
            }
        }

        plan
    }
}

/// Vanilla `stop()` (`Raid.java:242-246`) plus the boss-bar teardown the caller
/// must perform.
fn stop_into(inner: &mut RaidInner, plan: &mut TickPlan) {
    inner.state.stop();
    plan.remove_bossbar = std::mem::take(&mut inner.bossbar_players);
}

fn set_title(inner: &mut RaidInner, plan: &mut TickPlan, title: BarTitle) {
    if inner.last_title != title {
        inner.last_title = title;
        plan.title = Some(title);
    }
}

fn set_progress(inner: &mut RaidInner, plan: &mut TickPlan, progress: f32) {
    if (inner.last_progress - progress).abs() > f32::EPSILON {
        inner.last_progress = progress;
        plan.progress = Some(progress);
    }
}

/// Vanilla `Raid.HERO_OF_THE_VILLAGE` grant (`Raid.java:341-353`).
///
/// Vanilla resolves each stored UUID through `level.getEntity`, skips spectators,
/// and applies the effect with `duration = 48000`, `amplifier = raidOmenLevel - 1`,
/// `ambient = false`, `showParticles = false`, `showIcon = true`.
pub async fn grant_heroes_of_the_village(world: &Arc<World>, heroes: &[Uuid], amplifier: i32) {
    for hero in heroes {
        let Some(entity) = world.get_entity_by_uuid(*hero) else {
            continue;
        };
        if entity.is_spectator() {
            continue;
        }
        let Some(living) = entity.get_living_entity() else {
            continue;
        };
        let amplifier = u8::try_from(amplifier.max(0)).unwrap_or(u8::MAX);
        living
            .add_effect(Effect {
                effect_type: &StatusEffect::HERO_OF_THE_VILLAGE,
                duration: HERO_OF_THE_VILLAGE_DURATION,
                amplifier,
                ambient: false,
                show_particles: false,
                show_icon: true,
                blend: false,
            })
            .await;

        // Raid.java:349-351 — players also get the RAID_WIN stat.
        if let Some(player) = world.get_player_by_uuid(*hero) {
            player.stats.lock().await.increment(
                StatisticCategory::Custom,
                CustomStatistic::RaidWin as i32,
                1,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::raid::state::RaidState;

    fn facts() -> WorldFacts {
        WorldFacts {
            center_chunk_loaded: true,
            difficulty: Difficulty::Normal,
            center_is_village: true,
            relocated_center: None,
            relocated_is_village: false,
        }
    }

    fn raid() -> Raid {
        Raid::new(1, BlockPos::new(0, 64, 0), Difficulty::Normal)
    }

    #[test]
    fn peaceful_difficulty_stops_the_raid() {
        let raid = raid();
        let mut facts = facts();
        facts.difficulty = Difficulty::Peaceful;
        raid.advance(&facts);
        assert!(raid.is_stopped());
    }

    #[test]
    fn unloaded_center_only_hides_the_bar() {
        let raid = raid();
        let mut facts = facts();
        facts.center_chunk_loaded = false;
        let plan = raid.advance(&facts);
        assert_eq!(plan.visible, Some(false));
        assert!(!raid.is_active());
        assert!(
            !raid.is_stopped(),
            "an unloaded raid waits, it does not end"
        );
    }

    #[test]
    fn losing_the_village_before_any_wave_stops_the_raid() {
        let raid = raid();
        let mut facts = facts();
        facts.center_is_village = false;
        raid.advance(&facts);
        assert!(raid.is_stopped());
        assert!(!raid.is_loss());
    }

    #[test]
    fn losing_the_village_mid_raid_is_a_defeat() {
        let raid = raid();
        raid.with(|inner| {
            inner.state.groups_spawned = 2;
            inner.state.started = true;
        });
        let mut facts = facts();
        facts.center_is_village = false;
        raid.advance(&facts);
        assert!(raid.is_loss());
    }

    #[test]
    fn relocating_to_a_nearby_village_section_moves_the_center() {
        let raid = raid();
        raid.with(|inner| inner.state.groups_spawned = 1);
        let mut facts = facts();
        facts.center_is_village = false;
        facts.relocated_center = Some(BlockPos::new(24, 72, 24));
        facts.relocated_is_village = true;
        raid.advance(&facts);
        assert_eq!(raid.center(), BlockPos::new(24, 72, 24));
        assert!(!raid.is_loss(), "a valid relocation keeps the raid alive");
    }

    #[test]
    fn cooldown_counts_down_and_drives_the_bar() {
        let raid = raid();
        let plan = raid.advance(&facts());
        assert_eq!(raid.with(|i| i.state.raid_cooldown_ticks), 299);
        // 300 -> the first tick both refreshes players and moves progress.
        assert!(plan.refresh_players);
        assert!(plan.progress.is_some());
    }

    #[test]
    fn timeout_stops_the_raid() {
        let raid = raid();
        raid.with(|inner| inner.state.ticks_active = RAID_TIMEOUT_TICKS - 1);
        raid.advance(&facts());
        assert!(raid.is_stopped());
    }

    #[test]
    fn wave_spawns_once_the_cooldown_expires() {
        let raid = raid();
        raid.with(|inner| inner.state.raid_cooldown_ticks = 0);
        let plan = raid.advance(&facts());
        assert_eq!(plan.waves_to_spawn, 1);
    }

    #[test]
    fn between_wave_cooldown_restarts_after_a_cleared_wave() {
        let raid = raid();
        raid.with(|inner| {
            inner.state.started = true;
            inner.state.groups_spawned = 1;
            inner.state.raid_cooldown_ticks = 0;
        });
        let plan = raid.advance(&facts());
        assert_eq!(
            raid.with(|i| i.state.raid_cooldown_ticks),
            DEFAULT_PRE_RAID_TICKS
        );
        assert_eq!(plan.waves_to_spawn, 0, "the tick returns before spawning");
    }

    #[test]
    fn post_raid_grace_precedes_victory() {
        let raid = raid();
        raid.with(|inner| {
            inner.state.started = true;
            inner.state.groups_spawned = 5;
            inner.state.raid_cooldown_ticks = 0;
            inner.heroes_of_the_village.insert(Uuid::from_u128(7));
        });
        // 40 ticks of grace (Raid.java:338).
        for _ in 0..POST_RAID_TICK_LIMIT {
            let plan = raid.advance(&facts());
            assert!(plan.heroes.is_empty());
        }
        let plan = raid.advance(&facts());
        assert!(raid.with(|i| i.state.is_victory()));
        assert_eq!(plan.heroes, vec![Uuid::from_u128(7)]);
    }

    #[test]
    fn hero_amplifier_is_omen_level_minus_one() {
        let raid = raid();
        raid.with(|inner| {
            inner.state.started = true;
            inner.state.groups_spawned = 5;
            inner.state.raid_cooldown_ticks = 0;
            inner.state.raid_omen_level = 1;
            inner.state.post_raid_ticks = POST_RAID_TICK_LIMIT;
            inner.heroes_of_the_village.insert(Uuid::from_u128(7));
        });
        let plan = raid.advance(&facts());
        assert_eq!(plan.hero_amplifier, 0);
    }

    #[test]
    fn celebration_expires_into_stopped() {
        let raid = raid();
        raid.with(|inner| {
            inner.state.status = RaidStatus::Victory;
            inner.state.celebration_ticks = MAX_CELEBRATION_TICKS - 1;
        });
        raid.advance(&facts());
        assert!(raid.is_stopped());
    }

    #[test]
    fn victory_celebration_shows_the_victory_bar() {
        let raid = raid();
        raid.with(|inner| {
            inner.state.status = RaidStatus::Victory;
            inner.state.celebration_ticks = 19;
            inner.last_progress = 1.0;
        });
        let plan = raid.advance(&facts());
        assert_eq!(plan.title, Some(BarTitle::Victory));
        assert_eq!(plan.progress, Some(0.0));
    }

    #[test]
    fn loss_celebration_shows_the_defeat_bar() {
        let raid = raid();
        raid.with(|inner| {
            inner.state.status = RaidStatus::Loss;
            inner.state.celebration_ticks = 19;
        });
        let plan = raid.advance(&facts());
        assert_eq!(plan.title, Some(BarTitle::Defeat));
    }

    #[test]
    fn stopped_raid_does_nothing() {
        let raid = raid();
        raid.stop();
        let before = raid.with(|i| i.state.clone());
        raid.advance(&facts());
        let after = raid.with(|i| i.state.clone());
        assert_eq!(before.ticks_active, after.ticks_active);
    }

    // ── updateRaiders ────────────────────────────────────────────────────────

    fn healthy(uuid: u128) -> RaiderFacts {
        RaiderFacts {
            uuid: Uuid::from_u128(uuid),
            wave: 1,
            health: 24.0,
            gone: false,
            distance_sq_to_center: 16.0,
            tick_count: 1000,
            in_village: true,
            no_action_time: 0,
            ticks_outside_raid: 0,
            is_patrol_leader: false,
        }
    }

    #[test]
    fn healthy_raider_is_kept() {
        let outcome = update_raiders(BlockPos::new(0, 64, 0), &[healthy(1)]);
        assert!(outcome.drop.is_empty());
        assert!(outcome.increment_outside.is_empty());
    }

    #[test]
    fn removed_raider_is_dropped_with_its_health() {
        let mut raider = healthy(1);
        raider.gone = true;
        let outcome = update_raiders(BlockPos::new(0, 64, 0), &[raider]);
        assert_eq!(outcome.drop.len(), 1);
        let (uuid, wave, health, remove_health) = outcome.drop[0];
        assert_eq!(uuid, Uuid::from_u128(1));
        assert_eq!(wave, 1);
        assert!((health - 24.0).abs() < f32::EPSILON);
        assert!(remove_health);
    }

    #[test]
    fn raider_beyond_the_removal_radius_is_dropped() {
        let mut raider = healthy(1);
        raider.distance_sq_to_center = RAID_REMOVAL_THRESHOLD_SQR;
        let outcome = update_raiders(BlockPos::new(0, 64, 0), &[raider]);
        assert_eq!(outcome.drop.len(), 1);
    }

    #[test]
    fn young_raider_skips_the_wander_checks() {
        let mut raider = healthy(1);
        raider.tick_count = 600;
        raider.in_village = false;
        raider.no_action_time = MAX_NO_ACTION_TIME + 1;
        raider.ticks_outside_raid = OUTSIDE_RAID_BOUNDS_TIMEOUT;
        let outcome = update_raiders(BlockPos::new(0, 64, 0), &[raider]);
        assert!(outcome.drop.is_empty());
        assert!(outcome.increment_outside.is_empty());
    }

    #[test]
    fn idle_raider_outside_the_village_accrues_outside_ticks() {
        let mut raider = healthy(1);
        raider.in_village = false;
        raider.no_action_time = MAX_NO_ACTION_TIME + 1;
        let outcome = update_raiders(BlockPos::new(0, 64, 0), &[raider]);
        assert_eq!(outcome.increment_outside, vec![Uuid::from_u128(1)]);
        assert!(outcome.drop.is_empty());
    }

    #[test]
    fn busy_raider_outside_the_village_does_not_accrue() {
        let mut raider = healthy(1);
        raider.in_village = false;
        raider.no_action_time = MAX_NO_ACTION_TIME;
        let outcome = update_raiders(BlockPos::new(0, 64, 0), &[raider]);
        assert!(outcome.increment_outside.is_empty());
    }

    #[test]
    fn raider_outside_too_long_is_dropped() {
        let mut raider = healthy(1);
        raider.in_village = false;
        raider.no_action_time = MAX_NO_ACTION_TIME + 1;
        raider.ticks_outside_raid = OUTSIDE_RAID_BOUNDS_TIMEOUT - 1;
        let outcome = update_raiders(BlockPos::new(0, 64, 0), &[raider]);
        assert_eq!(outcome.drop.len(), 1);
    }

    #[test]
    fn dropping_a_patrol_leader_clears_its_wave_slot() {
        let mut raider = healthy(1);
        raider.gone = true;
        raider.is_patrol_leader = true;
        raider.wave = 3;
        let outcome = update_raiders(BlockPos::new(0, 64, 0), &[raider]);
        assert_eq!(outcome.clear_leader_waves, vec![3]);
    }

    #[test]
    fn state_clone_probe_does_not_mutate_the_raid() {
        // `advance` probes wave spawning on a clone; the real counter must only
        // move when the caller reports a successful spawn.
        let raid = raid();
        raid.with(|inner| inner.state.raid_cooldown_ticks = 0);
        let plan = raid.advance(&facts());
        assert_eq!(plan.waves_to_spawn, 1);
        assert_eq!(raid.groups_spawned(), 0);
    }

    #[test]
    fn raid_state_is_cloneable_for_the_probe() {
        let state = RaidState::new(Difficulty::Hard);
        let clone = state.clone();
        assert_eq!(clone.num_groups, state.num_groups);
    }
}
