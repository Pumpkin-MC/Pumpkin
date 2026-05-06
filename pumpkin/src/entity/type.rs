use std::sync::Arc;

use pumpkin_data::entity::EntityType;
use pumpkin_util::math::vector3::Vector3;
use uuid::Uuid;

use crate::entity::boss::ender_dragon::EnderDragonEntity;
use crate::entity::boss::wither::WitherEntity;
use crate::entity::decoration::{
    armor_stand::ArmorStandEntity, end_crystal::EndCrystalEntity, painting::PaintingEntity,
};
use crate::entity::experience_orb::ExperienceOrbEntity;
use crate::entity::falling::FallingEntity;
use crate::entity::item::ItemEntity;
use crate::entity::living::LivingEntity;
use crate::entity::mob::bat::BatEntity;
use crate::entity::mob::blaze::BlazeEntity;
use crate::entity::mob::breeze::BreezeEntity;
use crate::entity::mob::cave_spider::CaveSpiderEntity;
use crate::entity::mob::creaking::CreakingEntity;
use crate::entity::mob::creeper::CreeperEntity;
use crate::entity::mob::elder_guardian::ElderGuardianEntity;
use crate::entity::mob::enderman::EndermanEntity;
use crate::entity::mob::endermite::EndermiteEntity;
use crate::entity::mob::evoker::EvokerEntity;
use crate::entity::mob::ghast::GhastEntity;
use crate::entity::mob::giant::GiantEntity;
use crate::entity::mob::guardian::GuardianEntity;
use crate::entity::mob::hoglin::HoglinEntity;
use crate::entity::mob::illusioner::IllusionerEntity;
use crate::entity::mob::magma_cube::MagmaCubeEntity;
use crate::entity::mob::phantom::PhantomEntity;
use crate::entity::mob::piglin::PiglinEntity;
use crate::entity::mob::piglin_brute::PiglinBruteEntity;
use crate::entity::mob::pillager::PillagerEntity;
use crate::entity::mob::ravager::RavagerEntity;
use crate::entity::mob::shulker::ShulkerEntity;
use crate::entity::mob::silverfish::SilverfishEntity;
use crate::entity::mob::skeleton::{
    bogged::BoggedSkeletonEntity, parched::ParchedSkeletonEntity, skeleton::SkeletonEntity,
    stray::StraySkeletonEntity, wither::WitherSkeletonEntity,
};
use crate::entity::mob::slime::SlimeEntity;
use crate::entity::mob::spider::SpiderEntity;
use crate::entity::mob::vex::VexEntity;
use crate::entity::mob::vindicator::VindicatorEntity;
use crate::entity::mob::warden::WardenEntity;
use crate::entity::mob::witch::WitchEntity;
use crate::entity::mob::zoglin::ZoglinEntity;
use crate::entity::mob::zombie::zombie_villager::ZombieVillagerEntity;
use crate::entity::mob::zombie::{drowned::DrownedEntity, husk::HuskEntity, zombie::ZombieEntity};
use crate::entity::mob::zombified_piglin::ZombifiedPiglinEntity;
use crate::entity::passive::allay::AllayEntity;
use crate::entity::passive::armadillo::ArmadilloEntity;
use crate::entity::passive::axolotl::AxolotlEntity;
use crate::entity::passive::bee::BeeEntity;
use crate::entity::passive::camel::CamelEntity;
use crate::entity::passive::cat::CatEntity;
use crate::entity::passive::chicken::ChickenEntity;
use crate::entity::passive::cod::CodEntity;
use crate::entity::passive::cow::CowEntity;
use crate::entity::passive::dolphin::DolphinEntity;
use crate::entity::passive::donkey::DonkeyEntity;
use crate::entity::passive::fox::FoxEntity;
use crate::entity::passive::frog::FrogEntity;
use crate::entity::passive::glow_squid::GlowSquidEntity;
use crate::entity::passive::goat::GoatEntity;
use crate::entity::passive::horse::HorseEntity;
use crate::entity::passive::iron_golem::IronGolemEntity;
use crate::entity::passive::llama::LlamaEntity;
use crate::entity::passive::mooshroom::MooshroomEntity;
use crate::entity::passive::mule::MuleEntity;
use crate::entity::passive::ocelot::OcelotEntity;
use crate::entity::passive::panda::PandaEntity;
use crate::entity::passive::parrot::ParrotEntity;
use crate::entity::passive::pig::PigEntity;
use crate::entity::passive::polar_bear::PolarBearEntity;
use crate::entity::passive::pufferfish::PufferfishEntity;
use crate::entity::passive::rabbit::RabbitEntity;
use crate::entity::passive::salmon::SalmonEntity;
use crate::entity::passive::sheep::SheepEntity;
use crate::entity::passive::skeleton_horse::SkeletonHorseEntity;
use crate::entity::passive::sniffer::SnifferEntity;
use crate::entity::passive::snow_golem::SnowGolemEntity;
use crate::entity::passive::squid::SquidEntity;
use crate::entity::passive::strider::StriderEntity;
use crate::entity::passive::tadpole::TadpoleEntity;
use crate::entity::passive::trader_llama::TraderLlamaEntity;
use crate::entity::passive::tropical_fish::TropicalFishEntity;
use crate::entity::passive::turtle::TurtleEntity;
use crate::entity::passive::villager::VillagerEntity;
use crate::entity::passive::wandering_trader::WanderingTraderEntity;
use crate::entity::passive::wolf::WolfEntity;
use crate::entity::passive::zombie_horse::ZombieHorseEntity;
use crate::entity::projectile::ThrownItemEntity;
use crate::entity::projectile::arrow::ArrowEntity;
use crate::entity::projectile::egg::EggEntity;
use crate::entity::projectile::ender_pearl::EnderPearlEntity;
use crate::entity::projectile::eye_of_ender::EyeOfEnder;
use crate::entity::projectile::fireball::FireballEntity;
use crate::entity::projectile::firework_rocket::FireworkRocketEntity;
use crate::entity::projectile::lingering_potion::LingeringPotionEntity;
use crate::entity::projectile::shulker_bullet::ShulkerBulletEntity;
use crate::entity::projectile::small_fireball::SmallFireballEntity;
use crate::entity::projectile::snowball::SnowballEntity;
use crate::entity::projectile::splash_potion::SplashPotionEntity;
use crate::entity::projectile::wind_charge::WindChargeEntity;
use crate::entity::tnt::TNTEntity;
use crate::entity::vehicle::boat::BoatEntity;
use crate::entity::{Entity, EntityBase};
use crate::world::World;
use pumpkin_data::Block;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use std::sync::atomic::AtomicBool;

