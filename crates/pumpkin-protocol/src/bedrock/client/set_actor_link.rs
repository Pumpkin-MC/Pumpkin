// Last verified for v2169

use pumpkin_macros::packet;

use crate::{bedrock::client::common::ActorLink, serial::PacketWrite};

/// Sent by the server to create, modify, or remove an attachment link between two entities (e.g. riding).
#[derive(PacketWrite)]
#[packet(41)]
pub struct CSetActorLink {
    /// Attachment link descriptor defining the vehicle and passenger entities.
    pub link: ActorLink,
}
