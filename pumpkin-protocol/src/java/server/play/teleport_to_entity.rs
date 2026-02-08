use pumpkin_data::packet::serverbound::PLAY_TELEPORT_TO_ENTITY;
use pumpkin_macros::java_packet;

/// Sent by the client in spectator mode to teleport to a specific entity.
#[derive(serde::Deserialize, serde::Serialize)]
#[java_packet(PLAY_TELEPORT_TO_ENTITY)]
pub struct STeleportToEntity {
    pub target: uuid::Uuid,
}
