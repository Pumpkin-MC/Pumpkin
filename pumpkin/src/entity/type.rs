use std::sync::Arc;

use pumpkin_data::entity::{EntityType, MobCategory};
use pumpkin_util::math::position::BlockPos;
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
use crate::entity::mob::bat::{self, BatEntity};
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
use crate::entity::projectile::trident::TridentEntity;
use crate::entity::projectile::wind_charge::{WIND_CHARGE_GRAVITY, WindChargeEntity};
use crate::entity::tnt::TNTEntity;
use crate::entity::vehicle::boat::BoatEntity;
use crate::entity::vehicle::minecart::MinecartEntity;
use crate::entity::{Entity, EntityBase, mob};
use crate::world::World;
use pumpkin_data::Block;
use std::sync::atomic::AtomicBool;

#[expect(clippy::too_many_lines)]
pub fn from_type(
    entity_type: &'static EntityType,
    position: Vector3<f64>,
    world: &Arc<World>,
    uuid: Uuid,
) -> Arc<dyn EntityBase> {
    let entity = Entity::from_uuid(uuid, world.clone(), position, entity_type);

    let mob: Arc<dyn EntityBase> = match entity_type.id {
        // Zombie
        id if id == EntityType::ZOMBIE.id => ZombieEntity::new(entity),
        id if id == EntityType::DROWNED.id => DrownedEntity::new(entity),
        id if id == EntityType::HUSK.id => HuskEntity::new(entity),
        id if id == EntityType::ZOMBIE_VILLAGER.id => ZombieVillagerEntity::new(entity),
        id if id == EntityType::ZOMBIFIED_PIGLIN.id => ZombifiedPiglinEntity::new(entity),

        // Skeleton
        id if id == EntityType::SKELETON.id => SkeletonEntity::new(entity),
        id if id == EntityType::BOGGED.id => BoggedSkeletonEntity::new(entity),
        id if id == EntityType::PARCHED.id => ParchedSkeletonEntity::new(entity),
        id if id == EntityType::WITHER_SKELETON.id => WitherSkeletonEntity::new(entity),
        id if id == EntityType::STRAY.id => StraySkeletonEntity::new(entity),

        id if id == EntityType::BAT.id => BatEntity::new(entity),
        id if id == EntityType::CREEPER.id => CreeperEntity::new(entity),
        id if id == EntityType::ENDERMAN.id => EndermanEntity::new(entity),

        id if id == EntityType::BLAZE.id => BlazeEntity::new(entity),
        id if id == EntityType::SPIDER.id => SpiderEntity::new(entity),
        id if id == EntityType::CAVE_SPIDER.id => CaveSpiderEntity::new(entity),
        id if id == EntityType::GHAST.id => GhastEntity::new(entity),
        id if id == EntityType::MAGMA_CUBE.id => MagmaCubeEntity::new(entity),
        id if id == EntityType::PHANTOM.id => PhantomEntity::new(entity),
        id if id == EntityType::WITCH.id => WitchEntity::new(entity),
        id if id == EntityType::PIGLIN.id => PiglinEntity::new(entity),
        id if id == EntityType::PIGLIN_BRUTE.id => PiglinBruteEntity::new(entity),
        id if id == EntityType::PILLAGER.id => PillagerEntity::new(entity),
        id if id == EntityType::VINDICATOR.id => VindicatorEntity::new(entity),
        id if id == EntityType::EVOKER.id => EvokerEntity::new(entity),
        id if id == EntityType::RAVAGER.id => RavagerEntity::new(entity),
        id if id == EntityType::GUARDIAN.id => GuardianEntity::new(entity),
        id if id == EntityType::ELDER_GUARDIAN.id => ElderGuardianEntity::new(entity),
        id if id == EntityType::WARDEN.id => WardenEntity::new(entity),
        id if id == EntityType::HOGLIN.id => HoglinEntity::new(entity),
        id if id == EntityType::ZOGLIN.id => ZoglinEntity::new(entity),
        id if id == EntityType::BREEZE.id => BreezeEntity::new(entity),
        id if id == EntityType::CREAKING.id => CreakingEntity::new(entity),
        id if id == EntityType::ILLUSIONER.id => IllusionerEntity::new(entity),
        id if id == EntityType::VEX.id => VexEntity::new(entity),
        id if id == EntityType::ENDERMITE.id => EndermiteEntity::new(entity),
        id if id == EntityType::GIANT.id => GiantEntity::new(entity),

        id if id == EntityType::CAT.id => CatEntity::new(entity),
        id if id == EntityType::CHICKEN.id => ChickenEntity::new(entity),
        id if id == EntityType::COW.id => CowEntity::new(entity),
        id if id == EntityType::PIG.id => PigEntity::new(entity),
        id if id == EntityType::SHEEP.id => SheepEntity::new(entity),
        id if id == EntityType::WOLF.id => WolfEntity::new(entity),
        id if id == EntityType::FOX.id => FoxEntity::new(entity),
        id if id == EntityType::RABBIT.id => RabbitEntity::new(entity),
        id if id == EntityType::TURTLE.id => TurtleEntity::new(entity),
        id if id == EntityType::VILLAGER.id => VillagerEntity::new(entity),
        id if id == EntityType::SQUID.id => SquidEntity::new(entity),
        id if id == EntityType::HORSE.id => HorseEntity::new(entity),
        id if id == EntityType::DONKEY.id => DonkeyEntity::new(entity),
        id if id == EntityType::MULE.id => MuleEntity::new(entity),
        id if id == EntityType::ZOMBIE_HORSE.id => ZombieHorseEntity::new(entity),
        id if id == EntityType::SKELETON_HORSE.id => SkeletonHorseEntity::new(entity),
        id if id == EntityType::LLAMA.id => LlamaEntity::new(entity),
        id if id == EntityType::TRADER_LLAMA.id => TraderLlamaEntity::new(entity),
        id if id == EntityType::WANDERING_TRADER.id => WanderingTraderEntity::new(entity),
        id if id == EntityType::ALLAY.id => AllayEntity::new(entity),
        id if id == EntityType::ARMADILLO.id => ArmadilloEntity::new(entity),
        id if id == EntityType::AXOLOTL.id => AxolotlEntity::new(entity),
        id if id == EntityType::BEE.id => BeeEntity::new(entity),
        id if id == EntityType::CAMEL.id => CamelEntity::new(entity),
        id if id == EntityType::FROG.id => FrogEntity::new(entity),
        id if id == EntityType::GOAT.id => GoatEntity::new(entity),
        id if id == EntityType::MOOSHROOM.id => MooshroomEntity::new(entity),
        id if id == EntityType::OCELOT.id => OcelotEntity::new(entity),
        id if id == EntityType::PANDA.id => PandaEntity::new(entity),
        id if id == EntityType::PARROT.id => ParrotEntity::new(entity),
        id if id == EntityType::POLAR_BEAR.id => PolarBearEntity::new(entity),
        id if id == EntityType::SNIFFER.id => SnifferEntity::new(entity),
        id if id == EntityType::STRIDER.id => StriderEntity::new(entity),
        id if id == EntityType::GLOW_SQUID.id => GlowSquidEntity::new(entity),
        id if id == EntityType::COD.id => CodEntity::new(entity),
        id if id == EntityType::SALMON.id => SalmonEntity::new(entity),
        id if id == EntityType::PUFFERFISH.id => PufferfishEntity::new(entity),
        id if id == EntityType::TROPICAL_FISH.id => TropicalFishEntity::new(entity),
        id if id == EntityType::TADPOLE.id => TadpoleEntity::new(entity),
        id if id == EntityType::DOLPHIN.id => DolphinEntity::new(entity),

        id if id == EntityType::SNOW_GOLEM.id => SnowGolemEntity::new(entity),
        id if id == EntityType::IRON_GOLEM.id => IronGolemEntity::new(entity),

        id if id == EntityType::WITHER.id => WitherEntity::new(entity),
        id if id == EntityType::ENDER_DRAGON.id => EnderDragonEntity::new(entity),

        id if id == EntityType::AREA_EFFECT_CLOUD.id => {
            crate::entity::area_effect_cloud::AreaEffectCloudEntity::new(entity)
        }
        id if id == EntityType::ARMOR_STAND.id => Arc::new(ArmorStandEntity::new(entity)),
        id if id == EntityType::PAINTING.id => Arc::new(PaintingEntity::new(entity)),
        id if id == EntityType::END_CRYSTAL.id => Arc::new(EndCrystalEntity::new(entity)),
        id if id == EntityType::ENDER_PEARL.id => Arc::new(EnderPearlEntity::new(entity)),
        id if id == EntityType::SNOWBALL.id => Arc::new(SnowballEntity::new(entity)),
        id if id == EntityType::EGG.id => Arc::new(EggEntity::new(entity)),
        id if id == EntityType::SILVERFISH.id => SilverfishEntity::new(entity),
        id if id == EntityType::SLIME.id => SlimeEntity::new(entity),
        id if id == EntityType::SHULKER.id => ShulkerEntity::new(entity),
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
        id if id == EntityType::ITEM.id => Arc::new(ItemEntity::new_for_restore(entity)),
        id if id == EntityType::ARROW.id => Arc::new(ArrowEntity::new(entity, None)),
        id if id == EntityType::SPECTRAL_ARROW.id => Arc::new(ArrowEntity::new(entity, None)),
        id if id == EntityType::TRIDENT.id => Arc::new(TridentEntity::new(entity, None)),
        id if id == EntityType::MINECART.id
            || id == EntityType::CHEST_MINECART.id
            || id == EntityType::FURNACE_MINECART.id
            || id == EntityType::TNT_MINECART.id
            || id == EntityType::HOPPER_MINECART.id
            || id == EntityType::COMMAND_BLOCK_MINECART.id
            || id == EntityType::SPAWNER_MINECART.id =>
        {
            Arc::new(MinecartEntity::new(entity))
        }
        id if id == EntityType::FIREBALL.id => Arc::new(FireballEntity::new(entity)),
        id if id == EntityType::SMALL_FIREBALL.id => Arc::new(SmallFireballEntity::new(entity)),
        id if id == EntityType::WIND_CHARGE.id => {
            let thrown = ThrownItemEntity {
                entity,
                owner_id: None,
                collides_with_projectiles: false,
                has_hit: AtomicBool::new(false),
                gravity: WIND_CHARGE_GRAVITY,
            };
            Arc::new(WindChargeEntity::new_normal(thrown))
        }
        id if id == EntityType::BREEZE_WIND_CHARGE.id => {
            let thrown = ThrownItemEntity {
                entity,
                owner_id: None,
                collides_with_projectiles: false,
                has_hit: AtomicBool::new(false),
                gravity: WIND_CHARGE_GRAVITY,
            };
            Arc::new(WindChargeEntity::new_breeze(thrown))
        }
        id if id == EntityType::FIREWORK_ROCKET.id => Arc::new(FireworkRocketEntity::new(entity)),
        id if id == EntityType::SPLASH_POTION.id => Arc::new(SplashPotionEntity::new(entity)),
        id if id == EntityType::LINGERING_POTION.id => Arc::new(LingeringPotionEntity::new(entity)),
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

pub fn check_spawn_rules(
    entity_type: &'static EntityType,
    world: &World,
    pos: &BlockPos,
    is_thundering: bool,
) -> bool {
    let id = entity_type.id;

    // --- Surface undead (dark + canSeeSky): Stray / Parched ---
    if id == EntityType::STRAY.id || id == EntityType::PARCHED.id {
        return mob::MobEntity::check_surface_monster_spawn_rules(world, pos, is_thundering);
    }

    // --- Any-light monsters (peaceful only): Husk, Silverfish, Endermite ---
    // Vanilla Husk also requires canSeeSky for natural spawns.
    if id == EntityType::HUSK.id {
        if !mob::MobEntity::check_any_light_monster_spawn_rules(world, pos) {
            return false;
        }
        return world.get_sky_light_level(pos) >= 15;
    }
    if id == EntityType::SILVERFISH.id || id == EntityType::ENDERMITE.id {
        return mob::MobEntity::check_any_light_monster_spawn_rules(world, pos);
    }

    // Vanilla `PatrollingMonster.checkPatrollingMonsterSpawnRules`
    // (`PatrollingMonster.java:88-93`): block light at most 8 plus the any-light
    // monster predicate. Unlike `checkMonsterSpawnRules`, it deliberately skips
    // the normal sky-light/random-darkness test.
    if id == EntityType::PILLAGER.id {
        return mob::MobEntity::check_patrolling_monster_spawn_rules(world, pos);
    }

    // Blaze and breeze spawn through `checkAnyLightMonsterSpawnRules`, while
    // magma cubes only check that the difficulty is not peaceful. Applying the
    // generic darkness rule here makes lit nether fortresses and trial chambers
    // incorrectly sterile.
    if id == EntityType::BLAZE.id || id == EntityType::BREEZE.id || id == EntityType::MAGMA_CUBE.id
    {
        return mob::MobEntity::check_any_light_monster_spawn_rules(world, pos);
    }

    // Vanilla Ghast.checkGhastSpawnRules: peaceful gate plus a 1-in-20 roll.
    // Position obstruction is checked separately by the natural spawner.
    if id == EntityType::GHAST.id {
        return mob::MobEntity::check_any_light_monster_spawn_rules(world, pos)
            && rand::random_range(0..20) == 0;
    }

    // Piglins and hoglins deliberately do not use the generic monster-light
    // predicate. Their placement rule only rejects nether wart blocks.
    if id == EntityType::PIGLIN.id || id == EntityType::HOGLIN.id {
        return world.get_block(&pos.down()) != &Block::NETHER_WART_BLOCK;
    }

    // --- Water animals (vanilla SpawnPlacements.java:97-105) ---
    // Squid/dolphin (AgeableWaterCreature.checkSurfaceAgeableWaterCreatureSpawnRules,
    // AgeableWaterCreature.java:70-74) and cod/salmon/pufferfish
    // (WaterAnimal.checkSurfaceWaterAnimalSpawnRules, WaterAnimal.java:76-80)
    // share the same body: y within [seaLevel-13, seaLevel], water fluid below
    // and a water block above.
    if id == EntityType::SQUID.id
        || id == EntityType::DOLPHIN.id
        || id == EntityType::COD.id
        || id == EntityType::SALMON.id
        || id == EntityType::PUFFERFISH.id
    {
        return check_surface_water_animal_spawn_rules(world, pos);
    }
    // Vanilla TropicalFish.checkTropicalFishSpawnRules (TropicalFish.java:249-251):
    // water below+above, and either the any-height biome tag (lush caves) or the
    // surface-water window.
    if id == EntityType::TROPICAL_FISH.id {
        use pumpkin_data::tag::Taggable;
        let biome = world.level.get_rough_biome(pos);
        return water_below_and_water_above(world, pos)
            && (biome.has_tag(
                &pumpkin_data::tag::WorldgenBiome::MINECRAFT_ALLOWS_TROPICAL_FISH_SPAWNS_AT_ANY_HEIGHT,
            ) || check_surface_water_animal_spawn_rules(world, pos));
    }
    // Vanilla GlowSquid.checkGlowSquidSpawnRules (GlowSquid.java:117-119):
    // deep (y <= seaLevel - 33), pitch black, inside water. Without the depth
    // rule glow squid flood surface oceans that should hold regular squid.
    if id == EntityType::GLOW_SQUID.id {
        return pos.0.y <= world.sea_level - 33
            && world.get_light_level_with_darken(pos, 0) == 0
            && world.get_block(pos) == &Block::WATER;
    }
    // Vanilla Axolotl.checkAxolotlSpawnRules (Axolotl.java:528-530).
    if id == EntityType::AXOLOTL.id {
        use pumpkin_data::tag::Taggable;
        return world
            .get_block(&pos.down())
            .has_tag(&pumpkin_data::tag::Block::MINECRAFT_AXOLOTLS_SPAWNABLE_ON);
    }
    // Vanilla Drowned.checkDrownedSpawnRules (Drowned.java:136-154), natural
    // spawn path: water fluid below, monster gate (peaceful + darkness) while
    // inside water, then a rarity roll — 1/15 in river-like biomes
    // (MORE_FREQUENT_DROWNED_SPAWNS), else 1/40 and deeper than seaLevel - 5.
    if id == EntityType::DROWNED.id {
        use pumpkin_data::tag::Taggable;
        if !world
            .get_fluid(&pos.down())
            .has_tag(&pumpkin_data::tag::Fluid::MINECRAFT_WATER)
        {
            return false;
        }
        let can_monster_spawn =
            mob::MobEntity::check_monster_spawn_rules(world, pos, is_thundering)
                && world
                    .get_fluid(pos)
                    .has_tag(&pumpkin_data::tag::Fluid::MINECRAFT_WATER);
        let biome = world.level.get_rough_biome(pos);
        if biome.has_tag(&pumpkin_data::tag::WorldgenBiome::MINECRAFT_MORE_FREQUENT_DROWNED_SPAWNS)
        {
            return rand::random_range(0..15) == 0 && can_monster_spawn;
        }
        return rand::random_range(0..40) == 0
            && pos.0.y < world.sea_level - 5
            && can_monster_spawn;
    }

    // --- Standard monster light rules (26.2 Monster.checkMonsterSpawnRules) ---
    if id == EntityType::BOGGED.id
        || id == EntityType::CAVE_SPIDER.id
        || id == EntityType::CREEPER.id
        || id == EntityType::ENDERMAN.id
        || id == EntityType::GIANT.id
        || id == EntityType::RAVAGER.id
        || id == EntityType::SKELETON.id
        || id == EntityType::SPIDER.id
        || id == EntityType::WITCH.id
        || id == EntityType::WITHER.id
        || id == EntityType::WITHER_SKELETON.id
        || id == EntityType::ZOMBIE.id
        || id == EntityType::ZOMBIE_HORSE.id
        || id == EntityType::ZOMBIE_VILLAGER.id
        || id == EntityType::CREAKING.id
        || id == EntityType::EVOKER.id
        || id == EntityType::ILLUSIONER.id
        || id == EntityType::VEX.id
        || id == EntityType::VINDICATOR.id
        || id == EntityType::WARDEN.id
        || id == EntityType::PHANTOM.id
        || id == EntityType::ZOGLIN.id
        || id == EntityType::PIGLIN_BRUTE.id
        || id == EntityType::GUARDIAN.id
        || id == EntityType::ELDER_GUARDIAN.id
        || id == EntityType::SHULKER.id
        || id == EntityType::BREEZE.id
        || id == EntityType::ZOMBIFIED_PIGLIN.id
    {
        // Guardians also need water.
        if id == EntityType::GUARDIAN.id || id == EntityType::ELDER_GUARDIAN.id {
            let feet = world.get_block_state(pos);
            if !(feet.is_liquid() || feet.is_waterlogged()) {
                return false;
            }
        }
        return mob::MobEntity::check_monster_spawn_rules(world, pos, is_thundering);
    }
    if id == EntityType::BAT.id {
        return bat::BatEntity::check_bat_spawn_rules(world, pos);
    }
    if id == EntityType::SLIME.id {
        return SlimeEntity::check_slime_spawn_rules(world, pos);
    }

    // Vanilla PolarBear.checkPolarBearSpawnRules (26.2 CFR):
    // - In frozen-ocean alternate biomes: light + stand on ice (POLAR_BEARS_SPAWNABLE_ON_ALTERNATE)
    // - Else: Animal.checkAnimalSpawnRules (grass-like + light > 8)
    // Snowy plains / ice spikes use the animal path but often surface is snow/ice.
    if id == EntityType::POLAR_BEAR.id {
        return check_polar_bear_spawn_rules(world, pos);
    }

    // Vanilla per-species ground tags (not only grass_block). Taiga/podzol must
    // still get wolves/rabbits/foxes; plains get sheep/cows on grass.
    if entity_type.category == &MobCategory::CREATURE
        && matches!(
            entity_type.spawn_restriction.location,
            pumpkin_data::entity::SpawnLocation::OnGround
        )
    {
        return check_creature_spawn_rules(entity_type, world, pos);
    }

    true
}

/// Vanilla `Animal.isBrightEnoughToSpawn`: getRawBrightness(pos, 0) > 8.
fn is_bright_enough_to_spawn(world: &World, pos: &BlockPos) -> bool {
    world.get_raw_brightness_no_darken(pos) > 8
}

/// Vanilla `WaterAnimal.checkSurfaceWaterAnimalSpawnRules` (WaterAnimal.java:76-80)
/// == `AgeableWaterCreature.checkSurfaceAgeableWaterCreatureSpawnRules`
/// (AgeableWaterCreature.java:70-74): y within [seaLevel - 13, seaLevel] plus
/// the water-below / water-above sandwich.
fn check_surface_water_animal_spawn_rules(world: &World, pos: &BlockPos) -> bool {
    let sea_level = world.sea_level;
    pos.0.y >= sea_level - 13 && pos.0.y <= sea_level && water_below_and_water_above(world, pos)
}

/// Shared fish-rule body: water fluid below the spawn block and a water block
/// above it (WaterAnimal.java:79).
fn water_below_and_water_above(world: &World, pos: &BlockPos) -> bool {
    use pumpkin_data::tag::Taggable;
    world
        .get_fluid(&pos.down())
        .has_tag(&pumpkin_data::tag::Fluid::MINECRAFT_WATER)
        && world.get_block(&pos.up()) == &Block::WATER
}

/// Vanilla `Animal.checkAnimalSpawnRules` default: ANIMALS_SPAWNABLE_ON (grass).
fn check_animal_spawn_rules(world: &World, pos: &BlockPos) -> bool {
    use pumpkin_data::tag::Block as BlockTag;
    check_creature_spawn_on_tag(world, pos, &BlockTag::MINECRAFT_ANIMALS_SPAWNABLE_ON)
}

fn check_creature_spawn_on_tag(
    world: &World,
    pos: &BlockPos,
    ground_tag: &'static pumpkin_data::tag::Tag,
) -> bool {
    use pumpkin_data::tag::Taggable;
    let below = world.get_block(&pos.down());
    if !below.has_tag(ground_tag) {
        return false;
    }
    is_bright_enough_to_spawn(world, pos)
}

/// Pick the correct block tag for each OnGround creature (vanilla 26.2 tags).
///
/// Without this, every creature is forced onto `animals_spawnable_on` (= grass
/// only), so taiga (podzol/coarse dirt/snow) never re-spawns wolves/rabbits and
/// looks empty after generation packs despawn or unload.
fn check_creature_spawn_rules(
    entity_type: &'static EntityType,
    world: &World,
    pos: &BlockPos,
) -> bool {
    use pumpkin_data::tag::Block as BlockTag;
    let id = entity_type.id;
    let ground = if id == EntityType::WOLF.id {
        &BlockTag::MINECRAFT_WOLVES_SPAWNABLE_ON
    } else if id == EntityType::FOX.id {
        &BlockTag::MINECRAFT_FOXES_SPAWNABLE_ON
    } else if id == EntityType::RABBIT.id {
        &BlockTag::MINECRAFT_RABBITS_SPAWNABLE_ON
    } else if id == EntityType::PARROT.id {
        &BlockTag::MINECRAFT_PARROTS_SPAWNABLE_ON
    } else if id == EntityType::GOAT.id {
        &BlockTag::MINECRAFT_GOATS_SPAWNABLE_ON
    } else if id == EntityType::CAMEL.id {
        &BlockTag::MINECRAFT_CAMELS_SPAWNABLE_ON
    } else if id == EntityType::ARMADILLO.id {
        &BlockTag::MINECRAFT_ARMADILLO_SPAWNABLE_ON
    } else if id == EntityType::MOOSHROOM.id {
        &BlockTag::MINECRAFT_MOOSHROOMS_SPAWNABLE_ON
    } else if id == EntityType::FROG.id {
        &BlockTag::MINECRAFT_FROGS_SPAWNABLE_ON
    } else {
        // Sheep, pig, chicken, cow, horse, donkey, llama, etc.
        &BlockTag::MINECRAFT_ANIMALS_SPAWNABLE_ON
    };
    check_creature_spawn_on_tag(world, pos, ground)
}

/// Vanilla `PolarBear.checkPolarBearSpawnRules`.
fn check_polar_bear_spawn_rules(world: &World, pos: &BlockPos) -> bool {
    use pumpkin_data::tag::{self, Taggable};
    if !is_bright_enough_to_spawn(world, pos) {
        return false;
    }
    let below = world.get_block(&pos.down());
    // Alternate: polar_bears_spawnable_on_alternate (ice) in frozen-ocean style biomes.
    if below.has_tag(&tag::Block::MINECRAFT_POLAR_BEARS_SPAWNABLE_ON_ALTERNATE)
        || below == &Block::ICE
        || below == &Block::PACKED_ICE
        || below == &Block::BLUE_ICE
        || below == &Block::FROSTED_ICE
    {
        return true;
    }
    // Else animal rules (ANIMALS_SPAWNABLE_ON only).
    check_animal_spawn_rules(world, pos)
}
