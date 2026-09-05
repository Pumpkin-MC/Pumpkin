use core::f32;
use std::sync::atomic::Ordering;

use crate::entity::{Entity, EntityBase, living::LivingEntity};
use pumpkin_data::damage::DamageType;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::vector3::Vector3;

pub struct PaintingEntity {
    entity: Entity,
}

impl PaintingEntity {
    pub const fn new(entity: Entity) -> Self {
        Self { entity }
    }
}

impl EntityBase for PaintingEntity {
    fn write_custom_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_byte("facing", self.entity.data.load(Ordering::Relaxed) as i8);
    }

    fn read_custom_nbt(&self, nbt: &NbtCompound) {
        let facing = nbt.get_byte("facing").unwrap_or(3);
        self.entity.data.store(facing as i32, Ordering::Relaxed);
    }

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn damage_with_context(
        &self,
        _caller: &dyn EntityBase,
        _amount: f32,
        _damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        _source: Option<&dyn EntityBase>,
        _cause: Option<&dyn EntityBase>,
    ) -> bool {
        // TODO
        self.entity.remove();
        true
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}
