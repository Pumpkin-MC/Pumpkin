use core::f32;
use std::sync::{Arc, atomic::Ordering};

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture, living::LivingEntity,
};
use pumpkin_data::damage::DamageType;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::vector3::Vector3;

pub struct PaintingEntity {
    entity: Entity,
}

impl PaintingEntity {
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }
}

impl NBTStorage for PaintingEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            nbt.put_byte("facing", self.entity.data.load(Ordering::Relaxed) as i8);
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, _nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            // TODO
            self.entity.data.store(3, Ordering::Relaxed);
        })
    }
}

impl EntityBase for PaintingEntity {
    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn damage_with_context<'a>(
        &'a self,
        _caller: Arc<dyn EntityBase>,
        _amount: f32,
        _damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        _source: Option<&'a dyn EntityBase>,
        _cause: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async {
            // TODO
            self.entity.remove().await;
            true
        })
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }
}
