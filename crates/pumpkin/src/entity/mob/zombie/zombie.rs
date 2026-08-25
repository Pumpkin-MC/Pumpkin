use crate::entity::mob::equipment::RegionalDifficulty;
use crate::entity::mob::zombie::ZombieEntityBase;
use crate::entity::mob::{Mob, MobEntity};
use crate::entity::{Entity, EntityBaseFuture, NbtFuture};
use crate::world::World;
use pumpkin_nbt::compound::NbtCompound;
use std::sync::Arc;

pub struct ZombieEntity {
    entity: Arc<ZombieEntityBase>,
}

impl ZombieEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let entity = ZombieEntityBase::new(entity);
        let zombie = Self { entity };
        Arc::new(zombie)
    }

    #[must_use]
    pub fn with_can_break_doors(entity: Entity, can_break_doors: bool) -> Arc<Self> {
        let entity = ZombieEntityBase::with_can_break_doors(entity, can_break_doors);
        let zombie = Self { entity };
        Arc::new(zombie)
    }
}

impl Mob for ZombieEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }

    fn populate_default_equipment_slots<'a>(
        &'a self,
        world: &'a Arc<World>,
        difficulty: &'a RegionalDifficulty,
    ) -> EntityBaseFuture<'a, ()> {
        self.entity
            .populate_default_equipment_slots(world, difficulty)
    }

    fn populate_default_equipment_enchantments<'a>(
        &'a self,
        difficulty: &'a RegionalDifficulty,
    ) -> EntityBaseFuture<'a, ()> {
        self.entity
            .populate_default_equipment_enchantments(difficulty)
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
