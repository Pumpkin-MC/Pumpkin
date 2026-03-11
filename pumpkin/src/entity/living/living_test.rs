#[cfg(test)]
mod tests {
    use pumpkin_data::entity::EntityType;
    use pumpkin_data::sound::Sound;

    /// Test that entity types map to their correct hurt sounds
    #[test]
    fn test_slime_hurt_sound() {
        // Small slimes (size 1) should play the small hurt sound
        // Big slimes (size > 1) should play the regular hurt sound
        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::SLIME, Some(1)),
            Sound::EntitySlimeHurtSmall,
            "Small slime (size 1) should play EntitySlimeHurtSmall"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::SLIME, Some(2)),
            Sound::EntitySlimeHurt,
            "Big slime (size 2) should play EntitySlimeHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::SLIME, Some(4)),
            Sound::EntitySlimeHurt,
            "Big slime (size 4) should play EntitySlimeHurt"
        );
    }

    #[test]
    fn test_magma_cube_hurt_sound() {
        // Magma cubes follow the same pattern as slimes
        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::MAGMA_CUBE, Some(1)),
            Sound::EntityMagmaCubeHurtSmall,
            "Small magma cube (size 1) should play EntityMagmaCubeHurtSmall"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::MAGMA_CUBE, Some(2)),
            Sound::EntityMagmaCubeHurt,
            "Big magma cube (size 2) should play EntityMagmaCubeHurt"
        );
    }

    #[test]
    fn test_common_mob_hurt_sounds() {
        // Test a variety of common mobs have their specific hurt sounds
        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::ZOMBIE, None),
            Sound::EntityZombieHurt,
            "Zombie should play EntityZombieHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::SKELETON, None),
            Sound::EntitySkeletonHurt,
            "Skeleton should play EntitySkeletonHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::CREEPER, None),
            Sound::EntityCreeperHurt,
            "Creeper should play EntityCreeperHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::SPIDER, None),
            Sound::EntitySpiderHurt,
            "Spider should play EntitySpiderHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::ENDERMAN, None),
            Sound::EntityEndermanHurt,
            "Enderman should play EntityEndermanHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::WITCH, None),
            Sound::EntityWitchHurt,
            "Witch should play EntityWitchHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::BLAZE, None),
            Sound::EntityBlazeHurt,
            "Blaze should play EntityBlazeHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::GHAST, None),
            Sound::EntityGhastHurt,
            "Ghast should play EntityGhastHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::WOLF, None),
            Sound::EntityWolfHurt,
            "Wolf should play EntityWolfHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::CAT, None),
            Sound::EntityCatHurt,
            "Cat should play EntityCatHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::CHICKEN, None),
            Sound::EntityChickenHurt,
            "Chicken should play EntityChickenHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::COW, None),
            Sound::EntityCowHurt,
            "Cow should play EntityCowHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::PIG, None),
            Sound::EntityPigHurt,
            "Pig should play EntityPigHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::SHEEP, None),
            Sound::EntitySheepHurt,
            "Sheep should play EntitySheepHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::HORSE, None),
            Sound::EntityHorseHurt,
            "Horse should play EntityHorseHurt"
        );
    }

    #[test]
    fn test_nether_mob_hurt_sounds() {
        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::PIGLIN, None),
            Sound::EntityPiglinHurt,
            "Piglin should play EntityPiglinHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::HOGLIN, None),
            Sound::EntityHoglinHurt,
            "Hoglin should play EntityHoglinHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::PIGLIN_BRUTE, None),
            Sound::EntityPiglinBruteHurt,
            "Piglin Brute should play EntityPiglinBruteHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::STRIDER, None),
            Sound::EntityStriderHurt,
            "Strider should play EntityStriderHurt"
        );
    }

    #[test]
    fn test_ocean_mob_hurt_sounds() {
        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::GUARDIAN, None),
            Sound::EntityGuardianHurt,
            "Guardian should play EntityGuardianHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::ELDER_GUARDIAN, None),
            Sound::EntityElderGuardianHurt,
            "Elder Guardian should play EntityElderGuardianHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::DOLPHIN, None),
            Sound::EntityDolphinHurt,
            "Dolphin should play EntityDolphinHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::TURTLE, None),
            Sound::EntityTurtleHurt,
            "Turtle should play EntityTurtleHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::COD, None),
            Sound::EntityCodHurt,
            "Cod should play EntityCodHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::SALMON, None),
            Sound::EntitySalmonHurt,
            "Salmon should play EntitySalmonHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::PUFFERFISH, None),
            Sound::EntityPufferFishHurt,
            "Pufferfish should play EntityPufferFishHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::TROPICAL_FISH, None),
            Sound::EntityTropicalFishHurt,
            "Tropical Fish should play EntityTropicalFishHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::SQUID, None),
            Sound::EntitySquidHurt,
            "Squid should play EntitySquidHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::GLOW_SQUID, None),
            Sound::EntityGlowSquidHurt,
            "Glow Squid should play EntityGlowSquidHurt"
        );
    }

    #[test]
    fn test_boss_hurt_sounds() {
        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::ENDER_DRAGON, None),
            Sound::EntityEnderDragonHurt,
            "Ender Dragon should play EntityEnderDragonHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::WITHER, None),
            Sound::EntityWitherHurt,
            "Wither should play EntityWitherHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::WARDEN, None),
            Sound::EntityWardenHurt,
            "Warden should play EntityWardenHurt"
        );
    }

    #[test]
    fn test_villager_and_raider_hurt_sounds() {
        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::VILLAGER, None),
            Sound::EntityVillagerHurt,
            "Villager should play EntityVillagerHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::WANDERING_TRADER, None),
            Sound::EntityWanderingTraderHurt,
            "Wandering Trader should play EntityWanderingTraderHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::VINDICATOR, None),
            Sound::EntityVindicatorHurt,
            "Vindicator should play EntityVindicatorHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::EVOKER, None),
            Sound::EntityEvokerHurt,
            "Evoker should play EntityEvokerHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::VEX, None),
            Sound::EntityVexHurt,
            "Vex should play EntityVexHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::ILLUSIONER, None),
            Sound::EntityIllusionerHurt,
            "Illusioner should play EntityIllusionerHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::RAVAGER, None),
            Sound::EntityRavagerHurt,
            "Ravager should play EntityRavagerHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::ZOMBIE_VILLAGER, None),
            Sound::EntityZombieVillagerHurt,
            "Zombie Villager should play EntityZombieVillagerHurt"
        );
    }

    #[test]
    fn test_illager_hurt_sounds() {
        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::PILLAGER, None),
            Sound::EntityPillagerHurt,
            "Pillager should play EntityPillagerHurt"
        );
    }

    #[test]
    fn test_golem_hurt_sounds() {
        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::IRON_GOLEM, None),
            Sound::EntityIronGolemHurt,
            "Iron Golem should play EntityIronGolemHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::SNOW_GOLEM, None),
            Sound::EntitySnowGolemHurt,
            "Snow Golem should play EntitySnowGolemHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::SHULKER, None),
            Sound::EntityShulkerHurt,
            "Shulker should play EntityShulkerHurt"
        );
    }

    #[test]
    fn test_other_mob_hurt_sounds() {
        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::PHANTOM, None),
            Sound::EntityPhantomHurt,
            "Phantom should play EntityPhantomHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::DROWNED, None),
            Sound::EntityDrownedHurt,
            "Drowned should play EntityDrownedHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::HUSK, None),
            Sound::EntityHuskHurt,
            "Husk should play EntityHuskHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::STRAY, None),
            Sound::EntityStrayHurt,
            "Stray should play EntityStrayHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::WITHER_SKELETON, None),
            Sound::EntityWitherSkeletonHurt,
            "Wither Skeleton should play EntityWitherSkeletonHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::ZOMBIFIED_PIGLIN, None),
            Sound::EntityZombifiedPiglinHurt,
            "Zombified Piglin should play EntityZombifiedPiglinHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::ZOGLIN, None),
            Sound::EntityZoglinHurt,
            "Zoglin should play EntityZoglinHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::ENDERMITE, None),
            Sound::EntityEndermiteHurt,
            "Endermite should play EntityEndermiteHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::SILVERFISH, None),
            Sound::EntitySilverfishHurt,
            "Silverfish should play EntitySilverfishHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::BAT, None),
            Sound::EntityBatHurt,
            "Bat should play EntityBatHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::FOX, None),
            Sound::EntityFoxHurt,
            "Fox should play EntityFoxHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::PANDA, None),
            Sound::EntityPandaHurt,
            "Panda should play EntityPandaHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::POLAR_BEAR, None),
            Sound::EntityPolarBearHurt,
            "Polar Bear should play EntityPolarBearHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::TURTLE, None),
            Sound::EntityTurtleHurt,
            "Turtle should play EntityTurtleHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::RABBIT, None),
            Sound::EntityRabbitHurt,
            "Rabbit should play EntityRabbitHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::FROG, None),
            Sound::EntityFrogHurt,
            "Frog should play EntityFrogHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::AXOLOTL, None),
            Sound::EntityAxolotlHurt,
            "Axolotl should play EntityAxolotlHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::GOAT, None),
            Sound::EntityGoatHurt,
            "Goat should play EntityGoatHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::BEE, None),
            Sound::EntityBeeHurt,
            "Bee should play EntityBeeHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::OCELOT, None),
            Sound::EntityOcelotHurt,
            "Ocelot should play EntityOcelotHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::PARROT, None),
            Sound::EntityParrotHurt,
            "Parrot should play EntityParrotHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::LLAMA, None),
            Sound::EntityLlamaHurt,
            "Llama should play EntityLlamaHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::DONKEY, None),
            Sound::EntityDonkeyHurt,
            "Donkey should play EntityDonkeyHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::MULE, None),
            Sound::EntityMuleHurt,
            "Mule should play EntityMuleHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::CAMEL, None),
            Sound::EntityCamelHurt,
            "Camel should play EntityCamelHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::HORSE, None),
            Sound::EntityHorseHurt,
            "Horse should play EntityHorseHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::SKELETON_HORSE, None),
            Sound::EntitySkeletonHorseHurt,
            "Skeleton Horse should play EntitySkeletonHorseHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::ZOMBIE_HORSE, None),
            Sound::EntityZombieHorseHurt,
            "Zombie Horse should play EntityZombieHorseHurt"
        );
    }

    #[test]
    fn test_newer_mob_hurt_sounds() {
        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::ALLAY, None),
            Sound::EntityAllayHurt,
            "Allay should play EntityAllayHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::BREEZE, None),
            Sound::EntityBreezeHurt,
            "Breeze should play EntityBreezeHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::ARMADILLO, None),
            Sound::EntityArmadilloHurt,
            "Armadillo should play EntityArmadilloHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::SNIFFER, None),
            Sound::EntitySnifferHurt,
            "Sniffer should play EntitySnifferHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::TADPOLE, None),
            Sound::EntityTadpoleHurt,
            "Tadpole should play EntityTadpoleHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::NAUTILUS, None),
            Sound::EntityNautilusHurt,
            "Nautilus should play EntityNautilusHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::BOGGED, None),
            Sound::EntityBoggedHurt,
            "Bogged should play EntityBoggedHurt"
        );
    }

    #[test]
    fn test_fallback_to_generic_hurt() {
        // Entities without specific hurt sounds should fall back to generic
        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::ITEM, None),
            Sound::EntityGenericHurt,
            "Unknown entity should fall back to EntityGenericHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::EXPERIENCE_ORB, None),
            Sound::EntityGenericHurt,
            "Experience Orb should fall back to EntityGenericHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::FISHING_BOBBER, None),
            Sound::EntityGenericHurt,
            "Fishing Bobber should fall back to EntityGenericHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::LIGHTNING_BOLT, None),
            Sound::EntityGenericHurt,
            "Lightning Bolt should fall back to EntityGenericHurt"
        );

        assert_eq!(
            crate::entity::living::get_hurt_sound_for_entity(&EntityType::PLAYER, None),
            Sound::EntityPlayerHurt,
            "Player should use EntityPlayerHurt"
        );
    }
}
