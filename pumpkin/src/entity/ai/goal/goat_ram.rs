use std::sync::Arc;

use super::{Controls, Goal, GoalFuture};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::predicate::EntityPredicate;
use crate::entity::{EntityBase, mob::Mob};
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};
use rand::RngExt;

/// Vanilla: `GoatAi.RAM_PREPARE_TIME` -- ticks the goat stares down its target before charging.
const RAM_PREPARE_TIME: i32 = 20;
/// Vanilla: `GoatAi.RAM_MAX_DISTANCE` -- detection range for a ram target.
const RAM_MAX_DISTANCE: f64 = 7.0;
/// Vanilla: `RamTarget.RAM_SPEED_FORCE_FACTOR` field name reused for the charge navigator speed.
/// The actual value (3.0) comes from `GoatAi.SPEED_MULTIPLIER_WHEN_RAMMING`.
const RAM_CHARGE_SPEED: f64 = 3.0;
/// Vanilla: `GoatAi.ADULT_RAM_KNOCKBACK_FORCE`. Pumpkin's `GoatEntity` does not yet implement
/// `AgeableMob`, so baby goats always use the adult knockback (`BABY_RAM_KNOCKBACK_FORCE` = 1.0
/// is not distinguished) -- documented simplification.
const RAM_KNOCKBACK_FORCE: f64 = 2.5;
/// Vanilla: `RamTarget.TIME_OUT_DURATION` -- abandon a charge that never connects.
const RAM_TIMEOUT_TICKS: i32 = 200;
/// Vanilla: `GoatAi.TIME_BETWEEN_RAMS` (`UniformInt.of(600, 6000)`).
const TIME_BETWEEN_RAMS: std::ops::Range<i32> = 600..6000;
/// Squared distance at which a charging goat is considered to have connected with its target.
/// Vanilla's `RamTarget` instead checks for any `LivingEntity` overlapping the goat's own
/// bounding box every tick; we approximate that with a fixed short-range check.
const RAM_HIT_RANGE_SQ: f64 = 2.25; // 1.5 * 1.5

enum RamPhase {
    /// Vanilla: `PrepareRamNearestTarget` -- the goat walks up to a ram-off position and stares
    /// at its target for `RAM_PREPARE_TIME` ticks before actually charging.
    Preparing {
        target: Arc<dyn EntityBase>,
        ticks_left: i32,
    },
    /// Vanilla: `RamTarget` -- the goat sprints at the target and knocks it back on contact.
    Charging {
        target: Arc<dyn EntityBase>,
        /// Normalized horizontal direction the goat is charging in, used for the knockback
        /// vector on impact (vanilla: `RamTarget.ramDirection`).
        direction: (f64, f64),
        timeout: i32,
    },
}

/// Makes a goat charge at, and knock back, a nearby living entity.
///
/// This is a simplified, single-`Goal` port of vanilla's Brain-based ram behavior, which is
/// normally split across `PrepareRamNearestTarget` and `RamTarget` (two `Behavior`s driven by
/// `MemoryModuleType.RAM_TARGET`/`RAM_COOLDOWN_TICKS`). Notable simplifications:
/// - No dedicated "run-up" position search (`PrepareRamNearestTarget.calculateRammingStartPosition`,
///   which walks up to 7 blocks away from the target along a walkable line); the goat just faces
///   its target in place before charging.
/// - No horn-breaking-block check (`RamTarget.hasRammedHornBreakingBlock`) or horn drop.
/// - No distinction between regular and "screaming" goats (different cooldains/sounds in vanilla).
/// - No baby vs. adult knockback distinction (see `RAM_KNOCKBACK_FORCE`).
/// - Armor stands / world border / `mobGriefing` nuances of `RAM_TARGET_CONDITIONS` are skipped;
///   any living entity other than another goat (subject to the standard creative/spectator
///   exclusion) is a valid target.
///
/// Vanilla source: `net/minecraft/world/entity/ai/behavior/{PrepareRamNearestTarget,RamTarget}.java`,
/// `net/minecraft/world/entity/animal/goat/GoatAi.java`.
pub struct GoatRamGoal {
    goal_control: Controls,
    phase: Option<RamPhase>,
    cooldown: i32,
}

impl GoatRamGoal {
    #[must_use]
    pub fn new() -> Self {
        Self {
            goal_control: Controls::MOVE | Controls::LOOK,
            phase: None,
            // Vanilla seeds RAM_COOLDOWN_TICKS with TIME_BETWEEN_RAMS.sample() at spawn
            // (GoatAi.initMemories), rather than letting the goat ram immediately.
            cooldown: 0,
        }
    }

    async fn find_ram_target(mob: &dyn Mob) -> Option<Arc<dyn EntityBase>> {
        let self_entity = mob.get_entity();
        let self_uuid = self_entity.entity_uuid;
        let pos = self_entity.pos.load();
        let world = self_entity.world.load();

        let mut best: Option<(Arc<dyn EntityBase>, f64)> = None;
        for (uuid, candidate) in world.get_nearby_entities(pos, RAM_MAX_DISTANCE) {
            if uuid == self_uuid {
                continue;
            }
            // Vanilla: RAM_TARGET_CONDITIONS excludes other goats.
            if candidate.get_entity().entity_type == &EntityType::GOAT {
                continue;
            }
            // Only living entities are ram-able.
            if candidate.get_living_entity().is_none() {
                continue;
            }
            if !candidate.get_entity().is_alive() {
                continue;
            }
            if !EntityPredicate::ExceptCreativeOrSpectator
                .test(candidate.get_entity())
                .await
            {
                continue;
            }

            let dist_sq = candidate
                .get_entity()
                .pos
                .load()
                .squared_distance_to_vec(&pos);
            if best
                .as_ref()
                .is_none_or(|(_, best_dist)| dist_sq < *best_dist)
            {
                best = Some((candidate, dist_sq));
            }
        }

        best.map(|(entity, _)| entity)
    }

