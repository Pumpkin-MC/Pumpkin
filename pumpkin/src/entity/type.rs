use std::sync::Arc;

use pumpkin_data::entity::EntityType;
use pumpkin_util::math::vector3::Vector3;
use uuid::Uuid;

use crate::{
    entity::{
        Entity, EntityBase,
        decoration::{end_crystal::EndCrystalEntity, painting::PaintingEntity},
        living::LivingEntity,
        mob::{drowned::DrownedEntity, zombie::ZombieEntity},
    },
    world::World,
};

pub async fn from_type(
    entity_type: &'static EntityType,
    position: Vector3<f64>,
    world: &Arc<World>,
    uuid: Uuid,
) -> Arc<dyn EntityBase> {
    let entity = Entity::new(uuid, world.clone(), position, entity_type, false);

    let mob: Arc<dyn EntityBase> = match entity_type.id {
        id if id == EntityType::ZOMBIE.id => ZombieEntity::make(entity).await,
        id if id == EntityType::DROWNED.id => DrownedEntity::make(entity).await,
        id if id == EntityType::PAINTING.id => Arc::new(PaintingEntity::new(entity)),
        id if id == EntityType::END_CRYSTAL.id => Arc::new(EndCrystalEntity::new(entity)),
        // Fallback Entity
        _ => {
            if entity_type.max_health.is_some() {
                Arc::new(LivingEntity::new(entity))
            } else {
                Arc::new(entity)
            }
        }
    };

    mob
}
