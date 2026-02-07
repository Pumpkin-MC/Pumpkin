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
            blaze::BlazeEntity, bogged::BoggedEntity, breeze::BreezeEntity,
            cave_spider::CaveSpiderEntity, creaking::CreakingEntity, creeper::CreeperEntity,
            drowned::DrownedEntity, elder_guardian::ElderGuardianEntity,
            enderman::EndermanEntity, endermite::EndermiteEntity, evoker::EvokerEntity,
            ghast::GhastEntity, giant::GiantEntity, guardian::GuardianEntity,
            hoglin::HoglinEntity, husk::HuskEntity, illusioner::IllusionerEntity,
            magma_cube::MagmaCubeEntity, phantom::PhantomEntity, piglin::PiglinEntity,
            piglin_brute::PiglinBruteEntity, pillager::PillagerEntity, ravager::RavagerEntity,
            shulker::ShulkerEntity, silverfish::SilverfishEntity, slime::SlimeEntity,
            spider::SpiderEntity, stray::StrayEntity, vex::VexEntity,
            vindicator::VindicatorEntity, warden::WardenEntity, witch::WitchEntity,
            wither_skeleton::WitherSkeletonEntity, zombie::ZombieEntity,
            zombie_villager::ZombieVillagerEntity, zombified_piglin::ZombifiedPiglinEntity,
            zoglin::ZoglinEntity,
        },
        passive::{
            allay::AllayEntity, armadillo::ArmadilloEntity, axolotl::AxolotlEntity,
            bat::BatEntity, bee::BeeEntity, camel::CamelEntity, cat::CatEntity,
            chicken::ChickenEntity, cod::CodEntity, cow::CowEntity, dolphin::DolphinEntity,
            donkey::DonkeyEntity, fox::FoxEntity, frog::FrogEntity, goat::GoatEntity,
            horse::HorseEntity, iron_golem::IronGolemEntity, llama::LlamaEntity,
            mooshroom::MooshroomEntity, mule::MuleEntity, ocelot::OcelotEntity,
            panda::PandaEntity, parrot::ParrotEntity, pig::PigEntity,
            polar_bear::PolarBearEntity, pufferfish::PufferfishEntity, rabbit::RabbitEntity,
            salmon::SalmonEntity, sheep::SheepEntity, skeleton_horse::SkeletonHorseEntity,
            sniffer::SnifferEntity, snow_golem::SnowGolemEntity, squid::SquidEntity,
            strider::StriderEntity, tadpole::TadpoleEntity,
            trader_llama::TraderLlamaEntity, tropical_fish::TropicalFishEntity,
            turtle::TurtleEntity, wolf::WolfEntity, zombie_horse::ZombieHorseEntity,
        },
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
        // Existing mobs
        id if id == EntityType::ZOMBIE.id => ZombieEntity::new(entity).await,
        id if id == EntityType::DROWNED.id => DrownedEntity::new(entity).await,
        id if id == EntityType::ZOMBIE_VILLAGER.id => ZombieVillagerEntity::new(entity).await,
        id if id == EntityType::CREEPER.id => CreeperEntity::new(entity).await,
        id if id == EntityType::SNOW_GOLEM.id => SnowGolemEntity::new(entity).await,
        id if id == EntityType::IRON_GOLEM.id => IronGolemEntity::new(entity).await,
        id if id == EntityType::WOLF.id => WolfEntity::new(entity).await,
        id if id == EntityType::WITHER.id => WitherEntity::new(entity).await,
        id if id == EntityType::ARMOR_STAND.id => Arc::new(ArmorStandEntity::new(entity)),
        id if id == EntityType::PAINTING.id => Arc::new(PaintingEntity::new(entity)),
        id if id == EntityType::END_CRYSTAL.id => Arc::new(EndCrystalEntity::new(entity)),
        id if id == EntityType::SILVERFISH.id => SilverfishEntity::new(entity).await,
        id if id == EntityType::SPIDER.id => SpiderEntity::new(entity).await,
        id if id == EntityType::ENDERMAN.id => EndermanEntity::new(entity).await,
        // Passive mobs (sessions 005-006)
        id if id == EntityType::CHICKEN.id => ChickenEntity::new(entity).await,
        id if id == EntityType::COW.id => CowEntity::new(entity).await,
        id if id == EntityType::PIG.id => PigEntity::new(entity).await,
        id if id == EntityType::SHEEP.id => SheepEntity::new(entity).await,
        id if id == EntityType::BAT.id => BatEntity::new(entity).await,
        id if id == EntityType::SQUID.id => SquidEntity::new(entity).await,
        id if id == EntityType::RABBIT.id => RabbitEntity::new(entity).await,
        id if id == EntityType::OCELOT.id => OcelotEntity::new(entity).await,
        // Hostile mob variants (sessions 005-006)
        id if id == EntityType::CAVE_SPIDER.id => CaveSpiderEntity::new(entity).await,
        id if id == EntityType::HUSK.id => HuskEntity::new(entity).await,
        id if id == EntityType::STRAY.id => StrayEntity::new(entity).await,
        id if id == EntityType::WITCH.id => WitchEntity::new(entity).await,
        id if id == EntityType::SLIME.id => SlimeEntity::new(entity).await,
        // Session 007 mobs
        id if id == EntityType::PHANTOM.id => PhantomEntity::new(entity).await,
        id if id == EntityType::ENDERMITE.id => EndermiteEntity::new(entity).await,
        id if id == EntityType::MAGMA_CUBE.id => MagmaCubeEntity::new(entity).await,
        id if id == EntityType::DOLPHIN.id => DolphinEntity::new(entity).await,
        id if id == EntityType::FOX.id => FoxEntity::new(entity).await,
        id if id == EntityType::BEE.id => BeeEntity::new(entity).await,
        id if id == EntityType::GOAT.id => GoatEntity::new(entity).await,
        id if id == EntityType::FROG.id => FrogEntity::new(entity).await,
        id if id == EntityType::CAT.id => CatEntity::new(entity).await,
        // Session 008 — structure mobs
        id if id == EntityType::PILLAGER.id => PillagerEntity::new(entity).await,
        id if id == EntityType::VINDICATOR.id => VindicatorEntity::new(entity).await,
        id if id == EntityType::EVOKER.id => EvokerEntity::new(entity).await,
        id if id == EntityType::WITHER_SKELETON.id => WitherSkeletonEntity::new(entity).await,
        id if id == EntityType::GUARDIAN.id => GuardianEntity::new(entity).await,
        id if id == EntityType::ELDER_GUARDIAN.id => ElderGuardianEntity::new(entity).await,
        // Session 008 — Nether mobs
        id if id == EntityType::BLAZE.id => BlazeEntity::new(entity).await,
        id if id == EntityType::GHAST.id => GhastEntity::new(entity).await,
        id if id == EntityType::PIGLIN.id => PiglinEntity::new(entity).await,
        id if id == EntityType::PIGLIN_BRUTE.id => PiglinBruteEntity::new(entity).await,
        id if id == EntityType::ZOMBIFIED_PIGLIN.id => ZombifiedPiglinEntity::new(entity).await,
        id if id == EntityType::HOGLIN.id => HoglinEntity::new(entity).await,
        id if id == EntityType::ZOGLIN.id => ZoglinEntity::new(entity).await,
        // Session 008 — remaining hostile mobs
        id if id == EntityType::WARDEN.id => WardenEntity::new(entity).await,
        id if id == EntityType::RAVAGER.id => RavagerEntity::new(entity).await,
        id if id == EntityType::VEX.id => VexEntity::new(entity).await,
        id if id == EntityType::SHULKER.id => ShulkerEntity::new(entity).await,
        id if id == EntityType::BREEZE.id => BreezeEntity::new(entity).await,
        id if id == EntityType::CREAKING.id => CreakingEntity::new(entity).await,
        id if id == EntityType::BOGGED.id => BoggedEntity::new(entity).await,
        id if id == EntityType::GIANT.id => GiantEntity::new(entity).await,
        id if id == EntityType::ILLUSIONER.id => IllusionerEntity::new(entity).await,
        // Session 008 — horses and undead horses
        id if id == EntityType::HORSE.id => HorseEntity::new(entity).await,
        id if id == EntityType::DONKEY.id => DonkeyEntity::new(entity).await,
        id if id == EntityType::MULE.id => MuleEntity::new(entity).await,
        id if id == EntityType::SKELETON_HORSE.id => SkeletonHorseEntity::new(entity).await,
        id if id == EntityType::ZOMBIE_HORSE.id => ZombieHorseEntity::new(entity).await,
        // Session 008 — high-value passive mobs
        id if id == EntityType::ALLAY.id => AllayEntity::new(entity).await,
        id if id == EntityType::AXOLOTL.id => AxolotlEntity::new(entity).await,
        id if id == EntityType::TURTLE.id => TurtleEntity::new(entity).await,
        id if id == EntityType::PANDA.id => PandaEntity::new(entity).await,
        id if id == EntityType::PARROT.id => ParrotEntity::new(entity).await,
        id if id == EntityType::CAMEL.id => CamelEntity::new(entity).await,
        id if id == EntityType::SNIFFER.id => SnifferEntity::new(entity).await,
        // Session 008 — remaining passive mobs
        id if id == EntityType::LLAMA.id => LlamaEntity::new(entity).await,
        id if id == EntityType::TRADER_LLAMA.id => TraderLlamaEntity::new(entity).await,
        id if id == EntityType::MOOSHROOM.id => MooshroomEntity::new(entity).await,
        id if id == EntityType::POLAR_BEAR.id => PolarBearEntity::new(entity).await,
        id if id == EntityType::STRIDER.id => StriderEntity::new(entity).await,
        id if id == EntityType::ARMADILLO.id => ArmadilloEntity::new(entity).await,
        // Session 008 — fish and aquatic
        id if id == EntityType::COD.id => CodEntity::new(entity).await,
        id if id == EntityType::SALMON.id => SalmonEntity::new(entity).await,
        id if id == EntityType::PUFFERFISH.id => PufferfishEntity::new(entity).await,
        id if id == EntityType::TROPICAL_FISH.id => TropicalFishEntity::new(entity).await,
        id if id == EntityType::TADPOLE.id => TadpoleEntity::new(entity).await,
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
