use std::sync::Arc;

use crate::entity::{
    Entity, EntityBaseFuture, NBTStorage,
    mob::{Mob, MobEntity, skeleton::SkeletonEntityBase},
};

pub struct SkeletonEntity {
    entity: Arc<SkeletonEntityBase>,
}

impl SkeletonEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let entity = SkeletonEntityBase::new(entity);
        let skeleton = Self { entity };
        Arc::new(skeleton)
    }
}

impl NBTStorage for SkeletonEntity {
    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut pumpkin_nbt::compound::NbtCompound,
    ) -> crate::entity::NbtFuture<'a, ()> {
        self.get_mob_entity().living_entity.write_nbt(nbt)
    }

    fn read_nbt_non_mut<'a>(
        &'a self,
        nbt: &'a pumpkin_nbt::compound::NbtCompound,
    ) -> crate::entity::NbtFuture<'a, ()> {
        self.get_mob_entity().living_entity.read_nbt_non_mut(nbt)
    }
}

impl Mob for SkeletonEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }

    fn mob_init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        // Forward to base so the bow is actually equipped (wrapper previously
        // used the default Mob impl and left skeletons empty-handed).
        self.entity.mob_init_data_tracker()
    }
}
