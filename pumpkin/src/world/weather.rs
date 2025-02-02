use super::World;
use pumpkin_protocol::client::play::{CGameEvent, GameEvent};

pub struct Weather {
    pub raining: bool,
    pub rain_level: f32,
    pub thunder_level: f32,
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
            raining: false,
            rain_level: 0.0,
            thunder_level: 0.0,
        }
    }

    pub async fn set_rain(&mut self, world: &World, enabled: bool) {
        self.raining = enabled;
        world
            .broadcast_packet_all(&CGameEvent::new(
                if enabled {
                    GameEvent::BeginRaining
                } else {
                    GameEvent::EndRaining
                },
                0.0,
            ))
            .await;
    }

    pub async fn set_rain_level(&mut self, world: &World, level: f32) {
        self.rain_level = level.clamp(0.0, 1.0);
        world
            .broadcast_packet_all(&CGameEvent::new(
                GameEvent::RainLevelChange,
                self.rain_level,
            ))
            .await;
    }

    pub async fn set_thunder_level(&mut self, world: &World, level: f32) {
        self.thunder_level = level.clamp(0.0, 1.0);
        world
            .broadcast_packet_all(&CGameEvent::new(
                GameEvent::ThunderLevelChange,
                self.thunder_level,
            ))
            .await;
    }
}
