use std::sync::Arc;

use pumpkin_data::entity::EntityType;
use pumpkin_util::math::vector3::Vector3;
use uuid::Uuid;

use crate::{
    entity::{
        Entity, EntityBase,
        boss::wither::WitherEntity,
        decoration::{
            armor_stand::ArmorStandEntity, end_crystal::EndCrystalEntity, painting::PaintingEntity,
        },
        living::LivingEntity,
        mob::{
            creeper::CreeperEntity, drowned::DrownedEntity, zombie::ZombieEntity,
            zombie_villager::ZombieVillagerEntity,
        },
        passive::{iron_golem::IronGolemEntity, snow_golem::SnowGolemEntity, wolf::WolfEntity},
    },
    world::World,
};

pub async fn from_type(
    entity_type: &'static EntityType,
    position: Vector3<f64>,
    world: &Arc<World>,
    uuid: Uuid,
) -> Arc<dyn EntityBase> {
    let entity = Entity::from_uuid(uuid, world.clone(), position, entity_type);

    let mob: Arc<dyn EntityBase> = match entity_type.id {
        id if id == EntityType::ZOMBIE.id => ZombieEntity::make(entity).await,
        id if id == EntityType::DROWNED.id => DrownedEntity::make(entity).await,
        id if id == EntityType::ZOMBIE_VILLAGER.id => ZombieVillagerEntity::make(entity).await,
        id if id == EntityType::CREEPER.id => CreeperEntity::make(entity).await,
        id if id == EntityType::SNOW_GOLEM.id => SnowGolemEntity::make(entity).await,
        id if id == EntityType::IRON_GOLEM.id => IronGolemEntity::make(entity).await,
        id if id == EntityType::WOLF.id => WolfEntity::make(entity).await,
        id if id == EntityType::WITHER.id => WitherEntity::make(entity).await,
        id if id == EntityType::ARMOR_STAND.id => Arc::new(ArmorStandEntity::new(entity)),
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
