use super::{Controls, Goal};
use crate::entity::EntityBase;
use crate::entity::ai::goal::GoalFuture;
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

pub struct MeleeAttackGoal {
    goal_control: Controls,
    speed: f64,
    pause_when_mob_idle: bool,
    #[expect(dead_code)]
    target_location: Vector3<f64>,
    update_countdown_ticks: i32,
    pub cooldown: i32,
    #[expect(dead_code)]
    attack_interval_ticks: i32,
    last_target_position: Option<Vector3<f64>>,
    /// Vanilla `lastCanUseCheck` — throttle pathfinding in `canUse` to every 20 ticks.
    last_can_use_check: i64,
}

impl MeleeAttackGoal {
    #[must_use]
    pub fn new(speed: f64, pause_when_mob_idle: bool) -> Self {
        Self {
            goal_control: Controls::MOVE | Controls::LOOK,
            // Speed *modifier* (vanilla navigation speed), not absolute blocks/tick.
            speed: speed.max(0.01),
            pause_when_mob_idle,
            target_location: Vector3::new(0.0, 0.0, 0.0),
            update_countdown_ticks: 0,
            cooldown: 0,
            attack_interval_ticks: 20,
            last_target_position: None,
            last_can_use_check: i64::MIN,
        }
    }

    #[must_use]
    pub fn get_max_cooldown(&self) -> i32 {
        self.get_tick_count(20)
    }

    /// Vanilla-compatible: living health/death, not merely Entity::is_alive (removal).
    fn target_is_valid(target: &dyn EntityBase) -> bool {
        if let Some(living) = target.get_living_entity() {
            return living.is_alive();
        }
        target.get_entity().is_alive()
    }

    /// Vanilla `Navigation.moveTo(Entity)` uses the living target's feet position
    /// (not a snapped block center). Snapping to block centers made A* prefer
    /// side/back approaches before charging.
    fn path_destination(target: &dyn EntityBase) -> Vector3<f64> {
        target.get_entity().pos.load()
    }

    /// Prefer dry ground near the target. Iron golems (and any mob with water
    /// malus < 0) must not path into water when a zombie is knocked into a pond.
    ///
    /// `avoid_water` must be computed by the caller **without** holding
    /// `navigator`'s mutex — `start()` already locks the navigator.
    fn path_destination_for(
        mob: &dyn Mob,
        target: &dyn EntityBase,
        avoid_water: bool,
    ) -> Vector3<f64> {
        let default = Self::path_destination(target);
        if !avoid_water {
            return default;
        }

        let world = mob.get_entity().world.load();
        let pos = target.get_entity().pos.load();
        let feet = pos.to_block_pos();

        let is_water_at = |p: pumpkin_util::math::position::BlockPos| {
            use pumpkin_data::tag::Taggable;
            let state_id = world.get_block_state_id(&p);
            pumpkin_data::fluid::Fluid::from_state_id(state_id)
                .is_some_and(|f| f.has_tag(&pumpkin_data::tag::Fluid::MINECRAFT_WATER))
        };

        // Target on dry land → normal destination.
        if !is_water_at(feet) && !is_water_at(feet.down()) {
            return default;
        }

        // Search for nearest solid bank within 8 blocks (spiral).
        let origin = feet.0;
        for r in 1i32..=8 {
            for dx in -r..=r {
                for dz in -r..=r {
                    if dx.unsigned_abs() != r as u32 && dz.unsigned_abs() != r as u32 {
                        continue;
                    }
                    for dy in -2i32..=2 {
                        let p = pumpkin_util::math::position::BlockPos::new(
                            origin.x + dx,
                            origin.y + dy,
                            origin.z + dz,
                        );
                        if is_water_at(p) {
                            continue;
                        }
                        let below = p.down();
                        let below_state = world.get_block_state(&below);
                        let feet_state = world.get_block_state(&p);
                        if below_state.is_solid() && !feet_state.is_solid() && !is_water_at(p) {
                            return Vector3::new(
                                f64::from(p.0.x) + 0.5,
                                f64::from(p.0.y),
                                f64::from(p.0.z) + 0.5,
                            );
                        }
                    }
                }
            }
        }

        // No bank found — keep current position so we don't march into the water.
        let me = mob.get_entity().pos.load();
        Vector3::new(me.x, me.y, me.z)
    }

