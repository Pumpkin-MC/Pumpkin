use pumpkin_macros::packet;

use crate::serial::PacketWrite;

#[derive(PacketWrite)]
#[packet(82)]
pub struct CResourcePackDataInfo {
    pub resource_name: String,
    pub chunk_size: u32,
    pub number_of_chunks: u32,
    pub file_size: u64,
    pub file_hash: Vec<u8>,
    pub is_premium_pack: bool,
    pub pack_type: u8,
}
