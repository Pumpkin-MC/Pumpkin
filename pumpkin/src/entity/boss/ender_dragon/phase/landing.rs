use super::EnderDragonPhase;
use crate::entity::boss::ender_dragon::{EnderDragonEntity, Vector3Ext};
use futures::future::BoxFuture;

pub struct LandingPhase;

impl super::Phase for LandingPhase {
    fn get_type(&self) -> EnderDragonPhase {
        EnderDragonPhase::Landing
    }

    fn begin<'a>(&'a self, dragon: &'a EnderDragonEntity) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            *dragon.target_location.lock().await = None;
        })
    }

    fn tick<'a>(&'a self, dragon: &'a EnderDragonEntity) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            let mut target = dragon.target_location.lock().await;
            if target.is_none() {
                *target = Some(dragon.portal_top().await);
            }

            // Java doesn't snap the dragon to the target here - it just checks whether
            // the normal `steer_toward` movement (driven by `getPathTarget`) has
            // naturally brought it within a block, then switches phase; no teleport.
            let pos = dragon.mob_entity.living_entity.entity.pos.load();
            if let Some(t) = *target
                && pos.distance_squared(t) < 1.0
            {
                *dragon.sitting_flaming_times_run.lock().await = 0;
                drop(target);
                dragon.set_phase(EnderDragonPhase::SittingScanning).await;
            }
        })
    }

    fn get_max_y_acceleration(&self) -> f32 {
        1.5
    }

    fn get_fly_speed(&self) -> f32 {
        1.5
    }

    fn get_yaw_acceleration<'a>(&'a self, dragon: &'a EnderDragonEntity) -> BoxFuture<'a, f32> {
        Box::pin(async move {
            let vel = dragon.mob_entity.living_entity.entity.velocity.load();
            let f = vel.horizontal_length() as f32 + 1.0;
            let g = f.min(40.0);
            g / f
        })
    }
}
