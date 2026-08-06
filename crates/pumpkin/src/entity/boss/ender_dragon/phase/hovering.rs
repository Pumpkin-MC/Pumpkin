use super::EnderDragonPhase;
use crate::entity::boss::ender_dragon::{EnderDragonEntity, NODE_Y};
use futures::future::BoxFuture;
use pumpkin_util::math::vector3::Vector3;

pub struct HoveringPhase;

impl super::Phase for HoveringPhase {
    fn get_type(&self) -> EnderDragonPhase {
        EnderDragonPhase::Hovering
    }

    fn tick<'a>(&'a self, dragon: &'a EnderDragonEntity) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            let origin = {
                let guard = dragon.fight_origin.lock().await;
                guard.0
            };
            let target = Vector3::new(origin.x as f64, NODE_Y as f64 + 10.0, origin.z as f64);

            if rand::random_bool(0.01) {
                dragon.set_phase(EnderDragonPhase::TakingOff).await;
                return;
            }

            *dragon.target_location.lock().await = Some(target);
        })
    }
}
