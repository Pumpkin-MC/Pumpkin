use std::sync::Arc;

use pumpkin_data::entity::EntityType;
use pumpkin_data::tag::Taggable;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use uuid::Uuid;

use crate::entity::boss::ender_dragon::EnderDragonEntity;
use crate::entity::boss::wither::WitherEntity;
use crate::entity::decoration::{
    armor_stand::ArmorStandEntity,
    display::{BlockDisplayEntity, ItemDisplayEntity, TextDisplayEntity},
    end_crystal::EndCrystalEntity,
    item_frame::ItemFrameEntity,
    painting::PaintingEntity,
};
use crate::entity::experience_orb::ExperienceOrbEntity;
use crate::entity::falling::FallingEntity;
use crate::entity::interaction::InteractionEntity;
use crate::entity::item::ItemEntity;
use crate::entity::lightning::LightningBoltEntity;
use crate::entity::living::LivingEntity;
use crate::entity::marker::MarkerEntity;
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
use crate::entity::passive::copper_golem::CopperGolemEntity;
use crate::entity::passive::cow::CowEntity;
use crate::entity::passive::dolphin::DolphinEntity;
use crate::entity::passive::donkey::DonkeyEntity;
use crate::entity::passive::fox::FoxEntity;
use crate::entity::passive::frog::FrogEntity;
use crate::entity::passive::glow_squid::GlowSquidEntity;
use crate::entity::passive::goat::GoatEntity;
use crate::entity::passive::happy_ghast::HappyGhastEntity;
use crate::entity::passive::horse::HorseEntity;
use crate::entity::passive::iron_golem::IronGolemEntity;
use crate::entity::passive::llama::LlamaEntity;
use crate::entity::passive::mooshroom::MooshroomEntity;
use crate::entity::passive::mule::MuleEntity;
use crate::entity::passive::nautilus::NautilusEntity;
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
use crate::entity::projectile::llama_spit::LlamaSpitEntity;
use crate::entity::projectile::shulker_bullet::ShulkerBulletEntity;
use crate::entity::projectile::small_fireball::SmallFireballEntity;
use crate::entity::projectile::snowball::SnowballEntity;
use crate::entity::projectile::splash_potion::SplashPotionEntity;
use crate::entity::projectile::trident::TridentEntity;
use crate::entity::projectile::wind_charge::{WIND_CHARGE_GRAVITY, WindChargeEntity};
use crate::entity::projectile::wither_skull::WitherSkullEntity;
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
        id if id == EntityType::COPPER_GOLEM.id => CopperGolemEntity::new(entity),
        id if id == EntityType::FROG.id => FrogEntity::new(entity),
        id if id == EntityType::GOAT.id => GoatEntity::new(entity),
        id if id == EntityType::HAPPY_GHAST.id => HappyGhastEntity::new(entity),
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
        id if id == EntityType::NAUTILUS.id => NautilusEntity::new(entity),

        id if id == EntityType::SNOW_GOLEM.id => SnowGolemEntity::new(entity),
        id if id == EntityType::IRON_GOLEM.id => IronGolemEntity::new(entity),

        id if id == EntityType::WITHER.id => WitherEntity::new(entity),
        id if id == EntityType::ENDER_DRAGON.id => EnderDragonEntity::new(entity),

        id if id == EntityType::AREA_EFFECT_CLOUD.id => {
            crate::entity::area_effect_cloud::AreaEffectCloudEntity::new(entity)
        }
        id if id == EntityType::ARMOR_STAND.id => Arc::new(ArmorStandEntity::new(entity)),
        id if id == EntityType::BLOCK_DISPLAY.id => BlockDisplayEntity::new(entity),
        id if id == EntityType::ITEM_DISPLAY.id => ItemDisplayEntity::new(entity),
        id if id == EntityType::TEXT_DISPLAY.id => TextDisplayEntity::new(entity),
        id if id == EntityType::PAINTING.id => Arc::new(PaintingEntity::new(entity)),
        id if id == EntityType::ITEM_FRAME.id || id == EntityType::GLOW_ITEM_FRAME.id => {
            Arc::new(ItemFrameEntity::new(entity))
        }
        id if id == EntityType::EVOKER_FANGS.id => Arc::new(
            crate::entity::projectile::evoker_fangs::EvokerFangsEntity::new(entity, 0, 0.0, None),
        ),
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
        id if id == EntityType::ITEM.id => Arc::new(ItemEntity::new_empty(entity)),
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
        id if id == EntityType::WITHER_SKULL.id => Arc::new(WitherSkullEntity::new(entity)),
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
        id if id == EntityType::LIGHTNING_BOLT.id => Arc::new(LightningBoltEntity::new(entity)),
        id if id == EntityType::MARKER.id => MarkerEntity::new(entity),
        id if id == EntityType::INTERACTION.id => InteractionEntity::new(entity),
        id if id == EntityType::FIREWORK_ROCKET.id => Arc::new(FireworkRocketEntity::new(entity)),
        id if id == EntityType::SPLASH_POTION.id => Arc::new(SplashPotionEntity::new(entity)),
        id if id == EntityType::LINGERING_POTION.id => Arc::new(LingeringPotionEntity::new(entity)),
        id if id == EntityType::LLAMA_SPIT.id => Arc::new(LlamaSpitEntity::new(entity)),
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

