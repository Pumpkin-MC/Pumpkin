use std::sync::Arc;

use crate::entity::mob::zombie::ZombieEntityBase;
use crate::entity::{
    Entity, NBTStorage,
    mob::{Mob, MobEntity},
};

pub struct HuskEntity {
    pub(crate) entity: Arc<ZombieEntityBase>,
}

impl HuskEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let entity = ZombieEntityBase::new(entity);
        let zombie = Self { entity };
        Arc::new(zombie)
    }
}

impl NBTStorage for HuskEntity {
    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut pumpkin_nbt::compound::NbtCompound,
    ) -> crate::entity::NbtFuture<'a, ()> {
        self.entity.write_nbt(nbt)
    }

    fn read_nbt_non_mut<'a>(
        &'a self,
        nbt: &'a pumpkin_nbt::compound::NbtCompound,
    ) -> crate::entity::NbtFuture<'a, ()> {
        self.entity.read_nbt_non_mut(nbt)
    }
}

impl Mob for HuskEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }

    fn is_mob_baby(&self) -> bool {
        self.entity.is_mob_baby()
    }

    fn supports_break_door_goal(&self) -> bool {
        true
    }
}
