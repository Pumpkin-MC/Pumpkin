use pumpkin_data::packet::serverbound::PLAY_TELEPORT_TO_ENTITY;
use pumpkin_macros::java_packet;
use serde::Deserialize;

#[derive(Deserialize)]
#[java_packet(PLAY_TELEPORT_TO_ENTITY)]
pub struct STeleportToEntity {
    #[serde(with = "uuid::serde::compact")]
    pub target: uuid::Uuid,
}
