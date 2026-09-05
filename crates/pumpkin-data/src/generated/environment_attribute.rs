/* This file is generated. Do not edit manually. */
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EnvironmentAttributeType {
    RgbColor,
    ArgbColor,
    Float,
    AngleDegrees,
    Boolean,
    TriState,
    MoonPhase,
    Activity,
    BedRule,
    Particle,
    AmbientParticles,
    BackgroundMusic,
    AmbientSounds,
    Integer,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EnvironmentAttribute {
    VisualFogColor,
    VisualFogStartDistance,
    VisualFogEndDistance,
    VisualSkyFogEndDistance,
    VisualCloudFogEndDistance,
    VisualWaterFogColor,
    VisualWaterFogStartDistance,
    VisualWaterFogEndDistance,
    VisualSkyColor,
    VisualSunriseSunsetColor,
    VisualCloudColor,
    VisualCloudHeight,
    VisualSunAngle,
    VisualMoonAngle,
    VisualStarAngle,
    VisualMoonPhase,
    VisualStarBrightness,
    VisualBlockLightTint,
    VisualSkyLightColor,
    VisualSkyLightFactor,
    VisualNightVisionColor,
    VisualAmbientLightColor,
    VisualDefaultDripstoneParticle,
    VisualAmbientParticles,
    AudioBackgroundMusic,
    AudioMusicVolume,
    AudioAmbientSounds,
    AudioFireflyBushSounds,
    GameplaySkyLightLevel,
    GameplayCanStartRaid,
    GameplayWaterEvaporates,
    GameplayBedRule,
    GameplayRespawnAnchorWorks,
    GameplayNetherPortalSpawnsPiglin,
    GameplayFastLava,
    GameplayIncreasedFireBurnout,
    GameplayEyeblossomOpen,
    GameplayTurtleEggHatchChance,
    GameplayPiglinsZombify,
    GameplaySnowGolemMelts,
    GameplayCreakingActive,
    GameplaySurfaceSlimeSpawnChance,
    GameplayCatWakingUpGiftChance,
    GameplayBeesStayInHive,
    GameplayMonstersBurn,
    GameplayCanPillagerPatrolSpawn,
    GameplayVillagerActivity,
    GameplayBabyVillagerActivity,
}
impl EnvironmentAttribute {
    #[must_use]
    pub const fn id(&self) -> u8 {
        match self {
            Self::VisualFogColor => 0u8,
            Self::VisualFogStartDistance => 1u8,
            Self::VisualFogEndDistance => 2u8,
            Self::VisualSkyFogEndDistance => 3u8,
            Self::VisualCloudFogEndDistance => 4u8,
            Self::VisualWaterFogColor => 5u8,
            Self::VisualWaterFogStartDistance => 6u8,
            Self::VisualWaterFogEndDistance => 7u8,
            Self::VisualSkyColor => 8u8,
            Self::VisualSunriseSunsetColor => 9u8,
            Self::VisualCloudColor => 10u8,
            Self::VisualCloudHeight => 11u8,
            Self::VisualSunAngle => 12u8,
            Self::VisualMoonAngle => 13u8,
            Self::VisualStarAngle => 14u8,
            Self::VisualMoonPhase => 15u8,
            Self::VisualStarBrightness => 16u8,
            Self::VisualBlockLightTint => 17u8,
            Self::VisualSkyLightColor => 18u8,
            Self::VisualSkyLightFactor => 19u8,
            Self::VisualNightVisionColor => 20u8,
            Self::VisualAmbientLightColor => 21u8,
            Self::VisualDefaultDripstoneParticle => 22u8,
            Self::VisualAmbientParticles => 23u8,
            Self::AudioBackgroundMusic => 24u8,
            Self::AudioMusicVolume => 25u8,
            Self::AudioAmbientSounds => 26u8,
            Self::AudioFireflyBushSounds => 27u8,
            Self::GameplaySkyLightLevel => 28u8,
            Self::GameplayCanStartRaid => 29u8,
            Self::GameplayWaterEvaporates => 30u8,
            Self::GameplayBedRule => 31u8,
            Self::GameplayRespawnAnchorWorks => 32u8,
            Self::GameplayNetherPortalSpawnsPiglin => 33u8,
            Self::GameplayFastLava => 34u8,
            Self::GameplayIncreasedFireBurnout => 35u8,
            Self::GameplayEyeblossomOpen => 36u8,
            Self::GameplayTurtleEggHatchChance => 37u8,
            Self::GameplayPiglinsZombify => 38u8,
            Self::GameplaySnowGolemMelts => 39u8,
            Self::GameplayCreakingActive => 40u8,
            Self::GameplaySurfaceSlimeSpawnChance => 41u8,
            Self::GameplayCatWakingUpGiftChance => 42u8,
            Self::GameplayBeesStayInHive => 43u8,
            Self::GameplayMonstersBurn => 44u8,
            Self::GameplayCanPillagerPatrolSpawn => 45u8,
            Self::GameplayVillagerActivity => 46u8,
            Self::GameplayBabyVillagerActivity => 47u8,
        }
    }
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::VisualFogColor => "visual/fog_color",
            Self::VisualFogStartDistance => "visual/fog_start_distance",
            Self::VisualFogEndDistance => "visual/fog_end_distance",
            Self::VisualSkyFogEndDistance => "visual/sky_fog_end_distance",
            Self::VisualCloudFogEndDistance => "visual/cloud_fog_end_distance",
            Self::VisualWaterFogColor => "visual/water_fog_color",
            Self::VisualWaterFogStartDistance => "visual/water_fog_start_distance",
            Self::VisualWaterFogEndDistance => "visual/water_fog_end_distance",
            Self::VisualSkyColor => "visual/sky_color",
            Self::VisualSunriseSunsetColor => "visual/sunrise_sunset_color",
            Self::VisualCloudColor => "visual/cloud_color",
            Self::VisualCloudHeight => "visual/cloud_height",
            Self::VisualSunAngle => "visual/sun_angle",
            Self::VisualMoonAngle => "visual/moon_angle",
            Self::VisualStarAngle => "visual/star_angle",
            Self::VisualMoonPhase => "visual/moon_phase",
            Self::VisualStarBrightness => "visual/star_brightness",
            Self::VisualBlockLightTint => "visual/block_light_tint",
            Self::VisualSkyLightColor => "visual/sky_light_color",
            Self::VisualSkyLightFactor => "visual/sky_light_factor",
            Self::VisualNightVisionColor => "visual/night_vision_color",
            Self::VisualAmbientLightColor => "visual/ambient_light_color",
            Self::VisualDefaultDripstoneParticle => "visual/default_dripstone_particle",
            Self::VisualAmbientParticles => "visual/ambient_particles",
            Self::AudioBackgroundMusic => "audio/background_music",
            Self::AudioMusicVolume => "audio/music_volume",
            Self::AudioAmbientSounds => "audio/ambient_sounds",
            Self::AudioFireflyBushSounds => "audio/firefly_bush_sounds",
            Self::GameplaySkyLightLevel => "gameplay/sky_light_level",
            Self::GameplayCanStartRaid => "gameplay/can_start_raid",
            Self::GameplayWaterEvaporates => "gameplay/water_evaporates",
            Self::GameplayBedRule => "gameplay/bed_rule",
            Self::GameplayRespawnAnchorWorks => "gameplay/respawn_anchor_works",
            Self::GameplayNetherPortalSpawnsPiglin => "gameplay/nether_portal_spawns_piglin",
            Self::GameplayFastLava => "gameplay/fast_lava",
            Self::GameplayIncreasedFireBurnout => "gameplay/increased_fire_burnout",
            Self::GameplayEyeblossomOpen => "gameplay/eyeblossom_open",
            Self::GameplayTurtleEggHatchChance => "gameplay/turtle_egg_hatch_chance",
            Self::GameplayPiglinsZombify => "gameplay/piglins_zombify",
            Self::GameplaySnowGolemMelts => "gameplay/snow_golem_melts",
            Self::GameplayCreakingActive => "gameplay/creaking_active",
            Self::GameplaySurfaceSlimeSpawnChance => "gameplay/surface_slime_spawn_chance",
            Self::GameplayCatWakingUpGiftChance => "gameplay/cat_waking_up_gift_chance",
            Self::GameplayBeesStayInHive => "gameplay/bees_stay_in_hive",
            Self::GameplayMonstersBurn => "gameplay/monsters_burn",
            Self::GameplayCanPillagerPatrolSpawn => "gameplay/can_pillager_patrol_spawn",
            Self::GameplayVillagerActivity => "gameplay/villager_activity",
            Self::GameplayBabyVillagerActivity => "gameplay/baby_villager_activity",
        }
    }
    #[must_use]
    pub const fn resource_location(&self) -> &'static str {
        match self {
            Self::VisualFogColor => "minecraft:visual/fog_color",
            Self::VisualFogStartDistance => "minecraft:visual/fog_start_distance",
            Self::VisualFogEndDistance => "minecraft:visual/fog_end_distance",
            Self::VisualSkyFogEndDistance => "minecraft:visual/sky_fog_end_distance",
            Self::VisualCloudFogEndDistance => "minecraft:visual/cloud_fog_end_distance",
            Self::VisualWaterFogColor => "minecraft:visual/water_fog_color",
            Self::VisualWaterFogStartDistance => "minecraft:visual/water_fog_start_distance",
            Self::VisualWaterFogEndDistance => "minecraft:visual/water_fog_end_distance",
            Self::VisualSkyColor => "minecraft:visual/sky_color",
            Self::VisualSunriseSunsetColor => "minecraft:visual/sunrise_sunset_color",
            Self::VisualCloudColor => "minecraft:visual/cloud_color",
            Self::VisualCloudHeight => "minecraft:visual/cloud_height",
            Self::VisualSunAngle => "minecraft:visual/sun_angle",
            Self::VisualMoonAngle => "minecraft:visual/moon_angle",
            Self::VisualStarAngle => "minecraft:visual/star_angle",
            Self::VisualMoonPhase => "minecraft:visual/moon_phase",
            Self::VisualStarBrightness => "minecraft:visual/star_brightness",
            Self::VisualBlockLightTint => "minecraft:visual/block_light_tint",
            Self::VisualSkyLightColor => "minecraft:visual/sky_light_color",
            Self::VisualSkyLightFactor => "minecraft:visual/sky_light_factor",
            Self::VisualNightVisionColor => "minecraft:visual/night_vision_color",
            Self::VisualAmbientLightColor => "minecraft:visual/ambient_light_color",
            Self::VisualDefaultDripstoneParticle => "minecraft:visual/default_dripstone_particle",
            Self::VisualAmbientParticles => "minecraft:visual/ambient_particles",
            Self::AudioBackgroundMusic => "minecraft:audio/background_music",
            Self::AudioMusicVolume => "minecraft:audio/music_volume",
            Self::AudioAmbientSounds => "minecraft:audio/ambient_sounds",
            Self::AudioFireflyBushSounds => "minecraft:audio/firefly_bush_sounds",
            Self::GameplaySkyLightLevel => "minecraft:gameplay/sky_light_level",
            Self::GameplayCanStartRaid => "minecraft:gameplay/can_start_raid",
            Self::GameplayWaterEvaporates => "minecraft:gameplay/water_evaporates",
            Self::GameplayBedRule => "minecraft:gameplay/bed_rule",
            Self::GameplayRespawnAnchorWorks => "minecraft:gameplay/respawn_anchor_works",
            Self::GameplayNetherPortalSpawnsPiglin => {
                "minecraft:gameplay/nether_portal_spawns_piglin"
            }
            Self::GameplayFastLava => "minecraft:gameplay/fast_lava",
            Self::GameplayIncreasedFireBurnout => "minecraft:gameplay/increased_fire_burnout",
            Self::GameplayEyeblossomOpen => "minecraft:gameplay/eyeblossom_open",
            Self::GameplayTurtleEggHatchChance => "minecraft:gameplay/turtle_egg_hatch_chance",
            Self::GameplayPiglinsZombify => "minecraft:gameplay/piglins_zombify",
            Self::GameplaySnowGolemMelts => "minecraft:gameplay/snow_golem_melts",
            Self::GameplayCreakingActive => "minecraft:gameplay/creaking_active",
            Self::GameplaySurfaceSlimeSpawnChance => {
                "minecraft:gameplay/surface_slime_spawn_chance"
            }
            Self::GameplayCatWakingUpGiftChance => "minecraft:gameplay/cat_waking_up_gift_chance",
            Self::GameplayBeesStayInHive => "minecraft:gameplay/bees_stay_in_hive",
            Self::GameplayMonstersBurn => "minecraft:gameplay/monsters_burn",
            Self::GameplayCanPillagerPatrolSpawn => "minecraft:gameplay/can_pillager_patrol_spawn",
            Self::GameplayVillagerActivity => "minecraft:gameplay/villager_activity",
            Self::GameplayBabyVillagerActivity => "minecraft:gameplay/baby_villager_activity",
        }
    }
    #[must_use]
    pub const fn attribute_type(&self) -> EnvironmentAttributeType {
        match self {
            Self::VisualFogColor => EnvironmentAttributeType::RgbColor,
            Self::VisualFogStartDistance => EnvironmentAttributeType::Float,
            Self::VisualFogEndDistance => EnvironmentAttributeType::Float,
            Self::VisualSkyFogEndDistance => EnvironmentAttributeType::Float,
            Self::VisualCloudFogEndDistance => EnvironmentAttributeType::Float,
            Self::VisualWaterFogColor => EnvironmentAttributeType::RgbColor,
            Self::VisualWaterFogStartDistance => EnvironmentAttributeType::Float,
            Self::VisualWaterFogEndDistance => EnvironmentAttributeType::Float,
            Self::VisualSkyColor => EnvironmentAttributeType::RgbColor,
            Self::VisualSunriseSunsetColor => EnvironmentAttributeType::ArgbColor,
            Self::VisualCloudColor => EnvironmentAttributeType::ArgbColor,
            Self::VisualCloudHeight => EnvironmentAttributeType::Float,
            Self::VisualSunAngle => EnvironmentAttributeType::AngleDegrees,
            Self::VisualMoonAngle => EnvironmentAttributeType::AngleDegrees,
            Self::VisualStarAngle => EnvironmentAttributeType::AngleDegrees,
            Self::VisualMoonPhase => EnvironmentAttributeType::MoonPhase,
            Self::VisualStarBrightness => EnvironmentAttributeType::Float,
            Self::VisualBlockLightTint => EnvironmentAttributeType::RgbColor,
            Self::VisualSkyLightColor => EnvironmentAttributeType::RgbColor,
            Self::VisualSkyLightFactor => EnvironmentAttributeType::Float,
            Self::VisualNightVisionColor => EnvironmentAttributeType::RgbColor,
            Self::VisualAmbientLightColor => EnvironmentAttributeType::RgbColor,
            Self::VisualDefaultDripstoneParticle => EnvironmentAttributeType::Particle,
            Self::VisualAmbientParticles => EnvironmentAttributeType::AmbientParticles,
            Self::AudioBackgroundMusic => EnvironmentAttributeType::BackgroundMusic,
            Self::AudioMusicVolume => EnvironmentAttributeType::Float,
            Self::AudioAmbientSounds => EnvironmentAttributeType::AmbientSounds,
            Self::AudioFireflyBushSounds => EnvironmentAttributeType::Boolean,
            Self::GameplaySkyLightLevel => EnvironmentAttributeType::Float,
            Self::GameplayCanStartRaid => EnvironmentAttributeType::Boolean,
            Self::GameplayWaterEvaporates => EnvironmentAttributeType::Boolean,
            Self::GameplayBedRule => EnvironmentAttributeType::BedRule,
            Self::GameplayRespawnAnchorWorks => EnvironmentAttributeType::Boolean,
            Self::GameplayNetherPortalSpawnsPiglin => EnvironmentAttributeType::Boolean,
            Self::GameplayFastLava => EnvironmentAttributeType::Boolean,
            Self::GameplayIncreasedFireBurnout => EnvironmentAttributeType::Boolean,
            Self::GameplayEyeblossomOpen => EnvironmentAttributeType::TriState,
            Self::GameplayTurtleEggHatchChance => EnvironmentAttributeType::Float,
            Self::GameplayPiglinsZombify => EnvironmentAttributeType::Boolean,
            Self::GameplaySnowGolemMelts => EnvironmentAttributeType::Boolean,
            Self::GameplayCreakingActive => EnvironmentAttributeType::Boolean,
            Self::GameplaySurfaceSlimeSpawnChance => EnvironmentAttributeType::Float,
            Self::GameplayCatWakingUpGiftChance => EnvironmentAttributeType::Float,
            Self::GameplayBeesStayInHive => EnvironmentAttributeType::Boolean,
            Self::GameplayMonstersBurn => EnvironmentAttributeType::Boolean,
            Self::GameplayCanPillagerPatrolSpawn => EnvironmentAttributeType::Boolean,
            Self::GameplayVillagerActivity => EnvironmentAttributeType::Activity,
            Self::GameplayBabyVillagerActivity => EnvironmentAttributeType::Activity,
        }
    }
    #[must_use]
    pub const fn is_syncable(&self) -> bool {
        match self {
            Self::VisualFogColor => true,
            Self::VisualFogStartDistance => true,
            Self::VisualFogEndDistance => true,
            Self::VisualSkyFogEndDistance => true,
            Self::VisualCloudFogEndDistance => true,
            Self::VisualWaterFogColor => true,
            Self::VisualWaterFogStartDistance => true,
            Self::VisualWaterFogEndDistance => true,
            Self::VisualSkyColor => true,
            Self::VisualSunriseSunsetColor => true,
            Self::VisualCloudColor => true,
            Self::VisualCloudHeight => true,
            Self::VisualSunAngle => true,
            Self::VisualMoonAngle => true,
            Self::VisualStarAngle => true,
            Self::VisualMoonPhase => true,
            Self::VisualStarBrightness => true,
            Self::VisualBlockLightTint => true,
            Self::VisualSkyLightColor => true,
            Self::VisualSkyLightFactor => true,
            Self::VisualNightVisionColor => true,
            Self::VisualAmbientLightColor => true,
            Self::VisualDefaultDripstoneParticle => true,
            Self::VisualAmbientParticles => true,
            Self::AudioBackgroundMusic => true,
            Self::AudioMusicVolume => true,
            Self::AudioAmbientSounds => true,
            Self::AudioFireflyBushSounds => true,
            Self::GameplaySkyLightLevel => true,
            Self::GameplayCanStartRaid => false,
            Self::GameplayWaterEvaporates => true,
            Self::GameplayBedRule => false,
            Self::GameplayRespawnAnchorWorks => false,
            Self::GameplayNetherPortalSpawnsPiglin => false,
            Self::GameplayFastLava => true,
            Self::GameplayIncreasedFireBurnout => false,
            Self::GameplayEyeblossomOpen => false,
            Self::GameplayTurtleEggHatchChance => false,
            Self::GameplayPiglinsZombify => true,
            Self::GameplaySnowGolemMelts => false,
            Self::GameplayCreakingActive => true,
            Self::GameplaySurfaceSlimeSpawnChance => false,
            Self::GameplayCatWakingUpGiftChance => false,
            Self::GameplayBeesStayInHive => false,
            Self::GameplayMonstersBurn => false,
            Self::GameplayCanPillagerPatrolSpawn => false,
            Self::GameplayVillagerActivity => false,
            Self::GameplayBabyVillagerActivity => false,
        }
    }
    #[must_use]
    pub const fn is_positional(&self) -> bool {
        match self {
            Self::VisualFogColor => true,
            Self::VisualFogStartDistance => true,
            Self::VisualFogEndDistance => true,
            Self::VisualSkyFogEndDistance => true,
            Self::VisualCloudFogEndDistance => true,
            Self::VisualWaterFogColor => true,
            Self::VisualWaterFogStartDistance => true,
            Self::VisualWaterFogEndDistance => true,
            Self::VisualSkyColor => true,
            Self::VisualSunriseSunsetColor => true,
            Self::VisualCloudColor => true,
            Self::VisualCloudHeight => true,
            Self::VisualSunAngle => true,
            Self::VisualMoonAngle => true,
            Self::VisualStarAngle => true,
            Self::VisualMoonPhase => true,
            Self::VisualStarBrightness => true,
            Self::VisualBlockLightTint => true,
            Self::VisualSkyLightColor => true,
            Self::VisualSkyLightFactor => true,
            Self::VisualNightVisionColor => true,
            Self::VisualAmbientLightColor => true,
            Self::VisualDefaultDripstoneParticle => true,
            Self::VisualAmbientParticles => true,
            Self::AudioBackgroundMusic => true,
            Self::AudioMusicVolume => true,
            Self::AudioAmbientSounds => true,
            Self::AudioFireflyBushSounds => true,
            Self::GameplaySkyLightLevel => false,
            Self::GameplayCanStartRaid => true,
            Self::GameplayWaterEvaporates => true,
            Self::GameplayBedRule => true,
            Self::GameplayRespawnAnchorWorks => true,
            Self::GameplayNetherPortalSpawnsPiglin => true,
            Self::GameplayFastLava => false,
            Self::GameplayIncreasedFireBurnout => true,
            Self::GameplayEyeblossomOpen => true,
            Self::GameplayTurtleEggHatchChance => true,
            Self::GameplayPiglinsZombify => true,
            Self::GameplaySnowGolemMelts => true,
            Self::GameplayCreakingActive => true,
            Self::GameplaySurfaceSlimeSpawnChance => true,
            Self::GameplayCatWakingUpGiftChance => true,
            Self::GameplayBeesStayInHive => true,
            Self::GameplayMonstersBurn => true,
            Self::GameplayCanPillagerPatrolSpawn => true,
            Self::GameplayVillagerActivity => true,
            Self::GameplayBabyVillagerActivity => true,
        }
    }
    #[must_use]
    pub const fn is_spatially_interpolated(&self) -> bool {
        match self {
            Self::VisualFogColor => true,
            Self::VisualFogStartDistance => true,
            Self::VisualFogEndDistance => true,
            Self::VisualSkyFogEndDistance => true,
            Self::VisualCloudFogEndDistance => true,
            Self::VisualWaterFogColor => true,
            Self::VisualWaterFogStartDistance => true,
            Self::VisualWaterFogEndDistance => true,
            Self::VisualSkyColor => true,
            Self::VisualSunriseSunsetColor => true,
            Self::VisualCloudColor => true,
            Self::VisualCloudHeight => true,
            Self::VisualSunAngle => true,
            Self::VisualMoonAngle => true,
            Self::VisualStarAngle => true,
            Self::VisualMoonPhase => false,
            Self::VisualStarBrightness => true,
            Self::VisualBlockLightTint => true,
            Self::VisualSkyLightColor => true,
            Self::VisualSkyLightFactor => true,
            Self::VisualNightVisionColor => true,
            Self::VisualAmbientLightColor => true,
            Self::VisualDefaultDripstoneParticle => false,
            Self::VisualAmbientParticles => false,
            Self::AudioBackgroundMusic => false,
            Self::AudioMusicVolume => false,
            Self::AudioAmbientSounds => false,
            Self::AudioFireflyBushSounds => false,
            Self::GameplaySkyLightLevel => false,
            Self::GameplayCanStartRaid => false,
            Self::GameplayWaterEvaporates => false,
            Self::GameplayBedRule => false,
            Self::GameplayRespawnAnchorWorks => false,
            Self::GameplayNetherPortalSpawnsPiglin => false,
            Self::GameplayFastLava => false,
            Self::GameplayIncreasedFireBurnout => false,
            Self::GameplayEyeblossomOpen => false,
            Self::GameplayTurtleEggHatchChance => false,
            Self::GameplayPiglinsZombify => false,
            Self::GameplaySnowGolemMelts => false,
            Self::GameplayCreakingActive => false,
            Self::GameplaySurfaceSlimeSpawnChance => false,
            Self::GameplayCatWakingUpGiftChance => false,
            Self::GameplayBeesStayInHive => false,
            Self::GameplayMonstersBurn => false,
            Self::GameplayCanPillagerPatrolSpawn => false,
            Self::GameplayVillagerActivity => false,
            Self::GameplayBabyVillagerActivity => false,
        }
    }
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "visual/fog_color" | "minecraft:visual/fog_color" => Some(Self::VisualFogColor),
            "visual/fog_start_distance" | "minecraft:visual/fog_start_distance" => {
                Some(Self::VisualFogStartDistance)
            }
            "visual/fog_end_distance" | "minecraft:visual/fog_end_distance" => {
                Some(Self::VisualFogEndDistance)
            }
            "visual/sky_fog_end_distance" | "minecraft:visual/sky_fog_end_distance" => {
                Some(Self::VisualSkyFogEndDistance)
            }
            "visual/cloud_fog_end_distance" | "minecraft:visual/cloud_fog_end_distance" => {
                Some(Self::VisualCloudFogEndDistance)
            }
            "visual/water_fog_color" | "minecraft:visual/water_fog_color" => {
                Some(Self::VisualWaterFogColor)
            }
            "visual/water_fog_start_distance" | "minecraft:visual/water_fog_start_distance" => {
                Some(Self::VisualWaterFogStartDistance)
            }
            "visual/water_fog_end_distance" | "minecraft:visual/water_fog_end_distance" => {
                Some(Self::VisualWaterFogEndDistance)
            }
            "visual/sky_color" | "minecraft:visual/sky_color" => Some(Self::VisualSkyColor),
            "visual/sunrise_sunset_color" | "minecraft:visual/sunrise_sunset_color" => {
                Some(Self::VisualSunriseSunsetColor)
            }
            "visual/cloud_color" | "minecraft:visual/cloud_color" => Some(Self::VisualCloudColor),
            "visual/cloud_height" | "minecraft:visual/cloud_height" => {
                Some(Self::VisualCloudHeight)
            }
            "visual/sun_angle" | "minecraft:visual/sun_angle" => Some(Self::VisualSunAngle),
            "visual/moon_angle" | "minecraft:visual/moon_angle" => Some(Self::VisualMoonAngle),
            "visual/star_angle" | "minecraft:visual/star_angle" => Some(Self::VisualStarAngle),
            "visual/moon_phase" | "minecraft:visual/moon_phase" => Some(Self::VisualMoonPhase),
            "visual/star_brightness" | "minecraft:visual/star_brightness" => {
                Some(Self::VisualStarBrightness)
            }
            "visual/block_light_tint" | "minecraft:visual/block_light_tint" => {
                Some(Self::VisualBlockLightTint)
            }
            "visual/sky_light_color" | "minecraft:visual/sky_light_color" => {
                Some(Self::VisualSkyLightColor)
            }
            "visual/sky_light_factor" | "minecraft:visual/sky_light_factor" => {
                Some(Self::VisualSkyLightFactor)
            }
            "visual/night_vision_color" | "minecraft:visual/night_vision_color" => {
                Some(Self::VisualNightVisionColor)
            }
            "visual/ambient_light_color" | "minecraft:visual/ambient_light_color" => {
                Some(Self::VisualAmbientLightColor)
            }
            "visual/default_dripstone_particle" | "minecraft:visual/default_dripstone_particle" => {
                Some(Self::VisualDefaultDripstoneParticle)
            }
            "visual/ambient_particles" | "minecraft:visual/ambient_particles" => {
                Some(Self::VisualAmbientParticles)
            }
            "audio/background_music" | "minecraft:audio/background_music" => {
                Some(Self::AudioBackgroundMusic)
            }
            "audio/music_volume" | "minecraft:audio/music_volume" => Some(Self::AudioMusicVolume),
            "audio/ambient_sounds" | "minecraft:audio/ambient_sounds" => {
                Some(Self::AudioAmbientSounds)
            }
            "audio/firefly_bush_sounds" | "minecraft:audio/firefly_bush_sounds" => {
                Some(Self::AudioFireflyBushSounds)
            }
            "gameplay/sky_light_level" | "minecraft:gameplay/sky_light_level" => {
                Some(Self::GameplaySkyLightLevel)
            }
            "gameplay/can_start_raid" | "minecraft:gameplay/can_start_raid" => {
                Some(Self::GameplayCanStartRaid)
            }
            "gameplay/water_evaporates" | "minecraft:gameplay/water_evaporates" => {
                Some(Self::GameplayWaterEvaporates)
            }
            "gameplay/bed_rule" | "minecraft:gameplay/bed_rule" => Some(Self::GameplayBedRule),
            "gameplay/respawn_anchor_works" | "minecraft:gameplay/respawn_anchor_works" => {
                Some(Self::GameplayRespawnAnchorWorks)
            }
            "gameplay/nether_portal_spawns_piglin"
            | "minecraft:gameplay/nether_portal_spawns_piglin" => {
                Some(Self::GameplayNetherPortalSpawnsPiglin)
            }
            "gameplay/fast_lava" | "minecraft:gameplay/fast_lava" => Some(Self::GameplayFastLava),
            "gameplay/increased_fire_burnout" | "minecraft:gameplay/increased_fire_burnout" => {
                Some(Self::GameplayIncreasedFireBurnout)
            }
            "gameplay/eyeblossom_open" | "minecraft:gameplay/eyeblossom_open" => {
                Some(Self::GameplayEyeblossomOpen)
            }
            "gameplay/turtle_egg_hatch_chance" | "minecraft:gameplay/turtle_egg_hatch_chance" => {
                Some(Self::GameplayTurtleEggHatchChance)
            }
            "gameplay/piglins_zombify" | "minecraft:gameplay/piglins_zombify" => {
                Some(Self::GameplayPiglinsZombify)
            }
            "gameplay/snow_golem_melts" | "minecraft:gameplay/snow_golem_melts" => {
                Some(Self::GameplaySnowGolemMelts)
            }
            "gameplay/creaking_active" | "minecraft:gameplay/creaking_active" => {
                Some(Self::GameplayCreakingActive)
            }
            "gameplay/surface_slime_spawn_chance"
            | "minecraft:gameplay/surface_slime_spawn_chance" => {
                Some(Self::GameplaySurfaceSlimeSpawnChance)
            }
            "gameplay/cat_waking_up_gift_chance"
            | "minecraft:gameplay/cat_waking_up_gift_chance" => {
                Some(Self::GameplayCatWakingUpGiftChance)
            }
            "gameplay/bees_stay_in_hive" | "minecraft:gameplay/bees_stay_in_hive" => {
                Some(Self::GameplayBeesStayInHive)
            }
            "gameplay/monsters_burn" | "minecraft:gameplay/monsters_burn" => {
                Some(Self::GameplayMonstersBurn)
            }
            "gameplay/can_pillager_patrol_spawn"
            | "minecraft:gameplay/can_pillager_patrol_spawn" => {
                Some(Self::GameplayCanPillagerPatrolSpawn)
            }
            "gameplay/villager_activity" | "minecraft:gameplay/villager_activity" => {
                Some(Self::GameplayVillagerActivity)
            }
            "gameplay/baby_villager_activity" | "minecraft:gameplay/baby_villager_activity" => {
                Some(Self::GameplayBabyVillagerActivity)
            }
            _ => None,
        }
    }
    #[must_use]
    pub const fn from_id(id: u8) -> Option<Self> {
        match id {
            0u8 => Some(Self::VisualFogColor),
            1u8 => Some(Self::VisualFogStartDistance),
            2u8 => Some(Self::VisualFogEndDistance),
            3u8 => Some(Self::VisualSkyFogEndDistance),
            4u8 => Some(Self::VisualCloudFogEndDistance),
            5u8 => Some(Self::VisualWaterFogColor),
            6u8 => Some(Self::VisualWaterFogStartDistance),
            7u8 => Some(Self::VisualWaterFogEndDistance),
            8u8 => Some(Self::VisualSkyColor),
            9u8 => Some(Self::VisualSunriseSunsetColor),
            10u8 => Some(Self::VisualCloudColor),
            11u8 => Some(Self::VisualCloudHeight),
            12u8 => Some(Self::VisualSunAngle),
            13u8 => Some(Self::VisualMoonAngle),
            14u8 => Some(Self::VisualStarAngle),
            15u8 => Some(Self::VisualMoonPhase),
            16u8 => Some(Self::VisualStarBrightness),
            17u8 => Some(Self::VisualBlockLightTint),
            18u8 => Some(Self::VisualSkyLightColor),
            19u8 => Some(Self::VisualSkyLightFactor),
            20u8 => Some(Self::VisualNightVisionColor),
            21u8 => Some(Self::VisualAmbientLightColor),
            22u8 => Some(Self::VisualDefaultDripstoneParticle),
            23u8 => Some(Self::VisualAmbientParticles),
            24u8 => Some(Self::AudioBackgroundMusic),
            25u8 => Some(Self::AudioMusicVolume),
            26u8 => Some(Self::AudioAmbientSounds),
            27u8 => Some(Self::AudioFireflyBushSounds),
            28u8 => Some(Self::GameplaySkyLightLevel),
            29u8 => Some(Self::GameplayCanStartRaid),
            30u8 => Some(Self::GameplayWaterEvaporates),
            31u8 => Some(Self::GameplayBedRule),
            32u8 => Some(Self::GameplayRespawnAnchorWorks),
            33u8 => Some(Self::GameplayNetherPortalSpawnsPiglin),
            34u8 => Some(Self::GameplayFastLava),
            35u8 => Some(Self::GameplayIncreasedFireBurnout),
            36u8 => Some(Self::GameplayEyeblossomOpen),
            37u8 => Some(Self::GameplayTurtleEggHatchChance),
            38u8 => Some(Self::GameplayPiglinsZombify),
            39u8 => Some(Self::GameplaySnowGolemMelts),
            40u8 => Some(Self::GameplayCreakingActive),
            41u8 => Some(Self::GameplaySurfaceSlimeSpawnChance),
            42u8 => Some(Self::GameplayCatWakingUpGiftChance),
            43u8 => Some(Self::GameplayBeesStayInHive),
            44u8 => Some(Self::GameplayMonstersBurn),
            45u8 => Some(Self::GameplayCanPillagerPatrolSpawn),
            46u8 => Some(Self::GameplayVillagerActivity),
            47u8 => Some(Self::GameplayBabyVillagerActivity),
            _ => None,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloatKeyframe {
    pub ticks: i32,
    pub value: f32,
}
impl FloatKeyframe {
    pub const fn new(ticks: i32, value: f32) -> Self {
        Self { ticks, value }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoolKeyframe {
    pub ticks: i32,
    pub value: bool,
}
impl BoolKeyframe {
    pub const fn new(ticks: i32, value: bool) -> Self {
        Self { ticks, value }
    }
}
pub struct DayTimeline;
impl DayTimeline {
    pub const PERIOD_TICKS: i32 = 24000i32;
    pub const SKY_LIGHT_LEVEL_KEYFRAMES: &'static [FloatKeyframe] = &[
        FloatKeyframe::new(133i32, 1f32),
        FloatKeyframe::new(11867i32, 1f32),
        FloatKeyframe::new(13670i32, 0.26666668f32),
        FloatKeyframe::new(22330i32, 0.26666668f32),
    ];
    pub const MONSTERS_BURN_KEYFRAMES: &'static [BoolKeyframe] = &[
        BoolKeyframe::new(12542i32, false),
        BoolKeyframe::new(23460i32, true),
    ];
    pub const BEES_STAY_IN_HIVE_KEYFRAMES: &'static [BoolKeyframe] = &[
        BoolKeyframe::new(12542i32, true),
        BoolKeyframe::new(23460i32, false),
    ];
    pub const CREAKING_ACTIVE_KEYFRAMES: &'static [BoolKeyframe] = &[
        BoolKeyframe::new(12600i32, true),
        BoolKeyframe::new(23401i32, false),
    ];
    pub const EYEBLOSSOM_OPEN_KEYFRAMES: &'static [BoolKeyframe] = &[
        BoolKeyframe::new(12600i32, true),
        BoolKeyframe::new(23401i32, false),
    ];
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MoonPhase {
    FullMoon,
    WaningGibbous,
    ThirdQuarter,
    WaningCrescent,
    NewMoon,
    WaxingCrescent,
    FirstQuarter,
    WaxingGibbous,
}
impl MoonPhase {
    #[must_use]
    pub const fn index(&self) -> u8 {
        match self {
            Self::FullMoon => 0,
            Self::WaningGibbous => 1,
            Self::ThirdQuarter => 2,
            Self::WaningCrescent => 3,
            Self::NewMoon => 4,
            Self::WaxingCrescent => 5,
            Self::FirstQuarter => 6,
            Self::WaxingGibbous => 7,
        }
    }
    #[must_use]
    pub const fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(Self::FullMoon),
            1 => Some(Self::WaningGibbous),
            2 => Some(Self::ThirdQuarter),
            3 => Some(Self::WaningCrescent),
            4 => Some(Self::NewMoon),
            5 => Some(Self::WaxingCrescent),
            6 => Some(Self::FirstQuarter),
            7 => Some(Self::WaxingGibbous),
            _ => None,
        }
    }
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::FullMoon => "full_moon",
            Self::WaningGibbous => "waning_gibbous",
            Self::ThirdQuarter => "third_quarter",
            Self::WaningCrescent => "waning_crescent",
            Self::NewMoon => "new_moon",
            Self::WaxingCrescent => "waxing_crescent",
            Self::FirstQuarter => "first_quarter",
            Self::WaxingGibbous => "waxing_gibbous",
        }
    }
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "full_moon" => Some(Self::FullMoon),
            "waning_gibbous" => Some(Self::WaningGibbous),
            "third_quarter" => Some(Self::ThirdQuarter),
            "waning_crescent" => Some(Self::WaningCrescent),
            "new_moon" => Some(Self::NewMoon),
            "waxing_crescent" => Some(Self::WaxingCrescent),
            "first_quarter" => Some(Self::FirstQuarter),
            "waxing_gibbous" => Some(Self::WaxingGibbous),
            _ => None,
        }
    }
    #[must_use]
    pub const fn start_tick(&self) -> i32 {
        self.index() as i32 * 24000
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoonPhaseKeyframe {
    pub ticks: i32,
    pub value: MoonPhase,
}
impl MoonPhaseKeyframe {
    pub const fn new(ticks: i32, value: MoonPhase) -> Self {
        Self { ticks, value }
    }
}
pub struct MoonTimeline;
impl MoonTimeline {
    pub const PERIOD_TICKS: i32 = 192000i32;
    pub const SURFACE_SLIME_SPAWN_CHANCE_KEYFRAMES: &'static [FloatKeyframe] = &[
        FloatKeyframe::new(0i32, 0.5f32),
        FloatKeyframe::new(24000i32, 0.375f32),
        FloatKeyframe::new(48000i32, 0.25f32),
        FloatKeyframe::new(72000i32, 0.125f32),
        FloatKeyframe::new(96000i32, 0f32),
        FloatKeyframe::new(120000i32, 0.125f32),
        FloatKeyframe::new(144000i32, 0.25f32),
        FloatKeyframe::new(168000i32, 0.375f32),
    ];
    pub const MOON_PHASE_KEYFRAMES: &'static [MoonPhaseKeyframe] = &[
        MoonPhaseKeyframe::new(0i32, MoonPhase::FullMoon),
        MoonPhaseKeyframe::new(24000i32, MoonPhase::WaningGibbous),
        MoonPhaseKeyframe::new(48000i32, MoonPhase::ThirdQuarter),
        MoonPhaseKeyframe::new(72000i32, MoonPhase::WaningCrescent),
        MoonPhaseKeyframe::new(96000i32, MoonPhase::NewMoon),
        MoonPhaseKeyframe::new(120000i32, MoonPhase::WaxingCrescent),
        MoonPhaseKeyframe::new(144000i32, MoonPhase::FirstQuarter),
        MoonPhaseKeyframe::new(168000i32, MoonPhase::WaxingGibbous),
    ];
}
pub struct EarlyGameTimeline;
impl EarlyGameTimeline {
    pub const CAN_PILLAGER_PATROL_SPAWN_KEYFRAMES: &'static [BoolKeyframe] = &[
        BoolKeyframe::new(0i32, false),
        BoolKeyframe::new(120000i32, true),
    ];
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Activity {
    Core,
    Idle,
    Work,
    Play,
    Rest,
    Meet,
    Panic,
    Raid,
    PreRaid,
    Hide,
    Fight,
    Celebrate,
    AdmireItem,
    Avoid,
    Ride,
    PlayDead,
    LongJump,
    Ram,
    Tongue,
    Swim,
    LaySpawn,
    Sniff,
    Investigate,
    Roar,
    Emerge,
    Dig,
}
impl Activity {
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Core => "core",
            Self::Idle => "idle",
            Self::Work => "work",
            Self::Play => "play",
            Self::Rest => "rest",
            Self::Meet => "meet",
            Self::Panic => "panic",
            Self::Raid => "raid",
            Self::PreRaid => "pre_raid",
            Self::Hide => "hide",
            Self::Fight => "fight",
            Self::Celebrate => "celebrate",
            Self::AdmireItem => "admire_item",
            Self::Avoid => "avoid",
            Self::Ride => "ride",
            Self::PlayDead => "play_dead",
            Self::LongJump => "long_jump",
            Self::Ram => "ram",
            Self::Tongue => "tongue",
            Self::Swim => "swim",
            Self::LaySpawn => "lay_spawn",
            Self::Sniff => "sniff",
            Self::Investigate => "investigate",
            Self::Roar => "roar",
            Self::Emerge => "emerge",
            Self::Dig => "dig",
        }
    }
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        let name = name.strip_prefix("minecraft:").unwrap_or(name);
        match name {
            "core" => Some(Self::Core),
            "idle" => Some(Self::Idle),
            "work" => Some(Self::Work),
            "play" => Some(Self::Play),
            "rest" => Some(Self::Rest),
            "meet" => Some(Self::Meet),
            "panic" => Some(Self::Panic),
            "raid" => Some(Self::Raid),
            "pre_raid" => Some(Self::PreRaid),
            "hide" => Some(Self::Hide),
            "fight" => Some(Self::Fight),
            "celebrate" => Some(Self::Celebrate),
            "admire_item" => Some(Self::AdmireItem),
            "avoid" => Some(Self::Avoid),
            "ride" => Some(Self::Ride),
            "play_dead" => Some(Self::PlayDead),
            "long_jump" => Some(Self::LongJump),
            "ram" => Some(Self::Ram),
            "tongue" => Some(Self::Tongue),
            "swim" => Some(Self::Swim),
            "lay_spawn" => Some(Self::LaySpawn),
            "sniff" => Some(Self::Sniff),
            "investigate" => Some(Self::Investigate),
            "roar" => Some(Self::Roar),
            "emerge" => Some(Self::Emerge),
            "dig" => Some(Self::Dig),
            _ => None,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActivityKeyframe {
    pub ticks: i32,
    pub value: Activity,
}
impl ActivityKeyframe {
    pub const fn new(ticks: i32, value: Activity) -> Self {
        Self { ticks, value }
    }
}
pub struct VillagerScheduleTimeline;
impl VillagerScheduleTimeline {
    pub const PERIOD_TICKS: i32 = 24000i32;
    pub const VILLAGER_ACTIVITY_KEYFRAMES: &'static [ActivityKeyframe] = &[
        ActivityKeyframe::new(10i32, Activity::Idle),
        ActivityKeyframe::new(2000i32, Activity::Work),
        ActivityKeyframe::new(9000i32, Activity::Meet),
        ActivityKeyframe::new(11000i32, Activity::Idle),
        ActivityKeyframe::new(12000i32, Activity::Rest),
    ];
    pub const BABY_VILLAGER_ACTIVITY_KEYFRAMES: &'static [ActivityKeyframe] = &[
        ActivityKeyframe::new(10i32, Activity::Idle),
        ActivityKeyframe::new(3000i32, Activity::Play),
        ActivityKeyframe::new(6000i32, Activity::Idle),
        ActivityKeyframe::new(10000i32, Activity::Play),
        ActivityKeyframe::new(12000i32, Activity::Rest),
    ];
}
#[doc = r" Samples a cyclic float keyframe track with linear interpolation across `period_ticks`."]
#[must_use]
pub fn sample_float_track(keyframes: &[FloatKeyframe], period_ticks: i32, ticks: i64) -> f32 {
    if keyframes.is_empty() {
        return 1.0;
    }
    if keyframes.len() == 1 {
        return keyframes[0].value;
    }
    let t = ticks.rem_euclid(period_ticks as i64) as f32;
    let n = keyframes.len();
    let first = &keyframes[0];
    let last = &keyframes[n - 1];
    if t <= first.ticks as f32 {
        let seg_len = (first.ticks + period_ticks - last.ticks) as f32;
        if seg_len <= 0.0 {
            return first.value;
        }
        let alpha = (t + period_ticks as f32 - last.ticks as f32) / seg_len;
        return last.value + alpha * (first.value - last.value);
    }
    if t >= last.ticks as f32 {
        let seg_len = (first.ticks + period_ticks - last.ticks) as f32;
        if seg_len <= 0.0 {
            return last.value;
        }
        let alpha = (t - last.ticks as f32) / seg_len;
        return last.value + alpha * (first.value - last.value);
    }
    for i in 0..n - 1 {
        let k1 = &keyframes[i];
        let k2 = &keyframes[i + 1];
        if t >= k1.ticks as f32 && t <= k2.ticks as f32 {
            let seg_len = (k2.ticks - k1.ticks) as f32;
            if seg_len <= 0.0 {
                return k1.value;
            }
            let alpha = (t - k1.ticks as f32) / seg_len;
            return k1.value + alpha * (k2.value - k1.value);
        }
    }
    last.value
}
#[doc = r" Samples a cyclic float keyframe track with step interpolation (constant easing) across `period_ticks`."]
#[must_use]
pub fn sample_step_float_track(keyframes: &[FloatKeyframe], period_ticks: i32, ticks: i64) -> f32 {
    if keyframes.is_empty() {
        return 0.0;
    }
    let t = ticks.rem_euclid(period_ticks as i64) as i32;
    let mut current = keyframes[keyframes.len() - 1].value;
    for kf in keyframes {
        if t >= kf.ticks {
            current = kf.value;
        } else {
            break;
        }
    }
    current
}
#[doc = r" Samples a cyclic boolean keyframe track (step function) across `period_ticks`."]
#[must_use]
pub fn sample_bool_track(keyframes: &[BoolKeyframe], period_ticks: i32, ticks: i64) -> bool {
    if keyframes.is_empty() {
        return false;
    }
    let t = ticks.rem_euclid(period_ticks as i64) as i32;
    let mut current = keyframes[keyframes.len() - 1].value;
    for kf in keyframes {
        if t >= kf.ticks {
            current = kf.value;
        } else {
            break;
        }
    }
    current
}
#[doc = r" Samples an unbounded (non-cyclic) boolean keyframe track (step function)."]
#[must_use]
pub fn sample_unbounded_bool_track(keyframes: &[BoolKeyframe], ticks: i64) -> bool {
    if keyframes.is_empty() {
        return false;
    }
    let mut current = keyframes[0].value;
    for kf in keyframes {
        if ticks >= kf.ticks as i64 {
            current = kf.value;
        } else {
            break;
        }
    }
    current
}
#[doc = r" Samples a cyclic moon phase keyframe track across `period_ticks`."]
#[must_use]
pub fn sample_moon_phase_track(
    keyframes: &[MoonPhaseKeyframe],
    period_ticks: i32,
    ticks: i64,
) -> MoonPhase {
    if keyframes.is_empty() {
        return MoonPhase::FullMoon;
    }
    let t = ticks.rem_euclid(period_ticks as i64) as i32;
    let mut current = keyframes[keyframes.len() - 1].value;
    for kf in keyframes {
        if t >= kf.ticks {
            current = kf.value;
        } else {
            break;
        }
    }
    current
}
#[doc = r" Samples a cyclic activity keyframe track across `period_ticks`."]
#[must_use]
pub fn sample_activity_track(
    keyframes: &[ActivityKeyframe],
    period_ticks: i32,
    ticks: i64,
) -> Activity {
    if keyframes.is_empty() {
        return Activity::Idle;
    }
    let t = ticks.rem_euclid(period_ticks as i64) as i32;
    let mut current = keyframes[keyframes.len() - 1].value;
    for kf in keyframes {
        if t >= kf.ticks {
            current = kf.value;
        } else {
            break;
        }
    }
    current
}
