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

impl EntityPredicate<'_> {
    /// Tests `entity` against this predicate.
    ///
    /// Takes `&dyn EntityBase` so that player-state checks (`get_player`, `is_spectator`) and
    /// per-type overrides (`can_hit`, `is_collidable`) dispatch to the concrete mob: on a bare
    /// `&Entity` they fall back to the trait defaults, which silently disabled every
    /// creative/spectator filter on this path.
    #[must_use]
    pub fn test(&self, entity: &dyn EntityBase) -> bool {
        match self {
            EntityPredicate::ValidEntity => entity.get_entity().is_alive(),
            EntityPredicate::ValidLivingEntity => {
                entity.get_entity().is_alive() && entity.get_living_entity().is_some()
            }
            EntityPredicate::NotMounted => {
                let entity = entity.get_entity();
                entity.is_alive() && !entity.has_passengers() && !entity.has_vehicle()
            }
            EntityPredicate::ValidInventories => {
                // TODO: implement
                false
            }
            EntityPredicate::ExceptCreativeOrSpectator => !entity
                .get_player()
                .is_some_and(|player| player.is_spectator() || player.is_creative()),
            EntityPredicate::ExceptSpectator => !entity.is_spectator(),
            EntityPredicate::CanCollide => {
                EntityPredicate::ExceptSpectator.test(entity) && entity.is_collidable(None)
            }
            EntityPredicate::CanHit => {
                EntityPredicate::ExceptSpectator.test(entity) && entity.can_hit()
            }
            EntityPredicate::Rides(target_entity) => {
                let target: &Entity = target_entity;

                let mut opt_vehicle_arc = entity.get_entity().get_vehicle();

                while let Some(vehicle_arc) = opt_vehicle_arc {
                    let vehicle_entity_base: &dyn EntityBase = &*vehicle_arc;
                    let target_base: &dyn EntityBase = target;

                    if std::ptr::eq(vehicle_entity_base, target_base) {
                        return false;
                    }

                    opt_vehicle_arc = vehicle_entity_base.get_entity().get_vehicle();
                }
                true
            }
        }
    }
}
