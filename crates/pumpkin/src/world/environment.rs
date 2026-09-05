use pumpkin_data::dimension::BedRule;
use pumpkin_data::environment_attribute::{
    Activity, DayTimeline, EarlyGameTimeline, EnvironmentAttribute, MoonPhase, MoonTimeline,
    VillagerScheduleTimeline, sample_activity_track, sample_bool_track, sample_float_track,
    sample_moon_phase_track, sample_step_float_track, sample_unbounded_bool_track,
};
use pumpkin_util::math::position::BlockPos;

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
                    let rain_adj = (rain_level - thunder_level).max(0.0);
                    if rain_adj > 0.0 {
                        let rain_target = 4.0;
                        level += rain_adj * 0.3125 * (rain_target - level);
                    }
                    if thunder_level > 0.0 {
                        let thunder_target = 4.0;
                        level += thunder_level * 0.52734375 * (thunder_target - level);
                    }
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
