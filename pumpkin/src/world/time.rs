use pumpkin_protocol::{bedrock::client::set_time::CSetTime, java::client::play::CUpdateTime};

use crate::net::ClientPlatform;

use super::World;

pub struct LevelTime {
    pub world_age: i64,
    pub time_of_day: i64,
    pub rain_time: i64,
}

impl Default for LevelTime {
    fn default() -> Self {
        Self::new()
    }
}

impl LevelTime {
    #[must_use]
    pub fn new() -> Self {
        Self {
            world_age: 0,
            time_of_day: 0,
            rain_time: 0,
        }
    }

    pub fn tick_time(&mut self) {
        self.world_age += 1;
        self.time_of_day += 1;
        self.rain_time += 1;
    }

    pub async fn send_time(&self, world: &World) {
        let current_players = world.players.read().await;
        for player in current_players.values() {
            match player.client.as_ref() {
                ClientPlatform::Java(java_client) => {
                    java_client
                        .enqueue_packet(&CUpdateTime::new(self.world_age, self.time_of_day, true))
                        .await;
                }
                ClientPlatform::Bedrock(bedrock_client) => {
                    bedrock_client
                        .send_game_packet(&CSetTime::new(self.time_of_day as _))
                        .await;
                }
            }
        }
    }

    pub fn add_time(&mut self, time: i64) {
        self.time_of_day += time;
    }

    pub fn set_time(&mut self, time: i64) {
        self.time_of_day = time;
    }

    #[must_use]
    pub const fn query_daytime(&self) -> i64 {
        self.time_of_day % 24000
    }

    #[must_use]
    pub const fn query_gametime(&self) -> i64 {
        self.world_age
    }

    #[must_use]
    pub const fn query_day(&self) -> i64 {
        self.time_of_day / 24000
    }
}
