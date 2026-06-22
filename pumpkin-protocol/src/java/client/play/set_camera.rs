use pumpkin_data::packet::clientbound::PLAY_SET_CAMERA;
use pumpkin_util::version::JavaMinecraftVersion;
use serde::Serialize;

use crate::{VarInt, packet::MultiVersionJavaPacket};

#[derive(Serialize)]
pub struct CSetCamera {
    pub camera_id: VarInt,
}

impl CSetCamera {
    #[must_use]
    pub const fn new(camera_id: VarInt) -> Self {
        Self { camera_id }
    }
}

impl MultiVersionJavaPacket for CSetCamera {
    fn to_id(version: JavaMinecraftVersion) -> i32 {
        PLAY_SET_CAMERA.to_id(version)
    }
}
