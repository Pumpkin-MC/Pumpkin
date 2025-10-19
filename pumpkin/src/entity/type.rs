use std::sync::Arc;

use pumpkin_data::entity::EntityType;
use pumpkin_util::math::vector3::Vector3;
use uuid::Uuid;

use crate::{
    entity::{
        Entity, EntityBase,
        decoration::{end_crystal::EndCrystalEntity, painting::PaintingEntity},
        mob::zombie::Zombie,
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

    #[allow(clippy::single_match)]
    let mob: Arc<dyn EntityBase> = match entity_type.id {
        id if id == EntityType::ZOMBIE.id => Zombie::make(entity).await,
        id if id == EntityType::PAINTING.id => Arc::new(PaintingEntity::new(entity)),
        id if id == EntityType::END_CRYSTAL.id => Arc::new(EndCrystalEntity::new(entity)),
        // TODO
        _ => Arc::new(entity), // Fallback Entity
    };

    mob
}
