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
            creeper::CreeperEntity,
            enderman::EndermanEntity,
            silverfish::SilverfishEntity,
            skeleton::{
                bogged::BoggedSkeletonEntity, parched::ParchedSkeletonEntity,
                skeleton::SkeletonEntity, stray::StraySkeletonEntity, wither::WitherSkeletonEntity,
            },
            zombie::{ZombieEntity, drowned::DrownedEntity, husk::HuskEntity},
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
        // Zombie
        id if id == EntityType::ZOMBIE.id => ZombieEntity::new(entity).await,
        id if id == EntityType::DROWNED.id => DrownedEntity::new(entity).await,
        id if id == EntityType::HUSK.id => HuskEntity::new(entity).await,
        id if id == EntityType::ZOMBIE_VILLAGER.id => ZombieVillagerEntity::new(entity).await,

        // Sekelton
        id if id == EntityType::SKELETON.id => SkeletonEntity::new(entity).await,
        id if id == EntityType::BOGGED.id => BoggedSkeletonEntity::new(entity).await,
        id if id == EntityType::PARCHED.id => ParchedSkeletonEntity::new(entity).await,
        id if id == EntityType::WITHER_SKELETON.id => WitherSkeletonEntity::new(entity).await,
        id if id == EntityType::STRAY.id => StraySkeletonEntity::new(entity).await,

        id if id == EntityType::CREEPER.id => CreeperEntity::new(entity).await,
        id if id == EntityType::ENDERMAN.id => EndermanEntity::new(entity).await,

        id if id == EntityType::SNOW_GOLEM.id => SnowGolemEntity::new(entity).await,
        id if id == EntityType::IRON_GOLEM.id => IronGolemEntity::new(entity).await,
        id if id == EntityType::WOLF.id => WolfEntity::new(entity).await,
        id if id == EntityType::WITHER.id => WitherEntity::new(entity).await,
        id if id == EntityType::ARMOR_STAND.id => Arc::new(ArmorStandEntity::new(entity)),
        id if id == EntityType::PAINTING.id => Arc::new(PaintingEntity::new(entity)),
        id if id == EntityType::END_CRYSTAL.id => Arc::new(EndCrystalEntity::new(entity)),
        id if id == EntityType::SILVERFISH.id => SilverfishEntity::new(entity).await,
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
