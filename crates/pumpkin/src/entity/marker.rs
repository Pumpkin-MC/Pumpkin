use std::sync::{Arc, Mutex, atomic::Ordering};

use crate::{
    entity::{Entity, EntityBase, EntityBaseFuture, living::LivingEntity},
    net::{bedrock::BedrockClient, java::JavaClient},
    server::Server,
};
use pumpkin_data::damage::DamageType;
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_util::math::vector3::Vector3;

pub struct MarkerEntity {
    pub entity: Entity,
    pub data: Mutex<NbtCompound>,
}

impl MarkerEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        entity.no_physics.store(true, Ordering::Relaxed);
        Arc::new(Self {
            entity,
            data: Mutex::new(NbtCompound::new()),
        })
    }
}

impl EntityBase for MarkerEntity {
    fn write_custom_nbt(&self, nbt: &mut NbtCompound) {
        let data = self
            .data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !data.is_empty() {
            nbt.put("data", NbtTag::Compound(data.clone()));
        }
    }

    fn read_custom_nbt(&self, nbt: &NbtCompound) {
        if let Some(data) = nbt.get_compound("data") {
            *self
                .data
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = data.clone();
        }
    }

    fn tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>, _server: &'a Server) {}

    fn init_data_tracker(&self) {}

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn is_pushable(&self) -> bool {
        false
    }

    fn is_pushed_by_fluids(&self) -> bool {
        false
    }

    fn can_hit(&self) -> bool {
        false
    }

    fn is_immune_to_explosion(&self) -> bool {
        true
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
        false
    }

    fn send_java_spawn_packet<'a>(&'a self, _client: &'a JavaClient) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {})
    }

    fn send_bedrock_spawn_packet<'a>(
        &'a self,
        _client: &'a BedrockClient,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {})
    }
}
