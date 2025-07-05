use std::pin::Pin;
use std::sync::Arc;
use crate::entity::{Entity, EntityBase};

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
    pub fn test<'b>(&'b self, entity: &'b Entity) -> Pin<Box<dyn Future<Output = bool> + Send + 'b>> {
        Box::pin(async move {
            match self {
                EntityPredicate::ValidEntity => {
                    entity.is_alive()
                }
                EntityPredicate::ValidLivingEntity => {
                    entity.is_alive() && entity.get_living_entity().is_some()
                }
                EntityPredicate::NotMounted => {
                    entity.is_alive() && !entity.has_passengers().await && !entity.has_vehicle()
                }
                EntityPredicate::ValidInventories => {
                    // TODO
                    false
                }
                EntityPredicate::ExceptCreativeOrSpectator => {
                    if let Some(player) = entity.get_player() {
                        player.is_spectator() || player.is_creative()
                    } else {
                        false
                    }
                }
                EntityPredicate::ExceptSpectator => {
                    !entity.is_spectator()
                }
                EntityPredicate::CanCollide => {
                    EntityPredicate::ExceptSpectator.test(entity).await && entity.is_collidable(None)
                }
                EntityPredicate::CanHit => {
                    EntityPredicate::ExceptSpectator.test(entity).await && entity.can_hit()
                }
                EntityPredicate::Rides(target_entity) => {
                    async fn check(entity: Arc<&Entity>) -> Option<Arc<&Entity>> {
                        if !entity.has_vehicle() {
                            return None;
                        }
                        let vehicle_lock = entity.vehicle.lock().await;
                        if let Some(vehicle) = &*vehicle_lock {
                            Some(Arc::new(vehicle.get_entity()))
                        } else {
                            None
                        }
                    }
                    let mut next = check(Arc::new(entity)).await;
                    loop {
                        if let Some(next_entity) = &next {
                            if std::ptr::eq(next_entity.get_entity(), *target_entity) {
                                return false;
                            }
                            next = check(next_entity.clone()).await;
                        } else {
                            break;
                        }
                    }
                    true
                }
            }
        })
    }
}
