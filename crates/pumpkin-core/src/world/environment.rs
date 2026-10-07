use pumpkin_data::dimension::BedRule;
use pumpkin_data::environment_attribute::{
    Activity, DayTimeline, EarlyGameTimeline, EnvironmentAttribute, MoonPhase, MoonTimeline,
    VillagerScheduleTimeline, sample_activity_track, sample_bool_track, sample_float_track,
    sample_moon_phase_track, sample_step_float_track, sample_unbounded_bool_track,
};
use pumpkin_util::math::{lerp, position::BlockPos};

use super::World;

/// Environment attribute accessor for a world, matching vanilla's `EnvironmentAttributeSystem`.
pub struct EnvironmentAttributes<'a> {
    world: &'a World,
}

impl<'a> EnvironmentAttributes<'a> {
    #[must_use]
    pub const fn new(world: &'a World) -> Self {
        Self { world }
    }

    /// Evaluates a float environment attribute at the dimension level (e.g. `EnvironmentAttributes.SKY_LIGHT_LEVEL`).
    #[must_use]
    pub fn get_dimension_value_f32(&self, attribute: EnvironmentAttribute) -> f32 {
        match attribute {
            EnvironmentAttribute::GameplaySkyLightLevel => {
                let base = self.world.dimension.effective_sky_light_level();
                let multiplier =
                    if self.world.dimension.has_skylight && !self.world.dimension.has_fixed_time {
                        sample_float_track(
                            DayTimeline::SKY_LIGHT_LEVEL_KEYFRAMES,
                            DayTimeline::PERIOD_TICKS,
                            self.world.get_time_of_day(),
                        )
                    } else {
                        1.0
                    };
                let mut level = base * multiplier;

                // Weather modifier (WeatherAttributes.RAIN and THUNDER)
                if self.world.dimension.has_skylight {
                    let (rain_level, thunder_level) = {
                        let weather = self
                            .world
                            .weather
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        (weather.rain_level, weather.thunder_level)
                    };
                    level = Self::apply_weather_sky_light(level, rain_level, thunder_level);
                }
                level.clamp(0.0, 15.0)
            }
            EnvironmentAttribute::GameplaySurfaceSlimeSpawnChance => {
                if self.world.dimension.has_skylight && !self.world.dimension.has_fixed_time {
                    sample_step_float_track(
                        MoonTimeline::SURFACE_SLIME_SPAWN_CHANCE_KEYFRAMES,
                        MoonTimeline::PERIOD_TICKS,
                        self.world.get_time_of_day(),
                    )
                } else {
                    0.0
                }
            }
            EnvironmentAttribute::VisualSunAngle => {
                super::calculate_celestial_angle(self.world.get_time_of_day()) * 360.0
            }
            EnvironmentAttribute::VisualMoonAngle => {
                (super::calculate_celestial_angle(self.world.get_time_of_day()) * 360.0 + 180.0)
                    .rem_euclid(360.0)
            }
            _ => 0.0,
        }
    }

    fn apply_weather_sky_light(mut level: f32, rain_level: f32, thunder_level: f32) -> f32 {
        // Level.getThunderLevel scales thunder by rain before WeatherAttributes.addLayer.
        let thunder_level = thunder_level * rain_level;
        let rain_adj = rain_level - thunder_level;
        if rain_adj > 0.0 {
            let rain_target = 4.0;
            let rain_value = lerp(0.3125, level, rain_target);
            level = lerp(rain_adj, level, rain_value);
        }
        if thunder_level > 0.0 {
            let thunder_target = 4.0;
            let thunder_value = lerp(0.52734375, level, thunder_target);
            level = lerp(thunder_level, level, thunder_value);
        }
        level
    }

    /// Evaluates a boolean environment attribute at the dimension level.
    #[must_use]
    pub fn get_dimension_value_bool(&self, attribute: EnvironmentAttribute) -> bool {
        match attribute {
            EnvironmentAttribute::GameplayFastLava => self.world.dimension.fast_lava,
            EnvironmentAttribute::GameplayWaterEvaporates => self.world.dimension.water_evaporates,
            EnvironmentAttribute::GameplayRespawnAnchorWorks => {
                self.world.dimension.respawn_anchor_works
            }
            EnvironmentAttribute::GameplayPiglinsZombify => self.world.dimension.piglins_zombify,
            EnvironmentAttribute::GameplaySnowGolemMelts => self.world.dimension.snow_golem_melts,
            EnvironmentAttribute::GameplayCanStartRaid => self.world.dimension.can_start_raid,
            EnvironmentAttribute::GameplayNetherPortalSpawnsPiglin => {
                self.world.dimension.nether_portal_spawns_piglin
            }
            EnvironmentAttribute::GameplayCanPillagerPatrolSpawn => {
                if self.world.dimension.has_skylight && !self.world.dimension.has_fixed_time {
                    sample_unbounded_bool_track(
                        EarlyGameTimeline::CAN_PILLAGER_PATROL_SPAWN_KEYFRAMES,
                        self.world.get_time_of_day(),
                    )
                } else {
                    false
                }
            }
            EnvironmentAttribute::GameplayMonstersBurn => {
                self.world.dimension.has_skylight
                    && !self.world.dimension.has_fixed_time
                    && sample_bool_track(
                        DayTimeline::MONSTERS_BURN_KEYFRAMES,
                        DayTimeline::PERIOD_TICKS,
                        self.world.get_time_of_day(),
                    )
            }
            EnvironmentAttribute::GameplayBeesStayInHive => {
                let raining = {
                    let weather = self
                        .world
                        .weather
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    weather.raining
                };
                raining
                    || !self.world.dimension.has_skylight
                    || sample_bool_track(
                        DayTimeline::BEES_STAY_IN_HIVE_KEYFRAMES,
                        DayTimeline::PERIOD_TICKS,
                        self.world.get_time_of_day(),
                    )
            }
            EnvironmentAttribute::GameplayCreakingActive => {
                self.world.dimension.has_skylight
                    && !self.world.dimension.has_fixed_time
                    && sample_bool_track(
                        DayTimeline::CREAKING_ACTIVE_KEYFRAMES,
                        DayTimeline::PERIOD_TICKS,
                        self.world.get_time_of_day(),
                    )
            }
            _ => false,
        }
    }

