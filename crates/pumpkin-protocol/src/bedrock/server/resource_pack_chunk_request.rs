use pumpkin_macros::packet;

use crate::serial::PacketRead;

#[derive(PacketRead)]
#[packet(84)]
pub struct SResourcePackChunkRequest {
    pub resource_name: String,
    pub chunk: i32,
}
