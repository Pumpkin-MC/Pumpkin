use std::sync::Arc;

use crate::entity::{
    Entity, EntityBase,
    mob::{Mob, MobEntity},
    passive::squid::SquidEntity,
};

/// Represents a Glow Squid, a passive aquatic mob that emits a glowing particle effect.
///
/// Wiki: <https://minecraft.wiki/w/Glow_Squid>
pub struct GlowSquidEntity {
    squid: SquidEntity,
}

impl GlowSquidEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        Arc::new(Self {
            squid: SquidEntity::create(entity),
        })
    }
}

impl Mob for GlowSquidEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.squid.mob_entity
    }

    fn mob_tick(&self, caller: &dyn EntityBase) {
        self.squid.mob_tick(caller);
    }

    fn post_tick(&self) {
        self.squid.post_tick();
    }

    fn mob_travel_in_water(&self, caller: &dyn EntityBase) -> bool {
        self.squid.mob_travel_in_water(caller)
    }
}
