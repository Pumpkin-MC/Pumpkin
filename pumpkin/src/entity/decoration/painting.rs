use std::sync::atomic::Ordering;

use async_trait::async_trait;

use crate::entity::{Entity, EntityBase, living::LivingEntity};

pub struct PaintingEntity {
    entity: Entity,
}

impl PaintingEntity {
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }
}

#[async_trait]
impl EntityBase for PaintingEntity {
    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }
    async fn write_nbt(&self, nbt: &mut pumpkin_nbt::compound::NbtCompound) {
        nbt.put_byte("facing", self.entity.data.load(Ordering::Relaxed) as i8);
    }

    async fn read_nbt(&self, _nbt: &pumpkin_nbt::compound::NbtCompound) {
        // TODO
        self.entity.data.store(3, Ordering::Relaxed);
    }
}
