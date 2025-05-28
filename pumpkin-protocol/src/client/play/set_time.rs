use pumpkin_data::packet::clientbound::PLAY_SET_TIME;
use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(PLAY_SET_TIME)]
pub struct CUpdateTime {
    world_age: i64,
    time_of_day: i64,
    time_of_day_increasing: bool,
}

impl CUpdateTime {
    pub fn new(world_age: i64, time_of_day: i64, time_of_day_increasing: bool) -> Self {
        Self {
            world_age,
            time_of_day,
            time_of_day_increasing,
        }
    }
}
