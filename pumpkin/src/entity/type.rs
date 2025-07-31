use std::sync::Arc;

use pumpkin_data::entity::EntityType;
use pumpkin_util::math::vector3::Vector3;
use uuid::Uuid;

use crate::{
    entity::{Entity, EntityBase, decoration::painting::PaintingEntity, mob::zombie::Zombie},
    world::World,
};

pub async fn from_type(
    entity_type: EntityType,
    position: Vector3<f64>,
    world: &Arc<World>,
    uuid: Uuid,
) -> Arc<dyn EntityBase> {
    let entity = Entity::new(uuid, world.clone(), position, entity_type, false);

    #[allow(clippy::single_match)]
    let mob: Arc<dyn EntityBase> = match entity_type {
        EntityType::ZOMBIE => Zombie::make(entity).await,
        EntityType::PAINTING => Arc::new(PaintingEntity::new(entity)),
        // TODO
        _ => Arc::new(entity), // Fallback Entity
    };

    mob
}
