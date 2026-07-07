use super::{Controls, Goal, GoalFuture};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use crate::entity::projectile::arrow::{ArrowEntity, ArrowPickup};
use crate::entity::{Entity, EntityBase};
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;
use std::sync::Arc;

/// Ranged crossbow-attack behavior for pillagers and piglins.
///
/// Loosely mirrors vanilla `CrossbowAttackGoal`: the mob closes in until the
/// target is in range, holds position (backing off when the target gets too
/// close), spends a fixed time charging the crossbow, then fires an arrow and
/// reloads. Mob line of sight isn't modelled upstream yet (see
/// `BlazeShootFireballGoal`), so the target counts as visible while the goal
/// runs.
pub struct CrossbowAttackGoal {
    /// Movement speed multiplier while repositioning.
    speed: f64,
    /// Squared maximum shooting range.
    squared_range: f64,
    /// Ticks needed to fully charge the crossbow before it can fire.
    charge_time: i32,
    /// Charging progress; counts up to `charge_time` while not yet charged.
    charge_progress: i32,
    /// Whether the crossbow is charged and ready to fire.
    charged: bool,
    /// Reload delay left after charging (and after a shot) before the next fire.
    cooldown: i32,
    /// How long the target has been continuously visible.
    target_seeing_ticks: i32,
}

impl CrossbowAttackGoal {
    /// `speed` is the reposition speed multiplier and `range` the maximum
    /// shooting distance in blocks.
    #[must_use]
    pub fn new(speed: f64, range: f32) -> Box<Self> {
        Box::new(Self {
            speed,
            squared_range: f64::from(range * range),
            charge_time: 25,
            charge_progress: 0,
            charged: false,
            cooldown: 0,
            target_seeing_ticks: 0,
        })
    }

    /// A position roughly six blocks directly away from the target, used to back
    /// off when it gets too close. Falls back to a fixed direction when the mob
    /// and target overlap horizontally.
    fn retreat_position(mob_pos: Vector3<f64>, target_pos: Vector3<f64>) -> Vector3<f64> {
        let mut away = Vector3::new(mob_pos.x - target_pos.x, 0.0, mob_pos.z - target_pos.z);
        if away.horizontal_length() < 1.0e-4 {
            away = Vector3::new(1.0, 0.0, 0.0);
        }
        mob_pos + away.normalize().multiply(6.0, 6.0, 6.0)
    }

    /// Spawns an arrow travelling from the mob toward `target`, matching vanilla
    /// crossbow ballistics: `velocity(dx, dy + horizontal * 0.2, dz, 1.6, spread)`.
    async fn shoot_at(mob: &dyn Mob, target: &dyn EntityBase, spread: f64) {
        let shooter = mob.get_entity();
        let world = shooter.world.load();

        let arrow_entity = Entity::from_uuid(
            uuid::Uuid::new_v4(),
            world.clone(),
            shooter.pos.load(),
            &EntityType::ARROW,
        );
        // `new_shot` repositions the arrow to the shooter's eye height. Mob-shot
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
            Sound::ItemCrossbowShoot,
            SoundCategory::Hostile,
            &shooter_pos,
        );
        world.spawn_entity(Arc::new(arrow)).await;
    }
}

impl Goal for CrossbowAttackGoal {
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
            self.charge_progress = 0;
            self.charged = false;
            self.cooldown = 0;
            self.target_seeing_ticks = 0;
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

            // Upstream has no mob line-of-sight raycast yet, so the target counts
            // as continuously visible while this goal runs.
            self.target_seeing_ticks += 1;

            let in_range = distance_sq <= self.squared_range && self.target_seeing_ticks >= 5;
            {
                let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
                if in_range {
                    if distance_sq < self.squared_range * 0.25 {
                        // Too close: retreat directly away from the target.
                        let retreat = Self::retreat_position(mob_pos, target_pos);
                        navigator.set_progress(NavigatorGoal::new(mob_pos, retreat, self.speed));
                    } else {
                        navigator.stop();
                    }
                } else {
                    navigator.set_progress(NavigatorGoal::new(mob_pos, target_pos, self.speed));
                }
            }

            // Always face the target while engaging.
            mob.get_mob_entity()
                .look_control
                .lock()
                .unwrap()
                .look_at_entity_with_range(&target, 30.0, 30.0);

            if self.charged {
                if self.cooldown > 0 {
                    self.cooldown -= 1;
                } else if in_range {
                    let difficulty = mob.get_entity().world.load().level_info.load().difficulty;
                    let spread = f64::from(14 - (difficulty as i32) * 4);
                    Self::shoot_at(mob, target.as_ref(), spread).await;
                    self.charged = false;
                    self.charge_progress = 0;
                }
            } else {
                if self.charge_progress == 0 {
                    let pos = mob.get_entity().pos.load();
                    mob.get_entity().world.load().play_sound(
                        Sound::ItemCrossbowLoadingStart,
                        SoundCategory::Hostile,
                        &pos,
                    );
                }
                self.charge_progress += 1;
                if self.charge_progress >= self.charge_time {
                    self.charged = true;
                    self.charge_progress = 0;
                    // Vanilla waits 20-40 ticks after charging before shooting.
                    self.cooldown = 20 + mob.get_random().random_range(0..21);
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
