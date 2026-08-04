use super::EnderDragonPhase;
use crate::entity::boss::ender_dragon::EnderDragonEntity;
use futures::future::BoxFuture;

pub struct SitAttackingPhase;

impl super::Phase for SitAttackingPhase {
    fn get_type(&self) -> EnderDragonPhase {
        EnderDragonPhase::SittingAttacking
    }

    fn is_sitting_or_hovering(&self) -> bool {
        true
    }

    fn begin<'a>(&'a self, dragon: &'a EnderDragonEntity) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            *dragon.sit_attack_timer.lock().await = 0;
            *dragon.target_location.lock().await = None;
        })
    }

    fn tick<'a>(&'a self, dragon: &'a EnderDragonEntity) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            // Java never repositions the dragon here (`SittingAttackingPhase.serverTick`
            // only counts ticks); it just stays wherever `LandingPhase` left it.
            let mut timer = dragon.sit_attack_timer.lock().await;
            *timer += 1;

            if *timer >= 40 {
                *timer = 0;
                drop(timer);
                dragon.set_phase(EnderDragonPhase::SittingFlaming).await;
            }
        })
    }
}
