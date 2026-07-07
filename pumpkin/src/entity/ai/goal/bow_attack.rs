use super::{Controls, Goal, GoalFuture};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use crate::entity::projectile::arrow::{ArrowEntity, ArrowPickup};
use crate::entity::{Entity, EntityBase};
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::Difficulty;
use rand::RngExt;
use std::sync::Arc;

/// Ranged bow-attack behavior used by skeletons, strays and bogged.
///
/// Mirrors vanilla `BowAttackGoal`: the mob keeps its distance from the target,
/// circle-strafes left and right, and fires arrows on a difficulty-based
/// interval while the target is within range.
pub struct BowAttackGoal {
    /// Movement speed multiplier used while repositioning toward the target.
    speed: f64,
    /// Base ticks between two shots. Doubled outside of Hard difficulty, matching
    /// vanilla's `20` (Hard) / `40` (otherwise) attack interval.
    attack_interval: i32,
    /// Squared maximum shooting range.
    squared_range: f64,
    /// Ticks until the next arrow can be fired (`<= 0` means ready).
    cooldown: i32,
    /// How long the target has been continuously visible.
    target_seeing_ticks: i32,
    /// Ticks spent holding position in range (`-1` while not yet in range).
    combat_ticks: i32,
    /// Whether the mob currently strafes to its left.
    moving_to_left: bool,
    /// Whether the mob currently backs away from the target.
    backward: bool,
}

impl BowAttackGoal {
    /// `speed` is the reposition speed multiplier, `attack_interval` the base
    /// number of ticks between shots, and `range` the maximum shooting distance
    /// in blocks.
    #[must_use]
    pub fn new(speed: f64, attack_interval: i32, range: f32) -> Box<Self> {
        Box::new(Self {
            speed,
            attack_interval,
            squared_range: f64::from(range * range),
            cooldown: -1,
            target_seeing_ticks: 0,
            combat_ticks: -1,
            moving_to_left: false,
            backward: false,
        })
    }

    /// Spawns an arrow travelling from the mob toward `target`, mirroring vanilla
    /// `SkeletonEntity::shootAt` ballistics: `velocity(dx, dy + horizontal * 0.2,
    /// dz, 1.6, spread)`, where `spread` grows on lower difficulties.
    async fn shoot_at(mob: &dyn Mob, target: &dyn EntityBase, spread: f64) {
        let shooter = mob.get_entity();
        let world = shooter.world.load();

        let arrow_entity = Entity::from_uuid(
            uuid::Uuid::new_v4(),
            world.clone(),
            shooter.pos.load(),
            &EntityType::ARROW,
        );
        // `new_shot` repositions the arrow to the shooter's eye height. Skeleton
        // arrows can never be picked up, hence `Disallowed`.
        let arrow = ArrowEntity::new_shot(arrow_entity, shooter, ArrowPickup::Disallowed);

        let shooter_pos = shooter.pos.load();
        let target_entity = target.get_entity();
        let target_pos = target_entity.pos.load();
        let arrow_pos = arrow.entity.pos.load();

        let dx = target_pos.x - shooter_pos.x;
        // Aim about a third of the way up the target's body, like `getBodyY(1/3)`.
        let dy = f64::from(target_entity.height()).mul_add(1.0 / 3.0, target_pos.y) - arrow_pos.y;
        let dz = target_pos.z - shooter_pos.z;
        let horizontal = dx.hypot(dz);

        arrow.set_velocity(dx, horizontal.mul_add(0.2, dy), dz, 1.6, spread);

        world.play_sound(
            Sound::EntitySkeletonShoot,
            SoundCategory::Hostile,
            &shooter_pos,
        );
        world.spawn_entity(Arc::new(arrow)).await;
    }
}

impl Goal for BowAttackGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let target = mob.get_mob_entity().target.lock().await;
            target.as_ref().is_some_and(|t| t.get_entity().is_alive())
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let target = mob.get_mob_entity().target.lock().await;
            target.as_ref().is_some_and(|t| t.get_entity().is_alive())
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            mob.get_mob_entity().set_attacking(true);
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            mob.get_mob_entity().set_attacking(false);
            mob.get_mob_entity().navigator.lock().unwrap().stop();
            self.target_seeing_ticks = 0;
            self.combat_ticks = -1;
            self.cooldown = -1;
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            let target = mob.get_mob_entity().target.lock().await.clone();
            let Some(target) = target else {
                return;
            };

            let mob_pos = mob.get_entity().pos.load();
            let target_pos = target.get_entity().pos.load();
            let distance_sq = mob_pos.squared_distance_to_vec(&target_pos);

            // Upstream has no mob line-of-sight raycast yet (see
            // `BlazeShootFireballGoal`), so the target counts as continuously
            // visible while this goal runs.
            self.target_seeing_ticks += 1;

            // Hold position and fire once in range and steadily visible,
            // otherwise path toward the target.
            let holding_position =
                distance_sq <= self.squared_range && self.target_seeing_ticks >= 20;
            {
                let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
                if holding_position {
                    navigator.stop();
                    self.combat_ticks += 1;
                } else {
                    navigator.set_progress(NavigatorGoal::new(mob_pos, target_pos, self.speed));
                    self.combat_ticks = -1;
                }
            }

            // Roughly once a second in combat, randomly flip the strafe
            // direction so the mob weaves instead of standing still.
            if self.combat_ticks >= 20 {
                if mob.get_random().random_range(0.0..1.0) < 0.3 {
                    self.moving_to_left = !self.moving_to_left;
                }
                if mob.get_random().random_range(0.0..1.0) < 0.3 {
                    self.backward = !self.backward;
                }
                self.combat_ticks = 0;
            }

            // Always face the target while engaging.
            mob.get_mob_entity()
                .look_control
                .lock()
                .unwrap()
                .look_at_entity_with_range(&target, 30.0, 30.0);

            if self.combat_ticks > -1 {
                // Back away when too close, close in when too far.
                if distance_sq > self.squared_range * 0.75 {
                    self.backward = false;
                } else if distance_sq < self.squared_range * 0.25 {
                    self.backward = true;
                }
                mob.get_mob_entity().move_control.lock().unwrap().strafe(
                    if self.backward { -0.5 } else { 0.5 },
                    if self.moving_to_left { 0.5 } else { -0.5 },
                );

                if self.cooldown > 0 {
                    self.cooldown -= 1;
                } else {
                    let difficulty = mob.get_entity().world.load().level_info.load().difficulty;
                    let spread = f64::from(14 - (difficulty as i32) * 4);
                    Self::shoot_at(mob, target.as_ref(), spread).await;
                    self.cooldown = if difficulty == Difficulty::Hard {
                        self.attack_interval
                    } else {
                        self.attack_interval * 2
                    };
                }
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}
