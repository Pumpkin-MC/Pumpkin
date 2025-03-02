use pumpkin_data::packet::clientbound::PLAY_INITIALIZE_BORDER;
use pumpkin_macros::packet;
use serde::Serialize;

use crate::{VarInt, codec::var_long::VarLong};

#[derive(Serialize)]
#[packet(PLAY_INITIALIZE_BORDER)]
pub struct CInitializeWorldBorder {
    x: f64,
    z: f64,
    old_diameter: f64,
    new_diameter: f64,
    speed: VarLong,
    portal_teleport_boundary: VarInt,
    warning_blocks: VarInt,
    warning_time: VarInt,
}

impl CInitializeWorldBorder {
    #[expect(clippy::too_many_arguments)]
    pub fn new(
        x: f64,
        z: f64,
        old_diameter: f64,
        new_diameter: f64,
        speed: VarLong,
        portal_teleport_boundary: VarInt,
        warning_blocks: VarInt,
        warning_time: VarInt,
    ) -> Self {
        Self {
            x,
            z,
            old_diameter,
            new_diameter,
            speed,
            portal_teleport_boundary,
            warning_blocks,
            warning_time,
        }
    }
}
