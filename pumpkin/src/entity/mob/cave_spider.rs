use std::sync::Arc;

use crate::entity::{
    Entity, NBTStorage,
    mob::{Mob, MobEntity, spider::SpiderEntity},
};

/// Cave spider — same AI as spider; poison applied in `MobEntity::try_attack`.
pub struct CaveSpiderEntity {
    pub spider: Arc<SpiderEntity>,
}

impl CaveSpiderEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let spider = SpiderEntity::new(entity);
        Arc::new(Self { spider })
    }
}

impl NBTStorage for CaveSpiderEntity {
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

impl Mob for CaveSpiderEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        self.spider.get_mob_entity()
    }
}
