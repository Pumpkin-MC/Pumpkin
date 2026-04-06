/* This file is generated. Do not edit manually. */
use crate::entity::EntityType;
use crate::sound::Sound;
pub const fn hurt_sound_for_entity_type(entity_type: &'static EntityType) -> Option<Sound> {
    match entity_type.id {
        id if id == EntityType::BOGGED.id => Some(Sound::EntityBoggedHurt),
        id if id == EntityType::DROWNED.id => Some(Sound::EntityDrownedHurt),
        id if id == EntityType::ENDERMAN.id => Some(Sound::EntityEndermanHurt),
        id if id == EntityType::HUSK.id => Some(Sound::EntityHuskHurt),
        id if id == EntityType::PARCHED.id => Some(Sound::EntityParchedHurt),
        id if id == EntityType::SKELETON.id => Some(Sound::EntitySkeletonHurt),
        id if id == EntityType::STRAY.id => Some(Sound::EntityStrayHurt),
        id if id == EntityType::WITHER_SKELETON.id => Some(Sound::EntityWitherSkeletonHurt),
        id if id == EntityType::ZOMBIE.id => Some(Sound::EntityZombieHurt),
        id if id == EntityType::ZOMBIE_VILLAGER.id => Some(Sound::EntityZombieVillagerHurt),
        _ => None,
    }
}
