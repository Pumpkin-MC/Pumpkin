// Last verified for v2169

use pumpkin_macros::packet;

use crate::{bedrock::client::common::ActorLink, serial::PacketWrite};

#[derive(PacketWrite)]
#[packet(41)]
pub struct CSetActorLink {
    pub link: ActorLink,
}