#[expect(clippy::too_many_lines)]
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
        id if id == EntityType::ZOMBIFIED_PIGLIN.id => ZombifiedPiglinEntity::new(entity).await,

        // Skeleton
        id if id == EntityType::SKELETON.id => SkeletonEntity::new(entity).await,
        id if id == EntityType::BOGGED.id => BoggedSkeletonEntity::new(entity).await,
        id if id == EntityType::PARCHED.id => ParchedSkeletonEntity::new(entity).await,
        id if id == EntityType::WITHER_SKELETON.id => WitherSkeletonEntity::new(entity).await,
        id if id == EntityType::STRAY.id => StraySkeletonEntity::new(entity).await,

        id if id == EntityType::BAT.id => BatEntity::new(entity).await,
        id if id == EntityType::CREEPER.id => CreeperEntity::new(entity).await,
        id if id == EntityType::ENDERMAN.id => EndermanEntity::new(entity).await,

        id if id == EntityType::BLAZE.id => BlazeEntity::new(entity).await,
        id if id == EntityType::SPIDER.id => SpiderEntity::new(entity).await,
        id if id == EntityType::CAVE_SPIDER.id => CaveSpiderEntity::new(entity).await,
        id if id == EntityType::GHAST.id => GhastEntity::new(entity).await,
        id if id == EntityType::MAGMA_CUBE.id => MagmaCubeEntity::new(entity).await,
        id if id == EntityType::PHANTOM.id => PhantomEntity::new(entity).await,
        id if id == EntityType::WITCH.id => WitchEntity::new(entity).await,
        id if id == EntityType::PIGLIN.id => PiglinEntity::new(entity).await,
        id if id == EntityType::PIGLIN_BRUTE.id => PiglinBruteEntity::new(entity).await,
        id if id == EntityType::PILLAGER.id => PillagerEntity::new(entity).await,
        id if id == EntityType::VINDICATOR.id => VindicatorEntity::new(entity).await,
        id if id == EntityType::EVOKER.id => EvokerEntity::new(entity).await,
        id if id == EntityType::RAVAGER.id => RavagerEntity::new(entity).await,
        id if id == EntityType::GUARDIAN.id => GuardianEntity::new(entity).await,
        id if id == EntityType::ELDER_GUARDIAN.id => ElderGuardianEntity::new(entity).await,
        id if id == EntityType::WARDEN.id => WardenEntity::new(entity).await,
        id if id == EntityType::HOGLIN.id => HoglinEntity::new(entity).await,
        id if id == EntityType::ZOGLIN.id => ZoglinEntity::new(entity).await,
        id if id == EntityType::BREEZE.id => BreezeEntity::new(entity).await,
        id if id == EntityType::CREAKING.id => CreakingEntity::new(entity).await,
        id if id == EntityType::ILLUSIONER.id => IllusionerEntity::new(entity).await,
        id if id == EntityType::VEX.id => VexEntity::new(entity).await,
        id if id == EntityType::ENDERMITE.id => EndermiteEntity::new(entity).await,
        id if id == EntityType::GIANT.id => GiantEntity::new(entity).await,

        id if id == EntityType::CAT.id => CatEntity::new(entity).await,
        id if id == EntityType::CHICKEN.id => ChickenEntity::new(entity).await,
        id if id == EntityType::COW.id => CowEntity::new(entity).await,
        id if id == EntityType::PIG.id => PigEntity::new(entity).await,
        id if id == EntityType::SHEEP.id => SheepEntity::new(entity).await,
        id if id == EntityType::WOLF.id => WolfEntity::new(entity).await,
        id if id == EntityType::FOX.id => FoxEntity::new(entity).await,
        id if id == EntityType::RABBIT.id => RabbitEntity::new(entity).await,
        id if id == EntityType::TURTLE.id => TurtleEntity::new(entity).await,
        id if id == EntityType::VILLAGER.id => VillagerEntity::new(entity).await,
        id if id == EntityType::SQUID.id => SquidEntity::new(entity).await,
        id if id == EntityType::HORSE.id => HorseEntity::new(entity).await,
        id if id == EntityType::DONKEY.id => DonkeyEntity::new(entity).await,
        id if id == EntityType::MULE.id => MuleEntity::new(entity).await,
        id if id == EntityType::ZOMBIE_HORSE.id => ZombieHorseEntity::new(entity).await,
        id if id == EntityType::SKELETON_HORSE.id => SkeletonHorseEntity::new(entity).await,
        id if id == EntityType::LLAMA.id => LlamaEntity::new(entity).await,
        id if id == EntityType::TRADER_LLAMA.id => TraderLlamaEntity::new(entity).await,
        id if id == EntityType::WANDERING_TRADER.id => WanderingTraderEntity::new(entity).await,
        id if id == EntityType::ALLAY.id => AllayEntity::new(entity).await,
        id if id == EntityType::ARMADILLO.id => ArmadilloEntity::new(entity).await,
        id if id == EntityType::AXOLOTL.id => AxolotlEntity::new(entity).await,
        id if id == EntityType::BEE.id => BeeEntity::new(entity).await,
        id if id == EntityType::CAMEL.id => CamelEntity::new(entity).await,
        id if id == EntityType::FROG.id => FrogEntity::new(entity).await,
        id if id == EntityType::GOAT.id => GoatEntity::new(entity).await,
        id if id == EntityType::MOOSHROOM.id => MooshroomEntity::new(entity).await,
        id if id == EntityType::OCELOT.id => OcelotEntity::new(entity).await,
        id if id == EntityType::PANDA.id => PandaEntity::new(entity).await,
        id if id == EntityType::PARROT.id => ParrotEntity::new(entity).await,
        id if id == EntityType::POLAR_BEAR.id => PolarBearEntity::new(entity).await,
        id if id == EntityType::SNIFFER.id => SnifferEntity::new(entity).await,
        id if id == EntityType::STRIDER.id => StriderEntity::new(entity).await,
        id if id == EntityType::GLOW_SQUID.id => GlowSquidEntity::new(entity).await,
        id if id == EntityType::COD.id => CodEntity::new(entity).await,
        id if id == EntityType::SALMON.id => SalmonEntity::new(entity).await,
        id if id == EntityType::PUFFERFISH.id => PufferfishEntity::new(entity).await,
        id if id == EntityType::TROPICAL_FISH.id => TropicalFishEntity::new(entity).await,
        id if id == EntityType::TADPOLE.id => TadpoleEntity::new(entity).await,
        id if id == EntityType::DOLPHIN.id => DolphinEntity::new(entity).await,

        id if id == EntityType::SNOW_GOLEM.id => SnowGolemEntity::new(entity).await,
        id if id == EntityType::IRON_GOLEM.id => IronGolemEntity::new(entity).await,

        id if id == EntityType::WITHER.id => WitherEntity::new(entity).await,
        id if id == EntityType::ENDER_DRAGON.id => EnderDragonEntity::new(entity).await,

        id if id == EntityType::AREA_EFFECT_CLOUD.id => {
            crate::entity::area_effect_cloud::AreaEffectCloudEntity::new(entity)
        }
        id if id == EntityType::ARMOR_STAND.id => Arc::new(ArmorStandEntity::new(entity)),
        id if id == EntityType::PAINTING.id => Arc::new(PaintingEntity::new(entity)),
        id if id == EntityType::END_CRYSTAL.id => Arc::new(EndCrystalEntity::new(entity)),
        id if id == EntityType::ENDER_PEARL.id => Arc::new(EnderPearlEntity::new(entity).await),
        id if id == EntityType::SNOWBALL.id => Arc::new(SnowballEntity::new(entity).await),
        id if id == EntityType::EGG.id => Arc::new(EggEntity::new(entity).await),
        id if id == EntityType::SILVERFISH.id => SilverfishEntity::new(entity).await,
        id if id == EntityType::SLIME.id => SlimeEntity::new(entity).await,
        id if id == EntityType::SHULKER.id => ShulkerEntity::new(entity).await,
        id if id == EntityType::SHULKER_BULLET.id => {
            // Shulker bullets are normally spawned by ShulkerEntity directly;
            // when loaded from the world we create a no-target bullet at the given position.
            Arc::new(ShulkerBulletEntity::orphan(entity))
        }
        id if id == EntityType::FALLING_BLOCK.id => {
            Arc::new(FallingEntity::new(entity, Block::SAND.default_state.id))
        }
        id if id == EntityType::EXPERIENCE_ORB.id => Arc::new(ExperienceOrbEntity::new(entity, 1)),
        id if id == EntityType::TNT.id => Arc::new(TNTEntity::new(entity, 4.0, 80)),
        id if id == EntityType::ITEM.id => {
            Arc::new(ItemEntity::new(entity, ItemStack::new(1, &Item::AIR)).await)
        }
        id if id == EntityType::ARROW.id => Arc::new(ArrowEntity::new(entity, None)),
        id if id == EntityType::SPECTRAL_ARROW.id => Arc::new(ArrowEntity::new(entity, None)),
        id if id == EntityType::FIREBALL.id => Arc::new(FireballEntity::new(entity)),
        id if id == EntityType::SMALL_FIREBALL.id => Arc::new(SmallFireballEntity::new(entity)),
        id if id == EntityType::WIND_CHARGE.id => {
            let thrown = ThrownItemEntity {
                entity,
                owner_id: None,
                collides_with_projectiles: false,
                has_hit: AtomicBool::new(false),
            };
            Arc::new(WindChargeEntity::new(thrown))
        }
        id if id == EntityType::BREEZE_WIND_CHARGE.id => {
            let thrown = ThrownItemEntity {
                entity,
                owner_id: None,
                collides_with_projectiles: false,
                has_hit: AtomicBool::new(false),
            };
            Arc::new(WindChargeEntity::new(thrown))
        }
        id if id == EntityType::FIREWORK_ROCKET.id => {
            Arc::new(FireworkRocketEntity::new(entity).await)
        }
        id if id == EntityType::SPLASH_POTION.id => Arc::new(SplashPotionEntity::new(entity).await),
        id if id == EntityType::LINGERING_POTION.id => {
            Arc::new(LingeringPotionEntity::new(entity).await)
        }
        id if id == EntityType::EYE_OF_ENDER.id => Arc::new(EyeOfEnder::new(entity)),
        id if id == EntityType::ACACIA_BOAT.id
            || id == EntityType::ACACIA_CHEST_BOAT.id
            || id == EntityType::BIRCH_BOAT.id
            || id == EntityType::BIRCH_CHEST_BOAT.id
            || id == EntityType::DARK_OAK_BOAT.id
            || id == EntityType::DARK_OAK_CHEST_BOAT.id
            || id == EntityType::JUNGLE_BOAT.id
            || id == EntityType::JUNGLE_CHEST_BOAT.id
            || id == EntityType::MANGROVE_BOAT.id
            || id == EntityType::MANGROVE_CHEST_BOAT.id
            || id == EntityType::OAK_BOAT.id
            || id == EntityType::OAK_CHEST_BOAT.id
            || id == EntityType::PALE_OAK_BOAT.id
            || id == EntityType::PALE_OAK_CHEST_BOAT.id
            || id == EntityType::SPRUCE_BOAT.id
            || id == EntityType::SPRUCE_CHEST_BOAT.id
            || id == EntityType::BAMBOO_RAFT.id
            || id == EntityType::BAMBOO_CHEST_RAFT.id
            || id == EntityType::CHERRY_BOAT.id
            || id == EntityType::CHERRY_CHEST_BOAT.id =>
        {
            Arc::new(BoatEntity::new(entity))
        }
        // Fallback Entity
        _ => {
            if entity_type.attributes.is_empty() {
                Arc::new(entity)
            } else {
                Arc::new(LivingEntity::new(entity))
            }
        }
    };

    mob
}
