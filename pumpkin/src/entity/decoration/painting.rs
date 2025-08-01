use core::f32;
use std::sync::atomic::Ordering;

use crate::entity::{Entity, EntityBase, NBTStorage, living::LivingEntity};
use async_trait::async_trait;
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

#[async_trait]
impl NBTStorage for PaintingEntity {
    async fn write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_byte("facing", self.entity.data.load(Ordering::Relaxed) as i8);
    }

    async fn read_nbt_non_mut(&self, _nbt: &NbtCompound) {
        // TODO
        self.entity.data.store(3, Ordering::Relaxed);
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

    async fn damage_with_context(
        &self,
        _amount: f32,
        _damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        _source: Option<&dyn EntityBase>,
        _cause: Option<&dyn EntityBase>,
    ) -> bool {
        // TODO
        self.entity.remove().await;
        true
    }
}
