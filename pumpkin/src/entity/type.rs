use std::sync::Arc;

use pumpkin_data::entity::EntityType;
use pumpkin_util::math::vector3::Vector3;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
    entity::{
        Entity, EntityBase,
        ai::path::Navigator,
        living::LivingEntity,
        mob::{MobEntity, zombie::Zombie},
    },
    world::World,
};

pub fn from_type(
    entity_type: EntityType,
    position: Vector3<f64>,
    world: &Arc<World>,
    uuid: Uuid,
) -> Arc<dyn EntityBase> {
    let entity = Entity::new(uuid, world.clone(), position, entity_type, false);

    #[allow(clippy::single_match)]
    let mob = match entity_type {
        EntityType::ZOMBIE => Zombie::make(entity),
        // TODO
        _ => MobEntity {
            living_entity: LivingEntity::new(entity),
            goals: Mutex::new(vec![]),
            navigator: Mutex::new(Navigator::default()),
        },
    };
    Arc::new(mob)
}
