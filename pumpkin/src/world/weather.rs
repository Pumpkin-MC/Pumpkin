use super::World;
use pumpkin_protocol::client::play::{CGameEvent, GameEvent};
use rand::Rng;

// Weather timing constants
const RAIN_DELAY_MIN: i32 = 12_000;
const RAIN_DELAY_MAX: i32 = 180_000;
const RAIN_DURATION_MIN: i32 = 12_000;
const RAIN_DURATION_MAX: i32 = 24_000;
const THUNDER_DELAY_MIN: i32 = 12_000;
const THUNDER_DELAY_MAX: i32 = 180_000;
const THUNDER_DURATION_MIN: i32 = 3_600;
const THUNDER_DURATION_MAX: i32 = 15_600;

const WEATHER_TRANSITION_SPEED: f32 = 0.01;

pub struct Weather {
    pub clear_weather_time: i32,
    pub raining: bool,
    pub rain_time: i32,
    pub thundering: bool,
    pub thunder_time: i32,

    pub rain_level: f32,
    pub old_rain_level: f32,
    pub thunder_level: f32,
    pub old_thunder_level: f32,

    pub weather_cycle_enabled: bool,
}

impl Default for Weather {
    fn default() -> Self {
        Self::new()
    }
}

impl Weather {
    #[must_use]
    pub fn new() -> Self {
        Self {
            clear_weather_time: 0,
            raining: false,
            rain_time: 0,
            thundering: false,
            thunder_time: 0,
            rain_level: 0.0,
            old_rain_level: 0.0,
            thunder_level: 0.0,
            old_thunder_level: 0.0,
            weather_cycle_enabled: true,
        }
    }

    pub async fn set_weather_parameters(
        &mut self,
        world: &World,
        clear_time: i32,
        rain_time: i32,
        raining: bool,
        thundering: bool,
    ) {
        let was_raining = self.raining;

        self.clear_weather_time = clear_time;
        self.rain_time = rain_time;
        self.thunder_time = rain_time;
        self.raining = raining;
        self.thundering = thundering;

        if was_raining != raining {
            if was_raining {
                world
                    .broadcast_packet_all(&CGameEvent::new(GameEvent::EndRaining, 0.0))
                    .await;
            } else {
                world
                    .broadcast_packet_all(&CGameEvent::new(GameEvent::BeginRaining, 0.0))
                    .await;
            }
        }
    }

    pub async fn tick_weather(&mut self, world: &World) {
        if !self.weather_cycle_enabled {
            self.advance_weather_cycle();
        }

        // Update visual transitions
        self.old_rain_level = self.rain_level;
        self.old_thunder_level = self.thunder_level;

        if self.raining {
            self.rain_level = (self.rain_level + WEATHER_TRANSITION_SPEED).min(1.0);
        } else {
            self.rain_level = (self.rain_level - WEATHER_TRANSITION_SPEED).max(0.0);
        }

        if self.thundering {
            self.thunder_level = (self.thunder_level + WEATHER_TRANSITION_SPEED).min(1.0);
        } else {
            self.thunder_level = (self.thunder_level - WEATHER_TRANSITION_SPEED).max(0.0);
        }

        // Broadcast level changes if needed
        if (self.old_rain_level - self.rain_level).abs() > f32::EPSILON {
            world
                .broadcast_packet_all(&CGameEvent::new(
                    GameEvent::RainLevelChange,
                    self.rain_level,
                ))
                .await;
        }

        if (self.old_thunder_level - self.thunder_level).abs() > f32::EPSILON {
            world
                .broadcast_packet_all(&CGameEvent::new(
                    GameEvent::ThunderLevelChange,
                    self.thunder_level,
                ))
                .await;
        }
    }

    fn advance_weather_cycle(&mut self) {
        // Removed async since there are no await calls
        if self.clear_weather_time > 0 {
            self.clear_weather_time -= 1;
            self.thunder_time = i32::from(!self.thundering);
            self.rain_time = i32::from(!self.raining);
            self.thundering = false;
            self.raining = false;
        } else {
            // Handle thunder timing
            if self.thunder_time > 0 {
                self.thunder_time -= 1;
                if self.thunder_time == 0 {
                    self.thundering = !self.thundering;
                }
            } else if self.thundering {
                self.thunder_time =
                    rand::thread_rng().gen_range(THUNDER_DURATION_MIN..=THUNDER_DURATION_MAX);
            } else {
                self.thunder_time =
                    rand::thread_rng().gen_range(THUNDER_DELAY_MIN..=THUNDER_DELAY_MAX);
            }

            // Handle rain timing
            if self.rain_time > 0 {
                self.rain_time -= 1;
                if self.rain_time == 0 {
                    self.raining = !self.raining;
                }
            } else if self.raining {
                self.rain_time =
                    rand::thread_rng().gen_range(RAIN_DURATION_MIN..=RAIN_DURATION_MAX);
            } else {
                self.rain_time = rand::thread_rng().gen_range(RAIN_DELAY_MIN..=RAIN_DELAY_MAX);
            }
        }
    }

    pub async fn reset_weather_cycle(&mut self, world: &World) {
        self.set_weather_parameters(world, 0, 0, false, false).await;
    }
}

impl Clone for Weather {
    fn clone(&self) -> Self {
        Self {
            clear_weather_time: self.clear_weather_time,
            raining: self.raining,
            rain_time: self.rain_time,
            thundering: self.thundering,
            thunder_time: self.thunder_time,
            rain_level: self.rain_level,
            old_rain_level: self.old_rain_level,
            thunder_level: self.thunder_level,
            old_thunder_level: self.old_thunder_level,
            weather_cycle_enabled: self.weather_cycle_enabled,
        }
    }
}
