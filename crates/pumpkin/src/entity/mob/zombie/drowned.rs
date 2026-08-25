use std::sync::Arc;

use crate::entity::mob::zombie::ZombieEntityBase;
use crate::entity::{
    Entity, NbtFuture,
    mob::{Mob, MobEntity},
};
use pumpkin_nbt::compound::NbtCompound;

pub struct DrownedEntity {
    entity: Arc<ZombieEntityBase>,
}

impl DrownedEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let entity = ZombieEntityBase::new(entity);
        let zombie = Self { entity };
        let mob_arc = Arc::new(zombie);
        // Fix duplicated since already in ZombieEntity::new()
        {
            //let mut target_selector = mob_arc.entity.mob_entity.target_selector.lock().unwrap_or_else(std::sync::PoisonError::into_inner);

            // TODO
            // target_selector.add_goal(
            //     2,
            //     ActiveTargetGoal::with_default(
            //         &mob_arc.entity.mob_entity,
            //         &EntityType::PLAYER,
            //         true,
            //     ),
            // );
        };

        mob_arc
    }

    #[must_use]
    pub fn with_can_break_doors(entity: Entity, can_break_doors: bool) -> Arc<Self> {
        let entity = ZombieEntityBase::with_can_break_doors(entity, can_break_doors);
        let zombie = Self { entity };
        Arc::new(zombie)
    }
}

impl Mob for DrownedEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }

    fn mob_write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.entity.mob_write_nbt(nbt).await;
        })
    }

    fn mob_read_nbt<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.entity.mob_read_nbt(nbt).await;
        })
    }
}