    fn mob_avoids_water(mob: &dyn Mob) -> bool {
        let is_golem =
            mob.get_entity().entity_type.id == pumpkin_data::entity::EntityType::IRON_GOLEM.id;
        if is_golem {
            return true;
        }
        mob.get_mob_entity()
            .navigator
            .lock()
            .unwrap()
            .avoids_water()
    }
}

impl Goal for MeleeAttackGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            // Vanilla MeleeAttackGoal.canUse (26.2):
            // - throttle canUse checks to every 20 game ticks
            // - require createPath(target) != null OR isWithinMeleeAttackRange
            // Without the path check, MeleeAttack always steals MOVE and blocks
            // MoveTowardsTargetGoal (iron golem 0.9 approach) when A* fails.
            let age = i64::from(
                mob.get_entity()
                    .age
                    .load(std::sync::atomic::Ordering::Relaxed),
            );
            if self.last_can_use_check != i64::MIN && age.wrapping_sub(self.last_can_use_check) < 20
            {
                return false;
            }
            self.last_can_use_check = age;

            let target = {
                let guard = mob.get_mob_entity().target.lock().await;
                guard.clone()
            };
            let Some(target) = target else {
                return false;
            };
            if !Self::target_is_valid(target.as_ref()) {
                return false;
            }
            if target
                .get_player()
                .is_some_and(|p| p.is_spectator() || p.is_creative())
            {
                return false;
            }

            // In melee range → can start without a full path (vanilla).
            if mob
                .get_mob_entity()
                .is_in_attack_range(target.as_ref())
                .await
            {
                return true;
            }

            // Must be able to path to the target (or a dry bank if golem).
            let avoid_water = Self::mob_avoids_water(mob);
            let dest = Self::path_destination_for(mob, target.as_ref(), avoid_water);
            // Path destination stuck on self → cannot path (e.g. no bank for water target).
            let me = mob.get_entity().pos.load();
            if me.squared_distance_to_vec(&dest) < 0.25 {
                return false;
            }
            // `std::sync::MutexGuard` is !Send — cannot hold it across `.await`.
            // Run the path probe on the current worker without requiring Send.
            let living = &mob.get_mob_entity().living_entity;
            let navigator = &mob.get_mob_entity().navigator;
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let mut nav = navigator.lock().unwrap();
                    nav.create_path_to(living, dest).await.is_some()
                })
            })
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let target = mob.get_mob_entity().target.lock().await.clone();

            let Some(target) = target else {
                return false;
            };
            // Critical: drop chase the moment the target dies (death animation still
            // has Entity::is_alive()==true until remove after 20 ticks).
            if !Self::target_is_valid(target.as_ref()) {
                return false;
            }

            let is_valid_target = !target
                .get_player()
                .is_some_and(|p| p.is_spectator() || p.is_creative());

            if !is_valid_target {
                return false;
            }

            if self.pause_when_mob_idle {
                return mob
                    .get_mob_entity()
                    .is_in_position_target_range_pos(&target.get_entity().block_pos.load());
            }

            true
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            // Vanilla setAggressive(true) — illager arms / attacking pose
            mob.get_mob_entity().set_attacking(true);

            let target = mob.get_mob_entity().target.lock().await.clone();
            if let Some(target) = target {
                if !Self::target_is_valid(target.as_ref()) {
                    return;
                }
                // Read avoid_water / dest *before* locking navigator (non-reentrant Mutex).
                let avoid_water = Self::mob_avoids_water(mob);
                let dest = Self::path_destination_for(mob, target.as_ref(), avoid_water);
                let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
                navigator.set_progress(NavigatorGoal {
                    current_progress: mob.get_entity().pos.load(),
                    destination: dest,
                    speed: self.speed,
                });
                self.last_target_position = Some(dest);
            }
            self.update_countdown_ticks = 0;
            self.cooldown = 0;
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            // Always clear target when melee ends if it is dead/invalid so
            // ActiveTargetGoal can pick the next living enemy (golem 2nd zombie,
            // vindicator next villager). Vanilla TargetGoal.stop clears the target.
            let should_clear = {
                let target = mob.get_mob_entity().target.lock().await;
                match target.as_deref() {
                    None => false,
                    Some(entity) => {
                        !Self::target_is_valid(entity)
                            || entity
                                .get_player()
                                .is_some_and(|p| p.is_spectator() || p.is_creative())
                    }
                }
            };
            if should_clear {
                mob.set_mob_target(None).await;
            }

            mob.get_mob_entity().set_attacking(false);
            mob.get_mob_entity().navigator.lock().unwrap().stop();
            self.last_target_position = None;
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            let target = mob.get_mob_entity().target.lock().await.clone();
            let Some(target) = target else {
                return;
            };

            // Bail out mid-tick if the target died this frame.
            if !Self::target_is_valid(target.as_ref()) {
                mob.set_mob_target(None).await;
                mob.get_mob_entity().set_attacking(false);
                mob.get_mob_entity().navigator.lock().unwrap().stop();
                return;
            }

            mob.get_mob_entity()
                .look_control
                .lock()
                .unwrap()
                .look_at_entity_with_range(&target, 30.0, 30.0);

            self.update_countdown_ticks = (self.update_countdown_ticks - 1).max(0);

            let avoid_water = Self::mob_avoids_water(mob);
            let dest = Self::path_destination_for(mob, target.as_ref(), avoid_water);
            let should_update_nav = self.update_countdown_ticks <= 0
                && (self
                    .last_target_position
                    .is_none_or(|last_pos| dest.squared_distance_to_vec(&last_pos) >= 1.0)
                    || mob.get_random().random_range(0..20) == 0);

            if should_update_nav {
                let mob_pos = mob.get_entity().pos.load();
                let dist_sq = mob_pos.squared_distance_to_vec(&dest);
                let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
                navigator.set_progress(NavigatorGoal {
                    current_progress: mob_pos,
                    destination: dest,
                    speed: self.speed,
                });
                self.last_target_position = Some(dest);
                // Vanilla-ish repath cadence: faster when close for tighter chase.
                self.update_countdown_ticks = if dist_sq < 16.0 {
                    2 + mob.get_random().random_range(0..3)
                } else {
                    4 + mob.get_random().random_range(0..7)
                };
                if dist_sq > 1024.0 {
                    self.update_countdown_ticks += 10;
                } else if dist_sq > 256.0 {
                    self.update_countdown_ticks += 5;
                }
            }

            self.cooldown = (self.cooldown - 1).max(0);

            let can_see = {
                let from = mob.get_entity().get_eye_pos();
                let to = target.get_entity().get_eye_pos();
                let world = mob.get_entity().world.load();
                world
                    .raycast(from, to, async |block_pos, w| {
                        let state = w.get_block_state(block_pos);
                        state.is_solid()
                    })
                    .await
                    .is_none()
            };

            if self.cooldown <= 0
                && can_see
                && mob
                    .get_mob_entity()
                    .is_in_attack_range(target.as_ref())
                    .await
            {
                self.cooldown = self.get_max_cooldown();
                let is_golem = mob.get_entity().entity_type.id
                    == pumpkin_data::entity::EntityType::IRON_GOLEM.id;
                // Iron golem: both arms raise via entity event 4 inside try_attack.
                // Other mobs: arm swing animation packet.
                if !is_golem {
                    mob.get_mob_entity().living_entity.swing_hand().await;
                }
                // `mob` is EntityBase (Mob: EntityBase) — used as damage cause/source.
                let caller: &dyn EntityBase = mob;
                mob.get_mob_entity()
                    .try_attack(caller, target.as_ref())
                    .await;

                // If the attack killed them, clear immediately so we don't keep
                // swinging at a corpse for the rest of the death animation.
                if !Self::target_is_valid(target.as_ref()) {
                    mob.set_mob_target(None).await;
                    mob.get_mob_entity().set_attacking(false);
                    mob.get_mob_entity().navigator.lock().unwrap().stop();
                }
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        // Vanilla MeleeAttackGoal.requiresUpdateEveryTick() == true
        true
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
