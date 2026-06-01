use super::EnderDragonPhase;
use crate::entity::boss::ender_dragon::{EnderDragonEntity, NODE_Y, Vector3Ext};
use futures::future::BoxFuture;
use pumpkin_util::math::vector3::Vector3;

pub struct TakingOffPhase;

impl super::Phase for TakingOffPhase {
    fn get_type(&self) -> EnderDragonPhase {
        EnderDragonPhase::TakingOff
    }

    fn tick<'a>(&'a self, dragon: &'a EnderDragonEntity) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            let origin = {
                let guard = dragon.fight_origin.lock().await;
                guard.0
            };
            let target = Vector3::new(origin.x as f64, NODE_Y as f64, origin.z as f64);
            let pos = dragon.mob_entity.living_entity.entity.pos.load();

            if pos.distance_squared(target) < 16.0 {
                dragon.set_phase(EnderDragonPhase::Circling).await;
                return;
            }

            *dragon.target_location.lock().await = Some(target);
        })
    }
}
