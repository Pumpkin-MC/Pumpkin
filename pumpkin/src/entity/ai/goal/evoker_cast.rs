//! Evoker combat stand-in for vanilla spell goals:
//! - Fang attack → magic damage at target feet
//! - Summon Vex occasionally near the target

use super::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::Entity;
use crate::entity::EntityBase;
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use crate::entity::mob::vex::VexEntity;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;
use std::sync::Arc;

const CAST_RANGE_SQ: f64 = 12.0 * 12.0;
const FANG_INTERVAL: i32 = 100;
const SUMMON_INTERVAL: i32 = 340;

pub struct EvokerCastGoal {
    fang_cooldown: i32,
    summon_cooldown: i32,
    speed: f64,
}

impl EvokerCastGoal {
    #[must_use]
    pub fn new(speed: f64) -> Box<Self> {
        Box::new(Self {
            fang_cooldown: 0,
            summon_cooldown: 40,
            speed: speed.max(0.25),
        })
    }

    async fn cast_fangs(mob: &dyn Mob, target: &dyn EntityBase) {
        let shooter = mob.get_entity();
        let world = shooter.world.load();
        let target_pos = target.get_entity().pos.load();

        world.play_sound(
            Sound::EntityEvokerPrepareAttack,
            SoundCategory::Hostile,
            &shooter.pos.load(),
        );
        world.play_sound(
            Sound::EntityEvokerCastSpell,
            SoundCategory::Hostile,
            &target_pos,
        );
        world.play_sound(
            Sound::EntityEvokerFangsAttack,
            SoundCategory::Hostile,
            &target_pos,
        );

        // Stand-in for fangs: magic damage (vanilla fang ~6).
        target.damage(target, 6.0, DamageType::MAGIC).await;
        mob.get_mob_entity().living_entity.swing_hand().await;
    }

    async fn summon_vexes(mob: &dyn Mob, target: Arc<dyn EntityBase>) {
        let shooter = mob.get_entity();
        let world = shooter.world.load();
        let base = target.get_entity().pos.load();

        world.play_sound(
            Sound::EntityEvokerPrepareSummon,
            SoundCategory::Hostile,
            &shooter.pos.load(),
        );
        world.play_sound(
            Sound::EntityEvokerCastSpell,
            SoundCategory::Hostile,
            &shooter.pos.load(),
        );

        // Collect spawn offsets first so ThreadRng is not held across await.
        let offsets: Vec<Vector3<f64>> = {
            let mut rng = mob.get_random();
            let count = 1 + rng.random_range(0..2); // 1–2 vexes
            (0..count)
                .map(|_| {
                    Vector3::new(
                        rng.random_range(-1.5..1.5),
                        rng.random_range(0.5..1.5),
                        rng.random_range(-1.5..1.5),
                    )
                })
                .collect()
        };
        for offset in offsets {
            let spawn_pos = base.add_raw(offset.x, offset.y, offset.z);
            let entity = Entity::new(world.clone(), spawn_pos, &EntityType::VEX);
            let vex = VexEntity::new(entity);
            // Copy target so vexes engage immediately.
            {
                *vex.get_mob_entity().target.lock().await = Some(target.clone());
            }
            world.spawn_entity(vex as Arc<dyn EntityBase>).await;
        }
    }
}

impl Goal for EvokerCastGoal {
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

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            let target = mob.get_mob_entity().target.lock().await.clone();
            let Some(target) = target else {
                return;
            };

            let mob_pos = mob.get_entity().pos.load();
            let target_pos = target.get_entity().pos.load();
            let dist_sq = mob_pos.squared_distance_to_vec(&target_pos);

            {
                let eye = target.get_entity().get_eye_pos();
                let mut look = mob.get_mob_entity().look_control.lock().unwrap();
                look.look_at_with_range(eye.x, eye.y, eye.z, 30.0, 30.0);
            }

            // Keep medium range like vanilla spellcasters.
            if dist_sq > CAST_RANGE_SQ {
                let mut nav = mob.get_mob_entity().navigator.lock().unwrap();
                nav.set_progress(NavigatorGoal::new(mob_pos, target_pos, self.speed));
            } else if dist_sq < 4.0 * 4.0 {
                // Back off slightly if too close.
                let away = mob_pos.add_raw(
                    (mob_pos.x - target_pos.x).signum() * 3.0,
                    0.0,
                    (mob_pos.z - target_pos.z).signum() * 3.0,
                );
                let mut nav = mob.get_mob_entity().navigator.lock().unwrap();
                nav.set_progress(NavigatorGoal::new(mob_pos, away, self.speed));
            } else {
                mob.get_mob_entity().navigator.lock().unwrap().stop();
            }

            self.fang_cooldown -= 1;
            self.summon_cooldown -= 1;

            if dist_sq <= CAST_RANGE_SQ {
                if self.fang_cooldown <= 0 {
                    Self::cast_fangs(mob, target.as_ref()).await;
                    self.fang_cooldown = to_goal_ticks(FANG_INTERVAL);
                }
                if self.summon_cooldown <= 0 {
                    Self::summon_vexes(mob, target.clone()).await;
                    self.summon_cooldown = to_goal_ticks(SUMMON_INTERVAL);
                }
            }
        })
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}
