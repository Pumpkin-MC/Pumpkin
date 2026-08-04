use super::EnderDragonPhase;
use crate::entity::boss::ender_dragon::{EnderDragonEntity, Vector3Ext};
use futures::future::BoxFuture;

pub struct ChargingPhase;

impl super::Phase for ChargingPhase {
    fn get_type(&self) -> EnderDragonPhase {
        EnderDragonPhase::ChargingPlayer
    }

    fn begin<'a>(&'a self, dragon: &'a EnderDragonEntity) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            *dragon.target_location.lock().await = None;
            *dragon.charging_ticks.lock().await = 0;
        })
    }

    fn tick<'a>(&'a self, dragon: &'a EnderDragonEntity) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            let target = *dragon.strafe_target.lock().await;

            if target.is_none() {
                dragon.set_phase(EnderDragonPhase::HoldingPattern).await;
                return;
            }

            let target_pos = target.unwrap();
            let pos = dragon.mob_entity.living_entity.entity.pos.load();

            let mut ticks = dragon.charging_ticks.lock().await;
            if *ticks > 0 {
                *ticks += 1;
                if *ticks >= 10 {
                    drop(ticks);
                    dragon.set_phase(EnderDragonPhase::HoldingPattern).await;
                    return;
                }
            } else {
                let d = pos.distance_squared(target_pos);
                if !(100.0..=22500.0).contains(&d)
                    || dragon
                        .mob_entity
                        .living_entity
                        .entity
                        .horizontal_collision
                        .load(std::sync::atomic::Ordering::Relaxed)
                    || dragon
                        .mob_entity
                        .living_entity
                        .entity
                        .vertical_collision
                        .load(std::sync::atomic::Ordering::Relaxed)
                {
                    *ticks += 1;
                }
            }
            drop(ticks);

            *dragon.target_location.lock().await = Some(target_pos);
        })
    }

    fn get_max_y_acceleration(&self) -> f32 {
        3.0
    }

    fn get_fly_speed(&self) -> f32 {
        3.0
    }
}
