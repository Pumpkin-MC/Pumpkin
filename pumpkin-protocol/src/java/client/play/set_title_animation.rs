use pumpkin_data::packet::clientbound::PLAY_SET_TITLES_ANIMATION;

use pumpkin_macros::java_packet;
use serde::Serialize;

#[derive(Serialize)]
#[java_packet(PLAY_SET_TITLES_ANIMATION)]
pub struct CTitleAnimation {
    pub fade_in_ticks: i32,
    pub stay_ticks: i32,
    pub fade_out_ticks: i32,
}

impl CTitleAnimation {
    #[must_use]
    pub const fn new(fade_in_ticks: i32, stay_ticks: i32, fade_out_ticks: i32) -> Self {
        Self {
            fade_in_ticks,
            stay_ticks,
            fade_out_ticks,
        }
    }
}
