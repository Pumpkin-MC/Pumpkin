use pumpkin_data::packet::clientbound::PLAY_PLAYER_ABILITIES;
use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(PLAY_PLAYER_ABILITIES)]
pub struct CPlayerAbilities {
    pub flags: i8,
    pub flying_speed: f32,
    pub field_of_view: f32,
}

impl CPlayerAbilities {
    #[must_use] 
    pub const fn new(flags: i8, flying_speed: f32, field_of_view: f32) -> Self {
        Self {
            flags,
            flying_speed,
            field_of_view,
        }
    }
}