    /// Ends the current ram attempt (hit, miss, or timeout) and rolls a fresh cooldown, mirroring
    /// vanilla's `RamTarget.finishRam` / `PrepareRamNearestTarget.stop` cooldown resets.
    fn finish(&mut self, mob: &dyn Mob) {
        self.phase = None;
        self.cooldown = mob.get_random().random_range(TIME_BETWEEN_RAMS);
    }

    /// Handles a tick while staring down the target before charging. Returns the phase to keep
    /// running (or `None` if the ram attempt finished/was aborted this tick).
    fn tick_preparing(
        &mut self,
        mob: &dyn Mob,
        target: Arc<dyn EntityBase>,
        ticks_left: i32,
    ) -> Option<RamPhase> {
        if !target.get_entity().is_alive() {
            self.finish(mob);
            return None;
        }

        mob.get_mob_entity()
            .look_control
            .lock()
            .unwrap()
            .look_at_entity_with_range(&target, 30.0, 30.0);

        let ticks_left = ticks_left - 1;
        if ticks_left > 0 {
            return Some(RamPhase::Preparing { target, ticks_left });
        }

        // Prepare time elapsed: start the charge.
        let mob_pos = mob.get_entity().pos.load();
        let target_pos = target.get_entity().pos.load();
        let dx = target_pos.x - mob_pos.x;
        let dz = target_pos.z - mob_pos.z;
        let len = dx.hypot(dz).max(1.0e-4);
        let direction = (dx / len, dz / len);

        mob.get_mob_entity()
            .navigator
            .lock()
            .unwrap()
            .set_progress(NavigatorGoal {
                current_progress: mob_pos,
                destination: target_pos,
                speed: RAM_CHARGE_SPEED,
            });

        let world = mob.get_entity().world.load();
        world.play_sound(
            Sound::EntityGoatPrepareRam,
            SoundCategory::Neutral,
            &mob_pos,
        );

        Some(RamPhase::Charging {
            target,
            direction,
            timeout: RAM_TIMEOUT_TICKS,
        })
    }

    /// Handles a tick while sprinting at the target. Returns the phase to keep running (or
    /// `None` if the ram connected, timed out, or the target died this tick).
    async fn tick_charging(
        &mut self,
        mob: &dyn Mob,
        target: Arc<dyn EntityBase>,
        direction: (f64, f64),
        timeout: i32,
    ) -> Option<RamPhase> {
        if !target.get_entity().is_alive() {
            self.finish(mob);
            return None;
        }

        let mob_pos = mob.get_entity().pos.load();
        let target_pos = target.get_entity().pos.load();
        let dist_sq = mob_pos.squared_distance_to_vec(&target_pos);

        if dist_sq <= RAM_HIT_RANGE_SQ {
            mob.try_attack(target.as_ref()).await;
            if let Some(living) = target.get_living_entity() {
                living.knockback_with_resistance(RAM_KNOCKBACK_FORCE, direction.0, direction.1);
            } else {
                target
                    .get_entity()
                    .knockback(RAM_KNOCKBACK_FORCE, direction.0, direction.1);
            }

            let world = mob.get_entity().world.load();
            world.play_sound(Sound::EntityGoatRamImpact, SoundCategory::Neutral, &mob_pos);
            self.finish(mob);
            return None;
        }

        let navigator_idle = mob
            .get_mob_entity()
            .navigator
            .try_lock()
            .is_ok_and(|navigator| navigator.is_idle());
        let timeout = timeout - 1;
        if navigator_idle || timeout <= 0 {
            self.finish(mob);
            return None;
        }

        Some(RamPhase::Charging {
            target,
            direction,
            timeout,
        })
    }
}

impl Default for GoatRamGoal {
    fn default() -> Self {
        Self::new()
    }
}

impl Goal for GoatRamGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            self.cooldown = (self.cooldown - 1).max(0);
            if self.cooldown > 0 {
                return false;
            }

            let Some(target) = Self::find_ram_target(mob).await else {
                return false;
            };

            self.phase = Some(RamPhase::Preparing {
                target,
                ticks_left: RAM_PREPARE_TIME,
            });
            true
        })
    }

    fn should_continue<'a>(&'a self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            match &self.phase {
                Some(RamPhase::Preparing { target, .. } | RamPhase::Charging { target, .. }) => {
                    target.get_entity().is_alive()
                }
                None => false,
            }
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            mob.get_mob_entity().navigator.lock().unwrap().stop();
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            // If we still hold a phase here, the goal was interrupted (target died, or a
            // higher-priority goal preempted us) rather than finishing naturally through
            // `finish`, so roll a cooldown now -- vanilla always resets RAM_COOLDOWN_TICKS
            // whenever the ram behavior stops without a successful `finishRam` call.
            if self.phase.take().is_some() {
                self.cooldown = mob.get_random().random_range(TIME_BETWEEN_RAMS);
            }
            mob.get_mob_entity().navigator.lock().unwrap().stop();
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(phase) = self.phase.take() else {
                return;
            };

            self.phase = match phase {
                RamPhase::Preparing { target, ticks_left } => {
                    self.tick_preparing(mob, target, ticks_left)
                }
                RamPhase::Charging {
                    target,
                    direction,
                    timeout,
                } => self.tick_charging(mob, target, direction, timeout).await,
            };
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
