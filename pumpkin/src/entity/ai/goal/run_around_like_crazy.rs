//! Vanilla `RunAroundLikeCrazyGoal` (RunAroundLikeCrazyGoal.java) — an
//! untamed horse carrying a rider bolts to random spots and each second has a
//! 1-in-50 roll to either tame (temper check) or buck the rider off.

use std::sync::atomic::Ordering;

use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use super::{Controls, Goal, GoalFuture};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use crate::entity::passive::horse::HorseEntity;

pub struct RunAroundLikeCrazyGoal {
    speed: f64,
    target: Option<Vector3<f64>>,
}

impl RunAroundLikeCrazyGoal {
    #[must_use]
    pub const fn new(speed: f64) -> Self {
        Self {
            speed,
            target: None,
        }
    }

    fn as_horse(mob: &dyn Mob) -> Option<&HorseEntity> {
        mob.cast_any().downcast_ref::<HorseEntity>()
    }

    async fn has_rider(mob: &dyn Mob) -> bool {
        !mob.get_entity().passengers.lock().await.is_empty()
    }

    /// `DefaultRandomPos.getPos(mob, 5, 4)` stand-in: a random standable spot
    /// within ±5 blocks horizontally and ±4 vertically.
    fn random_run_pos(mob: &dyn Mob) -> Option<Vector3<f64>> {
        let entity = mob.get_entity();
        let world = entity.world.load();
        let pos = entity.pos.load();
        let mut rng = mob.get_random();
        for _ in 0..10 {
            let dx = rng.random_range(-5i32..=5);
            let dy = rng.random_range(-4i32..=4);
            let dz = rng.random_range(-5i32..=5);
            if dx == 0 && dz == 0 {
                continue;
            }
            let candidate = BlockPos::floored(
                pos.x + f64::from(dx),
                pos.y + f64::from(dy),
                pos.z + f64::from(dz),
            );
            let floor = world.get_block_state(&candidate.down());
            let feet = world.get_block_state(&candidate);
            let head = world.get_block_state(&candidate.up());
            if floor.is_side_solid(pumpkin_data::BlockDirection::Up)
                && !feet.is_solid()
                && !head.is_solid()
            {
                return Some(candidate.to_centered_f64());
            }
        }
        None
    }
}

impl Goal for RunAroundLikeCrazyGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(horse) = Self::as_horse(mob) else {
                return false;
            };
            if horse.tamed.load(Ordering::Relaxed) || !Self::has_rider(mob).await {
                return false;
            }
            self.target = Self::random_run_pos(mob);
            self.target.is_some()
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(target) = self.target {
                let mob_pos = mob.get_entity().pos.load();
                let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
                navigator.set_progress(NavigatorGoal::new(mob_pos, target, self.speed));
            }
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(horse) = Self::as_horse(mob) else {
                return false;
            };
            !horse.tamed.load(Ordering::Relaxed)
                && !mob.get_mob_entity().navigator.lock().unwrap().is_idle()
                && Self::has_rider(mob).await
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(horse) = Self::as_horse(mob) else {
                return;
            };
            if horse.tamed.load(Ordering::Relaxed) || mob.get_random().random_range(0..50) != 0 {
                return;
            }

            let entity = mob.get_entity();
            let world = entity.world.load_full();
            let rider = { entity.passengers.lock().await.first().cloned() };
            let Some(rider) = rider else {
                return;
            };

            if let Some(player) = rider.get_player() {
                // Vanilla: nextInt(maxTemper) < temper → tame.
                let temper = horse.temper.load(Ordering::Relaxed);
                if mob.get_random().random_range(0..HorseEntity::MAX_TEMPER) < temper {
                    // CriteriaTriggers.TAME_ANIMAL: advancement trigger not
                    // implemented yet.
                    let _ = player;
                    horse.set_tamed(true);
                    world.send_entity_status(
                        entity,
                        pumpkin_data::entity::EntityStatus::TamingSucceeded,
                    );
                    return;
                }
                horse.modify_temper(5);
            }

            // ejectPassengers + makeMad.
            let rider_ids: Vec<i32> = {
                let passengers = entity.passengers.lock().await;
                passengers
                    .iter()
                    .map(|p| p.get_entity().entity_id)
                    .collect()
            };
            for id in rider_ids {
                entity.remove_passenger(id).await;
            }
            world.play_sound(
                Sound::EntityHorseAngry,
                SoundCategory::Neutral,
                &entity.pos.load(),
            );
            world.send_entity_status(entity, pumpkin_data::entity::EntityStatus::TamingFailed);
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}
