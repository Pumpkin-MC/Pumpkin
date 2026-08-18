use pumpkin_macros::packet;

use crate::{bedrock::client::common::EntityLink, serial::PacketWrite};

/// Sent by the server to set the entity an actor is riding or to unmount an actor.
///
/// Packet ID: `41`
/// Ref: <https://mojang.github.io/bedrock-protocol-docs/html/SetActorLinkPacket.html>
#[derive(PacketWrite)]
#[packet(41)]
pub struct CSetActorLink {
    pub link: EntityLink,
}
