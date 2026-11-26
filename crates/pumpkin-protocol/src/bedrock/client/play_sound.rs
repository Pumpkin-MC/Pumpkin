// Last verified for v2225.

use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;

use crate::{codec::var_int::VarInt, serial::PacketWrite};

/// Plays a resource pack sound at the specified position.
#[derive(PacketWrite)]
#[packet(86)]
pub struct CPlaySound<'a> {
    name: &'a str,
    x: VarInt,
    y: VarInt,
    z: VarInt,
    volume: f32,
    pitch: f32,
    loop_count: VarInt,
    bypass_listener_range_check: bool,
    server_sound_handle: Option<u64>,
    playback_position_seconds: Option<f32>,
}

impl<'a> CPlaySound<'a> {
    #[must_use]
    pub fn once(name: &'a str, position: &Vector3<f64>, volume: f32, pitch: f32) -> Self {
        Self {
            name,
            x: VarInt((position.x * 8.0) as i32),
            y: VarInt((position.y * 8.0) as i32),
            z: VarInt((position.z * 8.0) as i32),
            volume,
            pitch,
            loop_count: VarInt(0),
            bypass_listener_range_check: false,
            server_sound_handle: None,
            playback_position_seconds: None,
        }
    }
}