#[expect(clippy::too_many_lines)]
pub fn check_spawn_rules(
    entity_type: &'static EntityType,
    world: &World,
    pos: &BlockPos,
    is_thundering: bool,
) -> bool {
    let id = entity_type.id;

    // Monsters (Standard Darkness + Peaceful check)
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
    {
        return mob::MobEntity::check_monster_spawn_rules(world, pos, is_thundering);
    }

    // Any-light monsters (Blaze, Breeze, Zoglin)
    if id == EntityType::BLAZE.id || id == EntityType::BREEZE.id || id == EntityType::ZOGLIN.id {
        return mob::MobEntity::check_any_light_monster_spawn_rules(world, pos);
    }

    // Surface monsters (Husk, Parched, Camel Husk)
    if id == EntityType::HUSK.id || id == EntityType::PARCHED.id || id == EntityType::CAMEL_HUSK.id
    {
        return mob::MobEntity::check_surface_monsters_spawn_rules(world, pos, is_thundering);
    }

    // Stray
    if id == EntityType::STRAY.id {
        if !mob::MobEntity::check_monster_spawn_rules(world, pos, is_thundering) {
            return false;
        }
        let mut check_sky_pos = *pos;
        while world.get_block(&check_sky_pos) == &pumpkin_data::Block::POWDER_SNOW {
            check_sky_pos = check_sky_pos.up();
        }
        return world.can_see_sky(&check_sky_pos.down());
    }

    // Pillager (Patrolling Monster)
    if id == EntityType::PILLAGER.id {
        if world.get_block_light_level(pos).unwrap_or(0) > 8 {
            return false;
        }
        return mob::MobEntity::check_any_light_monster_spawn_rules(world, pos);
    }

    // Endermite & Silverfish
    if id == EntityType::ENDERMITE.id || id == EntityType::SILVERFISH.id {
        if !mob::MobEntity::check_any_light_monster_spawn_rules(world, pos) {
            return false;
        }
        return world
            .get_closest_player(pos.to_centered_f64(), 5.0)
            .is_none();
    }

    // Ghast
    if id == EntityType::GHAST.id {
        if world.level_info.load().difficulty == pumpkin_util::Difficulty::Peaceful {
            return false;
        }
        if rand::random_range(0..20) != 0 {
            return false;
        }
        return mob::MobEntity::check_mob_spawn_rules(world, pos);
    }

    // Magma Cube
    if id == EntityType::MAGMA_CUBE.id {
        if world.level_info.load().difficulty == pumpkin_util::Difficulty::Peaceful {
            return false;
        }
        return mob::MobEntity::check_mob_spawn_rules(world, pos);
    }

    // Sulfur Cube
    if id == EntityType::SULFUR_CUBE.id {
        return true;
    }

    // Slime
    if id == EntityType::SLIME.id {
        return SlimeEntity::check_slime_spawn_rules(world, pos);
    }

    // Nether Mobs: Hoglin, Piglin, Zombified Piglin
    if id == EntityType::HOGLIN.id || id == EntityType::PIGLIN.id {
        let below = pos.down();
        if world.get_block(&below) == &pumpkin_data::Block::NETHER_WART_BLOCK {
            return false;
        }
        return mob::MobEntity::check_mob_spawn_rules(world, pos);
    }
    if id == EntityType::ZOMBIFIED_PIGLIN.id {
        if world.level_info.load().difficulty == pumpkin_util::Difficulty::Peaceful {
            return false;
        }
        let below = pos.down();
        if world.get_block(&below) == &pumpkin_data::Block::NETHER_WART_BLOCK {
            return false;
        }
        return mob::MobEntity::check_mob_spawn_rules(world, pos);
    }

    // Strider
    if id == EntityType::STRIDER.id {
        let mut check_pos = *pos;
        while world
            .get_fluid(&check_pos)
            .has_tag(&pumpkin_data::tag::Fluid::MINECRAFT_LAVA)
        {
            check_pos = check_pos.up();
        }
        return world.get_block(&check_pos).is_air();
    }

    // Bat
    if id == EntityType::BAT.id {
        return bat::BatEntity::check_bat_spawn_rules(world, pos);
    }

    // Animals (General)
    if id == EntityType::CHICKEN.id
        || id == EntityType::COW.id
        || id == EntityType::DONKEY.id
        || id == EntityType::HAPPY_GHAST.id
        || id == EntityType::HORSE.id
        || id == EntityType::LLAMA.id
        || id == EntityType::MULE.id
        || id == EntityType::PIG.id
        || id == EntityType::SHEEP.id
        || id == EntityType::CAT.id
        || id == EntityType::PANDA.id
        || id == EntityType::TRADER_LLAMA.id
    {
        return mob::MobEntity::check_animal_spawn_rules(world, pos);
    }

    // Tagged Animals: Armadillo, Camel, Frog, Goat, Mooshroom, Parrot, Rabbit, Wolf, Fox
    if id == EntityType::ARMADILLO.id {
        let below = pos.down();
        return world
            .get_block(&below)
            .has_tag(&pumpkin_data::tag::Block::MINECRAFT_ARMADILLO_SPAWNABLE_ON)
            && mob::MobEntity::is_bright_enough_to_spawn(world, pos);
    }
    if id == EntityType::CAMEL.id {
        let below = pos.down();
        return world
            .get_block(&below)
            .has_tag(&pumpkin_data::tag::Block::MINECRAFT_CAMELS_SPAWNABLE_ON)
            && mob::MobEntity::is_bright_enough_to_spawn(world, pos);
    }
    if id == EntityType::FROG.id {
        let below = pos.down();
        return world
            .get_block(&below)
            .has_tag(&pumpkin_data::tag::Block::MINECRAFT_FROGS_SPAWNABLE_ON)
            && mob::MobEntity::is_bright_enough_to_spawn(world, pos);
    }
    if id == EntityType::GOAT.id {
        let below = pos.down();
        return world
            .get_block(&below)
            .has_tag(&pumpkin_data::tag::Block::MINECRAFT_GOATS_SPAWNABLE_ON)
            && mob::MobEntity::is_bright_enough_to_spawn(world, pos);
    }
    if id == EntityType::MOOSHROOM.id {
        let below = pos.down();
        return world
            .get_block(&below)
            .has_tag(&pumpkin_data::tag::Block::MINECRAFT_MOOSHROOMS_SPAWNABLE_ON)
            && mob::MobEntity::is_bright_enough_to_spawn(world, pos);
    }
    if id == EntityType::PARROT.id {
        let below = pos.down();
        return world
            .get_block(&below)
            .has_tag(&pumpkin_data::tag::Block::MINECRAFT_PARROTS_SPAWNABLE_ON)
            && mob::MobEntity::is_bright_enough_to_spawn(world, pos);
    }
    if id == EntityType::RABBIT.id {
        let below = pos.down();
        return world
            .get_block(&below)
            .has_tag(&pumpkin_data::tag::Block::MINECRAFT_RABBITS_SPAWNABLE_ON)
            && mob::MobEntity::is_bright_enough_to_spawn(world, pos);
    }
    if id == EntityType::WOLF.id {
        let below = pos.down();
        return world
            .get_block(&below)
            .has_tag(&pumpkin_data::tag::Block::MINECRAFT_WOLVES_SPAWNABLE_ON)
            && mob::MobEntity::is_bright_enough_to_spawn(world, pos);
    }
    if id == EntityType::FOX.id {
        let below = pos.down();
        return world
            .get_block(&below)
            .has_tag(&pumpkin_data::tag::Block::MINECRAFT_FOXES_SPAWNABLE_ON)
            && mob::MobEntity::is_bright_enough_to_spawn(world, pos);
    }
    if id == EntityType::OCELOT.id {
        if rand::random_range(0..3) == 0 {
            return false;
        }
        return mob::MobEntity::check_mob_spawn_rules(world, pos);
    }
    if id == EntityType::POLAR_BEAR.id {
        let below = pos.down();
        let biome = world.get_biome(pos);
        if biome.has_tag(
            &pumpkin_data::tag::WorldgenBiome::MINECRAFT_POLAR_BEARS_SPAWN_ON_ALTERNATE_BLOCKS,
        ) {
            return mob::MobEntity::is_bright_enough_to_spawn(world, pos)
                && world.get_block(&below).has_tag(
                    &pumpkin_data::tag::Block::MINECRAFT_POLAR_BEARS_SPAWNABLE_ON_ALTERNATE,
                );
        }
        return mob::MobEntity::check_animal_spawn_rules(world, pos);
    }
    if id == EntityType::TURTLE.id {
        let sea_level = world.sea_level;
        let below = pos.down();
        return pos.0.y < sea_level + 4
            && world
                .get_block(&below)
                .has_tag(&pumpkin_data::tag::Block::MINECRAFT_SAND)
            && mob::MobEntity::is_bright_enough_to_spawn(world, pos);
    }
    if id == EntityType::SKELETON_HORSE.id {
        return mob::MobEntity::check_animal_spawn_rules(world, pos);
    }

    // Water Animals / Creatures
    if id == EntityType::COD.id || id == EntityType::PUFFERFISH.id || id == EntityType::SALMON.id {
        return mob::MobEntity::check_surface_water_animal_spawn_rules(world, pos);
    }
    if id == EntityType::DOLPHIN.id || id == EntityType::SQUID.id {
        return mob::MobEntity::check_surface_ageable_water_creature_spawn_rules(world, pos);
    }
    if id == EntityType::TROPICAL_FISH.id {
        let biome = world.get_biome(pos);
        let valid_water = world
            .get_fluid(&pos.down())
            .has_tag(&pumpkin_data::tag::Fluid::MINECRAFT_WATER)
            && (world.get_block(&pos.up()) == &pumpkin_data::Block::WATER
                || world
                    .get_fluid(&pos.up())
                    .has_tag(&pumpkin_data::tag::Fluid::MINECRAFT_WATER));
        if !valid_water {
            return false;
        }
        return biome.has_tag(
            &pumpkin_data::tag::WorldgenBiome::MINECRAFT_ALLOWS_TROPICAL_FISH_SPAWNS_AT_ANY_HEIGHT,
        ) || mob::MobEntity::check_surface_water_animal_spawn_rules(world, pos);
    }
    if id == EntityType::AXOLOTL.id {
        let below = pos.down();
        return world
            .get_block(&below)
            .has_tag(&pumpkin_data::tag::Block::MINECRAFT_AXOLOTLS_SPAWNABLE_ON);
    }
    if id == EntityType::GLOW_SQUID.id {
        let sea_level = world.sea_level;
        return pos.0.y <= sea_level - 33
            && world.get_max_local_raw_brightness(pos) == 0
            && (world.get_block(pos) == &pumpkin_data::Block::WATER
                || world
                    .get_fluid(pos)
                    .has_tag(&pumpkin_data::tag::Fluid::MINECRAFT_WATER));
    }
    if id == EntityType::NAUTILUS.id {
        let sea_level = world.sea_level;
        let min_spawn_level = sea_level - 25;
        return pos.0.y >= min_spawn_level
            && pos.0.y <= sea_level - 5
            && world
                .get_fluid(&pos.down())
                .has_tag(&pumpkin_data::tag::Fluid::MINECRAFT_WATER)
            && (world.get_block(&pos.up()) == &pumpkin_data::Block::WATER
                || world
                    .get_fluid(&pos.up())
                    .has_tag(&pumpkin_data::tag::Fluid::MINECRAFT_WATER));
    }
    if id == EntityType::DROWNED.id {
        if world.level_info.load().difficulty == pumpkin_util::Difficulty::Peaceful {
            return false;
        }
        if !world
            .get_fluid(&pos.down())
            .has_tag(&pumpkin_data::tag::Fluid::MINECRAFT_WATER)
            || !world
                .get_fluid(pos)
                .has_tag(&pumpkin_data::tag::Fluid::MINECRAFT_WATER)
        {
            return false;
        }
        if !mob::MobEntity::is_dark_enough_to_spawn(world, pos, is_thundering) {
            return false;
        }
        let biome = world.get_biome(pos);
        if biome.has_tag(&pumpkin_data::tag::WorldgenBiome::MINECRAFT_MORE_FREQUENT_DROWNED_SPAWNS)
        {
            return rand::random_range(0..15) == 0;
        }
        let sea_level = world.sea_level;
        return pos.0.y < sea_level - 5 && rand::random_range(0..40) == 0;
    }
    if id == EntityType::GUARDIAN.id || id == EntityType::ELDER_GUARDIAN.id {
        if world.level_info.load().difficulty == pumpkin_util::Difficulty::Peaceful {
            return false;
        }
        if !world
            .get_fluid(&pos.down())
            .has_tag(&pumpkin_data::tag::Fluid::MINECRAFT_WATER)
            || !world
                .get_fluid(pos)
                .has_tag(&pumpkin_data::tag::Fluid::MINECRAFT_WATER)
        {
            return false;
        }
        return rand::random_range(0..20) == 0 || !world.can_see_sky(pos);
    }

    // Generic mob spawn rules (Iron Golem, Snow Golem, Villager, Phantom, Shulker, Wandering Trader, Ender Dragon, etc.)
    if entity_type.mob {
        return mob::MobEntity::check_mob_spawn_rules(world, pos);
    }

    true
}
