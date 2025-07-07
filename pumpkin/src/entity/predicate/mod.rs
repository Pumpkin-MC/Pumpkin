use crate::entity::{Entity, EntityBase};
use std::pin::Pin;

pub enum EntityPredicate<'a> {
    ValidEntity,
    ValidLivingEntity,
    NotMounted,
    ValidInventories,
    ExceptCreativeOrSpectator,
    ExceptSpectator,
    CanCollide,
    CanHit,
    Rides(&'a Entity),
}

impl<'a> EntityPredicate<'a> {
    pub fn test<'b>(
        &'b self,
        entity: &'b Entity,
    ) -> Pin<Box<dyn Future<Output = bool> + Send + 'b>> {
        Box::pin(async move {
            match self {
                EntityPredicate::ValidEntity => entity.is_alive(),
                EntityPredicate::ValidLivingEntity => {
                    entity.is_alive() && entity.get_living_entity().is_some()
                }
                EntityPredicate::NotMounted => {
                    entity.is_alive() && !entity.has_passengers().await && !entity.has_vehicle().await
                }
                EntityPredicate::ValidInventories => {
                    // TODO: implement
                    false
                }
                EntityPredicate::ExceptCreativeOrSpectator => {
                    if let Some(player) = entity.get_player() {
                        player.is_spectator() || player.is_creative()
                    } else {
                        false
                    }
                }
                EntityPredicate::ExceptSpectator => !entity.is_spectator(),
                EntityPredicate::CanCollide => {
                    EntityPredicate::ExceptSpectator.test(entity).await
                        && entity.is_collidable(None)
                }
                EntityPredicate::CanHit => {
                    EntityPredicate::ExceptSpectator.test(entity).await && entity.can_hit()
                }
                EntityPredicate::Rides(target_entity) => {
                    let target: &Entity = *target_entity;

                    let mut opt_vehicle_arc = {
                        let vehicle_lock = entity.vehicle.lock().await;
                        vehicle_lock.clone()
                    };

                    while let Some(vehicle_arc) = opt_vehicle_arc {
                        let vehicle_entity_base: &dyn EntityBase = &*vehicle_arc;
                        let target_base: &dyn EntityBase = target;

                        if std::ptr::eq(vehicle_entity_base, target_base) {
                            return false;
                        }

                        opt_vehicle_arc = {
                            let vehicle_lock =
                                vehicle_entity_base.get_entity().vehicle.lock().await;
                            vehicle_lock.clone()
                        }
                    }
                    true
                }
            }
        })
    }
}