    /// Evaluates a `BedRule` environment attribute at the dimension level.
    #[must_use]
    pub const fn get_dimension_value_bed_rule(&self) -> BedRule {
        self.world.dimension.bed_rule
    }

    /// Evaluates a `TriState` environment attribute at the dimension level.
    #[must_use]
    pub fn get_dimension_value_tri_state(&self, attribute: EnvironmentAttribute) -> Option<bool> {
        match attribute {
            EnvironmentAttribute::GameplayEyeblossomOpen => (self.world.dimension.has_skylight
                && !self.world.dimension.has_fixed_time)
                .then(|| {
                    sample_bool_track(
                        DayTimeline::EYEBLOSSOM_OPEN_KEYFRAMES,
                        DayTimeline::PERIOD_TICKS,
                        self.world.get_time_of_day(),
                    )
                }),
            _ => None,
        }
    }

    /// Evaluates the current `MoonPhase` from the moon timeline.
    #[must_use]
    pub fn get_dimension_value_moon_phase(&self) -> MoonPhase {
        sample_moon_phase_track(
            MoonTimeline::MOON_PHASE_KEYFRAMES,
            MoonTimeline::PERIOD_TICKS,
            self.world.get_time_of_day(),
        )
    }

    /// Evaluates the current villager `Activity` from the villager schedule timeline.
    #[must_use]
    pub fn get_dimension_value_activity(&self, baby: bool) -> Activity {
        if baby {
            sample_activity_track(
                VillagerScheduleTimeline::BABY_VILLAGER_ACTIVITY_KEYFRAMES,
                VillagerScheduleTimeline::PERIOD_TICKS,
                self.world.get_time_of_day(),
            )
        } else {
            sample_activity_track(
                VillagerScheduleTimeline::VILLAGER_ACTIVITY_KEYFRAMES,
                VillagerScheduleTimeline::PERIOD_TICKS,
                self.world.get_time_of_day(),
            )
        }
    }

    /// Evaluates a boolean environment attribute at a position.
    #[must_use]
    pub fn get_value_bool(&self, attribute: EnvironmentAttribute, _pos: &BlockPos) -> bool {
        self.get_dimension_value_bool(attribute)
    }

    /// Evaluates a float environment attribute at a position.
    #[must_use]
    pub fn get_value_f32(&self, attribute: EnvironmentAttribute, _pos: &BlockPos) -> f32 {
        self.get_dimension_value_f32(attribute)
    }

    /// Evaluates a `BedRule` environment attribute at a position.
    #[must_use]
    pub const fn get_value_bed_rule(&self, _pos: &BlockPos) -> BedRule {
        self.get_dimension_value_bed_rule()
    }

    /// Evaluates a `TriState` environment attribute at a position.
    #[must_use]
    pub fn get_value_tri_state(
        &self,
        attribute: EnvironmentAttribute,
        _pos: &BlockPos,
    ) -> Option<bool> {
        self.get_dimension_value_tri_state(attribute)
    }

    /// Evaluates the current `MoonPhase` at a position.
    #[must_use]
    pub fn get_value_moon_phase(&self, _pos: &BlockPos) -> MoonPhase {
        self.get_dimension_value_moon_phase()
    }

    /// Evaluates the current villager `Activity` at a position.
    #[must_use]
    pub fn get_value_activity(&self, baby: bool, _pos: &BlockPos) -> Activity {
        self.get_dimension_value_activity(baby)
    }
}

#[cfg(test)]
mod tests {
    use super::EnvironmentAttributes;

    #[test]
    fn daylight_detector_sky_light_matches_vanilla_steady_weather() {
        // Captured from vanilla 26.3 EnvironmentAttributeSystem with WeatherAttributes layers.
        for (base, rain, thunder, expected) in [
            (15.0, 0.0, 0.0, 15.0f32),
            (15.0, 1.0, 0.0, 11.5625),
            (15.0, 1.0, 1.0, 9.199_219),
            (4.0, 1.0, 1.0, 4.0),
        ] {
            assert_eq!(
                EnvironmentAttributes::apply_weather_sky_light(base, rain, thunder).to_bits(),
                expected.to_bits(),
                "base={base}, rain={rain}, thunder={thunder}",
            );
        }
    }

    #[test]
    fn daylight_detector_sky_light_matches_vanilla_thunder_transitions() {
        // WeatherAccess uses Level.getThunderLevel(1), which scales raw thunder by rain.
        for (rain, thunder, expected) in [
            (0.2, 0.2, 14.229_57f32),
            (0.5, 0.5, 12.803_726),
            (0.0, 1.0, 15.0),
            (0.2, 1.0, 13.839_844),
        ] {
            assert_eq!(
                EnvironmentAttributes::apply_weather_sky_light(15.0, rain, thunder).to_bits(),
                expected.to_bits(),
                "rain={rain}, thunder={thunder}",
            );
        }
    }
}
