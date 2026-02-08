use pumpkin_data::packet::serverbound::PLAY_MOVE_VEHICLE;
use pumpkin_macros::java_packet;

/// Sent when the player moves while riding a vehicle (boat, horse, etc.).
#[derive(serde::Deserialize, serde::Serialize)]
#[java_packet(PLAY_MOVE_VEHICLE)]
pub struct SMoveVehicle {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
}
