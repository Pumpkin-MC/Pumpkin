use super::EnderDragonPhase;
use crate::entity::boss::ender_dragon::EnderDragonEntity;
use futures::future::BoxFuture;

pub struct HoveringPhase;

impl super::Phase for HoveringPhase {
    fn get_type(&self) -> EnderDragonPhase {
        EnderDragonPhase::Hover
    }

    fn is_sitting_or_hovering(&self) -> bool {
        true
    }

    fn begin<'a>(&'a self, dragon: &'a EnderDragonEntity) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            *dragon.target_location.lock().await = None;
        })
    }

    fn tick<'a>(&'a self, dragon: &'a EnderDragonEntity) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if dragon.target_location.lock().await.is_none() {
                let pos = dragon.mob_entity.living_entity.entity.pos.load();
                *dragon.target_location.lock().await = Some(pos);
            }
        })
    }

    fn get_max_y_acceleration(&self) -> f32 {
        1.0
    }

    fn get_fly_speed(&self) -> f32 {
        1.0
    }
}
