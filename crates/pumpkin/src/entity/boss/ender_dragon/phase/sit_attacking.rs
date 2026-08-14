use super::EnderDragonPhase;
use crate::entity::boss::ender_dragon::EnderDragonEntity;
use futures::future::BoxFuture;

pub struct SitAttackingPhase;

impl super::Phase for SitAttackingPhase {
    fn get_type(&self) -> EnderDragonPhase {
        EnderDragonPhase::SitAttacking
    }

    fn begin<'a>(&'a self, dragon: &'a EnderDragonEntity) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            *dragon.target_location.lock().await = None;
        })
    }

    fn tick<'a>(&'a self, dragon: &'a EnderDragonEntity) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            let mut timer = dragon.sit_attack_timer.lock().await;
            *timer += 1;

            if *timer > 40 {
                *timer = 0;
                let should_breathe = rand::random_bool(0.5);
                let should_take_off = *dragon.ticks_sitting.lock().await > 200;
                drop(timer);

                if should_breathe {
                    dragon.set_phase(EnderDragonPhase::SitBreathing).await;
                    *dragon.breathing_timer.lock().await = 0;
                } else if should_take_off {
                    dragon.set_phase(EnderDragonPhase::TakingOff).await;
                }
            } else {
                drop(timer);
            }

            let mut dmg = dragon.sitting_damage_received.lock().await;
            if *dmg > 150.0 {
                *dmg = 0.0;
                drop(dmg);
                dragon.set_phase(EnderDragonPhase::TakingOff).await;
            }
        })
    }
}
