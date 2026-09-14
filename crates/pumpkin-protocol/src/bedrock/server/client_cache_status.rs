// Last verified for v2169

use crate::serial::PacketRead;
use pumpkin_macros::packet;

#[derive(PacketRead)]
#[packet(129)]
pub struct SClientCacheStatus {
    pub is_cache_supported: bool,
}
