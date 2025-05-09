use std::sync::Arc;

use pumpkin_data::entity::EntityType;
use pumpkin_util::math::vector3::Vector3;

use crate::world::World;

use super::{
    Entity, EntityBase, experience_orb::ExperienceOrbEntity, item::ItemEntity,
    living::LivingEntity, mob::zombie::ZombieEntity, tnt::TNTEntity,
};

pub async fn entity_base_from_type(
    entity_type: EntityType,
    entity_uuid: uuid::Uuid,
    world: Arc<World>,
    position: Vector3<f64>,
    invulnerable: bool,
) -> Arc<dyn EntityBase> {
    let entity = Entity::new(entity_uuid, world, position, entity_type, invulnerable);
    // TODO
    match entity_type {
        EntityType::ZOMBIE => Arc::new(ZombieEntity::new(LivingEntity::new(entity), false, false)),
        EntityType::TNT => Arc::new(TNTEntity::new_default(entity)),
        EntityType::ITEM => Arc::new(ItemEntity::new(entity, 1, 1).await), // ?
        EntityType::EXPERIENCE_ORB => Arc::new(ExperienceOrbEntity::new(entity, 1)), // ?

        _ => Arc::new(entity),
    }
}
