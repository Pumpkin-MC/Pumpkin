use super::EnderDragonPhase;
use crate::entity::boss::ender_dragon::EnderDragonEntity;
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
            dragon.set_phase(EnderDragonPhase::SitAttacking).await;
            *dragon.ticks_sitting.lock().await = 0;
            *dragon.sit_attack_timer.lock().await = 0;
        })
    }
}
