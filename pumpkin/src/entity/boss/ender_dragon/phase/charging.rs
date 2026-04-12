use super::EnderDragonPhase;
use crate::entity::boss::ender_dragon::{EnderDragonEntity, NODE_Y, Vector3Ext};
use futures::future::BoxFuture;
use pumpkin_util::math::vector3::Vector3;

pub struct ChargingPhase;

impl super::Phase for ChargingPhase {
    fn get_type(&self) -> EnderDragonPhase {
        EnderDragonPhase::Charging
    }

    fn tick<'a>(&'a self, dragon: &'a EnderDragonEntity) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            let target_id = {
                let guard = dragon.target_player.lock().await;
                *guard
            };
            let world = dragon.mob_entity.living_entity.entity.world.load();
            let pos = dragon.mob_entity.living_entity.entity.pos.load();

            let target_pos = if let Some(id) = target_id
                && let Some(player) = world.players.load().iter().find(|p| p.gameprofile.id == id)
            {
                player.living_entity.entity.pos.load()
            } else {
                let origin = {
                    let guard = dragon.fight_origin.lock().await;
                    guard.0
                };
                Vector3::new(origin.x as f64, NODE_Y as f64 - 20.0, origin.z as f64)
            };

            if pos.distance_squared(target_pos) < 25.0 {
                dragon.set_phase(EnderDragonPhase::Hovering).await;
                return;
            }

            *dragon.target_location.lock().await = Some(target_pos);
        })
    }

    fn get_fly_speed(&self) -> f32 {
        3.0
    }
}
