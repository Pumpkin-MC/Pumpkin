//! Vanilla-style `FleeSunGoal` — undead walk into shade when burning in daylight.

use super::{Controls, Goal, GoalFuture};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

pub struct FleeSunGoal {
    speed: f64,
    shelter: Option<Vector3<f64>>,
}

impl FleeSunGoal {
    #[must_use]
    pub fn new(speed: f64) -> Self {
        Self {
            speed: speed.max(0.5),
            shelter: None,
        }
    }

    fn is_bright_enough_to_burn(mob: &dyn Mob) -> bool {
        let entity = mob.get_entity();
        let world = entity.world.load();
        if entity
            .touching_water
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            return false;
        }
        let feet = entity.block_pos.load();
        let sky = world.get_sky_light_level(&feet);
        if sky < 12 {
            return false;
        }
        // Under a roof? top solid above feet means shade.
        let top = world.get_top_block(pumpkin_util::math::vector2::Vector2::new(
            feet.0.x, feet.0.z,
        ));
        // Exposed to sky if top is at or below feet+1 (standing on surface).
        top <= feet.0.y + 1
    }

    fn find_shelter(mob: &dyn Mob) -> Option<Vector3<f64>> {
        let entity = mob.get_entity();
        let world = entity.world.load();
        let pos = entity.pos.load();
        let mut rng = mob.get_random();

        for _ in 0..24 {
            let dx = rng.random_range(-10i32..=10);
            let dy = rng.random_range(-3i32..=3);
            let dz = rng.random_range(-10i32..=10);
            let sample = BlockPos::floored(
                pos.x + f64::from(dx),
                pos.y + f64::from(dy),
                pos.z + f64::from(dz),
            );
            let below = sample.down();
            let below_state = world.get_block_state(&below);
            let feet_state = world.get_block_state(&sample);
            let head_state = world.get_block_state(&sample.up());
            if !below_state.is_solid() || feet_state.is_solid() || head_state.is_solid() {
                continue;
            }
            // Shade: sky light low, or solid block overhead within a few blocks.
            let sky = world.get_sky_light_level(&sample);
            let mut shaded = sky < 10;
            if !shaded {
                for up in 1..=4 {
                    let above = sample.up_height(up);
                    if world.get_block_state(&above).is_solid() {
                        shaded = true;
                        break;
                    }
                }
            }
            if shaded {
                return Some(Vector3::new(
                    f64::from(sample.0.x) + 0.5,
                    f64::from(sample.0.y),
                    f64::from(sample.0.z) + 0.5,
                ));
            }
        }
        None
    }
}

impl Goal for FleeSunGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            // Only when we have no combat target and sun is dangerous.
            {
                let t = mob.get_mob_entity().target.lock().await;
                if t.is_some() {
                    return false;
                }
            }
            if !Self::is_bright_enough_to_burn(mob) {
                return false;
            }
            // On fire or about to burn — seek shade.
            let on_fire = mob
                .get_entity()
                .fire_ticks
                .load(std::sync::atomic::Ordering::Relaxed)
                > 0;
            if !on_fire {
                // Still seek shade proactively in full sun.
                let sky = mob
                    .get_entity()
                    .world
                    .load()
                    .get_sky_light_level(&mob.get_entity().block_pos.load());
                if sky < 14 {
                    return false;
                }
            }
            self.shelter = Self::find_shelter(mob);
            self.shelter.is_some()
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            if mob.get_mob_entity().target.lock().await.is_some() {
                return false;
            }
            let Some(dest) = self.shelter else {
                return false;
            };
            let pos = mob.get_entity().pos.load();
            pos.squared_distance_to_vec(&dest) > 2.0
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            if let Some(dest) = self.shelter {
                let mut nav = mob.get_mob_entity().navigator.lock().unwrap();
                nav.set_progress(NavigatorGoal {
                    current_progress: mob.get_entity().pos.load(),
                    destination: dest,
                    speed: self.speed,
                });
            }
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.shelter = None;
            mob.get_mob_entity().navigator.lock().unwrap().stop();
        })
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}
